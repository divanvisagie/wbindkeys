use evdev_rs::enums::int_to_ev_key;
use input::event::keyboard::{KeyState, KeyboardEventTrait};
use input::event::pointer::PointerEventTrait;
use input::event::{DeviceEvent, EventTrait, PointerEvent};
use input::{Device, Event};
use std::ffi::CStr;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::parser::Keys;
use crate::script_manager::Outcome;
use crate::tracker::{Observed, Tracker};

const ISSUES_URL: &str = "https://github.com/divanvisagie/wbindkeys/issues";

/// Set by Ctrl+C so the recording can be finished off cleanly.
static STOP: AtomicBool = AtomicBool::new(false);

extern "C" fn on_sigint(_: libc::c_int) {
    STOP.store(true, Ordering::Relaxed);
}

/// Writes each line to the recording file and echoes events to the terminal.
struct Recording {
    file: File,
}

impl Recording {
    fn line(&mut self, line: &str) -> Result<(), String> {
        writeln!(self.file, "{}", line).map_err(|err| format!("writing the recording: {}", err))
    }

    fn event(&mut self, line: &str) -> Result<(), String> {
        println!("{}", line);
        self.line(line)
    }
}

/// Records every key, mouse button and scroll event wbindkeys sees, what it
/// made of each, and which binding it would run, until Ctrl+C.
pub fn record(output: &Path) -> Result<(), String> {
    let file = File::create(output).map_err(|err| format!("creating {}: {}", output.display(), err))?;
    let mut recording = Recording { file };

    eprintln!("Recording to {}.", output.display());
    eprintln!("Press the keys that don't work as you expect, then press Ctrl+C to stop.");
    eprintln!("Everything you type while recording is saved, so don't type passwords.");
    eprintln!();

    recording.line("# wbindkeys recording")?;
    recording.line(&format!("# Attach this file to a bug report at {}", ISSUES_URL))?;
    recording.line(&format!("# version: {}", env!("CARGO_PKG_VERSION")))?;
    recording.line(&format!("# system: {}", system()))?;
    recording.line(&format!(
        "# desktop: {} ({})",
        std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "unknown".into()),
        std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown session type".into())
    ))?;

    // Bindings are listed without their commands, which may be private.
    let config_path = crate::config_path();
    let (script_manager, layout) = match crate::load_config(&config_path, false) {
        Ok((script_manager, layout)) => {
            recording.line(&format!("# config: {}", config_path.display()))?;
            (Some(script_manager), layout)
        }
        Err(err) => {
            for (i, line) in err.lines().enumerate() {
                let prefix = if i == 0 { "# config: not loaded: " } else { "#   " };
                recording.line(&format!("{}{}", prefix, line))?;
            }
            (None, crate::choose_layout(None, false)?)
        }
    };
    recording.line(&format!("# layout from {}: {}", layout.source, layout.names().join(", ")))?;
    for (name, keys, on_release) in script_manager.iter().flat_map(|script_manager| script_manager.bindings()) {
        let trigger = if on_release { " on release" } else { "" };
        recording.line(&format!("# binding {}{} => {}", name, trigger, format_choices(&keys)))?;
    }

    recording.line("#")?;
    recording.line(&format!(
        "# {:>8}  {:<8} {:>5}  {:<18} {:<9} {:<20} {}",
        "time", "device", "code", "key", "state", "combo", "binding"
    ))?;

    unsafe {
        libc::signal(libc::SIGINT, on_sigint as *const () as libc::sighandler_t);
    }

    let mut input = crate::open_input();
    let mut tracker = Tracker::new(layout.modifier_keys());
    let mut devices = 0;
    let mut events = 0;
    let mut start_usec = None;

    while !STOP.load(Ordering::Relaxed) {
        input.dispatch().map_err(|err| format!("reading input events: {}", err))?;

        for event in &mut input {
            match &event {
                Event::Device(DeviceEvent::Added(_)) => {
                    devices += 1;
                    recording.line(&format!("# device added {}", describe_device(&event.device())))?;
                    continue;
                }
                Event::Device(DeviceEvent::Removed(_)) => {
                    recording.line(&format!("# device removed {}", event.device().sysname()))?;
                    continue;
                }
                _ => {}
            }

            let Some(observed) = tracker.handle(&event, Instant::now()) else {
                continue;
            };
            let time_usec = event_time_usec(&event).unwrap_or(0);
            let start = *start_usec.get_or_insert(time_usec);
            let pressed = observed.combo.is_some();
            let binding = match &script_manager {
                Some(script_manager) => describe_outcome(&script_manager.handle(&observed), pressed),
                None => describe_outcome(&Outcome { fired: Vec::new(), waiting: None }, pressed),
            };
            let seconds = time_usec.saturating_sub(start) as f64 / 1_000_000.0;
            recording.event(&format_event(seconds, event.device().sysname(), &observed, &binding))?;
            events += 1;
        }
        tracker.expire_scrolls(Instant::now());
        std::thread::sleep(Duration::from_millis(5));
    }

    recording.line(&format!("# stopped after {} events", events))?;
    eprintln!();
    if devices == 0 {
        eprintln!("warning: no input devices could be opened; run `wbindkeys permissions --check`");
    }
    eprintln!("Saved {} events to {}.", events, output.display());
    eprintln!("Attach it to a bug report at {}", ISSUES_URL);
    Ok(())
}

/// The bindings an event ran, e.g. "ALT+E", or "ALT+1 on release, waiting"
/// when a release binding's combo was pressed. "none" for a press that ran
/// nothing, "-" for a release that ran nothing.
fn describe_outcome(outcome: &Outcome, pressed: bool) -> String {
    let mut parts: Vec<String> = outcome.fired.iter().map(|fired| fired.name.clone()).collect();
    if let Some(waiting) = &outcome.waiting {
        parts.push(format!("{} on release, waiting", waiting));
    }
    match (parts.is_empty(), pressed) {
        (false, _) => parts.join(", "),
        (true, true) => "none".to_string(),
        (true, false) => "-".to_string(),
    }
}

/// One recorded event, in columns under the header written by `record`.
fn format_event(seconds: f64, device: &str, observed: &Observed, binding: &str) -> String {
    let state = match observed.state {
        KeyState::Pressed => "pressed",
        KeyState::Released => "released",
    };
    let combo = match &observed.combo {
        Some(combo) => format!("[{}]", combo.iter().map(u32::to_string).collect::<Vec<_>>().join(",")),
        None => "-".to_string(),
    };
    format!(
        "  {:>8.3}  {:<8} {:>5}  {:<18} {:<9} {:<20} {}",
        seconds,
        device,
        observed.key,
        key_name(observed.key),
        state,
        combo,
        binding
    )
}

/// The kernel's name for a key or button code, e.g. KEY_LEFTMETA or
/// BTN_LEFT, or wbindkeys' name for its virtual scroll keys.
fn key_name(code: u32) -> String {
    match code {
        c if c == Keys::ScrollUp as u32 => "SCROLLUP".to_string(),
        c if c == Keys::ScrollDown as u32 => "SCROLLDOWN".to_string(),
        c if c == Keys::ScrollLeft as u32 => "SCROLLLEFT".to_string(),
        c if c == Keys::ScrollRight as u32 => "SCROLLRIGHT".to_string(),
        _ => int_to_ev_key(code)
            .map(|key| format!("{:?}", key))
            .unwrap_or_else(|| format!("UNKNOWN_{}", code)),
    }
}

/// "[[56,100],[20]]" for ALT+T
fn format_choices(keys: &[Vec<u32>]) -> String {
    let parts: Vec<String> = keys
        .iter()
        .map(|choices| format!("[{}]", choices.iter().map(u32::to_string).collect::<Vec<_>>().join(",")))
        .collect();
    format!("[{}]", parts.join(","))
}

fn describe_device(device: &Device) -> String {
    // libinput only reports the bus type in newer versions, so read it from sysfs
    let bustype = std::fs::read_to_string(format!("/sys/class/input/{}/device/id/bustype", device.sysname()))
        .map(|bustype| bustype.trim().to_string())
        .unwrap_or_else(|_| "?".to_string());
    format!(
        "{} bus {} vendor {:04x} product {:04x} {}",
        device.sysname(),
        bustype,
        device.id_vendor(),
        device.id_product(),
        device.name()
    )
}

fn event_time_usec(event: &Event) -> Option<u64> {
    match event {
        Event::Keyboard(event) => Some(event.time_usec()),
        Event::Pointer(PointerEvent::Button(event)) => Some(event.time_usec()),
        Event::Pointer(PointerEvent::ScrollWheel(event)) => Some(event.time_usec()),
        _ => None,
    }
}

/// e.g. "Linux 6.17.0-5-generic x86_64"
fn system() -> String {
    unsafe {
        let mut name: libc::utsname = std::mem::zeroed();
        if libc::uname(&mut name) != 0 {
            return std::env::consts::OS.to_string();
        }
        let field = |chars: &[libc::c_char]| CStr::from_ptr(chars.as_ptr()).to_string_lossy().into_owned();
        format!("{} {} {}", field(&name.sysname), field(&name.release), field(&name.machine))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_are_written_in_columns() {
        let pressed = Observed {
            key: Keys::Num8 as u32,
            state: KeyState::Pressed,
            combo: Some(vec![Keys::LeftMod as u32, Keys::Num8 as u32]),
        };
        assert_eq!(
            format_event(1.5, "event3", &pressed, "MOD+8"),
            "     1.500  event3       9  KEY_8              pressed   [125,9]              MOD+8"
        );

        let released = Observed { key: Keys::LeftMod as u32, state: KeyState::Released, combo: None };
        assert_eq!(
            format_event(1.75, "event3", &released, "-"),
            "     1.750  event3     125  KEY_LEFTMETA       released  -                    -"
        );
    }

    #[test]
    fn keys_are_named_like_the_kernel_does() {
        assert_eq!(key_name(Keys::LeftMod as u32), "KEY_LEFTMETA");
        assert_eq!(key_name(Keys::Mouse1 as u32), "BTN_LEFT");
        assert_eq!(key_name(Keys::ScrollUp as u32), "SCROLLUP");
    }

    #[test]
    fn bindings_list_their_key_choices() {
        assert_eq!(format_choices(&[vec![56, 100], vec![20]]), "[[56,100],[20]]");
    }
}

use clap::{Args as ClapArgs, Parser, Subcommand};
use dirs::config_dir;
use input::event::PointerEvent;
use input::event::keyboard::{KeyState, KeyboardEventTrait};
use input::event::pointer::{ButtonState, PointerScrollEvent};
use input::{Event, Libinput, LibinputInterface};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use parser::Keys;
use script_manager::ScriptManager;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use std::u32;

const SCROLL_HOLD_MS: u64 = 500; // how long a scroll "press" lasts

/// Set by the `--debug` flag; enables `debug!` log output.
pub static DEBUG: AtomicBool = AtomicBool::new(false);

/// Prints to stderr when running with `--debug`.
macro_rules! debug {
    ($($arg:tt)*) => {
        if crate::DEBUG.load(std::sync::atomic::Ordering::Relaxed) {
            eprintln!("[debug] {}", format!($($arg)*));
        }
    };
}

mod parser;
mod permissions;
mod script_manager;

struct WBindKeysInterface;

impl LibinputInterface for WBindKeysInterface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        let file = OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) || (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) || (flags & O_RDWR != 0))
            .open(path);

        match file {
            Ok(f) => {
                debug!("Opened input device {:?}", path);
                Ok(f.into())
            }
            Err(err) => {
                debug!("Failed to open input device {:?}: {}", path, err);
                Err(err.raw_os_error().unwrap_or(-1))
            }
        }
    }

    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(File::from(fd));
    }
}

fn convert_button_to_key_state(button_state: ButtonState) -> KeyState {
    match button_state {
        ButtonState::Pressed => KeyState::Pressed,
        ButtonState::Released => KeyState::Released,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ScrollDir {
    Up,
    Down,
    Left,
    Right,
}

struct ScrollState {
    last_time: Instant,
    active: bool,
}

/// Bind keys, mouse buttons and scroll events to commands on Wayland.
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Print log output (input devices, config loading, bindings, key events)
    #[arg(short, long)]
    debug: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check or set up permission to read input devices
    Permissions(PermissionsArgs),
    /// Print the wbindkeys(1) manual page
    Man {
        /// Write it under ../share/man/man1 next to the wbindkeys binary
        /// (e.g. ~/.cargo/share/man/man1 after `cargo install`) instead
        #[arg(long)]
        install: bool,
    },
}

const MAN_PAGE: &str = include_str!("../man/wbindkeys.1");

#[derive(ClapArgs)]
#[group(required = true, multiple = false)]
struct PermissionsArgs {
    /// Check the udev rule is installed and input devices are readable
    #[arg(long)]
    check: bool,

    /// Install the udev rule and re-trigger input devices (needs root)
    #[arg(long)]
    set: bool,
}

fn man(install: bool) -> Result<(), String> {
    if !install {
        std::io::stdout().write_all(MAN_PAGE.as_bytes()).map_err(|err| err.to_string())?;
        return Ok(());
    }

    let exe = std::env::current_exe().map_err(|err| format!("locating the wbindkeys binary: {}", err))?;
    let prefix = exe
        .parent()
        .and_then(|bin| bin.parent())
        .ok_or("the wbindkeys binary has no parent directory to install under")?;
    let dir = prefix.join("share/man/man1");
    let path = dir.join("wbindkeys.1");
    std::fs::create_dir_all(&dir)
        .and_then(|()| std::fs::write(&path, MAN_PAGE))
        .map_err(|err| {
            format!(
                "writing {}: {} (run `wbindkeys man > <path>` to put it somewhere else)",
                path.display(),
                err
            )
        })?;
    println!("installed {}", path.display());
    Ok(())
}

fn main() {
    let args = Args::parse();
    DEBUG.store(args.debug, Ordering::Relaxed);

    match args.command {
        Some(Commands::Permissions(perms)) => {
            let ok = if perms.set {
                permissions::set().map_err(|err| eprintln!("error: {}", err)).is_ok()
            } else {
                permissions::check()
            };
            std::process::exit(if ok { 0 } else { 1 });
        }
        Some(Commands::Man { install }) => {
            let ok = man(install).map_err(|err| eprintln!("error: {}", err)).is_ok();
            std::process::exit(if ok { 0 } else { 1 });
        }
        None => {}
    }

    let mut input = Libinput::new_with_udev(WBindKeysInterface);
    input.udev_assign_seat("seat0").unwrap();

    let script_manager = ScriptManager::new();
    script_manager.register_functions().unwrap();

    //load from config dir
    let config_path = config_dir()
        .expect("Failed to load config directory.")
        .join("wbindkeys")
        .join("init.lua");

    if !config_path.exists() {
        panic!("Config file not found at {:?}", config_path);
    }
    debug!("Loading config from {:?}", config_path);
    let script = std::fs::read_to_string(config_path).unwrap();
    script_manager.load_script(&script).unwrap();
    debug!("Config loaded, listening for input events");

    let mut active_keys = Vec::new();
    let mut key_states: HashMap<u32, KeyState> = HashMap::new();
    let mut scroll_states: HashMap<ScrollDir, ScrollState> = HashMap::new();

    loop {
        let mut key: u32 = 0;
        let mut state: KeyState = KeyState::Released;

        input.dispatch().unwrap();

        // --- Handle libinput events ---
        for event in &mut input {
            match event {
                Event::Pointer(PointerEvent::Motion(_)) => {} // If event is mouse movement do nothing
                Event::Pointer(PointerEvent::Button(mouse_button)) => {
                    key = mouse_button.button();
                    state = convert_button_to_key_state(mouse_button.button_state());
                }
                Event::Pointer(PointerEvent::ScrollWheel(scroll_event)) => {
                    if let Some((scroll_dir, virtual_key)) = detect_scroll_direction(&scroll_event) {
                        let now = Instant::now();
                        let entry = scroll_states.entry(scroll_dir).or_insert(ScrollState {
                            last_time: now,
                            active: false,
                        });

                        // Only emit "Pressed" if not active or expired
                        if !entry.active
                            || now.duration_since(entry.last_time)
                                > Duration::from_millis(SCROLL_HOLD_MS)
                        {
                            debug!("Scroll {:?} => Pressed ({:#03x})", scroll_dir, virtual_key);

                            entry.active = true;
                            entry.last_time = now;

                            key = virtual_key;
                            state = KeyState::Pressed;
                        } else {
                            // ignore repeated scrolls in the same direction
                        }
                    }
                }
                Event::Keyboard(kb_event) => {
                    key = kb_event.key();
                    state = kb_event.key_state();
                    if key == Keys::LeftAlt as u32
                        || key == Keys::LeftCtrl as u32
                        || key == Keys::LeftMod as u32
                        || key == Keys::LeftShift as u32
                        || key == Keys::RightShift as u32
                        || key == Keys::Space as u32
                        || key == Keys::RightCtrl as u32
                        || key == Keys::RightMod as u32
                        || key == Keys::RightAlt as u32
                    {
                        match state {
                            KeyState::Pressed => active_keys.push(key),
                            KeyState::Released => active_keys.clear(),
                        }
                    }
                }
                _ => {} // Ignore all other events
            }

            // Only trigger on transition: Released → Pressed
            let prev_state = key_states.get(&key).copied().unwrap_or(KeyState::Released);
            key_states.insert(key, state);

            if state == KeyState::Pressed && prev_state == KeyState::Released {
                let total_combo = active_keys
                    .iter()
                    .chain(std::iter::once(&key))
                    .copied()
                    .collect::<Vec<u32>>();

                debug!("Pressed key {} ({:#x}), combo: {:?}", key, key, total_combo);
                script_manager.handle_action(total_combo, state);
            }
        }

        // --- Handle synthetic scroll releases ---
        let now = Instant::now();
        for (dir, state_entry) in scroll_states.iter_mut() {
            if state_entry.active
                && now.duration_since(state_entry.last_time) > Duration::from_millis(SCROLL_HOLD_MS)
            {
                let release_key = scroll_dir_to_key(*dir);
                let prev_state = key_states.get(&release_key).copied().unwrap_or(KeyState::Released);

                if prev_state == KeyState::Pressed {
                    debug!("Scroll {:?} => Released ({:#03x})", dir, release_key);

                    key_states.insert(release_key, KeyState::Released);
                    state_entry.active = false;
                }
            }
        }

        // small sleep to avoid busy loop (libinput often blocks anyway)
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn detect_scroll_direction<E>(scroll_event: &E) -> Option<(ScrollDir, u32)>
where
E: PointerScrollEvent,
{
    if scroll_event.has_axis(input::event::pointer::Axis::Vertical) {
        if scroll_event.scroll_value(input::event::pointer::Axis::Vertical) > 0.0 {
            Some((ScrollDir::Down, 0x999))
        } else {
            Some((ScrollDir::Up, 0x998))
        }
    } else if scroll_event.has_axis(input::event::pointer::Axis::Horizontal) {
        if scroll_event.scroll_value(input::event::pointer::Axis::Horizontal) > 0.0 {
            Some((ScrollDir::Right, 0x997))
        } else {
            Some((ScrollDir::Left, 0x996))
        }
    } else {
        None
    }
}

fn scroll_dir_to_key(dir: ScrollDir) -> u32 {
    match dir {
        ScrollDir::Up => 0x998,
        ScrollDir::Down => 0x999,
        ScrollDir::Left => 0x996,
        ScrollDir::Right => 0x997,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    /// The man page is written by hand, so make sure it keeps up with the
    /// CLI: every subcommand and long flag needs at least a mention.
    #[test]
    fn man_page_covers_cli() {
        fn check(cmd: &clap::Command, missing: &mut Vec<String>) {
            for arg in cmd.get_arguments() {
                // mdoc writes `--flag` as `Fl -flag`.
                if let Some(long) = arg.get_long().filter(|long| !MAN_PAGE.contains(&format!("Fl -{long}"))) {
                    missing.push(format!("--{long}"));
                }
            }
            for sub in cmd.get_subcommands() {
                if sub.get_name() != "help" && !MAN_PAGE.contains(&format!("Cm {}", sub.get_name())) {
                    missing.push(sub.get_name().to_string());
                }
                check(sub, missing);
            }
        }

        let mut missing = Vec::new();
        check(&Args::command(), &mut missing);
        assert!(missing.is_empty(), "not documented in man/wbindkeys.1: {}", missing.join(", "));
    }
}

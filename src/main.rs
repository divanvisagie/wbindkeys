use dirs::config_dir;
use input::event::{EventTrait, PointerEvent};
use input::event::keyboard::{KeyState, KeyboardEventTrait};
use input::event::pointer::{ButtonState, PointerScrollEvent};
use input::{Event, Device, Libinput, LibinputInterface};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use parser::Keys;
use script_manager::ScriptManager;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::Path;
use std::time::{Duration, Instant};
use std::u32;

const SCROLL_HOLD_MS: u64 = 500; // how long a scroll "press" lasts

mod parser;
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
            Ok(f) => Ok(f.into()),
            Err(err) => Err(err.raw_os_error().unwrap_or(-1)),
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

fn main() {
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
    let script = std::fs::read_to_string(config_path).unwrap();
    script_manager.load_script(&script).unwrap();

    let mut active_keys = Vec::new();
    let mut key_states: HashMap<u32, KeyState> = HashMap::new();
    let mut scroll_states: HashMap<ScrollDir, ScrollState> = HashMap::new();

    loop {
        let mut key: u32 = 0;
        let mut state: KeyState = KeyState::Released;

        input.dispatch().unwrap();

        // --- Handle libinput events ---
        for event in &mut input {
            let d: Device = event.device();

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
                            #[cfg(debug_assertions)]
                            println!("Scroll {:?} => Pressed ({:#03x})", scroll_dir, virtual_key);

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

                script_manager.handle_action(total_combo, state, d.id_vendor(), d.id_product());
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
                    #[cfg(debug_assertions)]
                    println!("Scroll {:?} => Released ({:#03x})", dir, release_key);

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

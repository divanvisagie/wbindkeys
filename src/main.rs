use dirs::config_dir;
use input::event::keyboard::{KeyState, KeyboardEventTrait};
use input::event::pointer::{ButtonState, PointerScrollEvent};
use input::event::PointerEvent;
use input::{Event, Libinput, LibinputInterface};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use parser::Keys;
use script_manager::ScriptManager;
use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::Path;
use std::u32;

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
    loop {
        let mut key:u32 = 0;
        let mut state: KeyState = KeyState::Released;

        input.dispatch().unwrap();
        for event in &mut input {
            match event {
                Event::Pointer(PointerEvent::Motion(_)) => {} // If event is mouse movement do nothing
                Event::Pointer(PointerEvent::Button(mouse_button)) => {
                    key = mouse_button.button();
                    state = convert_button_to_key_state(mouse_button.button_state());
                }
                Event::Pointer(PointerEvent::ScrollWheel(scroll_event)) => {
                    if scroll_event.has_axis(input::event::pointer::Axis::Vertical) == true {
                        if scroll_event.scroll_value(input::event::pointer::Axis::Vertical) > 0.0 {
                            println!("Scroll Down!");
                            key = 0x999
                        }else {
                            println!("Scroll Up!");
                            key = 0x998
                        }
                    } else {
                        if scroll_event.scroll_value(input::event::pointer::Axis::Horizontal) > 0.0 {
                            print!("Scroll Right!");
                            key = 0x997
                        } else {
                            println!("Scroll Left!");
                            key = 0x996
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
            if state == KeyState::Pressed {
                let total_combo = active_keys
                    .iter()
                    .chain(std::iter::once(&key))
                    .copied()
                    .collect::<Vec<u32>>();

                script_manager.handle_action(total_combo, state);
            }
        }
    }
}

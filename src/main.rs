use input::event::keyboard::{KeyState, KeyboardEventTrait};
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

fn main() {
    let mut input = Libinput::new_with_udev(WBindKeysInterface);
    input.udev_assign_seat("seat0").unwrap();

    let script_manager = ScriptManager::new();
    script_manager.register_functions().unwrap();

    let script = r#"
        bind("Alt+A", "alacritty")
        bind("Alt+C", "flatpak run org.telegram.desktop")
    "#;
    script_manager.load_script(script).unwrap();

    let mut active_keys = Vec::new();
    loop {
        input.dispatch().unwrap();
        for event in &mut input {
            match event {
                Event::Keyboard(kb_event) => {
                    let key = kb_event.key();
                    let state = kb_event.key_state();

                    if key == Keys::LeftAlt as u32
                        || key == Keys::LeftCtrl as u32
                        || key == Keys::LeftMod as u32
                        || key == Keys::Space as u32
                    {
                        match state {
                            KeyState::Pressed => active_keys.push(key),
                            KeyState::Released => active_keys.clear(),
                        }
                    }

                    let total_combo = active_keys
                        .iter()
                        .chain(std::iter::once(&key))
                        .copied()
                        .collect::<Vec<u32>>();

                    script_manager.handle_action(total_combo, state);
                    
                }
                _ => {} // Ignore non-keyboard events
            }
        }
    }
}

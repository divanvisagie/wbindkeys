use input::event::keyboard::{KeyState, KeyboardEventTrait};
use input::{Event, Libinput, LibinputInterface};
use mlua::{Function, Lua, Table};

use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::Path;
use std::sync::{Arc, Mutex};

struct Interface;

impl LibinputInterface for Interface {
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
enum Keys {
    A = 30,
    B = 31,
    LeftMod = 125,
    LeftAlt = 56,
}

fn main() {
    let mut input = Libinput::new_with_udev(Interface);
    input.udev_assign_seat("seat0").unwrap();

    let lua = Lua::new();
    let actions: HashMap<String, String> = HashMap::new();
    let lua = Arc::new(Mutex::new(lua));
    let actions = Arc::new(Mutex::new(actions));

    {
        let lua = Arc::clone(&lua);
        let actions = Arc::clone(&actions);

        let lock = lua.lock().unwrap();
        let r = lock
            .create_function(move |_, (binding, target): (String, String)| {
                println!("Binding key: {:?}", binding);
                println!("Target: {:?}", target);
                let mut actions_lock = actions.lock().unwrap();
                actions_lock.insert(binding, target);
                Ok(())
            })
            .unwrap();
        lock.globals().set("bind", r).unwrap();
    }

    {
        let lua = lua.clone();
        let actions = actions.clone();
        let script = r#"
            bind("Alt+A", "test")
            print("lol")
        "#;
        let lock = lua.lock().unwrap();
        let result = lock.load(script).exec();
        println!("Result output {:?}", result);

        //execute all actions
        for (key, action) in actions.lock().unwrap().iter() {
            println!("Key: {:?}, Action: {:?}", key, action);
        }
    }

    let mut active_keys = Vec::new();
    loop {
        input.dispatch().unwrap();
        for event in &mut input {
            match event {
                Event::Keyboard(kb_event) => {
                    let key = kb_event.key();
                    let state = kb_event.key_state();

                    if key == Keys::LeftAlt as u32 && state == KeyState::Pressed {
                        active_keys.push(key);
                    }
                    if key == Keys::LeftAlt as u32 && state == KeyState::Released {
                        active_keys.clear();
                    }

                    if active_keys.contains(&(Keys::LeftAlt as u32))
                        && key == Keys::A as u32
                        && state == KeyState::Pressed
                    {

                        if let Some(action) = actions.lock().unwrap().get("Alt+A") {
                            println!("Action: {:?}", action);
                        }
                    }
                }
                _ => {} // Ignore non-keyboard events
            }
        }
    }
}

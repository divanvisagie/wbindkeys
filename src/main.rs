use input::event::keyboard::{KeyState, KeyboardEventTrait};
use input::{Event, Libinput, LibinputInterface};
use mlua::{Function, Lua, Table};

use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::Path;

struct Interface;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file| file.into())
            .map_err(|err| err.raw_os_error().unwrap())
    }
    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(File::from(fd));
    }
}

fn key_to_string(key: u32) -> &'static str {
    match key {
        30 => "A",
        31 => "B",
        56 => "LeftAlt",
        125 => "LeftMod",
        _ => "Unknown",
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
    
    let r = lua.create_function(|_, (binding, target): (String, Function)| {
        println!("Binding key: {:?}", binding);
        println!("Target: {:?}", target);
        target.call::<_, ()>(()).unwrap();
        Ok(())
    }).unwrap();

    lua.globals().set("bind", r).unwrap();
    // Sample Lua script
    let script = r#"
        -- Run a Lua function when calling
        bind("alt+T", function()
            print("Alt key pressed")
            local result = os.execute("ls -l")
            print("Result: ", result)
        end) 
        print("lol");
    "#;
    let lua_ctx = lua.load(script);
    let result = lua_ctx.exec(); 
    println!("Result output {:?}", result);
   
    let mut active_keys = Vec::new();
    loop {
        input.dispatch().unwrap();
        for event in &mut input {
            match event {
                Event::Keyboard(kb_event) => {
                    let key = kb_event.key();
                    let state = kb_event.key_state();
                    println!("Keyboard event: key {:?}, state {:?}", key, state);
                    // Here, you can add logic to handle specific keys and states
                    if key == Keys::LeftAlt as u32 && state == KeyState::Pressed {
                        active_keys.push(key);
                        println!("Alt key pressed");
                    }
                    if key == Keys::LeftAlt as u32 && state == KeyState::Released {
                        println!("Alt key released");
                        active_keys.clear();
                    }

                    if active_keys.contains(&(Keys::LeftAlt as u32)) && key == Keys::A as u32 {
                        println!("Alt + A pressed");
                    }
                }
                _ => {} // Ignore non-keyboard events
            }
        }
    }
}

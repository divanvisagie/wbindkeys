use input::event::keyboard::{KeyState, KeyboardEventTrait};
use input::{Event, Libinput, LibinputInterface};
use mlua::{Function, Lua, Table};
use libc::{getegid, geteuid, getgid, setgid, setuid, O_RDONLY, O_RDWR, O_WRONLY};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::{env, u32};

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

#[derive(Debug)]
enum Bindtype {
    Command(String),
    Function(mlua::Function<'static>),
}

fn parse_binding(binding: &str) -> Vec<u32> {
    let strings: Vec<String> = binding.split('+').map(|s| s.to_string()).collect();

    let mut keys = Vec::new();
    for string in strings {
        match string.as_str() {
            "Alt" => keys.push(Keys::LeftAlt as u32),
            "A" => keys.push(Keys::A as u32),
            _ => {}
        }
    }
    keys
}

fn run_command_as_user(command: &str) {
    // Get the effective user ID and group ID
    let uid = unsafe { geteuid() };
    let gid = unsafe { getegid() };

    // Capture the environment variables
    let env_vars: Vec<(String, String)> = env::vars().collect();

    // Fork a new process
    match unsafe { libc::fork() } {
        -1 => panic!("Fork failed"),
        0 => {
            // Child process: Drop privileges and execute the command
            unsafe {
                setgid(gid);
                setuid(uid);
            }

            // Set environment variables
            for (key, value) in env_vars {
                env::set_var(key, value);
            }

            let output = Command::new("sh")
                .arg("-c")
                .arg(command)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .expect("Failed to execute command");

            println!("Command output: {:?}", output);
            std::process::exit(0);
        }
        _ => {
            // Parent process: Wait for the child to finish
            let mut status = 0;
            unsafe { libc::wait(&mut status) };
        }
    }
}

fn main() {
    let mut input = Libinput::new_with_udev(Interface);
    input.udev_assign_seat("seat0").unwrap();

    let lua = Lua::new();
    let lua =  Box::leak(Box::new(lua));
    let actions: HashMap<Vec<u32>, Bindtype> = HashMap::new();
    let actions = Arc::new(Mutex::new(actions));

    {
        let actions_str = Arc::clone(&actions);
        let actions_fun = Arc::clone(&actions);

        let basic_bind = lua
            .create_function(move |_, (binding, target): (String, String)| {
                println!("Binding key: {:?}", binding);
                println!("Target: {:?}", target);
                let mut actions_lock = actions_str.lock().unwrap();
                let binding = parse_binding(&binding);
                let target = Bindtype::Command(target);
                actions_lock.insert(binding, target);
                Ok(())
            })
            .unwrap();
        lua.globals().set("bind", basic_bind).unwrap();

        let fun_bind = lua.create_function(move |_, (binding, target): (String, mlua::Function)| {
            println!("Binding key: {:?}", binding);
            println!("Target: {:?}", target);
            let mut actions_lock = actions_fun.lock().unwrap();
            let binding = parse_binding(&binding);
            
            let target = Bindtype::Function(target);
            actions_lock.insert(binding, target);
            Ok(())
        }).unwrap();
        lua.globals().set("bind_fn", fun_bind).unwrap();
    }

    {
        let actions = actions.clone();
        let script = r#"
            bind_fn("Alt+A", function()
                print("Hello from lua")
                os.execute("xterm")
            end)
            print("lol")
        "#;
        let result = lua.load(script).exec();
        println!("Result output {:?}", result);

        //execute all actions
        for (key, action) in actions.lock().unwrap().iter() {
            println!("Registered Function: Key: {:?}, Action: {:?}", key, action);
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

                    // A merge of active keys and the current key pressed
                    let total_combo = active_keys
                        .iter()
                        .chain(std::iter::once(&key))
                        .copied()
                        .collect::<Vec<u32>>();

                    // is there something in the actions map that contains all of
                    // the keys in the total_combo
                    if let Some(action) = actions.lock().unwrap().get(&total_combo) {
                        if state == KeyState::Pressed {
                            println!("Action: {:?}", action);
                            
                            match action {
                                Bindtype::Function(func) => {
                                    let result = func.call::<_, ()>(());
                                    println!("Result output {:?}", result);
                                }
                                Bindtype::Command(command) => {
                                    run_command_as_user(command);
                                }
                            }
                        }
                    }

                }
                _ => {} // Ignore non-keyboard events
            }
        }
    }
}

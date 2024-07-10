use input::event::keyboard::{KeyState, KeyboardEventTrait};
use input::{Event, Libinput, LibinputInterface};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use mlua::Lua;
use parser::{parse_binding, Keys};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::u32;

mod parser;

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

#[derive(Debug)]
enum Bindtype {
    Command(String),
    Function(mlua::Function<'static>),
}

struct LuaManager {
    lua: &'static Lua,
    actions: Arc<Mutex<HashMap<Vec<u32>, Bindtype>>>,
}

impl LuaManager {
    fn new() -> Self {
        let lua = Box::leak(Box::new(Lua::new()));
        let actions = Arc::new(Mutex::new(HashMap::new()));

        LuaManager { lua, actions }
    }

    fn register_functions(&self) -> Result<(), mlua::Error> {
        let actions_str = Arc::clone(&self.actions);
        let actions_fun = Arc::clone(&self.actions);

        let basic_bind =
            self.lua
                .create_function(move |_, (binding, target): (String, String)| {
                    println!("Binding key: {:?}", binding);
                    println!("Target: {:?}", target);
                    let mut actions_lock = actions_str.lock().unwrap();
                    let binding = parse_binding(&binding);
                    let target = Bindtype::Command(target);
                    actions_lock.insert(binding, target);
                    Ok(())
                })?;
        self.lua.globals().set("bind", basic_bind)?;

        let fun_bind =
            self.lua
                .create_function(move |_, (binding, target): (String, mlua::Function)| {
                    println!("Binding key: {:?}", binding);
                    println!("Target: {:?}", target);
                    let mut actions_lock = actions_fun.lock().unwrap();
                    let binding = parse_binding(&binding);

                    let target = Bindtype::Function(target);
                    actions_lock.insert(binding, target);
                    Ok(())
                })?;
        self.lua.globals().set("bind_fn", fun_bind)?;

        Ok(())
    }

    fn load_script(&self, script: &str) -> Result<(), mlua::Error> {
        self.lua.load(script).exec()
    }

    fn handle_action(&self, total_combo: Vec<u32>, state: KeyState) {
        if let Some(action) = self.actions.lock().unwrap().get(&total_combo) {
            if state == KeyState::Pressed {
                println!("Action: {:?}", action);

                match action {
                    Bindtype::Function(func) => {
                        let result = func.call::<_, ()>(());
                        println!("Result output {:?}", result);
                    }
                    Bindtype::Command(command) => {
                        // run_command_as_user(command);
                        Command::new("sh")
                            .arg("-c")
                            .arg(command)
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn()
                            .expect("Failed to execute command");
                    }
                }
            }
        }
    }
}

fn main() {
    let mut input = Libinput::new_with_udev(Interface);
    input.udev_assign_seat("seat0").unwrap();

    let lua_manager = LuaManager::new();
    lua_manager.register_functions().unwrap();

    let script = r#"
        bind_fn("Alt+A", function()
            print("Hello from lua")
            os.execute("alacritty")
        end)
        bind("Alt+C", "alacritty")
    "#;
    lua_manager.load_script(script).unwrap();

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

                    lua_manager.handle_action(total_combo, state);
                    
                }
                _ => {} // Ignore non-keyboard events
            }
        }
    }
}

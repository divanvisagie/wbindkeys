
use input::event::keyboard::KeyState;
use mlua::Lua;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::u32;

use crate::parser::parse_binding;

#[derive(Debug)]
enum Bindtype {
    Command(String),
}

pub struct ScriptManager {
    lua: &'static Lua,
    actions: Arc<Mutex<HashMap<Vec<u32>, Bindtype>>>,
}

impl ScriptManager {
    pub fn new() -> Self {
        let lua = Box::leak(Box::new(Lua::new()));
        let actions = Arc::new(Mutex::new(HashMap::new()));

        ScriptManager { lua, actions }
    }

    pub fn register_functions(&self) -> Result<(), mlua::Error> {
        let actions_str = Arc::clone(&self.actions);

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

        Ok(())
    }

    pub fn load_script(&self, script: &str) -> Result<(), mlua::Error> {
        self.lua.load(script).exec()
    }

    pub fn handle_action(&self, total_combo: Vec<u32>, state: KeyState) {
        if let Some(action) = self.actions.lock().unwrap().get(&total_combo) {
            if state == KeyState::Pressed {
                println!("Action: {:?}", action);

                match action {
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

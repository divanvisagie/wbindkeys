use input::event::keyboard::KeyState;
use mlua::Lua;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::parser::parse_binding;

// ----------------------------------
// Data structures
// ----------------------------------

#[derive(Debug, Clone)]
struct DeviceFilter {
    vid: Option<u32>,
    pid: Option<u32>,
}

#[derive(Debug, Clone)]
enum Bindtype {
    Command(String),
}

#[derive(Debug, Clone)]
struct Binding {
    keys: Vec<u32>,
    action: Bindtype,
    filter: DeviceFilter,
}

// ----------------------------------
// ScriptManager definition
// ----------------------------------

pub struct ScriptManager {
    lua: &'static Lua,
    actions: Arc<Mutex<Vec<Binding>>>,
}

impl ScriptManager {
    pub fn new() -> Self {
        let lua = Box::leak(Box::new(Lua::new()));
        let actions = Arc::new(Mutex::new(Vec::new()));

        ScriptManager { lua, actions }
    }

    // --------------------------------------------------------
    // Register Lua functions
    // --------------------------------------------------------

    pub fn register_functions(&self) -> Result<(), mlua::Error> {
        let actions_ref = Arc::clone(&self.actions);

        // This defines the Lua function `bind{ ... }`
        let bind_func = self.lua.create_function(move |_, tbl: mlua::Table| {
            // Read required fields
            let keys: String = tbl.get("keys")?;
            let command: String = tbl.get("command")?;

            // Read optional fields
            let vid: Option<u32> = tbl.get("vid").ok();
            let pid: Option<u32> = tbl.get("pid").ok();

            println!("new binding: {:?}->{:?} (device filter: {:?}:{:?})", keys, command, vid, pid);

            let mut actions = actions_ref.lock().unwrap();

            // Build the binding
            let binding = Binding {
                keys: parse_binding(&keys),
                action: Bindtype::Command(command),
                filter: DeviceFilter { vid, pid },
            };

            actions.push(binding);
            Ok(())
        })?;

        // Make it available globally in Lua
        self.lua.globals().set("bind", bind_func)?;

        Ok(())
    }

    // --------------------------------------------------------
    // Load and execute a Lua script (e.g. config.lua)
    // --------------------------------------------------------

    pub fn load_script(&self, script: &str) -> Result<(), mlua::Error> {
        self.lua.load(script).exec()
    }

    // --------------------------------------------------------
    // Handle key events, considering device filters
    // --------------------------------------------------------

    pub fn handle_action(&self, combo: Vec<u32>, state: KeyState, device_vid: u32, device_pid: u32) {
        if state != KeyState::Pressed {
            return;
        }

        let actions = self.actions.lock().unwrap();

        for binding in actions.iter() {
            if binding.keys == combo {
                let f = &binding.filter;

                // Device filtering logic:
                let vid_ok = f.vid.map_or(true, |v| v == device_vid);
                let pid_ok = f.pid.map_or(true, |p| p == device_pid);

                if vid_ok && pid_ok {
                    match &binding.action {
                        Bindtype::Command(cmd) => {
                            println!(
                                "Executing {:?} (VID={:#06x}, PID={:#06x})",
                                cmd, device_vid, device_pid
                            );
                            Command::new("sh")
                                .arg("-c")
                                .arg(cmd)
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
}

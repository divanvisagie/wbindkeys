use input::event::keyboard::KeyState;
use mlua::Lua;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::u32;

use crate::parser::{matches, parse_binding};

#[derive(Debug)]
enum Bindtype {
    Command(String),
}

pub struct ScriptManager {
    lua: &'static Lua,
    /// Bindings in the order they were registered, each with the key codes
    /// that can satisfy every part of its combo (see `parse_binding`).
    actions: Arc<Mutex<Vec<(Vec<Vec<u32>>, Bindtype)>>>,
}

impl ScriptManager {
    pub fn new() -> Self {
        let lua = Box::leak(Box::new(Lua::new()));
        let actions = Arc::new(Mutex::new(Vec::new()));

        ScriptManager { lua, actions }
    }

    pub fn register_functions(&self) -> Result<(), mlua::Error> {
        let actions_str = Arc::clone(&self.actions);

        let basic_bind =
            self.lua
                .create_function(move |_, (binding, target): (String, String)| {
                    let mut actions_lock = actions_str.lock().unwrap();
                    let keys = parse_binding(&binding);
                    debug!("Registered binding {:?} => {:?} (keys: {:?})", binding, target, keys);
                    let target = Bindtype::Command(target);
                    // Binding the same combo again replaces the earlier binding.
                    match actions_lock.iter_mut().find(|(existing, _)| *existing == keys) {
                        Some((_, existing_target)) => *existing_target = target,
                        None => actions_lock.push((keys, target)),
                    }
                    Ok(())
                })?;
        self.lua.globals().set("bind", basic_bind)?;

        Ok(())
    }

    pub fn load_script(&self, script: &str) -> Result<(), mlua::Error> {
        self.lua.load(script).exec()
    }

    pub fn handle_action(&self, total_combo: Vec<u32>, state: KeyState) {
        let actions = self.actions.lock().unwrap();
        if let Some(action) = best_match(&actions, &total_combo) {
            if state == KeyState::Pressed {
                match action {
                    Bindtype::Command(command) => {
                        debug!("Matched combo {:?}, running: {}", total_combo, command);
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
        } else {
            debug!("No binding for combo {:?}", total_combo);
        }
    }
}

/// The binding to run for a pressed combo. When several match, e.g. ALT+E
/// and RIGHTALT+E for Right Alt+E, the most specific one (fewest accepted
/// keys) wins.
fn best_match<'a>(actions: &'a [(Vec<Vec<u32>>, Bindtype)], combo: &[u32]) -> Option<&'a Bindtype> {
    actions
        .iter()
        .filter(|(keys, _)| matches(keys, combo))
        .min_by_key(|(keys, _)| keys.iter().map(Vec::len).sum::<usize>())
        .map(|(_, action)| action)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Keys;

    fn command(action: Option<&Bindtype>) -> Option<&str> {
        action.map(|Bindtype::Command(command)| command.as_str())
    }

    #[test]
    fn one_sided_binding_wins_over_either_side() {
        // Registered in both orders, so the result doesn't depend on order.
        for bindings in [["ALT+E", "RIGHTALT+E"], ["RIGHTALT+E", "ALT+E"]] {
            let actions: Vec<_> = bindings
                .iter()
                .map(|binding| (parse_binding(binding), Bindtype::Command(binding.to_string())))
                .collect();

            let right = [Keys::RightAlt as u32, Keys::E as u32];
            let left = [Keys::LeftAlt as u32, Keys::E as u32];
            assert_eq!(command(best_match(&actions, &right)), Some("RIGHTALT+E"));
            assert_eq!(command(best_match(&actions, &left)), Some("ALT+E"));
        }
    }
}

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

struct Binding {
    /// The combo as written in the config, e.g. "ALT+T"
    name: String,
    /// The key codes that can satisfy every part of the combo (see `parse_binding`)
    keys: Vec<Vec<u32>>,
    action: Bindtype,
}

pub struct ScriptManager {
    lua: &'static Lua,
    /// Bindings in the order they were registered
    actions: Arc<Mutex<Vec<Binding>>>,
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
                    let new = Binding { name: binding, keys, action: Bindtype::Command(target) };
                    // Binding the same combo again replaces the earlier binding.
                    match actions_lock.iter_mut().find(|existing| existing.keys == new.keys) {
                        Some(existing) => *existing = new,
                        None => actions_lock.push(new),
                    }
                    Ok(())
                })?;
        self.lua.globals().set("bind", basic_bind)?;

        Ok(())
    }

    pub fn load_script(&self, script: &str) -> Result<(), mlua::Error> {
        self.lua.load(script).exec()
    }

    /// Every registered binding as written in the config, with its key codes.
    pub fn bindings(&self) -> Vec<(String, Vec<Vec<u32>>)> {
        let actions = self.actions.lock().unwrap();
        actions.iter().map(|binding| (binding.name.clone(), binding.keys.clone())).collect()
    }

    /// The binding a pressed combo would run, as written in the config.
    pub fn matching_binding(&self, combo: &[u32]) -> Option<String> {
        let actions = self.actions.lock().unwrap();
        best_match(&actions, combo).map(|binding| binding.name.clone())
    }

    pub fn handle_action(&self, total_combo: Vec<u32>, state: KeyState) {
        let actions = self.actions.lock().unwrap();
        if let Some(binding) = best_match(&actions, &total_combo) {
            if state == KeyState::Pressed {
                match &binding.action {
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
fn best_match<'a>(actions: &'a [Binding], combo: &[u32]) -> Option<&'a Binding> {
    actions
        .iter()
        .filter(|binding| matches(&binding.keys, combo))
        .min_by_key(|binding| binding.keys.iter().map(Vec::len).sum::<usize>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Keys;

    #[test]
    fn one_sided_binding_wins_over_either_side() {
        // Registered in both orders, so the result doesn't depend on order.
        for bindings in [["ALT+E", "RIGHTALT+E"], ["RIGHTALT+E", "ALT+E"]] {
            let actions: Vec<_> = bindings
                .iter()
                .map(|name| Binding {
                    name: name.to_string(),
                    keys: parse_binding(name),
                    action: Bindtype::Command(String::new()),
                })
                .collect();

            let right = [Keys::RightAlt as u32, Keys::E as u32];
            let left = [Keys::LeftAlt as u32, Keys::E as u32];
            let name = |combo: &[u32]| best_match(&actions, combo).map(|binding| binding.name.as_str());
            assert_eq!(name(&right), Some("RIGHTALT+E"));
            assert_eq!(name(&left), Some("ALT+E"));
        }
    }
}

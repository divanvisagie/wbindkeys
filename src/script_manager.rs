use input::event::keyboard::KeyState;
use mlua::{Lua, Table};
use std::cell::RefCell;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::layout::Layout;
use crate::parser::{matches, parse_binding, Keys};
use crate::tracker::Observed;

#[derive(Clone, Debug)]
enum Bindtype {
    Command(String),
}

/// When a binding runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Trigger {
    /// As soon as the combo is pressed (the default)
    Press,
    /// Once every key of the combo has been released, provided nothing else
    /// was pressed in between
    Release,
}

struct Binding {
    /// The combo as written in the config, e.g. "ALT+T"
    name: String,
    /// The key codes that can satisfy every part of the combo, once looked
    /// up in the layout by `resolve` (see `parse_binding`)
    keys: Vec<Vec<u32>>,
    trigger: Trigger,
    action: Bindtype,
}

/// A binding that an input event made run.
pub struct Fired {
    /// The combo as written in the config
    pub name: String,
    action: Bindtype,
}

impl Fired {
    fn new(binding: &Binding) -> Self {
        Fired { name: binding.name.clone(), action: binding.action.clone() }
    }

    pub fn run(&self) {
        match &self.action {
            Bindtype::Command(command) => {
                debug!("Running {}: {}", self.name, command);
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

/// What an input event led to.
pub struct Outcome {
    /// Bindings to run now
    pub fired: Vec<Fired>,
    /// A release binding whose combo was just pressed, which will run once
    /// its keys are released
    pub waiting: Option<String>,
}

/// A release binding whose combo was pressed, waiting for its keys to be released.
struct Pending {
    fired: Fired,
    held: Vec<u32>,
}

/// The layout set with keyboard{ layout = ..., variant = ..., options = ... }
#[derive(Clone, Debug, Default, PartialEq)]
pub struct KeyboardSetting {
    pub layout: String,
    pub variant: String,
    pub options: String,
}

pub struct ScriptManager {
    lua: &'static Lua,
    /// Bindings in the order they were registered
    actions: Arc<Mutex<Vec<Binding>>>,
    keyboard: Arc<Mutex<Option<KeyboardSetting>>>,
    pending: RefCell<Option<Pending>>,
}

impl ScriptManager {
    pub fn new() -> Self {
        let lua = Box::leak(Box::new(Lua::new()));
        let actions = Arc::new(Mutex::new(Vec::new()));

        ScriptManager { lua, actions, keyboard: Arc::new(Mutex::new(None)), pending: RefCell::new(None) }
    }

    pub fn register_functions(&self) -> Result<(), mlua::Error> {
        let actions_str = Arc::clone(&self.actions);

        let basic_bind = self.lua.create_function(
            move |_, (binding, target, options): (String, String, Option<Table>)| {
                let trigger = parse_options(&binding, options)?;
                debug!("Registered binding {:?} on {:?} => {:?}", binding, trigger, target);
                // Keys are looked up in `resolve`, once the whole config has
                // run and any keyboard{} setting is known.
                let new = Binding { name: binding, keys: Vec::new(), trigger, action: Bindtype::Command(target) };
                actions_str.lock().unwrap().push(new);
                Ok(())
            },
        )?;
        self.lua.globals().set("bind", basic_bind)?;

        let keyboard = Arc::clone(&self.keyboard);
        let set_keyboard = self.lua.create_function(move |_, options: Table| {
            let mut setting = KeyboardSetting::default();
            for pair in options.pairs::<String, String>() {
                let (key, value) = pair?;
                match key.as_str() {
                    "layout" => setting.layout = value,
                    "variant" => setting.variant = value,
                    "options" => setting.options = value,
                    other => {
                        return Err(mlua::Error::RuntimeError(format!(
                            "keyboard: unknown option `{}`, expected `layout`, `variant` or `options`",
                            other
                        )))
                    }
                }
            }
            if setting.layout.is_empty() {
                return Err(mlua::Error::RuntimeError("keyboard: `layout` is required".to_string()));
            }
            *keyboard.lock().unwrap() = Some(setting);
            Ok(())
        })?;
        self.lua.globals().set("keyboard", set_keyboard)?;

        Ok(())
    }

    /// The layout set with keyboard{} in the config, if any.
    pub fn keyboard(&self) -> Option<KeyboardSetting> {
        self.keyboard.lock().unwrap().clone()
    }

    /// Looks every binding's keys up in the layout. A binding with the same
    /// keys and trigger as an earlier one replaces it. Fails with every
    /// binding that names a key the layout doesn't have.
    pub fn resolve(&self, layout: &Layout) -> Result<(), String> {
        let mut actions = self.actions.lock().unwrap();
        let mut errors = Vec::new();
        let mut resolved: Vec<Binding> = Vec::new();
        for mut binding in actions.drain(..) {
            match parse_binding(&binding.name, layout) {
                Ok(keys) => {
                    debug!("Binding {:?} is keys {:?}", binding.name, keys);
                    binding.keys = keys;
                    let same = |existing: &&mut Binding| {
                        existing.trigger == binding.trigger && same_parts(&existing.keys, &binding.keys)
                    };
                    match resolved.iter_mut().find(|existing| same(existing)) {
                        Some(existing) => *existing = binding,
                        None => resolved.push(binding),
                    }
                }
                Err(err) => errors.push(format!("bind({:?}): {}", binding.name, err)),
            }
        }
        *actions = resolved;
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("\n"))
        }
    }

    pub fn load_script(&self, script: &str) -> Result<(), mlua::Error> {
        self.lua.load(script).exec()
    }

    /// Every registered binding as written in the config, with its key codes
    /// and whether it runs on release.
    pub fn bindings(&self) -> Vec<(String, Vec<Vec<u32>>, bool)> {
        let actions = self.actions.lock().unwrap();
        actions
            .iter()
            .map(|binding| (binding.name.clone(), binding.keys.clone(), binding.trigger == Trigger::Release))
            .collect()
    }

    /// Works out which bindings an input event runs, keeping track of release
    /// bindings that are waiting for their keys to be released. Doesn't run
    /// anything itself, so `wbindkeys record` can use it too.
    pub fn handle(&self, observed: &Observed) -> Outcome {
        let actions = self.actions.lock().unwrap();
        let mut pending = self.pending.borrow_mut();
        let mut outcome = Outcome { fired: Vec::new(), waiting: None };

        match &observed.combo {
            Some(combo) => {
                // Pressing anything else cancels a release binding that was waiting.
                if let Some(cancelled) = pending.take() {
                    debug!("Cancelled {} (on release): another key was pressed", cancelled.fired.name);
                }
                if let Some(binding) = best_match(&actions, combo, Trigger::Press) {
                    debug!("Matched combo {:?} to {}", combo, binding.name);
                    outcome.fired.push(Fired::new(binding));
                }
                if let Some(binding) = best_match(&actions, combo, Trigger::Release) {
                    // Scrolls have no release of their own, so don't wait for them.
                    let held: Vec<u32> = combo.iter().copied().filter(|key| !is_scroll(*key)).collect();
                    if held.is_empty() {
                        outcome.fired.push(Fired::new(binding));
                    } else {
                        debug!("Matched combo {:?} to {} (on release), waiting for release", combo, binding.name);
                        outcome.waiting = Some(binding.name.clone());
                        *pending = Some(Pending { fired: Fired::new(binding), held });
                    }
                }
                if outcome.fired.is_empty() && outcome.waiting.is_none() {
                    debug!("No binding for combo {:?}", combo);
                }
            }
            None if observed.state == KeyState::Released => {
                if let Some(waiting) = pending.as_mut() {
                    waiting.held.retain(|key| *key != observed.key);
                    if waiting.held.is_empty() {
                        outcome.fired.push(pending.take().unwrap().fired);
                    }
                }
            }
            None => {}
        }
        outcome
    }
}

/// Reads the optional third argument of bind(), e.g. { on = "release" }.
fn parse_options(binding: &str, options: Option<Table>) -> Result<Trigger, mlua::Error> {
    let Some(options) = options else {
        return Ok(Trigger::Press);
    };
    let mut trigger = Trigger::Press;
    for pair in options.pairs::<String, mlua::Value>() {
        let (key, value) = pair?;
        match (key.as_str(), value.as_str().as_deref()) {
            ("on", Some("press")) => trigger = Trigger::Press,
            ("on", Some("release")) => trigger = Trigger::Release,
            ("on", _) => {
                return Err(mlua::Error::RuntimeError(format!(
                    "bind({:?}): `on` must be \"press\" or \"release\"",
                    binding
                )))
            }
            (other, _) => {
                return Err(mlua::Error::RuntimeError(format!(
                    "bind({:?}): unknown option `{}`, expected `on`",
                    binding, other
                )))
            }
        }
    }
    Ok(trigger)
}

/// Whether two bindings have the same parts, in any order.
fn same_parts(a: &[Vec<u32>], b: &[Vec<u32>]) -> bool {
    a.len() == b.len() && a.iter().all(|part| b.contains(part))
}

fn is_scroll(key: u32) -> bool {
    [Keys::ScrollUp, Keys::ScrollDown, Keys::ScrollLeft, Keys::ScrollRight]
        .into_iter()
        .any(|scroll| scroll as u32 == key)
}

/// The binding with the given trigger to run for a pressed combo. When
/// several match, e.g. ALT+E and RIGHTALT+E for Right Alt+E, the most
/// specific one (fewest accepted keys) wins.
fn best_match<'a>(actions: &'a [Binding], combo: &[u32], trigger: Trigger) -> Option<&'a Binding> {
    actions
        .iter()
        .filter(|binding| binding.trigger == trigger && matches(&binding.keys, combo))
        .min_by_key(|binding| binding.keys.iter().map(Vec::len).sum::<usize>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::Tracker;

    fn manager(config: &str) -> ScriptManager {
        let manager = ScriptManager::new();
        manager.register_functions().unwrap();
        manager.load_script(config).unwrap();
        manager.resolve(&Layout::from_names("us", "", "", "test").unwrap()).unwrap();
        manager
    }

    /// Feeds key presses and releases through a tracker and the manager,
    /// returning the names of everything that fired.
    fn fire(manager: &ScriptManager, events: &[(Keys, KeyState)]) -> Vec<String> {
        let mut tracker = Tracker::new(us_modifiers());
        events
            .iter()
            .flat_map(|(key, state)| manager.handle(&tracker.key(*key as u32, *state)).fired)
            .map(|fired| fired.name)
            .collect()
    }

    use KeyState::{Pressed as Down, Released as Up};

    fn us_modifiers() -> Vec<u32> {
        Layout::from_names("us", "", "", "test").unwrap().modifier_keys()
    }

    #[test]
    fn one_sided_binding_wins_over_either_side() {
        // Registered in both orders, so the result doesn't depend on order.
        for config in [
            r#"bind("ALT+E", "") bind("RIGHTALT+E", "")"#,
            r#"bind("RIGHTALT+E", "") bind("ALT+E", "")"#,
        ] {
            let manager = manager(config);
            assert_eq!(fire(&manager, &[(Keys::RightAlt, Down), (Keys::E, Down)]), ["RIGHTALT+E"]);
            assert_eq!(fire(&manager, &[(Keys::LeftAlt, Down), (Keys::E, Down)]), ["ALT+E"]);
        }
    }

    #[test]
    fn release_binding_waits_until_every_key_is_released() {
        let manager = manager(r#"bind("ALT+1", "", { on = "release" })"#);
        let mut tracker = Tracker::new(us_modifiers());
        let mut step = |key: Keys, state| manager.handle(&tracker.key(key as u32, state));

        assert!(step(Keys::LeftAlt, Down).fired.is_empty());
        let pressed = step(Keys::Num1, Down);
        assert!(pressed.fired.is_empty());
        assert_eq!(pressed.waiting.as_deref(), Some("ALT+1"));
        assert!(step(Keys::Num1, Up).fired.is_empty(), "Alt is still held");
        let fired: Vec<_> = step(Keys::LeftAlt, Up).fired.into_iter().map(|fired| fired.name).collect();
        assert_eq!(fired, ["ALT+1"]);
    }

    #[test]
    fn release_binding_is_cancelled_by_another_key() {
        let manager = manager(r#"bind("MOD", "", { on = "release" }) bind("MOD+8", "")"#);
        // Tapping Super on its own runs the release binding...
        assert_eq!(fire(&manager, &[(Keys::LeftMod, Down), (Keys::LeftMod, Up)]), ["MOD"]);
        // ...but using it in Super+8 only runs MOD+8.
        let super_8 = [(Keys::LeftMod, Down), (Keys::Num8, Down), (Keys::Num8, Up), (Keys::LeftMod, Up)];
        assert_eq!(fire(&manager, &super_8), ["MOD+8"]);
    }

    #[test]
    fn press_and_release_bindings_can_share_a_combo() {
        let manager = manager(r#"bind("ALT+T", "") bind("ALT+T", "", { on = "release" })"#);
        let alt_t = [(Keys::LeftAlt, Down), (Keys::T, Down), (Keys::T, Up), (Keys::LeftAlt, Up)];
        assert_eq!(fire(&manager, &alt_t), ["ALT+T", "ALT+T"]);
        assert_eq!(manager.bindings().len(), 2);
    }

    #[test]
    fn bad_options_are_an_error() {
        let manager = ScriptManager::new();
        manager.register_functions().unwrap();
        let err = manager.load_script(r#"bind("ALT+T", "", { on = "hold" })"#).unwrap_err();
        assert!(err.to_string().contains("must be \"press\" or \"release\""), "{}", err);
        let err = manager.load_script(r#"bind("ALT+T", "", { when = "release" })"#).unwrap_err();
        assert!(err.to_string().contains("unknown option `when`"), "{}", err);
    }

    #[test]
    fn keyboard_sets_the_layout() {
        let manager = manager(r#"keyboard{ layout = "fr", options = "caps:swapescape" }"#);
        let expected = KeyboardSetting { layout: "fr".into(), variant: String::new(), options: "caps:swapescape".into() };
        assert_eq!(manager.keyboard(), Some(expected));
        assert_eq!(manager.keyboard().map(|setting| setting.layout), Some("fr".to_string()));

        let manager = ScriptManager::new();
        manager.register_functions().unwrap();
        let err = manager.load_script(r#"keyboard{ layuot = "fr" }"#).unwrap_err();
        assert!(err.to_string().contains("unknown option `layuot`"), "{}", err);
    }

    #[test]
    fn resolve_reports_every_unknown_key() {
        let manager = ScriptManager::new();
        manager.register_functions().unwrap();
        manager.load_script(r#"bind("ALT+NOPE", "") bind("ALT+T", "") bind("MOD+ж", "")"#).unwrap();
        let err = manager.resolve(&Layout::from_names("us", "", "", "test").unwrap()).unwrap_err();
        assert!(err.contains("bind(\"ALT+NOPE\")") && err.contains("bind(\"MOD+ж\")"), "{}", err);
        assert!(!err.contains("ALT+T"), "{}", err);
    }

    #[test]
    fn rebinding_the_same_keys_replaces_the_binding() {
        // Written differently, but the same keys: the later one wins
        let manager = manager(r#"bind("ALT+CTRL+T", "") bind("CTRL+ALT+T", "")"#);
        let names: Vec<_> = manager.bindings().into_iter().map(|(name, _, _)| name).collect();
        assert_eq!(names, ["CTRL+ALT+T"]);
    }
}

use input::event::keyboard::{KeyState, KeyboardEventTrait};
use input::event::pointer::{Axis, ButtonState, PointerScrollEvent};
use input::event::PointerEvent;
use input::Event;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::parser::Keys;

const SCROLL_HOLD_MS: u64 = 500; // how long a scroll "press" lasts

/// Keys that are remembered while held and put in front of the next key
/// pressed to make a combo.
const MODIFIERS: [u32; 9] = [
    Keys::LeftAlt as u32,
    Keys::LeftCtrl as u32,
    Keys::LeftMod as u32,
    Keys::LeftShift as u32,
    Keys::RightShift as u32,
    Keys::Space as u32,
    Keys::RightCtrl as u32,
    Keys::RightMod as u32,
    Keys::RightAlt as u32,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScrollDir {
    Up,
    Down,
    Left,
    Right,
}

struct ScrollState {
    last_time: Instant,
    active: bool,
}

/// A key, mouse button or scroll event as wbindkeys sees it.
pub struct Observed {
    pub key: u32,
    pub state: KeyState,
    /// The combo to match bindings against, when this is a new press.
    pub combo: Option<Vec<u32>>,
}

/// Turns input events into the combos bindings are matched against. Shared
/// by the main loop and `wbindkeys record`, so a recording shows exactly what
/// wbindkeys made of each event.
pub struct Tracker {
    active_keys: Vec<u32>,
    key_states: HashMap<u32, KeyState>,
    scroll_states: HashMap<ScrollDir, ScrollState>,
}

impl Tracker {
    pub fn new() -> Self {
        Tracker {
            active_keys: Vec::new(),
            key_states: HashMap::new(),
            scroll_states: HashMap::new(),
        }
    }

    /// Feeds one libinput event. Returns None for events wbindkeys ignores,
    /// such as pointer motion or a scroll that continues an earlier one.
    pub fn handle(&mut self, event: &Event, now: Instant) -> Option<Observed> {
        match event {
            Event::Keyboard(kb_event) => Some(self.key(kb_event.key(), kb_event.key_state())),
            Event::Pointer(PointerEvent::Button(button)) => {
                let state = match button.button_state() {
                    ButtonState::Pressed => KeyState::Pressed,
                    ButtonState::Released => KeyState::Released,
                };
                Some(self.button(button.button(), state))
            }
            Event::Pointer(PointerEvent::ScrollWheel(scroll_event)) => {
                let dir = detect_scroll_direction(scroll_event)?;
                self.scroll(dir, now)
            }
            _ => None,
        }
    }

    /// Feeds a keyboard key press or release.
    pub fn key(&mut self, key: u32, state: KeyState) -> Observed {
        if MODIFIERS.contains(&key) {
            match state {
                KeyState::Pressed => self.active_keys.push(key),
                KeyState::Released => self.active_keys.clear(),
            }
        }
        self.transition(key, state)
    }

    /// Feeds a mouse button press or release.
    pub fn button(&mut self, button: u32, state: KeyState) -> Observed {
        self.transition(button, state)
    }

    /// Feeds a scroll. A scroll acts as a press of a virtual key, which is
    /// held until SCROLL_HOLD_MS passes without scrolling that way again.
    pub fn scroll(&mut self, dir: ScrollDir, now: Instant) -> Option<Observed> {
        let virtual_key = scroll_dir_to_key(dir);
        let entry = self.scroll_states.entry(dir).or_insert(ScrollState {
            last_time: now,
            active: false,
        });

        // Only emit "Pressed" if not active or expired
        if entry.active && now.duration_since(entry.last_time) <= Duration::from_millis(SCROLL_HOLD_MS) {
            return None; // ignore repeated scrolls in the same direction
        }
        debug!("Scroll {:?} => Pressed ({:#03x})", dir, virtual_key);
        entry.active = true;
        entry.last_time = now;
        Some(self.transition(virtual_key, KeyState::Pressed))
    }

    /// Releases the virtual keys of scrolls that have stopped.
    pub fn expire_scrolls(&mut self, now: Instant) {
        for (dir, state_entry) in self.scroll_states.iter_mut() {
            if state_entry.active
                && now.duration_since(state_entry.last_time) > Duration::from_millis(SCROLL_HOLD_MS)
            {
                let release_key = scroll_dir_to_key(*dir);
                let prev_state = self.key_states.get(&release_key).copied().unwrap_or(KeyState::Released);

                if prev_state == KeyState::Pressed {
                    debug!("Scroll {:?} => Released ({:#03x})", dir, release_key);

                    self.key_states.insert(release_key, KeyState::Released);
                    state_entry.active = false;
                }
            }
        }
    }

    fn transition(&mut self, key: u32, state: KeyState) -> Observed {
        // Only trigger on transition: Released → Pressed
        let prev_state = self.key_states.get(&key).copied().unwrap_or(KeyState::Released);
        self.key_states.insert(key, state);

        let combo = (state == KeyState::Pressed && prev_state == KeyState::Released).then(|| {
            let combo: Vec<u32> = self.active_keys.iter().copied().chain(std::iter::once(key)).collect();
            debug!("Pressed key {} ({:#x}), combo: {:?}", key, key, combo);
            combo
        });
        Observed { key, state, combo }
    }
}

fn detect_scroll_direction<E>(scroll_event: &E) -> Option<ScrollDir>
where
    E: PointerScrollEvent,
{
    if scroll_event.has_axis(Axis::Vertical) {
        if scroll_event.scroll_value(Axis::Vertical) > 0.0 {
            Some(ScrollDir::Down)
        } else {
            Some(ScrollDir::Up)
        }
    } else if scroll_event.has_axis(Axis::Horizontal) {
        if scroll_event.scroll_value(Axis::Horizontal) > 0.0 {
            Some(ScrollDir::Right)
        } else {
            Some(ScrollDir::Left)
        }
    } else {
        None
    }
}

fn scroll_dir_to_key(dir: ScrollDir) -> u32 {
    match dir {
        ScrollDir::Up => Keys::ScrollUp as u32,
        ScrollDir::Down => Keys::ScrollDown as u32,
        ScrollDir::Left => Keys::ScrollLeft as u32,
        ScrollDir::Right => Keys::ScrollRight as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(tracker: &mut Tracker, key: Keys) -> Option<Vec<u32>> {
        tracker.key(key as u32, KeyState::Pressed).combo
    }

    fn release(tracker: &mut Tracker, key: Keys) {
        tracker.key(key as u32, KeyState::Released);
    }

    #[test]
    fn held_modifiers_go_in_front_of_the_key() {
        let mut tracker = Tracker::new();
        press(&mut tracker, Keys::LeftCtrl);
        press(&mut tracker, Keys::LeftAlt);
        assert_eq!(press(&mut tracker, Keys::T), Some(vec![Keys::LeftCtrl as u32, Keys::LeftAlt as u32, Keys::T as u32]));
    }

    #[test]
    fn releasing_a_modifier_forgets_held_modifiers() {
        let mut tracker = Tracker::new();
        press(&mut tracker, Keys::LeftMod);
        release(&mut tracker, Keys::LeftMod);
        assert_eq!(press(&mut tracker, Keys::Num8), Some(vec![Keys::Num8 as u32]));
    }

    #[test]
    fn a_held_key_only_makes_a_combo_once() {
        let mut tracker = Tracker::new();
        assert!(press(&mut tracker, Keys::T).is_some());
        assert_eq!(press(&mut tracker, Keys::T), None);
        release(&mut tracker, Keys::T);
        assert!(press(&mut tracker, Keys::T).is_some());
    }

    #[test]
    fn scrolling_presses_a_virtual_key_until_it_stops() {
        let mut tracker = Tracker::new();
        let start = Instant::now();
        let combo = tracker.scroll(ScrollDir::Up, start).and_then(|observed| observed.combo);
        assert_eq!(combo, Some(vec![Keys::ScrollUp as u32]));
        assert!(tracker.scroll(ScrollDir::Up, start + Duration::from_millis(100)).is_none());

        let later = start + Duration::from_millis(SCROLL_HOLD_MS + 200);
        tracker.expire_scrolls(later);
        let combo = tracker.scroll(ScrollDir::Up, later).and_then(|observed| observed.combo);
        assert_eq!(combo, Some(vec![Keys::ScrollUp as u32]));
    }
}

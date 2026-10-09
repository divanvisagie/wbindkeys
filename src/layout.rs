use std::io::Read;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use wayland_client::protocol::{wl_keyboard, wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, QueueHandle, WEnum};
use xkbcommon::xkb::{self, Keysym};

/// evdev key codes are xkb key codes minus this
const EVDEV_OFFSET: u32 = 8;

/// A modifier, as set by the keys that act as it in the keymap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modifier {
    Shift,
    AltGr,
    Ctrl,
    Alt,
    Super,
}

impl Modifier {
    const ALL: [Modifier; 5] = [Modifier::Shift, Modifier::AltGr, Modifier::Ctrl, Modifier::Alt, Modifier::Super];

    /// The xkb modifier names it goes by; newer keymaps use the virtual names.
    fn xkb_names(self) -> &'static [&'static str] {
        match self {
            Modifier::Shift => &["Shift"],
            Modifier::AltGr => &["Mod5", "LevelThree"],
            Modifier::Ctrl => &["Control"],
            Modifier::Alt => &["Mod1", "Alt"],
            Modifier::Super => &["Mod4", "Super"],
        }
    }
}

/// The keyboard layout that key names in bindings are looked up in.
pub struct Layout {
    keymap: xkb::Keymap,
    /// Where the layout came from, e.g. "compositor" or "config", for logs
    /// and recordings
    pub source: String,
}

impl Layout {
    /// A layout from xkb names, like the ones in `localectl`: "fr", or
    /// "us,se" for several.
    pub fn from_names(layout: &str, variant: &str, options: &str, source: &str) -> Result<Layout, String> {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let options = (!options.is_empty()).then(|| options.to_string());
        let keymap =
            xkb::Keymap::new_from_names(&context, "", "", layout, variant, options, xkb::KEYMAP_COMPILE_NO_FLAGS)
                .ok_or_else(|| format!("unknown keyboard layout {:?} (variant {:?})", layout, variant))?;
        Ok(Layout { keymap, source: source.to_string() })
    }

    /// The layout the compositor is using, read over Wayland.
    pub fn from_compositor() -> Result<Layout, String> {
        let text = compositor_keymap()?;
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let keymap = xkb::Keymap::new_from_string(
            &context,
            text,
            xkb::KEYMAP_FORMAT_TEXT_V1,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        )
        .ok_or("the compositor's keymap could not be compiled")?;
        Ok(Layout { keymap, source: "compositor".to_string() })
    }

    /// The names of the layouts in the keymap, e.g. ["English (US)", "Swedish"]
    pub fn names(&self) -> Vec<String> {
        (0..self.keymap.num_layouts()).map(|i| self.keymap.layout_get_name(i).to_string()).collect()
    }

    /// The keys that type a keysym and the modifiers they need, from the
    /// first layout that has it, at the lowest level. Keys that type it
    /// without modifiers all come back, e.g. both Return keys; otherwise
    /// only the first.
    pub fn find(&self, keysym: Keysym) -> Option<(Vec<u32>, Vec<Modifier>)> {
        for layout in 0..self.keymap.num_layouts() {
            for level in 0..4 {
                let mut keys = self
                    .keycodes()
                    .filter(|key| level < self.keymap.num_levels_for_key(*key, layout))
                    .filter(|key| self.keymap.key_get_syms_by_level(*key, layout, level).contains(&keysym));
                let Some(first) = keys.next() else {
                    continue;
                };
                let modifiers = self.modifiers_for_level(first, layout, level);
                let mut found = vec![first.raw() - EVDEV_OFFSET];
                if level == 0 {
                    found.extend(keys.map(|key| key.raw() - EVDEV_OFFSET));
                }
                return Some((found, modifiers));
            }
        }
        None
    }

    /// The keys that act as a modifier when held, e.g. both Ctrl keys, plus
    /// Caps Lock when it's set to act as Ctrl.
    pub fn keys_for(&self, modifier: Modifier) -> Vec<u32> {
        self.keycodes()
            .filter(|key| {
                let mut state = xkb::State::new(&self.keymap);
                state.update_key(*key, xkb::KeyDirection::Down);
                modifier
                    .xkb_names()
                    .iter()
                    .any(|name| state.mod_name_is_active(name, xkb::STATE_MODS_DEPRESSED))
            })
            .map(|key| key.raw() - EVDEV_OFFSET)
            .collect()
    }

    /// Every key that acts as a modifier when held.
    pub fn modifier_keys(&self) -> Vec<u32> {
        let mut keys: Vec<u32> = Modifier::ALL.iter().flat_map(|modifier| self.keys_for(*modifier)).collect();
        keys.sort_unstable();
        keys.dedup();
        keys
    }

    /// Every key in the keymap, except the placeholder keys xkb uses to set
    /// modifiers internally, which no keyboard sends.
    fn keycodes(&self) -> impl Iterator<Item = xkb::Keycode> + '_ {
        const PLACEHOLDERS: [&str; 6] = ["LVL3", "MDSW", "ALT", "META", "SUPR", "HYPR"];
        (self.keymap.min_keycode().raw()..=self.keymap.max_keycode().raw())
            .filter(|raw| *raw >= EVDEV_OFFSET)
            .map(xkb::Keycode::new)
            .filter(|key| !self.keymap.key_get_name(*key).is_some_and(|name| PLACEHOLDERS.contains(&name)))
    }

    /// The modifiers needed to reach a level of a key, ignoring ways that
    /// go through Caps Lock.
    fn modifiers_for_level(&self, key: xkb::Keycode, layout: u32, level: u32) -> Vec<Modifier> {
        let mut masks = [0; 16];
        let count = self.keymap.key_get_mods_for_level(key, layout, level, &mut masks);
        let lock = 1 << self.keymap.mod_get_index(xkb::MOD_NAME_CAPS);
        let Some(mask) = masks[..count]
            .iter()
            .copied()
            .filter(|mask| mask & lock == 0)
            .min_by_key(|mask| mask.count_ones())
        else {
            return Vec::new();
        };
        let mut modifiers: Vec<Modifier> = (0..self.keymap.num_mods())
            .filter(|index| mask & (1 << index) != 0)
            .filter_map(|index| {
                let name = self.keymap.mod_get_name(index);
                Modifier::ALL.into_iter().find(|modifier| modifier.xkb_names().contains(&name))
            })
            .collect();
        modifiers.dedup();
        modifiers
    }
}

/// Reads the keymap the compositor sends to every client that asks for a
/// keyboard. No window is needed.
fn compositor_keymap() -> Result<String, String> {
    let connection = connect().map_err(|err| format!("could not connect to the compositor: {}", err))?;
    let mut queue = connection.new_event_queue();
    connection.display().get_registry(&queue.handle(), ());

    let mut state = KeymapState::default();
    // The first round trip finds the seat, the next ones bring its keyboard's keymap.
    for _ in 0..4 {
        queue.roundtrip(&mut state).map_err(|err| format!("talking to the compositor: {}", err))?;
        if let Some(result) = state.keymap.take() {
            return result;
        }
    }
    Err("the compositor didn't send a keymap (is there a keyboard?)".to_string())
}

/// Connects to $WAYLAND_DISPLAY, or wayland-0 when it isn't set, as can
/// happen in a service that started before the desktop.
fn connect() -> Result<Connection, String> {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        return Connection::connect_to_env().map_err(|err| err.to_string());
    }
    let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR").ok_or("XDG_RUNTIME_DIR is not set")?;
    let socket = PathBuf::from(runtime_dir).join("wayland-0");
    let stream = UnixStream::connect(&socket).map_err(|err| format!("{}: {}", socket.display(), err))?;
    Connection::from_socket(stream).map_err(|err| err.to_string())
}

#[derive(Default)]
struct KeymapState {
    keymap: Option<Result<String, String>>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for KeymapState {
    fn event(
        _: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        queue: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { name, interface, version } = event {
            if interface == "wl_seat" {
                let seat: wl_seat::WlSeat = registry.bind(name, version.min(5), queue, ());
                seat.get_keyboard(queue, ());
            }
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for KeymapState {
    fn event(_: &mut Self, _: &wl_seat::WlSeat, _: wl_seat::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for KeymapState {
    fn event(
        state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_keyboard::Event::Keymap { format, fd, size } = event {
            state.keymap = Some(match format {
                WEnum::Value(wl_keyboard::KeymapFormat::XkbV1) => {
                    let mut text = vec![0; size as usize];
                    std::fs::File::from(fd)
                        .read_exact(&mut text)
                        .map(|()| String::from_utf8_lossy(&text).trim_end_matches('\0').to_string())
                        .map_err(|err| format!("reading the compositor's keymap: {}", err))
                }
                _ => Err("the compositor's keymap is not in xkb format".to_string()),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout(names: &str) -> Layout {
        Layout::from_names(names, "", "", "test").unwrap()
    }

    fn find_char(layout: &Layout, ch: char) -> Option<(Vec<u32>, Vec<Modifier>)> {
        layout.find(xkb::utf32_to_keysym(ch as u32))
    }

    #[test]
    fn characters_are_found_on_the_key_that_types_them() {
        let us = layout("us");
        assert_eq!(find_char(&us, '&'), Some((vec![8], vec![Modifier::Shift]))); // Shift+7
        assert_eq!(find_char(&us, 'a'), Some((vec![30], vec![])));

        let fr = layout("fr");
        assert_eq!(find_char(&fr, '&'), Some((vec![2], vec![]))); // the 1 key
        assert_eq!(find_char(&fr, '_'), Some((vec![9], vec![]))); // the 8 key
        assert_eq!(find_char(&fr, 'a'), Some((vec![16], vec![]))); // where Q is on US
        assert_eq!(find_char(&fr, '8'), Some((vec![9], vec![Modifier::Shift])));
        assert_eq!(find_char(&fr, '@'), Some((vec![11], vec![Modifier::AltGr]))); // AltGr+0
    }

    #[test]
    fn main_keys_are_preferred_over_the_keypad() {
        // KEY_EQUAL with Shift, not KEY_KPPLUS
        assert_eq!(find_char(&layout("us"), '+'), Some((vec![13], vec![Modifier::Shift])));
    }

    #[test]
    fn later_layouts_are_searched_when_the_first_lacks_a_character() {
        assert_eq!(find_char(&layout("us"), 'å'), None);
        assert_eq!(find_char(&layout("us,se"), 'å').map(|(keys, _)| keys), Some(vec![26]));
    }

    #[test]
    fn layout_options_are_followed() {
        let swapped = Layout::from_names("us", "", "caps:swapescape", "test").unwrap();
        assert_eq!(swapped.find(Keysym::Escape).map(|(keys, _)| keys), Some(vec![58])); // KEY_CAPSLOCK
        let caps_ctrl = Layout::from_names("us", "", "caps:ctrl_modifier", "test").unwrap();
        assert_eq!(caps_ctrl.keys_for(Modifier::Ctrl), vec![29, 58, 97]); // Left Ctrl, Caps Lock, Right Ctrl
        assert!(caps_ctrl.modifier_keys().contains(&58));
    }

    #[test]
    fn modifiers_follow_the_layout() {
        let us = layout("us");
        assert_eq!(us.keys_for(Modifier::Alt), vec![56, 100]);
        assert_eq!(us.keys_for(Modifier::Super), vec![125, 126]);
        // On French the right Alt key is AltGr
        let fr = layout("fr");
        assert_eq!(fr.keys_for(Modifier::Alt), vec![56]);
        assert_eq!(fr.keys_for(Modifier::AltGr), vec![100]);
    }
}

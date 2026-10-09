use crate::layout::{Layout, Modifier};
use xkbcommon::xkb::{self, Keysym};

/// Linux key codes. Keys in bindings are looked up in the keyboard layout,
/// so these are only used for mouse buttons and scrolls, for modifiers a
/// layout doesn't have, and in tests.
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Keys {
    A = 0x1E,
    B = 0x30,
    C = 0x2E,
    D = 0x20,
    E = 0x12,
    F = 0x21,
    G = 0x22,
    H = 0x23,
    I = 0x17,
    J = 0x24,
    K = 0x25,
    L = 0x26,
    M = 0x32,
    N = 0x31,
    O = 0x18,
    P = 0x19,
    Q = 0x10,
    R = 0x13,
    S = 0x1F,
    T = 0x14,
    U = 0x16,
    V = 0x2F,
    W = 0x11,
    X = 0x2D,
    Y = 0x15,
    Z = 0x2C,
    BackTick = 0x29,
    Num0 = 0x0B,
    Num1 = 0x02,
    Num2 = 0x03,
    Num3 = 0x04,
    Num4 = 0x05,
    Num5 = 0x06,
    Num6 = 0x07,
    Num7 = 0x08,
    Num8 = 0x09,
    Num9 = 0x0A,
    Escape = 0x01,
    F1 = 0x3B,
    F2 = 0x3C,
    F3 = 0x3D,
    F4 = 0x3E,
    F5 = 0x3F,
    F6 = 0x40,
    F7 = 0x41,
    F8 = 0x42,
    F9 = 0x43,
    F10 = 0x44,
    F11 = 0x57,
    F12 = 0x58,
    LeftCtrl = 0x1D,
    LeftShift = 0x2A,
    LeftMod = 0x7D,
    LeftAlt = 0x38,
    RightCtrl = 0x61,
    RightShift = 0x36,
    RightMod = 0x7e,
    RightAlt = 0x64,
    Space = 0x39,
    Enter = 0x1C,
    Backspace = 0x0E,
    Tab = 0x0F,
    Up = 0x67,
    Down = 0x6C,
    Left = 0x69,
    Right = 0x6A,
    Insert = 0x6E,
    Delete = 0x6F,
    Home = 0x66,
    End = 0x6B,
    PageUp = 0x68,
    PageDown = 0x6D,
    Comma = 0x33,
    Period = 0x34,
    Slash = 0x35,
    LeftSquare = 0x1a,
    RightSquare = 0x1b,
    SemiColon = 0x27,
    Quote = 0x28,
    BackSlash = 0x2b,
    Dash = 0x0c,
    Equal = 0x0d,
    Mouse1 = 0x110,
    Mouse2 = 0x111,
    Mouse3 = 0x112,
    Mouse4 = 0x113,
    Mouse5 = 0x114,
    Mouse6 = 0x115,
    Mouse7 = 0x116,
    Mouse8 = 0x117,
    Mouse9 = 0x118,
    Mouse10 = 0x119,
    ScrollLeft = 0x996,
    ScrollRight = 0x997,
    ScrollUp = 0x998,
    ScrollDown = 0x999,
    }

/// One part of a binding as written, before it is looked up in a layout.
enum Name {
    /// ALT, CTRL, SHIFT, MOD or ALTGR: every key that acts as that modifier
    Modifier(Modifier),
    /// LEFTALT and friends: the key that types that keysym, or the usual
    /// key when the layout has none
    OneSided(Keysym, Keys),
    /// Any other key, by the keysym it types
    Keysym(Keysym, String),
    /// Mouse buttons and scrolls, which have no layout
    Pointer(Keys),
}

/// Parses a binding like "ALT+T" into the key codes that can satisfy each of
/// its parts, looking every key up in the keyboard layout, so a binding
/// means the keys the user sees. Characters that need a modifier add it,
/// e.g. "MOD+&" on a US keyboard is Super+Shift+7. `ALT`, `CTRL`, `SHIFT`,
/// `MOD` and `ALTGR` accept every key that acts as that modifier.
pub fn parse_binding(binding: &str, layout: &Layout) -> Result<Vec<Vec<u32>>, String> {
    let mut parts: Vec<Vec<u32>> = Vec::new();
    let mut implied: Vec<Modifier> = Vec::new();

    for token in binding.split('+') {
        let keys = match name(token)? {
            Name::Modifier(modifier) => modifier_keys(layout, modifier),
            Name::OneSided(keysym, usual) => match layout.find(keysym) {
                Some((keys, _)) => keys,
                None => vec![usual as u32],
            },
            Name::Keysym(keysym, shown) => {
                let (keys, modifiers) = layout.find(keysym).ok_or_else(|| {
                    format!("no key types {} in your keyboard layout ({})", shown, layout.names().join(", "))
                })?;
                implied.extend(modifiers);
                keys
            }
            Name::Pointer(key) => vec![key as u32],
        };
        parts.push(sorted(keys));
    }

    // Add the modifiers characters need, unless the binding already has a
    // key that acts as that modifier, e.g. SHIFT or LEFTSHIFT.
    for modifier in implied {
        let keys = modifier_keys(layout, modifier);
        if !parts.iter().any(|part| part.iter().all(|key| keys.contains(key))) {
            parts.push(keys);
        }
    }
    Ok(parts)
}

/// What a token in a binding names, case-insensitively.
fn name(token: &str) -> Result<Name, String> {
    let upper = token.to_uppercase();
    let keysym = |sym: Keysym| Ok(Name::Keysym(sym, token.to_string()));
    match upper.as_str() {
        "ALT" => Ok(Name::Modifier(Modifier::Alt)),
        "CTRL" => Ok(Name::Modifier(Modifier::Ctrl)),
        "SHIFT" => Ok(Name::Modifier(Modifier::Shift)),
        "MOD" => Ok(Name::Modifier(Modifier::Super)),
        "ALTGR" => Ok(Name::Modifier(Modifier::AltGr)),
        "LEFTALT" => Ok(Name::OneSided(Keysym::Alt_L, Keys::LeftAlt)),
        "RIGHTALT" => Ok(Name::OneSided(Keysym::Alt_R, Keys::RightAlt)),
        "LEFTCTRL" => Ok(Name::OneSided(Keysym::Control_L, Keys::LeftCtrl)),
        "RIGHTCTRL" => Ok(Name::OneSided(Keysym::Control_R, Keys::RightCtrl)),
        "LEFTSHIFT" => Ok(Name::OneSided(Keysym::Shift_L, Keys::LeftShift)),
        "RIGHTSHIFT" => Ok(Name::OneSided(Keysym::Shift_R, Keys::RightShift)),
        "LEFTMOD" => Ok(Name::OneSided(Keysym::Super_L, Keys::LeftMod)),
        "RIGHTMOD" => Ok(Name::OneSided(Keysym::Super_R, Keys::RightMod)),
        "ESCAPE" => keysym(Keysym::Escape),
        "ENTER" => keysym(Keysym::Return),
        "BACKSPACE" => keysym(Keysym::BackSpace),
        "TAB" => keysym(Keysym::Tab),
        "SPACE" => keysym(Keysym::space),
        "UP" => keysym(Keysym::Up),
        "DOWN" => keysym(Keysym::Down),
        "LEFT" => keysym(Keysym::Left),
        "RIGHT" => keysym(Keysym::Right),
        "HOME" => keysym(Keysym::Home),
        "END" => keysym(Keysym::End),
        "PAGEUP" => keysym(Keysym::Prior),
        "PAGEDOWN" => keysym(Keysym::Next),
        "INSERT" => keysym(Keysym::Insert),
        "DELETE" => keysym(Keysym::Delete),
        "BACKTICK" => keysym(Keysym::grave),
        "COMMA" => keysym(Keysym::comma),
        "PERIOD" => keysym(Keysym::period),
        "SLASH" => keysym(Keysym::slash),
        "BACKSLASH" => keysym(Keysym::backslash),
        "SEMICOLON" => keysym(Keysym::semicolon),
        "QUOTE" => keysym(Keysym::apostrophe),
        "DASH" => keysym(Keysym::minus),
        "EQUAL" => keysym(Keysym::equal),
        "LEFTSQUARE" => keysym(Keysym::bracketleft),
        "RIGHTSQUARE" => keysym(Keysym::bracketright),
        // The separator can't be written on its own
        "PLUS" => keysym(Keysym::plus),
        "MOUSE1" => Ok(Name::Pointer(Keys::Mouse1)),
        "MOUSE2" => Ok(Name::Pointer(Keys::Mouse2)),
        "MOUSE3" => Ok(Name::Pointer(Keys::Mouse3)),
        "MOUSE4" => Ok(Name::Pointer(Keys::Mouse4)),
        "MOUSE5" => Ok(Name::Pointer(Keys::Mouse5)),
        "MOUSE6" => Ok(Name::Pointer(Keys::Mouse6)),
        "MOUSE7" => Ok(Name::Pointer(Keys::Mouse7)),
        "MOUSE8" => Ok(Name::Pointer(Keys::Mouse8)),
        "MOUSE9" => Ok(Name::Pointer(Keys::Mouse9)),
        "MOUSE10" => Ok(Name::Pointer(Keys::Mouse10)),
        "SCROLLUP" => Ok(Name::Pointer(Keys::ScrollUp)),
        "SCROLLDOWN" => Ok(Name::Pointer(Keys::ScrollDown)),
        "SCROLLLEFT" => Ok(Name::Pointer(Keys::ScrollLeft)),
        "SCROLLRIGHT" => Ok(Name::Pointer(Keys::ScrollRight)),
        _ => {
            // A single character, by the key labelled with it: letters in
            // either case mean the same key.
            let mut chars = token.chars();
            if let (Some(ch), None) = (chars.next(), chars.next()) {
                let lower = ch.to_lowercase().next().unwrap_or(ch);
                return keysym(xkb::utf32_to_keysym(lower as u32));
            }
            // Any other xkb keysym name, e.g. F13 or XF86AudioMute
            match xkb::keysym_from_name(token, xkb::KEYSYM_CASE_INSENSITIVE) {
                sym if sym.raw() != 0 => keysym(sym),
                _ if token.is_empty() => Err("empty key name (use PLUS for the + key)".to_string()),
                _ => Err(format!("unknown key name {:?}", token)),
            }
        }
    }
}

/// The keys that act as a modifier, or the usual ones if the layout has none.
fn modifier_keys(layout: &Layout, modifier: Modifier) -> Vec<u32> {
    let keys = layout.keys_for(modifier);
    if !keys.is_empty() {
        return keys;
    }
    let usual = match modifier {
        Modifier::Alt => vec![Keys::LeftAlt, Keys::RightAlt],
        Modifier::Ctrl => vec![Keys::LeftCtrl, Keys::RightCtrl],
        Modifier::Shift => vec![Keys::LeftShift, Keys::RightShift],
        Modifier::Super => vec![Keys::LeftMod, Keys::RightMod],
        Modifier::AltGr => vec![Keys::RightAlt],
    };
    usual.into_iter().map(|key| key as u32).collect()
}

fn sorted(mut keys: Vec<u32>) -> Vec<u32> {
    keys.sort_unstable();
    keys.dedup();
    keys
}

/// Whether a pressed combo (held keys and the key just pressed) satisfies a
/// parsed binding. Each part needs its own key, in any order.
pub fn matches(binding: &[Vec<u32>], combo: &[u32]) -> bool {
    fn assign(parts: &[Vec<u32>], keys: &mut Vec<u32>) -> bool {
        let Some((part, rest)) = parts.split_first() else {
            return keys.is_empty();
        };
        for i in 0..keys.len() {
            if part.contains(&keys[i]) {
                let key = keys.remove(i);
                if assign(rest, keys) {
                    return true;
                }
                keys.insert(i, key);
            }
        }
        false
    }
    binding.len() == combo.len() && assign(binding, &mut combo.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn us() -> Layout {
        Layout::from_names("us", "", "", "test").unwrap()
    }

    fn parse(binding: &str, layout: &Layout) -> Vec<Vec<u32>> {
        parse_binding(binding, layout).unwrap()
    }

    #[test]
    fn named_keys_on_a_us_keyboard() {
        let test_cases = [
            ("LeftShift", Keys::LeftShift),
            ("LeftCtrl", Keys::LeftCtrl),
            ("LeftMod", Keys::LeftMod),
            ("LeftAlt", Keys::LeftAlt),
            ("RightShift", Keys::RightShift),
            ("RightCtrl", Keys::RightCtrl),
            ("RightMod", Keys::RightMod),
            ("RightAlt", Keys::RightAlt),
            ("Space", Keys::Space),
            ("Enter", Keys::Enter),
            ("Backspace", Keys::Backspace),
            ("Tab", Keys::Tab),
            ("Escape", Keys::Escape),
            ("Up", Keys::Up),
            ("Down", Keys::Down),
            ("Left", Keys::Left),
            ("Right", Keys::Right),
            ("Insert", Keys::Insert),
            ("Delete", Keys::Delete),
            ("Home", Keys::Home),
            ("End", Keys::End),
            ("PageUp", Keys::PageUp),
            ("PageDown", Keys::PageDown),
            ("Comma", Keys::Comma),
            ("Period", Keys::Period),
            ("Slash", Keys::Slash),
            ("LeftSquare", Keys::LeftSquare),
            ("RightSquare", Keys::RightSquare),
            ("SemiColon", Keys::SemiColon),
            ("Quote", Keys::Quote),
            ("BackSlash", Keys::BackSlash),
            ("Dash", Keys::Dash),
            ("Equal", Keys::Equal),
            ("BackTick", Keys::BackTick),
            ("A", Keys::A),
            ("q", Keys::Q),
            ("Z", Keys::Z),
            ("0", Keys::Num0),
            ("1", Keys::Num1),
            ("9", Keys::Num9),
            ("F1", Keys::F1),
            ("F12", Keys::F12),
            ("Mouse1", Keys::Mouse1),
            ("Mouse10", Keys::Mouse10),
            ("ScrollUp", Keys::ScrollUp),
        ];

        let us = us();
        for (input, expected) in test_cases {
            assert_eq!(parse(input, &us), vec![vec![expected as u32]], "Failed for input: {}", input);
        }
    }

    #[test]
    fn keys_follow_the_layout() {
        let fr = Layout::from_names("fr", "", "", "test").unwrap();
        let super_keys = vec![Keys::LeftMod as u32, Keys::RightMod as u32];
        let shift_keys = vec![Keys::LeftShift as u32, Keys::RightShift as u32];
        // The keys labelled & and _ on AZERTY
        assert_eq!(parse("MOD+&", &fr), vec![super_keys.clone(), vec![Keys::Num1 as u32]]);
        assert_eq!(parse("MOD+_", &fr), vec![super_keys.clone(), vec![Keys::Num8 as u32]]);
        // A is where Q is on QWERTY, and digits need Shift
        assert_eq!(parse("ALT+A", &fr), vec![vec![Keys::LeftAlt as u32], vec![Keys::Q as u32]]);
        assert_eq!(parse("MOD+8", &fr), vec![super_keys.clone(), vec![Keys::Num8 as u32], shift_keys]);
        // @ is AltGr+0
        assert_eq!(
            parse("MOD+@", &fr),
            vec![super_keys, vec![Keys::Num0 as u32], vec![Keys::RightAlt as u32]]
        );
    }

    #[test]
    fn characters_add_the_modifiers_they_need() {
        let us = us();
        let super_keys = vec![Keys::LeftMod as u32, Keys::RightMod as u32];
        let shift_keys = vec![Keys::LeftShift as u32, Keys::RightShift as u32];
        assert_eq!(parse("MOD+&", &us), vec![super_keys.clone(), vec![Keys::Num7 as u32], shift_keys.clone()]);
        assert_eq!(parse("MOD+PLUS", &us), vec![super_keys.clone(), vec![Keys::Equal as u32], shift_keys.clone()]);
        // Writing the modifier as well doesn't add it twice
        assert_eq!(parse("MOD+SHIFT+_", &us), vec![super_keys.clone(), shift_keys, vec![Keys::Dash as u32]]);
        assert_eq!(
            parse("MOD+LEFTSHIFT+_", &us),
            vec![super_keys, vec![Keys::LeftShift as u32], vec![Keys::Dash as u32]]
        );
    }

    #[test]
    fn keysym_names_can_be_used() {
        assert_eq!(parse("XF86AudioMute", &us()), vec![vec![113]]); // KEY_MUTE
        assert_eq!(parse("ALT+XF86AudioRaiseVolume", &us()), vec![vec![56, 100], vec![115]]); // KEY_VOLUMEUP
    }

    #[test]
    fn unknown_names_are_an_error() {
        let us = us();
        assert_eq!(parse_binding("ALT+NOPE", &us), Err("unknown key name \"NOPE\"".to_string()));
        assert!(parse_binding("MOD+ж", &us).unwrap_err().starts_with("no key types ж in your keyboard layout"));
        assert!(parse_binding("MOD++", &us).unwrap_err().contains("use PLUS"));
    }

    #[test]
    fn generic_modifiers_match_either_side() {
        let us = us();
        let test_cases = [
            ("ALT+T", Keys::LeftAlt, Keys::RightAlt),
            ("CTRL+T", Keys::LeftCtrl, Keys::RightCtrl),
            ("SHIFT+T", Keys::LeftShift, Keys::RightShift),
            ("MOD+T", Keys::LeftMod, Keys::RightMod),
        ];

        for (input, left, right) in test_cases {
            let binding = parse(input, &us);
            assert!(matches(&binding, &[left as u32, Keys::T as u32]), "left side failed for {}", input);
            assert!(matches(&binding, &[right as u32, Keys::T as u32]), "right side failed for {}", input);
        }
    }

    #[test]
    fn sided_modifiers_match_one_side() {
        let us = us();
        let test_cases = [
            ("LEFTALT+T", Keys::LeftAlt, Keys::RightAlt),
            ("RIGHTALT+T", Keys::RightAlt, Keys::LeftAlt),
            ("LEFTCTRL+T", Keys::LeftCtrl, Keys::RightCtrl),
            ("RIGHTCTRL+T", Keys::RightCtrl, Keys::LeftCtrl),
            ("LEFTSHIFT+T", Keys::LeftShift, Keys::RightShift),
            ("RIGHTSHIFT+T", Keys::RightShift, Keys::LeftShift),
            ("LEFTMOD+T", Keys::LeftMod, Keys::RightMod),
            ("RIGHTMOD+T", Keys::RightMod, Keys::LeftMod),
        ];

        for (input, side, other) in test_cases {
            let binding = parse(input, &us);
            assert!(matches(&binding, &[side as u32, Keys::T as u32]), "{} should match its own side", input);
            assert!(!matches(&binding, &[other as u32, Keys::T as u32]), "{} should not match the other side", input);
        }
    }

    #[test]
    fn modifiers_can_be_pressed_in_any_order() {
        let binding = parse("CTRL+ALT+T", &us());
        let (ctrl, alt, t) = (Keys::LeftCtrl as u32, Keys::RightAlt as u32, Keys::T as u32);
        assert!(matches(&binding, &[ctrl, alt, t]));
        assert!(matches(&binding, &[alt, ctrl, t]));
        assert!(!matches(&binding, &[ctrl, t]), "missing a modifier");
        assert!(!matches(&binding, &[ctrl, alt, Keys::LeftShift as u32, t]), "extra modifier");
        assert!(!matches(&binding, &[ctrl, ctrl, t]), "each part needs its own key");
    }
}

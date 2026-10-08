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

/// Parses a binding like "ALT+T" into the key codes that can satisfy each of
/// its parts, in order. `ALT`, `CTRL`, `SHIFT` and `MOD` accept either the
/// left- or right-hand key; every other name accepts exactly one key.
pub fn parse_binding(binding: &str) -> Vec<Vec<u32>> {
    let strings: Vec<String> = binding.split('+').map(|s| s.to_string()).collect();

    let mut keys = Vec::new();
    for string in strings {
        match string.to_uppercase().as_str() {
            "A" => keys.push(vec![Keys::A as u32]),
            "B" => keys.push(vec![Keys::B as u32]),
            "C" => keys.push(vec![Keys::C as u32]),
            "D" => keys.push(vec![Keys::D as u32]),
            "E" => keys.push(vec![Keys::E as u32]),
            "F" => keys.push(vec![Keys::F as u32]),
            "G" => keys.push(vec![Keys::G as u32]),
            "H" => keys.push(vec![Keys::H as u32]),
            "I" => keys.push(vec![Keys::I as u32]),
            "J" => keys.push(vec![Keys::J as u32]),
            "K" => keys.push(vec![Keys::K as u32]),
            "L" => keys.push(vec![Keys::L as u32]),
            "M" => keys.push(vec![Keys::M as u32]),
            "N" => keys.push(vec![Keys::N as u32]),
            "O" => keys.push(vec![Keys::O as u32]),
            "P" => keys.push(vec![Keys::P as u32]),
            "Q" => keys.push(vec![Keys::Q as u32]),
            "R" => keys.push(vec![Keys::R as u32]),
            "S" => keys.push(vec![Keys::S as u32]),
            "T" => keys.push(vec![Keys::T as u32]),
            "U" => keys.push(vec![Keys::U as u32]),
            "V" => keys.push(vec![Keys::V as u32]),
            "W" => keys.push(vec![Keys::W as u32]),
            "X" => keys.push(vec![Keys::X as u32]),
            "Y" => keys.push(vec![Keys::Y as u32]),
            "Z" => keys.push(vec![Keys::Z as u32]),
            "0" => keys.push(vec![Keys::Num0 as u32]),
            "1" => keys.push(vec![Keys::Num1 as u32]),
            "2" => keys.push(vec![Keys::Num2 as u32]),
            "3" => keys.push(vec![Keys::Num3 as u32]),
            "4" => keys.push(vec![Keys::Num4 as u32]),
            "5" => keys.push(vec![Keys::Num5 as u32]),
            "6" => keys.push(vec![Keys::Num6 as u32]),
            "7" => keys.push(vec![Keys::Num7 as u32]),
            "8" => keys.push(vec![Keys::Num8 as u32]),
            "9" => keys.push(vec![Keys::Num9 as u32]),
            "BACKTICK" => keys.push(vec![Keys::BackTick as u32]),
            "ESCAPE" => keys.push(vec![Keys::Escape as u32]),
            "F1" => keys.push(vec![Keys::F1 as u32]),
            "F2" => keys.push(vec![Keys::F2 as u32]),
            "F3" => keys.push(vec![Keys::F3 as u32]),
            "F4" => keys.push(vec![Keys::F4 as u32]),
            "F5" => keys.push(vec![Keys::F5 as u32]),
            "F6" => keys.push(vec![Keys::F6 as u32]),
            "F7" => keys.push(vec![Keys::F7 as u32]),
            "F8" => keys.push(vec![Keys::F8 as u32]),
            "F9" => keys.push(vec![Keys::F9 as u32]),
            "F10" => keys.push(vec![Keys::F10 as u32]),
            "F11" => keys.push(vec![Keys::F11 as u32]),
            "F12" => keys.push(vec![Keys::F12 as u32]),
            "LEFTALT" => keys.push(vec![Keys::LeftAlt as u32]),
            "LEFTCTRL" => keys.push(vec![Keys::LeftCtrl as u32]),
            "LEFTSHIFT" => keys.push(vec![Keys::LeftShift as u32]),
            "CTRL" => keys.push(vec![Keys::LeftCtrl as u32, Keys::RightCtrl as u32]),
            "LEFTMOD" => keys.push(vec![Keys::LeftMod as u32]),
            "ALT" => keys.push(vec![Keys::LeftAlt as u32, Keys::RightAlt as u32]),
            "SHIFT" => keys.push(vec![Keys::LeftShift as u32, Keys::RightShift as u32]),
            "MOD" => keys.push(vec![Keys::LeftMod as u32, Keys::RightMod as u32]),
            "SPACE" => keys.push(vec![Keys::Space as u32]),
            "ENTER" => keys.push(vec![Keys::Enter as u32]),
            "BACKSPACE" => keys.push(vec![Keys::Backspace as u32]),
            "TAB" => keys.push(vec![Keys::Tab as u32]),
            "UP" => keys.push(vec![Keys::Up as u32]),
            "DOWN" => keys.push(vec![Keys::Down as u32]),
            "LEFT" => keys.push(vec![Keys::Left as u32]),
            "RIGHT" => keys.push(vec![Keys::Right as u32]),
            "INSERT" => keys.push(vec![Keys::Insert as u32]),
            "DELETE" => keys.push(vec![Keys::Delete as u32]),
            "HOME" => keys.push(vec![Keys::Home as u32]),
            "END" => keys.push(vec![Keys::End as u32]),
            "PAGEUP" => keys.push(vec![Keys::PageUp as u32]),
            "PAGEDOWN" => keys.push(vec![Keys::PageDown as u32]),
            "COMMA"=> keys.push(vec![Keys::Comma as u32]),
            "PERIOD"=> keys.push(vec![Keys::Period as u32]),
            "SLASH"=> keys.push(vec![Keys::Slash as u32]),
            "RIGHTCTRL"=> keys.push(vec![Keys::RightCtrl as u32]),
            "RIGHTALT"=> keys.push(vec![Keys::RightAlt as u32]),
            "RIGHTSHIFT"=> keys.push(vec![Keys::RightShift as u32]),
            "RIGHTMOD"=> keys.push(vec![Keys::RightMod as u32]),
            "LEFTSQUARE"=> keys.push(vec![Keys::LeftSquare as u32]),
            "RIGHTSQUARE"=> keys.push(vec![Keys::RightSquare as u32]),
            "SEMICOLON"=> keys.push(vec![Keys::SemiColon as u32]),
            "QUOTE"=> keys.push(vec![Keys::Quote as u32]),
            "BACKSLASH"=> keys.push(vec![Keys::BackSlash as u32]),
            "DASH"=> keys.push(vec![Keys::Dash as u32]),
            "EQUAL"=> keys.push(vec![Keys::Equal as u32]),
            "MOUSE1"=> keys.push(vec![Keys::Mouse1 as u32]),
            "MOUSE2"=> keys.push(vec![Keys::Mouse2 as u32]),
            "MOUSE3"=> keys.push(vec![Keys::Mouse3 as u32]),
            "MOUSE4"=> keys.push(vec![Keys::Mouse4 as u32]),
            "MOUSE5"=> keys.push(vec![Keys::Mouse5 as u32]),
            "MOUSE6"=> keys.push(vec![Keys::Mouse6 as u32]),
            "MOUSE7"=> keys.push(vec![Keys::Mouse7 as u32]),
            "MOUSE8"=> keys.push(vec![Keys::Mouse8 as u32]),
            "MOUSE9"=> keys.push(vec![Keys::Mouse9 as u32]),
            "MOUSE10"=> keys.push(vec![Keys::Mouse10 as u32]),
            "SCROLLLEFT"=> keys.push(vec![Keys::ScrollLeft as u32]),
            "SCROLLRIGHT"=> keys.push(vec![Keys::ScrollRight as u32]),
            "SCROLLUP"=> keys.push(vec![Keys::ScrollUp as u32]),
            "SCROLLDOWN"=> keys.push(vec![Keys::ScrollDown as u32]),
            _ => debug!("Unknown key {:?} in binding {:?}, ignoring", string, binding),
        }
    }
    keys
}

/// Whether a pressed combo (key codes in the order they were pressed)
/// satisfies a parsed binding.
pub fn matches(binding: &[Vec<u32>], combo: &[u32]) -> bool {
    binding.len() == combo.len()
        && binding.iter().zip(combo).all(|(choices, key)| choices.contains(key))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_single_key() {
        let test_cases = [
            ("LeftShift", Keys::LeftShift as u32),
            ("LeftCtrl", Keys::LeftCtrl as u32),
            ("LeftMod", Keys::LeftMod as u32),
            ("LeftAlt", Keys::LeftAlt as u32),
            ("RightShift", Keys::RightShift as u32),
            ("RightCtrl", Keys::RightCtrl as u32),
            ("RightMod", Keys::RightMod as u32),
            ("RightAlt", Keys::RightAlt as u32),
            ("Space", Keys::Space as u32),
            ("Enter", Keys::Enter as u32),
            ("Backspace", Keys::Backspace as u32),
            ("Tab", Keys::Tab as u32),
            ("Up", Keys::Up as u32),
            ("Down", Keys::Down as u32),
            ("Left", Keys::Left as u32),
            ("Right", Keys::Right as u32),
            ("Insert", Keys::Insert as u32),
            ("Delete", Keys::Delete as u32),
            ("Home", Keys::Home as u32),
            ("End", Keys::End as u32),
            ("PageUp", Keys::PageUp as u32),
            ("PageDown", Keys::PageDown as u32),
            ("Comma", Keys::Comma as u32),
            ("Period", Keys::Period as u32),
            ("Slash", Keys::Slash as u32),
            ("LeftSquare", Keys::LeftSquare as u32),
            ("RightSquare", Keys::RightSquare as u32),
            ("SemiColon", Keys::SemiColon as u32),
            ("Quote", Keys::Quote as u32),
            ("BackSlash", Keys::BackSlash as u32),
            ("Dash", Keys::Dash as u32),
            ("Equal", Keys::Equal as u32),
            ("A", Keys::A as u32),
            ("B", Keys::B as u32),
            ("C", Keys::C as u32),
            ("D", Keys::D as u32),
            ("E", Keys::E as u32),
            ("F", Keys::F as u32),
            ("G", Keys::G as u32),
            ("H", Keys::H as u32),
            ("I", Keys::I as u32),
            ("J", Keys::J as u32),
            ("K", Keys::K as u32),
            ("L", Keys::L as u32),
            ("M", Keys::M as u32),
            ("N", Keys::N as u32),
            ("O", Keys::O as u32),
            ("P", Keys::P as u32),
            ("Q", Keys::Q as u32),
            ("R", Keys::R as u32),
            ("S", Keys::S as u32),
            ("T", Keys::T as u32),
            ("U", Keys::U as u32),
            ("V", Keys::V as u32),
            ("W", Keys::W as u32),
            ("X", Keys::X as u32),
            ("Y", Keys::Y as u32),
            ("Z", Keys::Z as u32),
            ("BackTick", Keys::BackTick as u32),
            ("0", Keys::Num0 as u32),
            ("1", Keys::Num1 as u32),
            ("2", Keys::Num2 as u32),
            ("3", Keys::Num3 as u32),
            ("4", Keys::Num4 as u32),
            ("5", Keys::Num5 as u32),
            ("6", Keys::Num6 as u32),
            ("7", Keys::Num7 as u32),
            ("8", Keys::Num8 as u32),
            ("9", Keys::Num9 as u32),
            ("F1", Keys::F1 as u32),
            ("F2", Keys::F2 as u32),
            ("F3", Keys::F3 as u32),
            ("F4", Keys::F4 as u32),
            ("F5", Keys::F5 as u32),
            ("F6", Keys::F6 as u32),
            ("F7", Keys::F7 as u32),
            ("F8", Keys::F8 as u32),
            ("F9", Keys::F9 as u32),
            ("F10", Keys::F10 as u32),
            ("F11", Keys::F11 as u32),
            ("F12", Keys::F12 as u32),
            ("F12", Keys::F12 as u32),
        ];

        for (input, expected_output) in test_cases {
            assert_eq!(parse_binding(input), vec![vec![expected_output]], "Failed for input: {}", input);
        }
    }

    // Characters like `&` and `_` are bound by the physical key that types them.
    // Expected values are the raw evdev codes from linux/input-event-codes.h.
    #[test]
    fn test_parse_layout_character_bindings() {
        const KEY_1: u32 = 2;
        const KEY_7: u32 = 8;
        const KEY_8: u32 = 9;
        const KEY_MINUS: u32 = 12;
        const KEY_LEFTSHIFT: u32 = 42;
        const KEY_LEFTMETA: u32 = 125;

        let test_cases = [
            // French AZERTY: & and _ are unshifted on the 1 and 8 keys
            ("MOD+1", vec![KEY_LEFTMETA, KEY_1]),
            ("MOD+8", vec![KEY_LEFTMETA, KEY_8]),
            // US QWERTY: & is Shift+7, _ is Shift+-
            ("MOD+SHIFT+7", vec![KEY_LEFTMETA, KEY_LEFTSHIFT, KEY_7]),
            ("MOD+SHIFT+DASH", vec![KEY_LEFTMETA, KEY_LEFTSHIFT, KEY_MINUS]),
        ];

        for (input, pressed) in test_cases {
            assert!(matches(&parse_binding(input), &pressed), "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_generic_modifiers_match_either_side() {
        let test_cases = [
            ("ALT+T", Keys::LeftAlt, Keys::RightAlt),
            ("CTRL+T", Keys::LeftCtrl, Keys::RightCtrl),
            ("SHIFT+T", Keys::LeftShift, Keys::RightShift),
            ("MOD+T", Keys::LeftMod, Keys::RightMod),
        ];

        for (input, left, right) in test_cases {
            let binding = parse_binding(input);
            assert!(matches(&binding, &[left as u32, Keys::T as u32]), "left side failed for {}", input);
            assert!(matches(&binding, &[right as u32, Keys::T as u32]), "right side failed for {}", input);
        }
    }

    #[test]
    fn test_sided_modifiers_match_one_side() {
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
            let binding = parse_binding(input);
            assert!(matches(&binding, &[side as u32, Keys::T as u32]), "{} should match its own side", input);
            assert!(!matches(&binding, &[other as u32, Keys::T as u32]), "{} should not match the other side", input);
        }
    }

    #[test]
    fn test_matches_needs_the_whole_combo_in_order() {
        let binding = parse_binding("CTRL+ALT+T");
        let (ctrl, alt, t) = (Keys::LeftCtrl as u32, Keys::RightAlt as u32, Keys::T as u32);
        assert!(matches(&binding, &[ctrl, alt, t]));
        assert!(!matches(&binding, &[alt, ctrl, t]), "modifiers pressed out of order");
        assert!(!matches(&binding, &[ctrl, t]), "missing a modifier");
        assert!(!matches(&binding, &[ctrl, alt, Keys::LeftShift as u32, t]), "extra modifier");
    }
}

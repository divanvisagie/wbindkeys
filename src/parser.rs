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
    Space = 0x39,
    Enter = 0x1C,
    Backspace = 0x0E,
    Tab = 0x0F,
    Up = 0xC8,
    Down = 0xD0,
    Left = 0xCB,
    Right = 0xCD,
    Insert = 0xD2,
    Delete = 0xD3,
    Home = 0xC7,
    End = 0xCF,
    PageUp = 0xC9,
    PageDown = 0xD1,
}

pub fn parse_binding(binding: &str) -> Vec<u32> {
    let strings: Vec<String> = binding.split('+').map(|s| s.to_string()).collect();

    let mut keys = Vec::new();
    for string in strings {
        match string.as_str() {
            "A" => keys.push(Keys::A as u32),
            "B" => keys.push(Keys::B as u32),
            "C" => keys.push(Keys::C as u32),
            "D" => keys.push(Keys::D as u32),
            "E" => keys.push(Keys::E as u32),
            "F" => keys.push(Keys::F as u32),
            "G" => keys.push(Keys::G as u32),
            "H" => keys.push(Keys::H as u32),
            "I" => keys.push(Keys::I as u32),
            "J" => keys.push(Keys::J as u32),
            "K" => keys.push(Keys::K as u32),
            "L" => keys.push(Keys::L as u32),
            "M" => keys.push(Keys::M as u32),
            "N" => keys.push(Keys::N as u32),
            "O" => keys.push(Keys::O as u32),
            "P" => keys.push(Keys::P as u32),
            "Q" => keys.push(Keys::Q as u32),
            "R" => keys.push(Keys::R as u32),
            "S" => keys.push(Keys::S as u32),
            "T" => keys.push(Keys::T as u32),
            "U" => keys.push(Keys::U as u32),
            "V" => keys.push(Keys::V as u32),
            "W" => keys.push(Keys::W as u32),
            "X" => keys.push(Keys::X as u32),
            "Y" => keys.push(Keys::Y as u32),
            "Z" => keys.push(Keys::Z as u32),
            "0" => keys.push(Keys::Num0 as u32),
            "1" => keys.push(Keys::Num1 as u32),
            "2" => keys.push(Keys::Num2 as u32),
            "3" => keys.push(Keys::Num3 as u32),
            "4" => keys.push(Keys::Num4 as u32),
            "5" => keys.push(Keys::Num5 as u32),
            "6" => keys.push(Keys::Num6 as u32),
            "7" => keys.push(Keys::Num7 as u32),
            "8" => keys.push(Keys::Num8 as u32),
            "9" => keys.push(Keys::Num9 as u32),
            "Escape" => keys.push(Keys::Escape as u32),
            "F1" => keys.push(Keys::F1 as u32),
            "F2" => keys.push(Keys::F2 as u32),
            "F3" => keys.push(Keys::F3 as u32),
            "F4" => keys.push(Keys::F4 as u32),
            "F5" => keys.push(Keys::F5 as u32),
            "F6" => keys.push(Keys::F6 as u32),
            "F7" => keys.push(Keys::F7 as u32),
            "F8" => keys.push(Keys::F8 as u32),
            "F9" => keys.push(Keys::F9 as u32),
            "F10" => keys.push(Keys::F10 as u32),
            "F11" => keys.push(Keys::F11 as u32),
            "F12" => keys.push(Keys::F12 as u32),
            "LeftAlt" => keys.push(Keys::LeftAlt as u32),
            "LeftCtrl" => keys.push(Keys::LeftCtrl as u32),
            "LeftShift" => keys.push(Keys::LeftShift as u32),
            "Ctrl" => keys.push(Keys::LeftCtrl as u32), // Assuming "Ctrl" refers to "LeftCtrl"
            "LeftMod" => keys.push(Keys::LeftMod as u32),
            "Alt" => keys.push(Keys::LeftAlt as u32), // Assuming "Alt" refers to "LeftAlt"
            "Space" => keys.push(Keys::Space as u32),
            "Enter" => keys.push(Keys::Enter as u32),
            "Backspace" => keys.push(Keys::Backspace as u32),
            "Tab" => keys.push(Keys::Tab as u32),
            "Up" => keys.push(Keys::Up as u32),
            "Down" => keys.push(Keys::Down as u32),
            "Left" => keys.push(Keys::Left as u32),
            "Right" => keys.push(Keys::Right as u32),
            "Insert" => keys.push(Keys::Insert as u32),
            "Delete" => keys.push(Keys::Delete as u32),
            "Home" => keys.push(Keys::Home as u32),
            "End" => keys.push(Keys::End as u32),
            "PageUp" => keys.push(Keys::PageUp as u32),
            "PageDown" => keys.push(Keys::PageDown as u32),
            _ => {}
        }
    }
    keys
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_binding() {
        let binding = "Ctrl+Alt+Delete";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::LeftCtrl as u32, Keys::LeftAlt as u32, Keys::Delete as u32]);
    }
    #[test]
    fn test_parse_binding_a() {
        let binding: &str = "A";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::A as u32]);
        }
    #[test]
    fn test_parse_binding_b() {
        let binding: &str = "B";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::B as u32]);
        }
    #[test]
    fn test_parse_binding_c() {
        let binding: &str = "C";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::C as u32]);
        }
    #[test]
    fn test_parse_binding_d() {
        let binding: &str = "D";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::D as u32]);
        }
    #[test]
    fn test_parse_binding_e() {
        let binding: &str = "E";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::E as u32]);
        }
    #[test]
    fn test_parse_binding_f() {
        let binding: &str = "F";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::F as u32]);
        }
    #[test]
    fn test_parse_binding_g() {
        let binding: &str = "G";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::G as u32]);
        }
    #[test]
    fn test_parse_binding_h() {
        let binding: &str = "H";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::H as u32]);
        }
    #[test]
    fn test_parse_binding_i() {
        let binding: &str = "I";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::I as u32]);
        }
    #[test]
    fn test_parse_binding_j() {
        let binding: &str = "J";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::J as u32]);
        }
    #[test]
    fn test_parse_binding_k() {
        let binding: &str = "K";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::K as u32]);
        }
    #[test]
    fn test_parse_binding_l() {
        let binding: &str = "L";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::L as u32]);
        }
    #[test]
    fn test_parse_binding_m() {
        let binding: &str = "M";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::M as u32]);
        }
    #[test]
    fn test_parse_binding_n() {
        let binding: &str = "N";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::N as u32]);
        }
    #[test]
    fn test_parse_binding_o() {
        let binding: &str = "O";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::O as u32]);
        }
    #[test]
    fn test_parse_binding_p() {
        let binding: &str = "P";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::P as u32]);
        }
    #[test]
    fn test_parse_binding_q() {
        let binding: &str = "Q";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Q as u32]);
        }
    #[test]
    fn test_parse_binding_r() {
        let binding: &str = "R";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::R as u32]);
        }
    #[test]
    fn test_parse_binding_s() {
        let binding: &str = "S";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::S as u32]);
        }
    #[test]
    fn test_parse_binding_t() {
        let binding: &str = "T";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::T as u32]);
        }
    #[test]
    fn test_parse_binding_u() {
        let binding: &str = "U";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::U as u32]);
        }
    #[test]
    fn test_parse_binding_v() {
        let binding: &str = "V";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::V as u32]);
        }
    #[test]
    fn test_parse_binding_w() {
        let binding: &str = "W";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::W as u32]);
        }
    #[test]
    fn test_parse_binding_x() {
        let binding: &str = "X";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::X as u32]);
        }
    #[test]
    fn test_parse_binding_y() {
        let binding: &str = "Y";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Y as u32]);
        }
    #[test]
    fn test_parse_binding_z() {
        let binding: &str = "Z";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Z as u32]);
        }
    #[test]
    fn test_parse_binding_0() {
        let binding: &str = "0";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Num0 as u32]);
        }
    #[test]
    fn test_parse_binding_1() {
        let binding: &str = "1";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Num1 as u32]);
        }
    #[test]
    fn test_parse_binding_2() {
        let binding: &str = "2";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Num2 as u32]);
        }
    #[test]
    fn test_parse_binding_3() {
        let binding: &str = "3";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Num3 as u32]);
        }
    #[test]
    fn test_parse_binding_4() {
        let binding: &str = "4";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Num4 as u32]);
        }
    #[test]
    fn test_parse_binding_5() {
        let binding: &str = "5";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Num5 as u32]);
        }
    #[test]
    fn test_parse_binding_6() {
        let binding: &str = "6";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Num6 as u32]);
        }
    #[test]
    fn test_parse_binding_7() {
        let binding: &str = "7";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Num7 as u32]);
        }
    #[test]
    fn test_parse_binding_8() {
        let binding: &str = "8";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Num8 as u32]);
        }
    #[test]
    fn test_parse_binding_9() {
        let binding: &str = "9";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Num9 as u32]);
        }
    #[test]
    fn test_parse_binding_escape() {
        let binding: &str = "Escape";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Escape as u32]);
        }
    #[test]
    fn test_parse_binding_f1() {
        let binding: &str = "F1";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::F1 as u32]);
        }
    #[test]
    fn test_parse_binding_f2() {
        let binding: &str = "F2";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::F2 as u32]);
        }
    #[test]
    fn test_parse_binding_f3() {
        let binding: &str = "F3";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::F3 as u32]);
        }
    #[test]
    fn test_parse_binding_f4() {
        let binding: &str = "F4";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::F4 as u32]);
        }
    #[test]
    fn test_parse_binding_f5() {
        let binding: &str = "F5";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::F5 as u32]);
        }
    #[test]
    fn test_parse_binding_f6() {
        let binding: &str = "F6";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::F6 as u32]);
        }
    #[test]
    fn test_parse_binding_f7() {
        let binding: &str = "F7";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::F7 as u32]);
        }
    #[test]
    fn test_parse_binding_f8() {
        let binding: &str = "F8";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::F8 as u32]);
        }
    #[test]
    fn test_parse_binding_f9() {
        let binding: &str = "F9";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::F9 as u32]);
        }
    #[test]
    fn test_parse_binding_f10() {
        let binding: &str = "F10";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::F10 as u32]);
        }
    #[test]
    fn test_parse_binding_f11() {
        let binding: &str = "F11";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::F11 as u32]);
        }
    #[test]
    fn test_parse_binding_f12() {
        let binding: &str = "F12";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::F12 as u32]);
        }
    #[test]
    fn test_parse_binding_leftshift() {
        let binding: &str = "LeftShift";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::LeftShift as u32]);
        }
    #[test]
    fn test_parse_binding_leftctrl() {
        let binding: &str = "LeftCtrl";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::LeftCtrl as u32]);
        }
    #[test]
    fn test_parse_binding_leftmod() {
        let binding: &str = "LeftMod";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::LeftMod as u32]);
        }
    #[test]
    fn test_parse_binding_leftalt() {
        let binding: &str = "LeftAlt";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::LeftAlt as u32]);
        }
    #[test]
    fn test_parse_binding_space() {
        let binding: &str = "Space";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Space as u32]);
        }
    #[test]
    fn test_parse_binding_enter() {
        let binding: &str = "Enter";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Enter as u32]);
        }
    #[test]
    fn test_parse_binding_backspace() {
        let binding: &str = "Backspace";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Backspace as u32]);
        }
    #[test]
    fn test_parse_binding_tab() {
        let binding: &str = "Tab";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Tab as u32]);
        }
    #[test]
    fn test_parse_binding_up() {
        let binding: &str = "Up";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Up as u32]);
        }
    #[test]
    fn test_parse_binding_down() {
        let binding: &str = "Down";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Down as u32]);
        }
    #[test]
    fn test_parse_binding_left() {
        let binding: &str = "Left";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Left as u32]);
        }
    #[test]
    fn test_parse_binding_right() {
        let binding: &str = "Right";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Right as u32]);
        }
    #[test]
    fn test_parse_binding_insert() {
        let binding: &str = "Insert";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Insert as u32]);
        }
    #[test]
    fn test_parse_binding_delete() {
        let binding: &str = "Delete";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Delete as u32]);
        }
    #[test]
    fn test_parse_binding_home() {
        let binding: &str = "Home";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::Home as u32]);
        }
    #[test]
    fn test_parse_binding_end() {
        let binding: &str = "End";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::End as u32]);
        }
    #[test]
    fn test_parse_binding_pageup() {
        let binding: &str = "PageUp";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::PageUp as u32]);
        }
    #[test]
    fn test_parse_binding_pagedown() {
        let binding: &str = "PageDown";
        let keys = parse_binding(binding);
        assert_eq!(keys, vec![Keys::PageDown as u32]);
        }
    
}

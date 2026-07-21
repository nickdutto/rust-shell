use reedline::{KeyCode, KeyModifiers};

pub fn parse_key_modifier(key_modifier_str: &str) -> KeyModifiers {
    match key_modifier_str {
        "alt" => KeyModifiers::ALT,
        "control" => KeyModifiers::CONTROL,
        "hyper" => KeyModifiers::HYPER,
        "meta" => KeyModifiers::META,
        "shift" => KeyModifiers::SHIFT,
        "super" => KeyModifiers::SUPER,
        _ => KeyModifiers::NONE,
    }
}

pub fn parse_key_code(key_code_str: &str) -> KeyCode {
    match key_code_str {
        "escape" => KeyCode::Esc,
        "tab" => KeyCode::Tab,
        "backspace" => KeyCode::Backspace,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "backtab" => KeyCode::BackTab,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        code => code.chars().next().map_or(KeyCode::Null, KeyCode::Char),
    }
}

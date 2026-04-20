use evdev::{Device, KeyCode, RelativeAxisCode};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceKind {
    Keyboard,
    Mouse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Side {
    Left,
    Right,
}

pub fn classify(dev: &Device) -> Option<DeviceKind> {
    let keys = dev.supported_keys();
    let rels = dev.supported_relative_axes();

    let has_alpha = keys
        .map(|k| k.contains(KeyCode::KEY_A) || k.contains(KeyCode::KEY_Z))
        .unwrap_or(false);

    let has_mouse_motion = rels
        .map(|r| r.contains(RelativeAxisCode::REL_X) && r.contains(RelativeAxisCode::REL_Y))
        .unwrap_or(false);

    let has_mouse_button = keys
        .map(|k| k.contains(KeyCode::BTN_LEFT))
        .unwrap_or(false);

    if has_alpha {
        Some(DeviceKind::Keyboard)
    } else if has_mouse_motion || has_mouse_button {
        Some(DeviceKind::Mouse)
    } else {
        None
    }
}

/// Map a physical keycode/button to a hand side based on touch-typing conventions.
/// Returns `None` for unclassified keys (they don't move the animation but still count).
pub fn key_side(code: KeyCode, kind: DeviceKind) -> Option<Side> {
    match kind {
        DeviceKind::Mouse => match code {
            KeyCode::BTN_LEFT => Some(Side::Left),
            KeyCode::BTN_RIGHT => Some(Side::Right),
            _ => None,
        },
        DeviceKind::Keyboard => keyboard_side(code),
    }
}

fn keyboard_side(code: KeyCode) -> Option<Side> {
    use KeyCode as K;
    match code {
        K::KEY_GRAVE
        | K::KEY_1 | K::KEY_2 | K::KEY_3 | K::KEY_4 | K::KEY_5
        | K::KEY_TAB
        | K::KEY_Q | K::KEY_W | K::KEY_E | K::KEY_R | K::KEY_T
        | K::KEY_CAPSLOCK
        | K::KEY_A | K::KEY_S | K::KEY_D | K::KEY_F | K::KEY_G
        | K::KEY_LEFTSHIFT
        | K::KEY_Z | K::KEY_X | K::KEY_C | K::KEY_V | K::KEY_B
        | K::KEY_LEFTCTRL | K::KEY_LEFTMETA | K::KEY_LEFTALT
        | K::KEY_ESC
        | K::KEY_F1 | K::KEY_F2 | K::KEY_F3 | K::KEY_F4 | K::KEY_F5 | K::KEY_F6
        | K::KEY_102ND
        | K::KEY_SPACE => Some(Side::Left),

        K::KEY_6 | K::KEY_7 | K::KEY_8 | K::KEY_9 | K::KEY_0
        | K::KEY_MINUS | K::KEY_EQUAL | K::KEY_BACKSPACE
        | K::KEY_Y | K::KEY_U | K::KEY_I | K::KEY_O | K::KEY_P
        | K::KEY_LEFTBRACE | K::KEY_RIGHTBRACE | K::KEY_BACKSLASH
        | K::KEY_H | K::KEY_J | K::KEY_K | K::KEY_L
        | K::KEY_SEMICOLON | K::KEY_APOSTROPHE | K::KEY_ENTER
        | K::KEY_N | K::KEY_M
        | K::KEY_COMMA | K::KEY_DOT | K::KEY_SLASH | K::KEY_RIGHTSHIFT
        | K::KEY_RIGHTALT | K::KEY_RIGHTMETA | K::KEY_RIGHTCTRL
        | K::KEY_F7 | K::KEY_F8 | K::KEY_F9 | K::KEY_F10 | K::KEY_F11 | K::KEY_F12
        | K::KEY_UP | K::KEY_DOWN | K::KEY_LEFT | K::KEY_RIGHT
        | K::KEY_HOME | K::KEY_END | K::KEY_PAGEUP | K::KEY_PAGEDOWN
        | K::KEY_INSERT | K::KEY_DELETE
        | K::KEY_NUMLOCK
        | K::KEY_KP0 | K::KEY_KP1 | K::KEY_KP2 | K::KEY_KP3 | K::KEY_KP4
        | K::KEY_KP5 | K::KEY_KP6 | K::KEY_KP7 | K::KEY_KP8 | K::KEY_KP9
        | K::KEY_KPDOT | K::KEY_KPENTER | K::KEY_KPPLUS | K::KEY_KPMINUS
        | K::KEY_KPASTERISK | K::KEY_KPSLASH | K::KEY_KPEQUAL => Some(Side::Right),

        _ => None,
    }
}

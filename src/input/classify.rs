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

    let has_mouse_button = keys.map(|k| k.contains(KeyCode::BTN_LEFT)).unwrap_or(false);

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
        | K::KEY_1
        | K::KEY_2
        | K::KEY_3
        | K::KEY_4
        | K::KEY_5
        | K::KEY_TAB
        | K::KEY_Q
        | K::KEY_W
        | K::KEY_E
        | K::KEY_R
        | K::KEY_T
        | K::KEY_CAPSLOCK
        | K::KEY_A
        | K::KEY_S
        | K::KEY_D
        | K::KEY_F
        | K::KEY_G
        | K::KEY_LEFTSHIFT
        | K::KEY_Z
        | K::KEY_X
        | K::KEY_C
        | K::KEY_V
        | K::KEY_B
        | K::KEY_LEFTCTRL
        | K::KEY_LEFTMETA
        | K::KEY_LEFTALT
        | K::KEY_ESC
        | K::KEY_F1
        | K::KEY_F2
        | K::KEY_F3
        | K::KEY_F4
        | K::KEY_F5
        | K::KEY_F6
        | K::KEY_102ND
        | K::KEY_SPACE => Some(Side::Left),

        K::KEY_6
        | K::KEY_7
        | K::KEY_8
        | K::KEY_9
        | K::KEY_0
        | K::KEY_MINUS
        | K::KEY_EQUAL
        | K::KEY_BACKSPACE
        | K::KEY_Y
        | K::KEY_U
        | K::KEY_I
        | K::KEY_O
        | K::KEY_P
        | K::KEY_LEFTBRACE
        | K::KEY_RIGHTBRACE
        | K::KEY_BACKSLASH
        | K::KEY_H
        | K::KEY_J
        | K::KEY_K
        | K::KEY_L
        | K::KEY_SEMICOLON
        | K::KEY_APOSTROPHE
        | K::KEY_ENTER
        | K::KEY_N
        | K::KEY_M
        | K::KEY_COMMA
        | K::KEY_DOT
        | K::KEY_SLASH
        | K::KEY_RIGHTSHIFT
        | K::KEY_RIGHTALT
        | K::KEY_RIGHTMETA
        | K::KEY_RIGHTCTRL
        | K::KEY_F7
        | K::KEY_F8
        | K::KEY_F9
        | K::KEY_F10
        | K::KEY_F11
        | K::KEY_F12
        | K::KEY_UP
        | K::KEY_DOWN
        | K::KEY_LEFT
        | K::KEY_RIGHT
        | K::KEY_HOME
        | K::KEY_END
        | K::KEY_PAGEUP
        | K::KEY_PAGEDOWN
        | K::KEY_INSERT
        | K::KEY_DELETE
        | K::KEY_NUMLOCK
        | K::KEY_KP0
        | K::KEY_KP1
        | K::KEY_KP2
        | K::KEY_KP3
        | K::KEY_KP4
        | K::KEY_KP5
        | K::KEY_KP6
        | K::KEY_KP7
        | K::KEY_KP8
        | K::KEY_KP9
        | K::KEY_KPDOT
        | K::KEY_KPENTER
        | K::KEY_KPPLUS
        | K::KEY_KPMINUS
        | K::KEY_KPASTERISK
        | K::KEY_KPSLASH
        | K::KEY_KPEQUAL => Some(Side::Right),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mouse_left_button_is_left() {
        assert_eq!(
            key_side(KeyCode::BTN_LEFT, DeviceKind::Mouse),
            Some(Side::Left)
        );
    }

    #[test]
    fn mouse_right_button_is_right() {
        assert_eq!(
            key_side(KeyCode::BTN_RIGHT, DeviceKind::Mouse),
            Some(Side::Right)
        );
    }

    #[test]
    fn mouse_middle_button_is_unclassified() {
        assert_eq!(key_side(KeyCode::BTN_MIDDLE, DeviceKind::Mouse), None);
    }

    #[test]
    fn mouse_extra_buttons_are_unclassified() {
        assert_eq!(key_side(KeyCode::BTN_SIDE, DeviceKind::Mouse), None);
        assert_eq!(key_side(KeyCode::BTN_EXTRA, DeviceKind::Mouse), None);
        assert_eq!(key_side(KeyCode::BTN_FORWARD, DeviceKind::Mouse), None);
        assert_eq!(key_side(KeyCode::BTN_BACK, DeviceKind::Mouse), None);
    }

    #[test]
    fn mouse_kind_ignores_keyboard_alpha_keys() {
        assert_eq!(key_side(KeyCode::KEY_A, DeviceKind::Mouse), None);
        assert_eq!(key_side(KeyCode::KEY_Z, DeviceKind::Mouse), None);
        assert_eq!(key_side(KeyCode::KEY_SPACE, DeviceKind::Mouse), None);
    }

    #[test]
    fn keyboard_kind_ignores_mouse_buttons() {
        assert_eq!(key_side(KeyCode::BTN_LEFT, DeviceKind::Keyboard), None);
        assert_eq!(key_side(KeyCode::BTN_RIGHT, DeviceKind::Keyboard), None);
        assert_eq!(key_side(KeyCode::BTN_MIDDLE, DeviceKind::Keyboard), None);
    }

    #[test]
    fn keyboard_left_alpha_row() {
        for k in [
            KeyCode::KEY_Q,
            KeyCode::KEY_W,
            KeyCode::KEY_E,
            KeyCode::KEY_R,
            KeyCode::KEY_T,
            KeyCode::KEY_A,
            KeyCode::KEY_S,
            KeyCode::KEY_D,
            KeyCode::KEY_F,
            KeyCode::KEY_G,
            KeyCode::KEY_Z,
            KeyCode::KEY_X,
            KeyCode::KEY_C,
            KeyCode::KEY_V,
            KeyCode::KEY_B,
        ] {
            assert_eq!(
                key_side(k, DeviceKind::Keyboard),
                Some(Side::Left),
                "{k:?} should be Left"
            );
        }
    }

    #[test]
    fn keyboard_right_alpha_row() {
        for k in [
            KeyCode::KEY_Y,
            KeyCode::KEY_U,
            KeyCode::KEY_I,
            KeyCode::KEY_O,
            KeyCode::KEY_P,
            KeyCode::KEY_H,
            KeyCode::KEY_J,
            KeyCode::KEY_K,
            KeyCode::KEY_L,
            KeyCode::KEY_N,
            KeyCode::KEY_M,
        ] {
            assert_eq!(
                key_side(k, DeviceKind::Keyboard),
                Some(Side::Right),
                "{k:?} should be Right"
            );
        }
    }

    #[test]
    fn keyboard_numbers_split_at_six() {
        for k in [
            KeyCode::KEY_1,
            KeyCode::KEY_2,
            KeyCode::KEY_3,
            KeyCode::KEY_4,
            KeyCode::KEY_5,
        ] {
            assert_eq!(key_side(k, DeviceKind::Keyboard), Some(Side::Left));
        }
        for k in [
            KeyCode::KEY_6,
            KeyCode::KEY_7,
            KeyCode::KEY_8,
            KeyCode::KEY_9,
            KeyCode::KEY_0,
        ] {
            assert_eq!(key_side(k, DeviceKind::Keyboard), Some(Side::Right));
        }
    }

    #[test]
    fn keyboard_f_keys_split_at_seven() {
        for k in [
            KeyCode::KEY_F1,
            KeyCode::KEY_F2,
            KeyCode::KEY_F3,
            KeyCode::KEY_F4,
            KeyCode::KEY_F5,
            KeyCode::KEY_F6,
        ] {
            assert_eq!(key_side(k, DeviceKind::Keyboard), Some(Side::Left));
        }
        for k in [
            KeyCode::KEY_F7,
            KeyCode::KEY_F8,
            KeyCode::KEY_F9,
            KeyCode::KEY_F10,
            KeyCode::KEY_F11,
            KeyCode::KEY_F12,
        ] {
            assert_eq!(key_side(k, DeviceKind::Keyboard), Some(Side::Right));
        }
    }

    #[test]
    fn keyboard_space_is_left() {
        assert_eq!(
            key_side(KeyCode::KEY_SPACE, DeviceKind::Keyboard),
            Some(Side::Left)
        );
    }

    #[test]
    fn keyboard_modifiers_split_by_physical_side() {
        let left = [
            KeyCode::KEY_LEFTSHIFT,
            KeyCode::KEY_LEFTCTRL,
            KeyCode::KEY_LEFTALT,
            KeyCode::KEY_LEFTMETA,
        ];
        let right = [
            KeyCode::KEY_RIGHTSHIFT,
            KeyCode::KEY_RIGHTCTRL,
            KeyCode::KEY_RIGHTALT,
            KeyCode::KEY_RIGHTMETA,
        ];
        for k in left {
            assert_eq!(
                key_side(k, DeviceKind::Keyboard),
                Some(Side::Left),
                "{k:?} should be Left"
            );
        }
        for k in right {
            assert_eq!(
                key_side(k, DeviceKind::Keyboard),
                Some(Side::Right),
                "{k:?} should be Right"
            );
        }
    }

    #[test]
    fn keyboard_esc_grave_tab_capslock_are_left() {
        for k in [
            KeyCode::KEY_ESC,
            KeyCode::KEY_GRAVE,
            KeyCode::KEY_TAB,
            KeyCode::KEY_CAPSLOCK,
            KeyCode::KEY_102ND,
        ] {
            assert_eq!(
                key_side(k, DeviceKind::Keyboard),
                Some(Side::Left),
                "{k:?} should be Left"
            );
        }
    }

    #[test]
    fn keyboard_enter_and_backspace_are_right() {
        assert_eq!(
            key_side(KeyCode::KEY_ENTER, DeviceKind::Keyboard),
            Some(Side::Right)
        );
        assert_eq!(
            key_side(KeyCode::KEY_BACKSPACE, DeviceKind::Keyboard),
            Some(Side::Right)
        );
    }

    #[test]
    fn keyboard_right_side_punctuation() {
        for k in [
            KeyCode::KEY_MINUS,
            KeyCode::KEY_EQUAL,
            KeyCode::KEY_LEFTBRACE,
            KeyCode::KEY_RIGHTBRACE,
            KeyCode::KEY_BACKSLASH,
            KeyCode::KEY_SEMICOLON,
            KeyCode::KEY_APOSTROPHE,
            KeyCode::KEY_COMMA,
            KeyCode::KEY_DOT,
            KeyCode::KEY_SLASH,
        ] {
            assert_eq!(
                key_side(k, DeviceKind::Keyboard),
                Some(Side::Right),
                "{k:?} should be Right"
            );
        }
    }

    #[test]
    fn keyboard_navigation_cluster_is_right() {
        for k in [
            KeyCode::KEY_UP,
            KeyCode::KEY_DOWN,
            KeyCode::KEY_LEFT,
            KeyCode::KEY_RIGHT,
            KeyCode::KEY_HOME,
            KeyCode::KEY_END,
            KeyCode::KEY_PAGEUP,
            KeyCode::KEY_PAGEDOWN,
            KeyCode::KEY_INSERT,
            KeyCode::KEY_DELETE,
        ] {
            assert_eq!(
                key_side(k, DeviceKind::Keyboard),
                Some(Side::Right),
                "{k:?} should be Right"
            );
        }
    }

    #[test]
    fn keyboard_numpad_is_right() {
        for k in [
            KeyCode::KEY_NUMLOCK,
            KeyCode::KEY_KP0,
            KeyCode::KEY_KP1,
            KeyCode::KEY_KP2,
            KeyCode::KEY_KP3,
            KeyCode::KEY_KP4,
            KeyCode::KEY_KP5,
            KeyCode::KEY_KP6,
            KeyCode::KEY_KP7,
            KeyCode::KEY_KP8,
            KeyCode::KEY_KP9,
            KeyCode::KEY_KPPLUS,
            KeyCode::KEY_KPMINUS,
            KeyCode::KEY_KPASTERISK,
            KeyCode::KEY_KPSLASH,
            KeyCode::KEY_KPENTER,
            KeyCode::KEY_KPDOT,
            KeyCode::KEY_KPEQUAL,
        ] {
            assert_eq!(
                key_side(k, DeviceKind::Keyboard),
                Some(Side::Right),
                "{k:?} should be Right"
            );
        }
    }

    #[test]
    fn keyboard_media_keys_are_unclassified() {
        for k in [
            KeyCode::KEY_POWER,
            KeyCode::KEY_VOLUMEUP,
            KeyCode::KEY_VOLUMEDOWN,
            KeyCode::KEY_MUTE,
            KeyCode::KEY_PLAYPAUSE,
            KeyCode::KEY_BRIGHTNESSUP,
        ] {
            assert_eq!(
                key_side(k, DeviceKind::Keyboard),
                None,
                "{k:?} should be unclassified"
            );
        }
    }

    #[test]
    fn key_side_is_deterministic() {
        let probes = [
            KeyCode::KEY_A,
            KeyCode::KEY_L,
            KeyCode::KEY_SPACE,
            KeyCode::BTN_LEFT,
            KeyCode::KEY_POWER,
        ];
        for k in probes {
            for kind in [DeviceKind::Keyboard, DeviceKind::Mouse] {
                assert_eq!(key_side(k, kind), key_side(k, kind));
            }
        }
    }
}

//! The boundary between winit and the engine's own input types.
//!
//! This is one of only two modules that import winit (the other is `app.rs`). Keeping the
//! translation in one place means the rest of the engine and every game see only [`Key`] and
//! [`MouseButton`]; replacing winit, or upgrading it when its key names change, touches this file
//! alone.

use winit::event::MouseButton as WinitMouseButton;
use winit::keyboard::KeyCode;

use crate::input::{Key, MouseButton};

/// Translates a physical key code. Keys the engine does not know yet give `None`.
// Used by app.rs from Task 7 on; until then only tests call these.
#[allow(dead_code)]
pub(crate) fn key_from_winit(code: KeyCode) -> Option<Key> {
    let key = match code {
        KeyCode::KeyA => Key::A,
        KeyCode::KeyB => Key::B,
        KeyCode::KeyC => Key::C,
        KeyCode::KeyD => Key::D,
        KeyCode::KeyE => Key::E,
        KeyCode::KeyF => Key::F,
        KeyCode::KeyG => Key::G,
        KeyCode::KeyH => Key::H,
        KeyCode::KeyI => Key::I,
        KeyCode::KeyJ => Key::J,
        KeyCode::KeyK => Key::K,
        KeyCode::KeyL => Key::L,
        KeyCode::KeyM => Key::M,
        KeyCode::KeyN => Key::N,
        KeyCode::KeyO => Key::O,
        KeyCode::KeyP => Key::P,
        KeyCode::KeyQ => Key::Q,
        KeyCode::KeyR => Key::R,
        KeyCode::KeyS => Key::S,
        KeyCode::KeyT => Key::T,
        KeyCode::KeyU => Key::U,
        KeyCode::KeyV => Key::V,
        KeyCode::KeyW => Key::W,
        KeyCode::KeyX => Key::X,
        KeyCode::KeyY => Key::Y,
        KeyCode::KeyZ => Key::Z,
        KeyCode::Digit0 => Key::Digit0,
        KeyCode::Digit1 => Key::Digit1,
        KeyCode::Digit2 => Key::Digit2,
        KeyCode::Digit3 => Key::Digit3,
        KeyCode::Digit4 => Key::Digit4,
        KeyCode::Digit5 => Key::Digit5,
        KeyCode::Digit6 => Key::Digit6,
        KeyCode::Digit7 => Key::Digit7,
        KeyCode::Digit8 => Key::Digit8,
        KeyCode::Digit9 => Key::Digit9,
        KeyCode::ArrowUp => Key::Up,
        KeyCode::ArrowDown => Key::Down,
        KeyCode::ArrowLeft => Key::Left,
        KeyCode::ArrowRight => Key::Right,
        KeyCode::Space => Key::Space,
        KeyCode::Enter => Key::Enter,
        KeyCode::Escape => Key::Escape,
        KeyCode::Tab => Key::Tab,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::ShiftLeft => Key::LeftShift,
        KeyCode::ShiftRight => Key::RightShift,
        KeyCode::ControlLeft => Key::LeftControl,
        KeyCode::ControlRight => Key::RightControl,
        KeyCode::AltLeft => Key::LeftAlt,
        KeyCode::AltRight => Key::RightAlt,
        KeyCode::F1 => Key::F1,
        KeyCode::F2 => Key::F2,
        KeyCode::F3 => Key::F3,
        KeyCode::F4 => Key::F4,
        KeyCode::F5 => Key::F5,
        KeyCode::F6 => Key::F6,
        KeyCode::F7 => Key::F7,
        KeyCode::F8 => Key::F8,
        KeyCode::F9 => Key::F9,
        KeyCode::F10 => Key::F10,
        KeyCode::F11 => Key::F11,
        KeyCode::F12 => Key::F12,
        _ => return None,
    };
    Some(key)
}

/// Translates a mouse button. Extra buttons (back, forward, …) give `None`.
// Used by app.rs from Task 7 on; until then only tests call these.
#[allow(dead_code)]
pub(crate) fn mouse_button_from_winit(button: WinitMouseButton) -> Option<MouseButton> {
    match button {
        WinitMouseButton::Left => Some(MouseButton::Left),
        WinitMouseButton::Right => Some(MouseButton::Right),
        WinitMouseButton::Middle => Some(MouseButton::Middle),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_digits_and_arrows_translate() {
        assert_eq!(key_from_winit(KeyCode::KeyW), Some(Key::W));
        assert_eq!(key_from_winit(KeyCode::KeyZ), Some(Key::Z));
        assert_eq!(key_from_winit(KeyCode::Digit7), Some(Key::Digit7));
        assert_eq!(key_from_winit(KeyCode::ArrowLeft), Some(Key::Left));
    }

    #[test]
    fn control_and_function_keys_translate() {
        assert_eq!(key_from_winit(KeyCode::Escape), Some(Key::Escape));
        assert_eq!(key_from_winit(KeyCode::ShiftRight), Some(Key::RightShift));
        assert_eq!(key_from_winit(KeyCode::F12), Some(Key::F12));
    }

    #[test]
    fn unknown_keys_are_none() {
        assert_eq!(key_from_winit(KeyCode::NumLock), None);
    }

    #[test]
    fn mouse_buttons_translate() {
        assert_eq!(
            mouse_button_from_winit(WinitMouseButton::Left),
            Some(MouseButton::Left)
        );
        assert_eq!(
            mouse_button_from_winit(WinitMouseButton::Middle),
            Some(MouseButton::Middle)
        );
        assert_eq!(mouse_button_from_winit(WinitMouseButton::Back), None);
    }
}

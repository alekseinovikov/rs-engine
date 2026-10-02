//! The engine's own key and mouse button types.
//!
//! Keys are *physical*: [`Key::W`] is the key in the W position of a US keyboard, whatever the
//! active layout prints on it. That is what games want: WASD stays under the left hand on a
//! QWERTY, AZERTY or Russian layout. Text input, which needs the printed character, is a
//! different feature and not part of the MVP.
//!
//! The windowing library has its own types for the same idea; `platform.rs` translates them, so
//! games depend only on these enums. This is the platform abstraction layer that engines such as
//! SDL and raylib provide too: a new backend or input device means a new translation function,
//! not changes to every game.

/// A physical keyboard key, named after its label on a US QWERTY keyboard.
#[allow(missing_docs)] // Each variant is named after the key it stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Up,
    Down,
    Left,
    Right,
    Space,
    Enter,
    Escape,
    Tab,
    Backspace,
    LeftShift,
    RightShift,
    LeftControl,
    RightControl,
    LeftAlt,
    RightAlt,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

/// A mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// The primary (usually left) button.
    Left,
    /// The secondary (usually right) button.
    Right,
    /// The middle button, often the scroll wheel.
    Middle,
}

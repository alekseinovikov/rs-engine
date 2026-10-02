//! Keyboard and mouse input, as the game sees it through `ctx.input`.
//!
//! [`Input`] is plain data: the app writes OS events into it, the game reads it in `update`, and
//! after every fixed tick the app calls `end_tick` so that "just pressed" and "just released"
//! last exactly one tick (see `buttons.rs` for why). Because nothing here knows about the
//! windowing library, the whole module is tested with made-up events.

mod buttons;
mod key;

pub use key::{Key, MouseButton};

use buttons::ButtonState;

/// The keyboard and mouse state for the current tick.
#[derive(Debug, Clone)]
pub struct Input {
    keys: ButtonState<Key>,
    mouse_buttons: ButtonState<MouseButton>,
    mouse_position: (f32, f32),
}

impl Input {
    /// Creates an input state with nothing held and the mouse at the origin.
    pub(crate) fn new() -> Self {
        Self {
            keys: ButtonState::new(),
            mouse_buttons: ButtonState::new(),
            mouse_position: (0.0, 0.0),
        }
    }

    /// Is `key` held?
    pub fn is_down(&self, key: Key) -> bool {
        self.keys.is_down(key)
    }

    /// Did `key` go down since the last tick? True for exactly one tick per press.
    pub fn just_pressed(&self, key: Key) -> bool {
        self.keys.just_pressed(key)
    }

    /// Did `key` go up since the last tick? True for exactly one tick per release.
    pub fn just_released(&self, key: Key) -> bool {
        self.keys.just_released(key)
    }

    /// The keys pressed since the last tick, in no particular order.
    pub fn just_pressed_keys(&self) -> impl Iterator<Item = Key> + '_ {
        self.keys.pressed()
    }

    /// The keys released since the last tick, in no particular order.
    pub fn just_released_keys(&self) -> impl Iterator<Item = Key> + '_ {
        self.keys.released()
    }

    /// Is mouse `button` held?
    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons.is_down(button)
    }

    /// Did mouse `button` go down since the last tick?
    pub fn mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons.just_pressed(button)
    }

    /// Did mouse `button` go up since the last tick?
    pub fn mouse_just_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons.just_released(button)
    }

    /// The cursor position in physical window pixels, from the top-left corner (+y points down).
    ///
    /// Converting it to world coordinates needs the camera, which arrives in M3.
    pub fn mouse_position(&self) -> (f32, f32) {
        self.mouse_position
    }

    pub(crate) fn press_key(&mut self, key: Key) {
        self.keys.press(key);
    }

    pub(crate) fn release_key(&mut self, key: Key) {
        self.keys.release(key);
    }

    pub(crate) fn press_mouse(&mut self, button: MouseButton) {
        self.mouse_buttons.press(button);
    }

    pub(crate) fn release_mouse(&mut self, button: MouseButton) {
        self.mouse_buttons.release(button);
    }

    pub(crate) fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position = (x, y);
    }

    /// Releases all keys and mouse buttons; called when the window loses focus.
    pub(crate) fn release_all(&mut self) {
        self.keys.release_all();
        self.mouse_buttons.release_all();
    }

    /// Ends a fixed tick: clears the "just pressed" and "just released" edges.
    pub(crate) fn end_tick(&mut self) {
        self.keys.end_tick();
        self.mouse_buttons.end_tick();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_and_mouse_buttons_are_tracked_separately() {
        let mut input = Input::new();
        input.press_key(Key::Space);
        input.press_mouse(MouseButton::Left);
        assert!(input.just_pressed(Key::Space));
        assert!(input.mouse_just_pressed(MouseButton::Left));
        assert!(!input.is_mouse_down(MouseButton::Right));
    }

    #[test]
    fn end_tick_clears_edges_for_keys_and_mouse() {
        let mut input = Input::new();
        input.press_key(Key::Space);
        input.press_mouse(MouseButton::Left);
        input.end_tick();
        assert!(!input.just_pressed(Key::Space));
        assert!(!input.mouse_just_pressed(MouseButton::Left));
        assert!(input.is_down(Key::Space));
        assert!(input.is_mouse_down(MouseButton::Left));
    }

    #[test]
    fn release_all_releases_keys_and_mouse() {
        let mut input = Input::new();
        input.press_key(Key::A);
        input.press_mouse(MouseButton::Right);
        input.release_all();
        assert!(!input.is_down(Key::A));
        assert!(!input.is_mouse_down(MouseButton::Right));
        assert!(input.just_released(Key::A));
    }

    #[test]
    fn the_mouse_position_is_stored() {
        let mut input = Input::new();
        input.set_mouse_position(12.5, 40.0);
        assert_eq!(input.mouse_position(), (12.5, 40.0));
    }
}

//! The state of a set of buttons: which are held, and which changed during this tick.
//!
//! The OS reports *edges* ("W went down", "W went up"); games ask about *levels* ("is W held?")
//! and edges ("was Space pressed this tick?"). [`ButtonState`] turns the first into the second
//! with three sets:
//!
//! - `down`: buttons held right now;
//! - `pressed` / `released`: buttons whose state changed since the last tick.
//!
//! The app feeds OS events in as they arrive, the game reads the sets in `update`, and the app
//! calls [`ButtonState::end_tick`] after each tick, which clears the edges. So an edge is seen by
//! exactly one tick: a press that arrives during a frame with no ticks waits for the next tick,
//! and a frame with two ticks does not see it twice.
//!
//! The same logic serves keys and mouse buttons, hence the generic parameter.

use std::collections::HashSet;
use std::hash::Hash;

/// Held buttons plus this tick's presses and releases, for any button type `T`.
#[derive(Debug, Clone)]
pub(crate) struct ButtonState<T> {
    down: HashSet<T>,
    pressed: HashSet<T>,
    released: HashSet<T>,
}

// Used by app.rs from Task 7 on; until then only tests call these.
#[allow(dead_code)]
impl<T: Copy + Eq + Hash> ButtonState<T> {
    /// Creates a state with nothing held.
    pub(crate) fn new() -> Self {
        Self {
            down: HashSet::new(),
            pressed: HashSet::new(),
            released: HashSet::new(),
        }
    }

    /// Records that `button` went down. A button that is already held is ignored, which also
    /// filters out the OS key auto-repeat.
    pub(crate) fn press(&mut self, button: T) {
        // `insert` returns false if the button was already held.
        if self.down.insert(button) {
            self.pressed.insert(button);
        }
    }

    /// Records that `button` went up. A button that is not held is ignored.
    pub(crate) fn release(&mut self, button: T) {
        if self.down.remove(&button) {
            self.released.insert(button);
        }
    }

    /// Releases every held button, for example when the window loses focus: the OS will not send
    /// us the key-up events that happen in another window.
    pub(crate) fn release_all(&mut self) {
        self.released.extend(self.down.drain());
    }

    /// Clears this tick's presses and releases. Held buttons stay held.
    pub(crate) fn end_tick(&mut self) {
        self.pressed.clear();
        self.released.clear();
    }

    /// Is `button` held?
    pub(crate) fn is_down(&self, button: T) -> bool {
        self.down.contains(&button)
    }

    /// Did `button` go down since the last tick?
    pub(crate) fn just_pressed(&self, button: T) -> bool {
        self.pressed.contains(&button)
    }

    /// Did `button` go up since the last tick?
    pub(crate) fn just_released(&self, button: T) -> bool {
        self.released.contains(&button)
    }

    /// The buttons that went down since the last tick, in no particular order.
    pub(crate) fn pressed(&self) -> impl Iterator<Item = T> + '_ {
        self.pressed.iter().copied()
    }

    /// The buttons that went up since the last tick, in no particular order.
    pub(crate) fn released(&self) -> impl Iterator<Item = T> + '_ {
        self.released.iter().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Plain chars stand in for keys: the logic does not care about the button type.
    fn state() -> ButtonState<char> {
        ButtonState::new()
    }

    #[test]
    fn a_press_is_down_and_just_pressed() {
        let mut s = state();
        s.press('w');
        assert!(s.is_down('w'));
        assert!(s.just_pressed('w'));
        assert!(!s.just_released('w'));
    }

    #[test]
    fn end_tick_clears_the_edge_but_keeps_the_button_held() {
        let mut s = state();
        s.press('w');
        s.end_tick();
        assert!(s.is_down('w'));
        assert!(!s.just_pressed('w'));
    }

    #[test]
    fn a_release_is_up_and_just_released() {
        let mut s = state();
        s.press('w');
        s.end_tick();
        s.release('w');
        assert!(!s.is_down('w'));
        assert!(s.just_released('w'));
        s.end_tick();
        assert!(!s.just_released('w'));
    }

    #[test]
    fn a_tap_between_two_ticks_is_not_lost() {
        let mut s = state();
        s.press('w');
        s.release('w');
        assert!(!s.is_down('w'));
        assert!(
            s.just_pressed('w'),
            "the press must still be seen by the next tick"
        );
        assert!(s.just_released('w'));
    }

    #[test]
    fn auto_repeat_does_not_press_again() {
        let mut s = state();
        s.press('w');
        s.end_tick();
        s.press('w'); // the OS repeats a held key
        assert!(!s.just_pressed('w'));
        assert!(s.is_down('w'));
    }

    #[test]
    fn releasing_a_button_that_is_not_held_is_ignored() {
        let mut s = state();
        s.release('w');
        assert!(!s.just_released('w'));
    }

    #[test]
    fn release_all_releases_every_held_button() {
        let mut s = state();
        s.press('a');
        s.press('d');
        s.end_tick();
        s.release_all();
        assert!(!s.is_down('a') && !s.is_down('d'));
        assert!(s.just_released('a') && s.just_released('d'));
    }

    #[test]
    fn pressed_lists_this_ticks_presses() {
        let mut s = state();
        s.press('a');
        s.press('b');
        let mut pressed: Vec<char> = s.pressed().collect();
        pressed.sort();
        assert_eq!(pressed, ['a', 'b']);
        s.end_tick();
        assert_eq!(s.pressed().count(), 0);
    }
}

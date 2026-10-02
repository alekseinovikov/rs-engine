//! Colors as artists write them (sRGB) and as the GPU blends them (linear).
//!
//! Screens do not show brightness linearly. The sRGB standard spends more of its 256 steps on
//! dark shades, where our eyes notice differences, so the value 0.5 is not "half as bright" as
//! 1.0 but about a fifth. Every color picker, image editor and CSS color is in sRGB.
//!
//! Blending and interpolation, however, are only correct in *linear* space, where 0.5 really is
//! half the light. So the engine works like this:
//!
//! 1. The game writes colors in sRGB ([`Color::rgb`], [`Color::from_hex`]), exactly as in an
//!    image editor.
//! 2. The engine converts them to linear on the CPU ([`Color::to_linear`]) before they reach the
//!    GPU.
//! 3. The GPU interpolates and blends in linear space.
//! 4. The window's surface has an `...Srgb` format, so the GPU converts each pixel back to sRGB
//!    when it writes it, and the screen shows the color the artist picked.

/// A color in sRGB space with straight (not premultiplied) alpha. Each channel is in `0.0..=1.0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red, in sRGB.
    pub r: f32,
    /// Green, in sRGB.
    pub g: f32,
    /// Blue, in sRGB.
    pub b: f32,
    /// Opacity: `0.0` is fully transparent, `1.0` fully opaque. Alpha is not gamma-encoded.
    pub a: f32,
}

impl Color {
    /// Opaque black.
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    /// Opaque white.
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    /// Opaque red.
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
    /// Opaque green.
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
    /// Opaque blue.
    pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
    /// Fully transparent black.
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

    /// An opaque color from sRGB channels in `0.0..=1.0`.
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
    }

    /// A color from sRGB channels and alpha, each in `0.0..=1.0`.
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// An opaque color from a `0xRRGGBB` number, as written in CSS or an image editor.
    ///
    /// ```
    /// use engine::Color;
    /// assert_eq!(Color::from_hex(0xFF0000), Color::RED);
    /// ```
    pub const fn from_hex(rgb: u32) -> Self {
        let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
        let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
        let b = (rgb & 0xFF) as f32 / 255.0;
        Self::rgb(r, g, b)
    }

    /// The same color with a different alpha.
    pub const fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    /// Converts to linear space as `[r, g, b, a]`, the form the GPU works with.
    pub fn to_linear(self) -> [f32; 4] {
        [
            srgb_to_linear(self.r),
            srgb_to_linear(self.g),
            srgb_to_linear(self.b),
            self.a,
        ]
    }
}

/// The sRGB transfer function, inverted: an sRGB channel value to linear light.
///
/// Near black the curve is a straight line (dividing by 12.92); above 0.04045 it is a power
/// curve with exponent 2.4. The two pieces meet smoothly, which avoids an infinitely steep slope
/// at zero.
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 1e-5,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn black_and_white_are_the_same_in_both_spaces() {
        assert_eq!(Color::BLACK.to_linear(), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(Color::WHITE.to_linear(), [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn mid_grey_in_srgb_is_about_a_fifth_of_the_light() {
        let [r, g, b, _] = Color::rgb(0.5, 0.5, 0.5).to_linear();
        assert_close(r, 0.214_041);
        assert_eq!((r, r), (g, b));
    }

    #[test]
    fn the_linear_and_power_pieces_meet_at_the_threshold() {
        let below = srgb_to_linear(0.04045);
        let above = ((0.04045_f32 + 0.055) / 1.055).powf(2.4);
        assert_close(below, 0.003_130_8);
        assert_close(below, above);
    }

    #[test]
    fn alpha_is_not_converted() {
        let [_, _, _, a] = Color::rgba(0.5, 0.5, 0.5, 0.5).to_linear();
        assert_eq!(a, 0.5);
    }

    #[test]
    fn hex_splits_into_channels() {
        let color = Color::from_hex(0x3366CC);
        assert_close(color.r, 0.2);
        assert_close(color.g, 0.4);
        assert_close(color.b, 0.8);
        assert_eq!(color.a, 1.0);
        assert_eq!(Color::from_hex(0x00FF00), Color::GREEN);
    }

    #[test]
    fn with_alpha_keeps_the_channels() {
        assert_eq!(
            Color::RED.with_alpha(0.25),
            Color::rgba(1.0, 0.0, 0.0, 0.25)
        );
    }
}

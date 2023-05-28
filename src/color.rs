use stagebridge::num::Float;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Color {
    pub a: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub w: f32
}

impl Color {
    pub const OFF:     Self = Self::argbw(0.0, 0.0, 0.0, 0.0, 0.0);
    pub const WHITE:   Self = Self::w(1.0);
    pub const RGB:     Self = Self::rgb(1.0,   1.0,   1.0);

    pub const RED:     Self = Self::rgb(1.0,   0.0,   0.0);
    pub const ORANGE:  Self = Self::rgb(1.0,   0.251, 0.0);
    pub const YELLOW:  Self = Self::rgb(1.0,   1.0,   0.0);
    pub const PEA:     Self = Self::rgb(0.533, 1.0,   0.0);
    pub const LIME:    Self = Self::rgb(0.0,   1.0,   0.0);
    pub const MINT:    Self = Self::rgb(0.0,   1.0,   0.267);
    pub const CYAN:    Self = Self::rgb(0.0,   0.8,   1.0);
    pub const BLUE:    Self = Self::rgb(0.0,   0.0,   1.0);
    pub const VIOLET:  Self = Self::rgb(0.533, 0.0,   1.0);
    pub const MAGENTA: Self = Self::rgb(1.0,   0.0,   1.0);
    pub const PINK:    Self = Self::rgb(1.0,   0.38,  0.8);

    pub const fn argbw(a: f32, r: f32, g: f32, b: f32, w: f32) -> Self { Self { a, r, g, b, w } }
    pub const fn argb(a: f32, r: f32, g: f32, b: f32) -> Self { Self::argbw(a, r, g, b, 0.0) }
    pub const fn aw(a: f32, w: f32) -> Self { Self::argbw(a, 0.0, 0.0, 0.0, w) }
    pub const fn rgbw(r: f32, g: f32, b: f32, w: f32) -> Self { Self::argbw(1.0, r, g, b, w) }
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self { Self::argb(1.0, r, g, b) }
    pub const fn w(w: f32) -> Self { Self::aw(1.0, w) }

    pub fn hsv(h: f32, s: f32, v: f32) -> Self {
        use stagebridge::util::ease::mix;
        let r = v * mix(1.0, (((h + 1.0      ).fract() * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0), s);
        let g = v * mix(1.0, (((h + 0.6666666).fract() * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0), s);
        let b = v * mix(1.0, (((h + 0.3333333).fract() * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0), s);
        Self::rgb(r, g, b)
    }
}

impl Color {
    pub fn a(self, a: f32) -> Self {
        Self { a, r: self.r, g: self.g, b: self.b, w: self.w }
    }

    pub fn a_mul(self, a: f32) -> Self {
        Self { a: self.a * a, r: self.r, g: self.g, b: self.b, w: self.w }
    }
}

use stagebridge::midi::device::launchpad_x::types::PaletteColor;
impl From<PaletteColor> for Color {
    fn from(p: PaletteColor) -> Self {
        match p {
            PaletteColor::Index(_)   => Color::WHITE,
            PaletteColor::Off        => Color::OFF,
            PaletteColor::White      => Color::WHITE,
            PaletteColor::Red        => Color::RED,
            PaletteColor::Orange     => Color::ORANGE,
            PaletteColor::Yellow     => Color::YELLOW,
            PaletteColor::Pea        => Color::PEA,
            PaletteColor::Lime       => Color::LIME,
            PaletteColor::Mint       => Color::MINT,
            PaletteColor::Cyan       => Color::CYAN,
            PaletteColor::Blue       => Color::BLUE,
            PaletteColor::Violet     => Color::VIOLET,
            PaletteColor::Magenta    => Color::MAGENTA,
            PaletteColor::Pink       => Color::PINK,
        }
    }
}

pub use stagebridge::midi::device::launchpad_x::types::Color as PadColor;
impl From<Color> for PadColor {
    fn from(color: Color) -> Self {
        if color == Color::WHITE || color == Color::RGB {
            PadColor::Palette(PaletteColor::White)
        } else {
            let Color { r, g, b, a, .. } = color;
            let r = (r * a).midi_byte();
            let g = (g * a).midi_byte();
            let b = (b * a).midi_byte();
            PadColor::Rgb(r, g, b)
        }
    }
}

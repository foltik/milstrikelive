use stagebridge::dmx::DMXDevice;
use stagebridge::num::Float;

use crate::ColorMode;

#[derive(Clone, Copy, Debug)]
pub struct Laser {
    pub active: bool,

    pub pattern: LaserPattern,
    pub color: LaserColor,
    pub stroke: LaserStroke,

    pub rotate: f32,
    pub xflip: f32,
    pub yflip: f32,
    pub x: f32,
    pub y: f32,
    pub size: f32,
}

impl std::default::Default for Laser {
    fn default() -> Self {
        Self {
            active: false,
            pattern: LaserPattern::Raw(0),
            stroke: LaserStroke::Solid(1.0),
            color: LaserColor::RGB,

            rotate: 0.0,
            xflip: 0.0,
            yflip: 0.0,
            x: 0.0,
            y: 0.0,
            size: 0.0,
        }
    }
}

impl DMXDevice for Laser {
    fn size(&self) -> usize { 10 }

    #[rustfmt::skip]
    fn encode(&self, buffer: &mut [u8]) {
        buffer[0] = if self.active { 64 } else { 0 };
        buffer[1] = self.pattern.byte();
        buffer[2] = self.rotate.lerp_byte(0..127);
        buffer[3] = self.yflip.lerp_byte(0..127);
        buffer[4] = self.xflip.lerp_byte(0..127);
        buffer[5] = self.x.lerp_byte(0..127);
        buffer[6] = self.y.lerp_byte(0..127);
        buffer[7] = self.size.lerp_byte(0..63);
        buffer[8] = self.color.byte();
        buffer[9] = self.stroke.byte();
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LaserColor {
    Raw(u8),

    Rgb(bool, bool, bool),
    Mix(usize),
}
impl From<ColorMode> for LaserColor {
    fn from(mode: ColorMode) -> Self {
        match mode {
            ColorMode::Red => LaserColor::RED,
            ColorMode::Green => LaserColor::GREEN,
            ColorMode::Blue => LaserColor::BLUE,
            ColorMode::Other => LaserColor::RGB,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LaserStroke {
    Solid(f32),
    Dots(f32),
}

#[derive(Clone, Copy, Debug)]
pub enum LaserPattern {
    Raw(u8),

    Square,
    SquareWide,
    SquareXWide,
    SquareBlock,
    Circle,
    CircleWide,
    CircleDash,
    CircleQuad,
    CircleCircle,
    CircleSquare,
    CircleX,
    CircleY,
    LineX,
    LineY,
    LineXY,
    LineDX,
    LineDY,
    Line2X,
    Line2Y,
    LinePenta,
    LineStair,
    Tri,
    TriX,
    TriY,
    Tri3d,
    TriTri,
    TriCircle,
    TriWing,
    TriArch,
    Penta,
    Squig1,
    Squig2,
    Three,
    Two,
    One,
    Music,
    Tree,
    Star,
    Sin,
    Heart,
    Elephant,
    Apple,
    Plus,
    PlusOval,
    PlusArrow,
    PlusDia,
    Arrow,
    ArrowInvert,
    Hourglass1,
    Hourglass2,
}

impl LaserColor {
    pub const RED: Self = LaserColor::Rgb(true, false, false);
    pub const GREEN: Self = LaserColor::Rgb(false, true, false);
    pub const BLUE: Self = LaserColor::Rgb(false, false, true);
    pub const RGB: Self = LaserColor::Rgb(true, true, true);

    pub fn byte(self) -> u8 {
        match self {
            LaserColor::Raw(i) => i,

            LaserColor::Rgb(r, g, b) => match (r, g, b) {
                (true, false, false) => 76, // R
                (false, true, false) => 98, // G
                (false, false, true) => 116, // B

                (true, true, false) => 86, // RG
                (true, false, true) => 122, // RB
                (false, true, true) => 104, // BG

                (true, true, true) => 64, // RGB
                (false, false, false) => 0, // unused
            }

            LaserColor::Mix(i) => match i % 7 {
                0 => 0,
                1 => 10,
                2 => 20,
                3 => 28,
                4 => 38,
                5 => 50,
                6 => 58,
                _ => 0,
            }
        }
    }
}

impl LaserStroke {
    pub fn byte(self) -> u8 {
        match self {
            LaserStroke::Solid(fr) => fr.inv().lerp_byte(0..127),
            LaserStroke::Dots(fr) => fr.inv().lerp_byte(128..255),
        }
    }
}

impl LaserPattern {
    pub fn byte(self) -> u8 {
        match self {
            LaserPattern::Raw(i) => i,

            LaserPattern::Square => 0,
            LaserPattern::SquareWide => 232,
            LaserPattern::SquareXWide => 255,
            LaserPattern::SquareBlock => 224,
            LaserPattern::Circle => 6,
            LaserPattern::CircleWide => 82,
            LaserPattern::CircleDash => 138,
            LaserPattern::CircleQuad => 144,
            LaserPattern::CircleCircle => 146,
            LaserPattern::CircleSquare => 162,
            LaserPattern::CircleX => 26,
            LaserPattern::CircleY => 32,
            LaserPattern::LineX => 12,
            LaserPattern::LineY => 16,
            LaserPattern::LineXY => 22,
            LaserPattern::LineDX => 46,
            LaserPattern::LineDY => 52,
            LaserPattern::Line2X => 56,
            LaserPattern::Line2Y => 62,
            LaserPattern::LinePenta => 172,
            LaserPattern::LineStair => 182,
            LaserPattern::Tri => 36,
            LaserPattern::TriX => 42,
            LaserPattern::TriY => 100,
            LaserPattern::Tri3d => 152,
            LaserPattern::TriTri => 168,
            LaserPattern::TriCircle => 214,
            LaserPattern::TriWing => 218,
            LaserPattern::TriArch => 224,
            LaserPattern::Penta => 186,
            LaserPattern::Squig1 => 66,
            LaserPattern::Squig2 => 72,
            LaserPattern::Three => 94,
            LaserPattern::Two => 112,
            LaserPattern::One => 116,
            LaserPattern::Music => 76,
            LaserPattern::Tree => 86,
            LaserPattern::Star => 104,
            LaserPattern::Sin => 108,
            LaserPattern::Heart => 122,
            LaserPattern::Elephant => 126,
            LaserPattern::Apple => 132,
            LaserPattern::Plus => 156,
            LaserPattern::PlusOval => 194,
            LaserPattern::PlusArrow => 196,
            LaserPattern::PlusDia => 250,
            LaserPattern::Arrow => 204,
            LaserPattern::ArrowInvert => 228,
            LaserPattern::Hourglass1 => 238,
            LaserPattern::Hourglass2 => 210,
        }
    }
}

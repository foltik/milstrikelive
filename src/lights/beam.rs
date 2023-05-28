use stagebridge::dmx::DMXDevice;
use stagebridge::num::Float;

use crate::color::Color;

#[derive(Clone, Copy, Debug)]
pub struct Beam {
    pub mode: BeamMode,

    pub pitch: f32,
    pub yaw: f32,
    pub speed: f32,

    pub color: Color,

    pub ring: BeamRing,
}

impl std::default::Default for Beam {
    fn default() -> Self {
        Self {
            mode: BeamMode::Manual,

            pitch: 0.0,
            yaw: 2.0 / 3.0,
            speed: 1.0,

            color: Color::OFF,

            ring: BeamRing::Off,
        }
    }
}

impl DMXDevice for Beam {
    fn size(&self) -> usize { 15 }

    fn encode(&self, buffer: &mut [u8]) {
        buffer[0] = self.yaw.byte();
        // buffer[0] = (self.yaw * (2.0 / 3.0)).byte();
        buffer[0] = self.yaw.lerp((1.0/3.0)..1.0).byte();
        // buffer[1]: yaw fine
        buffer[2] = self.pitch.byte();
        // buffer[3]: pitch fine
        buffer[4] = (1.0 - self.speed).byte();
        buffer[5] = self.color.a.byte();
        // buffer[6]: strobe
        buffer[7] = self.color.r.byte();
        buffer[8] = self.color.g.byte();
        buffer[9] = self.color.b.byte();
        buffer[10] = self.color.w.byte();
        // buffer[11]: color preset
        buffer[12] = self.mode.byte();
        // buffer[13]: auto pitch/yaw, reset
        buffer[14] = self.ring.byte();
    }
}


#[derive(Clone, Copy, Debug)]
pub enum BeamMode {
    Manual,
    ColorCycle,
    Auto,
}

impl BeamMode {
    pub fn byte(&self) -> u8 {
        match self {
            BeamMode::Manual => 0,
            BeamMode::ColorCycle => 159,
            BeamMode::Auto => 60,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BeamRing {
    Off,

    Red,
    Green,
    Blue,
    Yellow,
    Purple,
    Teal,
    White,

    RedYellow,
    RedPurple,
    RedWhite,

    GreenYellow,
    GreenBlue,
    GreenWhite,

    BluePurple,
    BlueTeal,
    BlueWhite,

    Cycle,
    Raw(u8),
}

impl BeamRing {
    pub fn byte(&self) -> u8 {
        match self {
            BeamRing::Off => 0,

            BeamRing::Red => 4,
            BeamRing::Green => 22,
            BeamRing::Blue => 36,
            BeamRing::Yellow => 56,
            BeamRing::Purple => 74,
            BeamRing::Teal => 84,
            BeamRing::White => 104,

            BeamRing::RedYellow => 116,
            BeamRing::RedPurple => 128,
            BeamRing::RedWhite => 140,

            BeamRing::GreenYellow => 156,
            BeamRing::GreenBlue => 176,
            BeamRing::GreenWhite => 192,

            BeamRing::BluePurple => 206,
            BeamRing::BlueTeal => 216,
            BeamRing::BlueWhite => 242,

            BeamRing::Cycle => 248,
            BeamRing::Raw(i) => *i,
        }
    }
}

use stagebridge::dmx::DMXDevice;
use stagebridge::num::Float;

use crate::color::Color;

#[derive(Clone, Copy, Debug)]
pub struct Spider {
    // pub mode: SpiderMode,
    // pub speed: f32,

    pub color0: Color,
    pub pos0: f32,

    pub color1: Color,
    pub pos1: f32,
}

impl std::default::Default for Spider {
    fn default() -> Self {
        Self {
            // mode: SpiderMode::Manual,
            // speed: 1.0,

            pos0: 0.0,
            pos1: 0.0,

            color0: Color::OFF,
            color1: Color::OFF,
        }
    }
}

impl DMXDevice for Spider {
    fn size(&self) -> usize { 15 }

    fn encode(&self, buffer: &mut [u8]) {
        buffer[0] = self.pos0.byte();
        buffer[1] = self.pos1.byte();
        buffer[2] = self.color0.a.byte();
        // buffer[3]: strobe

        buffer[4] = self.color0.r.byte();
        buffer[5] = self.color0.g.byte();
        buffer[6] = self.color0.b.byte();
        buffer[7] = self.color0.w.byte();
        buffer[8] = self.color0.r.byte();
        buffer[9] = self.color0.g.byte();
        buffer[10] = self.color0.b.byte();
        buffer[11] = self.color0.w.byte();
        // buffer[12]: effect preset
        // buffer[13]: effect speed
        // buffer[14]: reset
    }
}


// pub enum SpiderMode {
//     Manual,
//     ColorCycle,
//     Auto,
// }

// impl SpiderMode {
//     pub fn byte(&self) -> u8 {
//         match self {
//             SpiderMode::Manual => 0,
//             SpiderMode::ColorCycle => 159,
//             SpiderMode::Auto => 60,
//         }
//     }
// }

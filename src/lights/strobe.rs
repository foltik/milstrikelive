use stagebridge::dmx::DMXDevice;
use stagebridge::num::Float;

use crate::color::Color;

#[derive(Clone, Copy, Debug)]
pub struct Strobe {
    // pub mode: StrobeMode,

    pub color: Color,
}

impl std::default::Default for Strobe {
    fn default() -> Self {
        Self {
            // mode: StrobeMode::Manual,

            color: Color::OFF,
        }
    }
}

impl DMXDevice for Strobe {
    fn size(&self) -> usize { 6 }

    fn encode(&self, buffer: &mut [u8]) {
        buffer[0] = self.color.a.byte();
        // buffer[1]: mode
        if self.color.w == 0.0 {
            buffer[2] = self.color.r.byte();
            buffer[3] = self.color.g.byte();
            buffer[4] = self.color.b.byte();
        } else {
            buffer[2] = self.color.w.byte();
            buffer[3] = self.color.w.byte();
            buffer[4] = self.color.w.byte();
        }
        // buffer[5]: sound control
    }
}


// pub enum StrobeMode {
//     Manual,
//     ColorCycle,
//     Auto,
// }

// impl StrobeMode {
//     pub fn byte(&self) -> u8 {
//         match self {
//             StrobeMode::Manual => 0,
//             StrobeMode::ColorCycle => 159,
//             StrobeMode::Auto => 60,
//         }
//     }
// }

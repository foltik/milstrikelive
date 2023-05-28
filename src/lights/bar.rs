use stagebridge::dmx::DMXDevice;
use stagebridge::num::Float;

use crate::color::Color;

#[derive(Clone, Copy, Debug)]
pub struct Bar {
    // pub mode: BarMode,

    pub color: Color,
}

impl std::default::Default for Bar {
    fn default() -> Self {
        Self {
            // mode: BarMode::Manual,

            color: Color::OFF,
        }
    }
}

impl DMXDevice for Bar {
    fn size(&self) -> usize { 7 }

    fn encode(&self, buffer: &mut [u8]) {
        if self.color.w == 0.0 {
            buffer[0] = self.color.r.byte();
            buffer[1] = self.color.g.byte();
            buffer[2] = self.color.b.byte();
        } else {
            buffer[0] = self.color.w.byte();
            buffer[1] = self.color.w.byte();
            buffer[2] = self.color.w.byte();
        }
        // buffer[3]: preset colors
        // buffer[4]: strobe
        // buffer[5]: mode
        buffer[6] = self.color.a.byte();
    }
}


// pub enum BarMode {
//     Manual,
//     ColorCycle,
//     Auto,
// }

// impl BarMode {
//     pub fn byte(&self) -> u8 {
//         match self {
//             BarMode::Manual => 0,
//             BarMode::ColorCycle => 159,
//             BarMode::Auto => 60,
//         }
//     }
// }

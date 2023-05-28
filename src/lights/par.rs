use stagebridge::dmx::DMXDevice;
use stagebridge::num::Float;

use crate::Color;

pub struct Par {
    pub color: Color,
}

impl std::default::Default for Par {
    fn default() -> Self {
        Self {
            color: Color::OFF,
        }
    }
}

impl DMXDevice for Par {
    fn size(&self) -> usize { 8 }

    fn encode(&self, buffer: &mut [u8]) {
        // buffer[0]: ?
        // buffer[1]: ?
        // buffer[2]: ?
        buffer[3] = self.color.a.byte();
        buffer[4] = self.color.r.byte();
        buffer[5] = self.color.g.byte();
        buffer[6] = self.color.b.byte();
        buffer[7] = self.color.w.byte();
    }
}

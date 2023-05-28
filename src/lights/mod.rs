use std::default::Default;

use stagebridge::dmx::{DMX, DMXDevice};

mod beam; pub use beam::*;
mod spider; pub use spider::*;
mod par; pub use par::*;
mod bar; pub use bar::*;
mod strobe; pub use strobe::*;
mod laser; pub use laser::*;

use crate::color::Color;

#[derive(Default)]
pub struct Lights {
    pub pars: [Par; 10],
    pub beams: [Beam; 4],
    pub strobe: Strobe,
    pub bars: [Bar; 2],
    pub spiders: [Spider; 2],
    pub laser: Laser,
}

impl Lights {
    pub fn write(&self, dmx: &mut DMX) {
        for (i, par) in self.pars.iter().enumerate() {
            par.write(dmx, 1 + 8*i);
        }

        for (i, beam) in self.beams.iter().enumerate() {
            beam.write(dmx, 81 + 15 * i);
        }

        self.strobe.write(dmx, 142);

        for (i, bar) in self.bars.iter().enumerate() {
            bar.write(dmx, 149 + 7 * i);
        }

        self.laser.write(dmx, 164);

        for (i, spider) in self.spiders.iter().enumerate() {
            spider.write(dmx, 175 + 15*i);
        }
    }

    pub fn all(mut self, color: Color) -> Self {
        for par in &mut self.pars {
            par.color = color;
        }
        for beam in &mut self.beams {
            beam.color = color;
        }
        for bar in &mut self.bars {
            bar.color = color;
        }
        self.strobe.color = color;
        for spider in &mut self.spiders {
            spider.color0 = color;
            spider.color1 = color;
        }
        self
    }

    pub fn brightness(&mut self, fr: f32) {
        for par in &mut self.pars {
            par.color = par.color.a_mul(fr);
        }
        for beam in &mut self.beams {
            beam.color = beam.color.a_mul(fr);
        }
        for bar in &mut self.bars {
            bar.color = bar.color.a_mul(fr);
        }
        self.strobe.color = self.strobe.color.a_mul(fr);
        for spider in &mut self.spiders {
            spider.color0 = spider.color0.a_mul(fr);
            spider.color1 = spider.color1.a_mul(fr);
        }
    }
}

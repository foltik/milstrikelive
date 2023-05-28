#![feature(trait_alias)]
#![feature(let_chains)]

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(illegal_floating_point_literal_pattern)]

#[macro_use]
extern crate msmacros;

use std::time::{Duration, Instant};

use stagebridge::midi::device::launchpad_x::types::{Pos, Color as PadColor, Coord};
use stagebridge::util::future::Broadcast;
use stagebridge::{cast, osc::{self, Osc, Message as OscMessage, Value as OscValue}};
use stagebridge::midi::device::{launchpad_x, launch_control_xl};
use stagebridge::{dmx::DMX, e131::{E131, E131_PORT}};
use stagebridge::num::{Float, Range};

mod context; use context::*;
// mod time; use time::*;
mod color; use color::*;
mod lights; use lights::*;
mod logic; use logic::*;
mod fx; use fx::*;

#[derive(Clone)]
pub struct State {
    t0: f32,
    t: f32,
    phi: f32,
    phi_mul: f32,
    bpm: f32,

    viz_pd: Pd,
    viz_beat: bool,
    viz_beat_last: bool,
    viz_alpha: f32,

    color_mode: ColorMode,
    color0: ColorOp,
    color1: ColorOp,
    map0: ColorMapOp,
    map1: ColorMapOp,

    fr0: f32,
    fr1: f32,

    off: bool,
    alpha: f32,
}
#[derive(Clone, Copy, Debug)]
pub enum ColorMode {
    Red,
    Green,
    Blue,
    Other,
}
impl State {
    pub fn phi(&self, pd: Pd) -> f32 {
        self.phi.mod_div(pd.fr() * self.phi_mul)
    }

    pub fn color0(&self) -> Color {
        self.map0.apply(self, self.color0.apply(self))
    }
    pub fn color1(&self) -> Color {
        self.map1.apply(self, self.color1.apply(self))
    }

    pub fn color0_phase(&self, pd: Pd, offset: f32) -> Color {
        let mut state = self.clone();
        state.phi = state.phi.phase(pd.fr(), offset);
        self.map0.apply(&state, self.color0.apply(&state))
    }
    pub fn color1_phase(&self, pd: Pd, offset: f32) -> Color {
        let mut state = self.clone();
        state.phi = state.phi.phase(pd.fr(), offset);
        self.map1.apply(&state, self.color1.apply(&state))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ColorSelect {
    All,
    Color0,
    Color1,
}
#[derive(Clone, Copy, Debug)]
pub struct Pd(pub usize, pub usize);
impl Pd {
    pub fn fr(&self) -> f32 {
        self.0 as f32 / self.1 as f32
    }
    pub fn mul(&self, mul: usize) -> Self {
        Self(self.0 * mul, self.1)
    }
    pub fn div(&self, div: usize) -> Self {
        Self(self.0, self.1 * div)
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            t0: 0.0,
            t: 0.0,
            phi: 0.0,
            phi_mul: 1.0,
            bpm: 120.0,

            viz_pd: Pd(1, 1),
            viz_beat: true,
            viz_beat_last: false,
            viz_alpha: 1.0,

            color_mode: ColorMode::Other,
            color0: ColorOp::value(Color::WHITE),
            color1: ColorOp::value(Color::WHITE),
            map0: fx::id(),
            map1: fx::id(),

            fr0: 0.0,
            fr1: 0.0,

            off: false,
            alpha: 1.0,
        }
    }
}

#[tokio::main]
async fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() > 1 && args[1] == "-v" {
        std::env::set_var("RUST_LOG", "trace");
    } else {
        std::env::set_var("RUST_LOG", "debug");
    }
    pretty_env_logger::init();

    let ctx = Context::new().await;

    let mut dmx = DMX::new(205);
    let mut e131 = E131::new("10.16.4.1".parse().unwrap(), E131_PORT, 1).unwrap();

    let osc = Osc::new(7777).await;
    let osc_rx = osc.subscribe_sync();

    let pad_rx = ctx.subscribe_pad();
    let ctrl_rx = ctx.subscribe_ctrl();

    // Main loop runs at 200fps
    let mut state = State::default();

    let mut time = logic::Time::new();
    let mut pads = logic::Pads::new();
    let mut beams = logic::Beams::new();
    let mut lasers = logic::Lasers::new();
    let mut bars = logic::Bars::new();
    let mut pars = logic::Pars::new();
    let mut spiders = logic::Spiders::new();
    let mut strobes = logic::Strobes::new();

    let start = Instant::now();
    let mut interval = tokio::time::interval(Duration::from_millis(5));
    loop {
        interval.tick().await;
        state.t0 = start.elapsed().as_secs_f32();
        if let ClockSource::Static { bpm } = time.source {
            state.t = state.t0;
            state.phi = (state.t0 * (bpm / 60.0)).fmod(16.0);
        }

        let viz_alpha = |fr| {
            osc.send("127.0.0.1:7778", OscMessage {
                addr: "/alpha".into(),
                args: vec![OscValue::Float(fr)],
            })
        };

        let viz_beat = || {
            osc.send("127.0.0.1:7778", OscMessage {
                addr: "/beat".into(),
                args: vec![],
            })
        };

        let viz_switch = |name: &str| {
            osc.send("127.0.0.1:7778", OscMessage {
                addr: format!("/stage/{}", name),
                args: vec![],
            })
        };

        let viz_pd = |bpm: f32, pd: f32| {
            let secs_per_beat = 60.0 / bpm;
            let secs = secs_per_beat * pd;
            osc.send("127.0.0.1:7778", OscMessage {
                addr: "/pd".into(),
                args: vec![osc::Value::Float(secs)],
            })
        };

        let viz_color = |color: Color| {
            let args = if color == Color::WHITE {
                vec![
                    osc::Value::Float(1.0),
                    osc::Value::Float(1.0),
                    osc::Value::Float(1.0),
                ]
            } else {
                vec![
                    osc::Value::Float(color.r),
                    osc::Value::Float(color.g),
                    osc::Value::Float(color.b),
                ]
            };
            osc.send("127.0.0.1:7778", OscMessage {
                addr: "/color".into(),
                args,
            })
        };

        let viz_param = |i: u8, fr: f32| {
            osc.send("127.0.0.1:7778", OscMessage {
                addr: format!("/param/{}", i),
                args: vec![OscValue::Float(fr)]
            })
        };

        for msg in osc_rx.try_iter() {
            log::trace!("OSC: {}: {:?}", &msg.addr, &msg.args);
            use osc::Value;
            if let ClockSource::Osc = time.source {
                match msg.addr.as_str() {
                    "/vdj/time" => state.t = cast!(msg.args[0], Value::Float),
                    "/vdj/phase16" => state.phi = cast!(msg.args[0], Value::Float) * 16.0,
                    "/vdj/bpm" => state.bpm = cast!(msg.args[0], Value::Float),
                    _ => {}
                }
            }
        }

        for input in pad_rx.try_iter() {
            log::trace!("Pad: {:?}", input);
            use launchpad_x::Input;
            match input {
                Input::Press(pos, _fr) => {
                    let Coord(x, y) = Pos::from(pos).into();
                    match (x, y) {
                        // manual
                        (0, 0) => {
                            state.viz_beat = false;
                            state.viz_pd = Pd(1, 4);
                            viz_beat().await;
                            map0!(fx::once(Pd(1, 4), fx::ramp(Pd(1, 4))));
                        },
                        (1, 0) => {
                            state.viz_beat = false;
                            state.viz_pd = Pd(1, 2);
                            viz_beat().await;
                            map0!(fx::once(Pd(1, 2), fx::ramp(Pd(1, 2))));
                        },
                        (2, 0) => {
                            state.viz_beat = false;
                            state.viz_pd = Pd(1, 1);
                            viz_beat().await;
                            map0!(fx::once(Pd(1, 1), fx::ramp(Pd(1, 1))));
                        },
                        (3, 0) => {
                            state.viz_beat = false;
                            state.viz_pd = Pd(2, 1);
                            viz_beat().await;
                            map0!(fx::once(Pd(2, 1), fx::ramp(Pd(2, 1))));
                        },
                        (4, 0) => map1!(fx::once(Pd(1, 4), fx::ramp(Pd(1, 4)))),
                        (5, 0) => map1!(fx::once(Pd(1, 2), fx::ramp(Pd(1, 2)))),
                        (6, 0) => map1!(fx::once(Pd(1, 1), fx::ramp(Pd(1, 1)))),
                        (7, 0) => map1!(fx::once(Pd(2, 1), fx::ramp(Pd(2, 1)))),

                        // colorz
                        (i, 1) => match i {
                            0  => { r!(); color0!(Color::RED.into()); color1!(Color::RED.into()) },
                            1  => { r!(); color0!(Color::RED.into()); color1!(Color::BLUE.into()) },
                            2  => { r!(); color0!(Color::RED.into()); color1!(Color::VIOLET.into()) },
                            3  => { r!(); color0!(Color::MAGENTA.into()); color1!(Color::MAGENTA.into()) },

                            4  => { g!(); color0!(Color::LIME.into()); color1!(Color::LIME.into()) },
                            5  => { g!(); color0!(Color::YELLOW.into()); color1!(Color::YELLOW.into()) },
                            6  => { g!(); color0!(Color::PEA.into()); color1!(Color::LIME.into()) },
                            7  => { g!(); color0!(Color::MINT.into()); color1!(Color::MINT.into()) },
                            _ => unreachable!(),
                        },
                        (i, 2) => match i {
                            0  => { b!(); color0!(Color::BLUE.into()); color1!(Color::BLUE.into()) },
                            1  => { b!(); color0!(Color::CYAN.into()); color1!(Color::BLUE.into()) },
                            2  => { b!(); color0!(Color::BLUE.into()); color1!(Color::VIOLET.into()) },
                            3  => { b!(); color0!(Color::BLUE.into()); color1!(Color::MINT.into()) },

                            4  => { o!(); color0!(Color::RGB.into()); color1!(Color::RGB.into()) },
                            5  => { o!(); color0!(Color::WHITE.into()); color1!(Color::WHITE.into()) },
                            6  => { o!(); color0!(fx::rainbow(Pd(16, 1))); color1!(fx::rainbow(Pd(16, 1))) },
                            7  => { o!(); color0!(fx::rainbow(Pd(4, 1))); color1!(fx::rainbow(Pd(4, 1))) },
                            _ => unreachable!(),
                        },

                        // low
                        (0, 3) => {
                            reset!();
                            map0!(fx::sin(Pd(8, 1), 0.2, 0.15));
                            map1!(fx::off());
                            beams.pattern = BeamPattern::WaveY { pd: Pd(8, 1) };
                            pars.color = ParColor::Color1;
                            spiders.color = SpiderColor::Color1;
                            state.viz_beat = false;
                        },
                        (1, 3) => {
                            reset!();
                            map0!(fx::sin(Pd(8, 1), 0.2, 0.15));
                            map1!(fx::off());
                            beams.pattern = BeamPattern::WaveY { pd: Pd(8, 1) };
                            pars.color = ParColor::Color1;
                            state.viz_beat = false;
                        },
                        (2, 3) => {
                            reset!();
                            map0!(fx::sin(Pd(8, 1), 0.2, 0.15));
                            map1!(fx::off());
                            beams.color = BeamColor::Color1;
                            spiders.color = SpiderColor::Color1;
                            state.viz_beat = false;
                        },
                        (3, 3) => {
                            reset!();
                            map0!(fx::sin(Pd(8, 1), 0.2, 0.15));
                            map1!(fx::off());
                            beams.color = BeamColor::Color1;
                            state.viz_beat = false;
                        },
                        (0, 4) => {
                            reset!();
                            map0!(fx::sin(Pd(8, 1), 0.3, 0.2));
                            map1!(fx::sin(Pd(8, 1), 0.3, 0.2));
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::SpreadOut;
                            spiders.color = SpiderColor::Color1;
                            spiders.pattern = SpiderPattern::Up;
                            state.viz_beat = false;
                        },
                        (1, 4) => {
                            reset!();
                            map0!(fx::sin(Pd(8, 1), 0.3, 0.2));
                            map1!(fx::sin(Pd(8, 1), 0.3, 0.2));
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::SpreadOut;
                            spiders.color = SpiderColor::Color1;
                            spiders.pattern = SpiderPattern::Alternate { pd: Pd(8, 1) };
                            state.viz_beat = false;
                        },
                        (2, 4) => {
                            reset!();
                            map0!(fx::sin(Pd(8, 1), 0.6, 0.2));
                            map1!(fx::sin(Pd(8, 1), 0.6, 0.2));
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::WaveY { pd: Pd(8, 1) };
                            spiders.color = SpiderColor::Color1;
                            spiders.pattern = SpiderPattern::Alternate { pd: Pd(8, 1) };
                            state.viz_beat = false;
                        },
                        (3, 4) => {
                            reset!();
                            map0!(fx::sin(Pd(8, 1), 0.6, 0.2));
                            map1!(fx::sin(Pd(8, 1), 0.6, 0.2));
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::Out;
                            spiders.color = SpiderColor::Color1;
                            spiders.pattern = SpiderPattern::Alternate { pd: Pd(8, 1) };
                            state.viz_beat = false;
                        },

                        // build
                        (0, 5) => {
                            // 2/1 short pulse beams
                            reset!();
                            map0!(fx::pulse_short(Pd(1, 1), 1.0..0.0));
                            map1!(fx::off());
                            spiders.color = SpiderColor::Off;
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::Out;
                            state.viz_alpha = 0.0;
                        },
                        (1, 5) => {
                            // 2/1 short pulse beams, 1/1 pars strobe
                            reset!();
                            map0!(fx::pulse_short(Pd(1, 2), 1.0..0.0));
                            map1!(fx::off());
                            spiders.color = SpiderColor::Off;
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::Out;
                            state.viz_alpha = 0.0;
                        },
                        (2, 5) => {
                            // 2/1 short pulse beams, 1/2 pars strobe
                            reset!();
                            map0!(fx::strobe(Pd(1, 4), 0.5, 0.0..1.0));
                            map1!(fx::alpha(0.1));
                            spiders.color = SpiderColor::Off;
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::Out;
                            bars.color = BarColor::Color0;
                            strobes.color = StrobeColor::Strobe { pd: Pd(1, 4), duty: 0.1, alpha: 1.0 };
                            state.viz_alpha = 0.0;
                        },
                        (3, 5) => {
                            // 2/1 short pulse beams, 1/2 pars strobe, 1/4 strobe light
                            reset!();
                            map0!(fx::strobe(Pd(1, 8), 0.4, 0.0..1.0));
                            map1!(fx::alpha(0.5));
                            spiders.color = SpiderColor::Off;
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::Out;
                            bars.color = BarColor::Color0;
                            strobes.color = StrobeColor::Strobe { pd: Pd(1, 4), duty: 0.1, alpha: 1.0 };
                            state.viz_alpha = 0.0;
                        },
                        (0, 6) => {
                            // 2/1 short pulse beams
                            reset!();
                            map0!(fx::off());
                            map1!(ColorMapOp::value(Color::WHITE).compose(fx::pulse_short(Pd(2, 1), 0.8..0.0)));
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::Square { pd: Pd(2, 1) };
                            state.viz_alpha = 0.0;
                        },
                        (1, 6) => {
                            // 2/1 short pulse beams, 1/1 pars strobe
                            reset!();
                            map0!(ColorMapOp::value(Color::WHITE).compose(fx::strobe(Pd(1, 1), 0.1, 0.0..0.2)));
                            map1!(ColorMapOp::value(Color::WHITE).compose(fx::pulse_short(Pd(2, 1), 0.8..0.0)));
                            spiders.color = SpiderColor::Off;
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::Square { pd: Pd(2, 1) };
                            state.viz_alpha = 0.0;
                        },
                        (2, 6) => {
                            // white roll
                            reset!();
                            map0!(fx::off());
                            map1!(Color::WHITE.into());
                            beams.pattern = BeamPattern::Square { pd: Pd(1, 1) };
                            beams.color = BeamColor::Roll { pd: Pd(1, 1), duty: 0.1, offset: 0.1, alpha: 1.0 };
                            pars.color = ParColor::StrobeAlt1 { pd: Pd(1, 1), duty: 0.1 };
                            strobes.color = StrobeColor::Strobe { pd: Pd(1, 2), duty: 0.1, alpha: 1.0 };
                            state.viz_alpha = 0.0;
                        },
                        (3, 6) => {
                            // mega white roll
                            reset!();
                            map0!(fx::off());
                            map1!(Color::WHITE.into());
                            beams.pattern = BeamPattern::Square { pd: Pd(1, 1) };
                            beams.color = BeamColor::Roll { pd: Pd(1, 1), duty: 0.25, offset: 0.1, alpha: 1.0 };
                            pars.color = ParColor::StrobeRoll1 { pd: Pd(1, 1), duty: 0.2, offset: 0.3 };
                            strobes.color = StrobeColor::Strobe { pd: Pd(1, 4), duty: 0.5, alpha: 1.0 };
                            state.viz_alpha = 0.0;
                        },

                        // break
                        (4, 3) => {
                            reset!();
                            map0!(fx::off());
                            map1!(fx::off());
                        },
                        (5, 3) => {
                            reset!();
                            map0!(fx::off());
                            map1!(fx::alpha(0.1));
                            pars.color = ParColor::Spotlight;
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::SpreadIn;
                            state.viz_alpha = 0.0;
                        },
                        (6, 3) => {
                            reset!();
                            map0!(fx::off());
                            map1!(fx::alpha(0.1));
                            pars.color = ParColor::UpDown;
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::Cross;
                            state.viz_alpha = 0.0;
                        },
                        (7, 3) => {
                            reset!();
                            map0!(fx::off());
                            map1!(fx::alpha(0.1));
                            pars.color = ParColor::UpDown;
                            beams.pattern = BeamPattern::Cross;
                            beams.color = BeamColor::Roll { pd: Pd(1, 1), duty: 0.75, offset: 0.1, alpha: 0.2 };
                            strobes.color = StrobeColor::Strobe { pd: Pd(1, 2), duty: 0.25, alpha: 0.2 };
                            state.viz_alpha = 0.0;
                        },
                        (4, 4) => {
                            reset!();
                            map0!(fx::alpha(0.25));
                            map1!(fx::off());
                            beams.color = BeamColor::Color1;
                            state.viz_alpha = 0.0;
                        },
                        (5, 4) => {
                            reset!();
                            map0!(fx::alpha(0.5));
                            map1!(fx::alpha(0.5));
                            beams.pattern = BeamPattern::Out;
                            strobes.color = StrobeColor::Color0;
                            state.viz_alpha = 0.0;
                        },
                        (6, 4) => {
                            reset!();
                            map0!(fx::strobe(Pd(1, 2), 0.5, 0.0..1.0));
                            map1!(fx::off());
                            strobes.color = StrobeColor::Color0;
                            spiders.color = SpiderColor::Color1;
                            beams.color = BeamColor::Color1;
                            state.viz_alpha = 0.0;
                        },
                        (7, 4) => {
                            reset!();
                            map0!(fx::strobe(Pd(1, 4), 0.5, 0.0..1.0));
                            map1!(fx::id());
                            strobes.color = StrobeColor::Color0;
                            spiders.color = SpiderColor::Color1;
                            spiders.pattern = SpiderPattern::Wave { pd: Pd(2, 1) };
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::Square { pd: Pd(2, 1) };
                            state.viz_alpha = 0.0;
                        },

                        // drop
                        (4, 5) => {
                            // solid with tri color1
                            reset!();
                            map0!(fx::sin(Pd(8, 1), 0.3, 0.2));
                            map1!(fx::tri(Pd(2, 1), 0.0..1.0));
                            spiders.pattern = SpiderPattern::Wave { pd: Pd(2, 1) };
                            spiders.color = SpiderColor::Color1;
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::Square { pd: Pd(2, 1) };
                            state.viz_pd = Pd(1, 1);
                        },
                        (5, 5) => {
                            reset!();
                            map0!(fx::sin(Pd(8, 1), 0.3, 0.2));
                            map1!(fx::tri(Pd(1, 1), 0.0..1.0));
                            spiders.pattern = SpiderPattern::Wave { pd: Pd(2, 1) };
                            spiders.color = SpiderColor::Color1;
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::Square { pd: Pd(2, 1) };
                            state.viz_pd = Pd(1, 1);
                        },
                        (6, 5) => {
                            reset!();
                            map0!(fx::sin(Pd(8, 1), 0.25, 0.25));
                            map1!(fx::tri(Pd(1, 1), 0.0..1.0));
                            spiders.pattern = SpiderPattern::Wave { pd: Pd(2, 1) };
                            spiders.color = SpiderColor::Color1;
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::Square { pd: Pd(2, 1) };
                            state.viz_pd = Pd(1, 1);
                        },
                        (7, 5) => {
                            reset!();
                            map0!(fx::sin(Pd(8, 1), 0.25, 0.25));
                            map1!(fx::tri(Pd(1, 2), 0.0..1.0));
                            spiders.pattern = SpiderPattern::Wave { pd: Pd(2, 1) };
                            spiders.color = SpiderColor::Color1;
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::Square { pd: Pd(2, 1) };
                            state.viz_pd = Pd(1, 2);
                        },
                        (4, 6) => {
                            // wave slow
                            reset!();
                            map0!(fx::pulse(Pd(1, 1), 1.0..0.0));
                            map1!(fx::id());
                            spiders.pattern = SpiderPattern::Wave { pd: Pd(2, 1) };
                            spiders.color = SpiderColor::Color1;
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::Square { pd: Pd(2, 1) };
                            state.viz_pd = Pd(1, 1);
                        },
                        (5, 6) => {
                            // snap slow
                            reset!();
                            map0!(fx::pulse(Pd(1, 1), 1.0..0.0));
                            map1!(fx::id());
                            spiders.pattern = SpiderPattern::Snap { pd: Pd(2, 1) };
                            spiders.color = SpiderColor::Color1;
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::Square { pd: Pd(1, 1) };
                            state.viz_pd = Pd(1, 1);
                        },
                        (6, 6) => {
                            // mini short pulse
                            reset!();
                            map0!(fx::pulse(Pd(1, 1), 1.0..0.0));
                            map1!(fx::id());
                            spiders.pattern = SpiderPattern::Snap { pd: Pd(1, 1) };
                            spiders.color = SpiderColor::Color1;
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::Square { pd: Pd(1, 1) };
                            state.viz_pd = Pd(1, 1);
                        },
                        (7, 6) => {
                            // mega short pulse
                            reset!();
                            map0!(fx::pulse(Pd(1, 2), 1.0..0.0));
                            map1!(fx::id());
                            spiders.pattern = SpiderPattern::Snap { pd: Pd(1, 1) };
                            spiders.color = SpiderColor::Color1;
                            beams.color = BeamColor::Color1;
                            beams.pattern = BeamPattern::Square { pd: Pd(1, 1) };
                            strobes.color = StrobeColor::Color0;
                            state.viz_pd = Pd(1, 2);
                        },

                        // lazors
                        (4, 7) => {
                            lasers.active = false;
                        },
                        (5, 7) => {
                            lasers.active = true;
                            lasers.pattern = LaserPattern::Line2X;
                            lasers.color = state.color_mode.into();
                            lasers.pos = LaserPos::WaveY { pd: Pd(8, 1) }
                        },
                        (6, 7) => {
                            lasers.active = true;
                            lasers.pattern = LaserPattern::LinePenta;
                            lasers.color = state.color_mode.into();
                            lasers.pos = LaserPos::Rotate { pd: Pd(8, 1) }
                        },
                        (7, 7) => {
                            lasers.active = true;
                            lasers.pattern = LaserPattern::TriWing;
                            lasers.color = state.color_mode.into();
                            lasers.pos = LaserPos::Rotate { pd: Pd(8, 1) }
                        }

                        _ => {}
                    };
                },
                _ => {}
            }
            time.pad(&mut state, input);
            pads.pad(&mut state, input);
            beams.pad(&mut state, input);
            lasers.pad(&mut state, input);
            bars.pad(&mut state, input);
            pars.pad(&mut state, input);
            spiders.pad(&mut state, input);
            strobes.pad(&mut state, input);
        }

        for input in ctrl_rx.try_iter() {
            log::trace!("Ctrl: {:?}", input);
            use launch_control_xl::Input;
            match input {
                Input::Slider(0, fr) => state.alpha = fr,
                Input::Slider(1, fr) => state.fr0 = fr,
                Input::Slider(2, fr) => state.fr1 = fr,

                Input::Focus(i, true) => match i {
                    0 => viz_switch("loading").await,
                    1 => viz_switch("code").await,
                    2 => viz_switch("tiles").await,
                    3 => viz_switch("primes").await,
                    4 => viz_switch("bubbles").await,
                    _ => {},
                },
                Input::Control(i, true) => match i {
                    0 => viz_switch("tri").await,
                    1 => viz_switch("linewave").await,
                    2 => viz_switch("torus").await,
                    3 => viz_switch("wormhole").await,
                    4 => viz_switch("stars").await,
                    _ => {},
                },

                Input::SendA(i, fr) => viz_param(i, (fr + 1.0) / 2.0).await,
                _ => {}
            }
            time.ctrl(&mut state, input);
            pads.ctrl(&mut state, input);
            beams.ctrl(&mut state, input);
            lasers.ctrl(&mut state, input);
            bars.ctrl(&mut state, input);
            pars.ctrl(&mut state, input);
            spiders.ctrl(&mut state, input);
            strobes.ctrl(&mut state, input);
        }

        let mut lights = Lights::default();

        if let Some(pad) = ctx.pad() {
            if let Some(ctrl) = ctx.ctrl() {
                time.output(&state, &mut lights, pad, ctrl).await;
                pads.output(&state, &mut lights, pad, ctrl).await;
                beams.output(&state, &mut lights, pad, ctrl).await;
                lasers.output(&state, &mut lights, pad, ctrl).await;
                bars.output(&state, &mut lights, pad, ctrl).await;
                pars.output(&state, &mut lights, pad, ctrl).await;
                spiders.output(&state, &mut lights, pad, ctrl).await;
                strobes.output(&state, &mut lights, pad, ctrl).await;
            }
        }

        viz_alpha(state.alpha * state.viz_alpha).await;
        viz_color(state.color0()).await;
        viz_pd(state.bpm, state.viz_pd.fr() * state.phi_mul).await;
        if state.viz_beat {
            let beat = state.phi(state.viz_pd).bsquare(1.0, 0.5);
            if beat == true && state.viz_beat_last == false {
                viz_beat().await;
            }
            state.viz_beat_last = beat;
        }

        if state.off {
            lights.brightness(0.0);
        } else {
            lights.brightness(state.alpha);
        }

        lights.write(&mut dmx);
        e131.send(dmx.buffer());
    }
}

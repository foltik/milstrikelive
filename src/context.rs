use std::thread;
use std::sync::mpsc;
use stagebridge::osc::{self, Osc};
use tokio::task;

use stagebridge::util::future::Broadcast;
// use stagebridge::osc::Osc;
use stagebridge::midi::Midi;
use stagebridge::midi::device::launchpad_x::{LaunchpadX, self};
use stagebridge::midi::device::launch_control_xl::{LaunchControlXL, self};

// use super::beatgrid::BeatGrid;
// use super::state::StateHolder;

pub struct Context {
    pad: Option<Midi<LaunchpadX>>,
    ctrl: Option<Midi<LaunchControlXL>>,

    // pub beats: BeatGrid,

    // pub state: StateHolder,
}

impl Context {
    pub async fn new() -> &'static Self {
        let pad = match Midi::<LaunchpadX>::open("Launchpad X:Launchpad X LPX MIDI") {
            Ok(pad) => {
                use stagebridge::midi::device::launchpad_x::{*, types::*};
                pad.send(Output::Mode(Mode::Programmer)).await;
                pad.send(Output::Pressure(Pressure::Off, PressureCurve::Medium)).await;
                pad.send(Output::Clear).await;
                Some(pad)
            },
            Err(e) => {
                log::warn!("Failed to open Launchpad: {:?}", e);
                None
            }
        };

        let ctrl = match Midi::<LaunchControlXL>::open("Launch Control XL:Launch Control XL") {
            Ok(ctrl) => Some(ctrl),
            Err(e) => {
                log::warn!("Failed to open LaunchControl: {:?}", e);
                None
            }
        };

        // let beats = BeatGrid::new(128);
        // beats.start(&osc);

        // let state = StateHolder::spawn();

        Box::leak(Box::new(Self {
            pad,
            ctrl,

            // beats,

            // state,
        }))
    }

    pub fn pad(&self) -> Option<&Midi<LaunchpadX>> {
        self.pad.as_ref()
    }

    pub fn ctrl(&self) -> Option<&Midi<LaunchControlXL>> {
        self.ctrl.as_ref()
    }

    pub fn subscribe_pad(&self) -> mpsc::Receiver<launchpad_x::Input> {
        match self.pad.as_ref() {
            Some(pad) => pad.subscribe_sync(),
            _ => {
                let (tx, rx) = mpsc::channel();
                thread::spawn(move || {
                    let _tx = tx;
                    thread::park();
                });
                rx
            }
        }
    }

    pub async fn send_pad(&self, output: launchpad_x::Output) {
        if let Some(pad) = self.pad.as_ref() {
            pad.send(output).await;
        }
    }

    pub fn subscribe_ctrl(&self) -> mpsc::Receiver<launch_control_xl::Input> {
        match self.ctrl.as_ref() {
            Some(ctrl) => ctrl.subscribe_sync(),
            _ => {
                let (tx, rx) = mpsc::channel();
                thread::spawn(move || {
                    let _tx = tx;
                    thread::park();
                });
                rx
            }
        }
    }

    pub async fn send_ctrl(&self, output: launch_control_xl::Output) {
        if let Some(ctrl) = self.ctrl.as_ref() {
            ctrl.send(output).await;
        }
    }
}

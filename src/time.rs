
use std::sync::Arc;
use atomic_float::AtomicF32;
use std::sync::atomic::{AtomicU16, Ordering};

use futures::Future;
use tokio::task::spawn;
use tokio::sync::{broadcast, mpsc};

type Fraction = fraction::Fraction;

use stagebridge::osc::{Osc, Message, Type};

#[derive(Clone)]
pub struct Time {
    bpm: f32,
    beat: u16,
    quantum: u16,
}

impl Time {
    pub fn new(quantum: u16) -> Self {
        let (tx, _) = broadcast::channel(16);

        Self {
            tx,
            bpm: Arc::new(AtomicF32::new(120.0)),
            beat: Arc::new(AtomicU16::new(0)),
            quantum,
        }
    }

    pub fn quantum(&self) -> u16 {
        self.quantum
    }

    pub fn bpm(&self) -> f32 {
        self.bpm.load(Ordering::Acquire)
    }

    pub fn beat(&self) -> u16 {
        self.beat.load(Ordering::Acquire)
    }

    pub fn start(&self, osc: &Osc) {
        let Self { tx, bpm, beat, .. } = (*self).clone();

        let mut rx = osc.subscribe();
        spawn(async move {
            loop {
                use broadcast::error::RecvError;
                match rx.recv().await {
                    Ok(Message { addr, args }) => match addr.as_str() {
                        "/bpm" => {
                            if let Some(Type::Float(f)) = args.get(0) {
                                log::debug!("BPM: {}", f);
                                bpm.store(*f, Ordering::Release);
                            } else {
                                log::warn!("invalid /bpm received: {:?}", args);
                            }
                        },
                        "/beats" => {
                            if let Some(Type::Int(i)) = args.get(0) {
                                let n = i.rem_euclid(std::u16::MAX as i32) as u16;
                                log::trace!("beat {}", n);
                                beat.store(n, Ordering::Release);
                                match tx.send(n) {
                                    Err(_) => log::trace!("Dropped beat {:?}", n),
                                    _ => {},
                                }
                            }
                        },
                        _ => {}
                    },
                    Err(RecvError::Lagged(n)) => log::warn!("osc receiver lagged by {}", n),
                    Err(RecvError::Closed) => log::error!("osc receiver closed"),
                }
            }
        });
    }

    pub fn subscribe(&self) -> mpsc::Receiver<u16> {
        let (tx, beat_rx) = mpsc::channel(16);

        let mut rx = self.tx.subscribe();
        spawn(async move {
            loop {
                use broadcast::error::RecvError;
                match rx.recv().await {
                    Ok(n) => match tx.send(n).await {
                        Err(_) => return,
                        _ => {}
                    },
                    Err(RecvError::Lagged(n)) => log::warn!("beat recv lagged by {:?}", n),
                    Err(RecvError::Closed) => log::error!("beat grid closed"),
                }
            }
        });

        beat_rx
    }

    pub fn subscribe_div(&self, num: u16, denom: u16) -> mpsc::Receiver<u16> {
        let div = (self.quantum * num) / denom;

        let (tx, beat_rx) = mpsc::channel(16);
        let mut rx = self.tx.subscribe();

        spawn(async move {
            loop {
                use broadcast::error::RecvError;
                match rx.recv().await {
                    Ok(i) => {
                        if i % div == 0 {
                            match tx.send(i / div).await {
                                Err(_) => return,
                                _ => {}
                            }
                        }
                    },
                    Err(RecvError::Lagged(n)) => log::warn!("beat recv lagged by {:?}", n),
                    Err(RecvError::Closed) => log::error!("beat recv closed"),
                }
            }
        });

        beat_rx
    }

    pub fn spawn<F, Fut>(&self, f: F)
    where
        F: FnOnce(Time) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send + 'static,
    {
        spawn(f((*self).clone()));
    }
}

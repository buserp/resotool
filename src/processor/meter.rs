use super::adsr::Stage;
use nice_plug::prelude::AtomicF32;
use std::sync::atomic::{AtomicU8, Ordering};

/// Voice state for the GUI, written by the audio thread
#[derive(Default)]
pub struct VoiceMeter {
    level: AtomicF32,
    stage: AtomicU8,
    freq: AtomicF32,
}

impl VoiceMeter {
    /// Envelope level in [0, 1]
    pub fn level(&self) -> f32 {
        self.level.load(Ordering::Relaxed)
    }

    pub fn stage(&self) -> Stage {
        self.stage.load(Ordering::Relaxed).into()
    }

    /// Resonator center in Hz
    pub fn freq(&self) -> f32 {
        self.freq.load(Ordering::Relaxed)
    }

    pub(super) fn set_envelope(&self, level: f32, stage: Stage) {
        self.level.store(level, Ordering::Relaxed);
        self.stage.store(stage as u8, Ordering::Relaxed);
    }

    pub(super) fn set_freq(&self, freq: f32) {
        self.freq.store(freq, Ordering::Relaxed);
    }
}

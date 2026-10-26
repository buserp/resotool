use super::meter::VoiceMeter;
use fundsp::DEFAULT_SR;
use fundsp::prelude32::{An, AudioNode, Frame, Shared, U1};
use std::sync::Arc;

#[derive(Clone)]
pub struct Adsr {
    pub attack: Shared,
    pub decay: Shared,
    pub sustain: Shared,
    pub release: Shared,
}

impl Default for Adsr {
    fn default() -> Self {
        Self {
            attack: Shared::new(1.0),
            decay: Shared::new(1.0),
            sustain: Shared::new(1.0),
            release: Shared::new(1.0),
        }
    }
}

/// ADSR envelope. Input: gate (> 0 = held). Output: level in [0, 1]
pub fn adsr_shared(params: Adsr, meter: Arc<VoiceMeter>) -> An<AdsrNode> {
    An(AdsrNode {
        params,
        meter,
        stage: Stage::Idle,
        level: 0.0,
        release_from: 0.0,
        sample_rate: DEFAULT_SR as f32,
    })
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Stage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

impl From<u8> for Stage {
    fn from(other: u8) -> Stage {
        match other {
            1 => Stage::Attack,
            2 => Stage::Decay,
            3 => Stage::Sustain,
            4 => Stage::Release,
            _ => Stage::Idle,
        }
    }
}

#[derive(Clone)]
pub struct AdsrNode {
    params: Adsr,
    meter: Arc<VoiceMeter>,
    stage: Stage,
    level: f32,
    // Level at note off, so Release is always the full fade time
    release_from: f32,
    sample_rate: f32,
}

impl AudioNode for AdsrNode {
    const ID: u64 = 0x5245_534f_4144_5352;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self) {
        self.stage = Stage::Idle;
        self.level = 0.0;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate as f32;
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let held = input[0] > 0.0;
        // Stages continue from the current level, so retrigger and early release don't jump
        match self.stage {
            Stage::Idle | Stage::Release if held => self.stage = Stage::Attack,
            Stage::Attack | Stage::Decay | Stage::Sustain if !held => {
                self.release_from = self.level;
                self.stage = Stage::Release;
            }
            _ => {}
        }

        let sustain = self.params.sustain.value();
        match self.stage {
            Stage::Idle => self.level = 0.0,
            Stage::Attack => {
                self.level += self.step(1.0, self.params.attack.value());
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.stage = Stage::Decay;
                }
            }
            Stage::Decay => {
                self.level -= self.step(1.0 - sustain, self.params.decay.value());
                if self.level <= sustain {
                    self.level = sustain;
                    self.stage = Stage::Sustain;
                }
            }
            Stage::Sustain => self.level = sustain,
            Stage::Release => {
                self.level -= self.step(self.release_from, self.params.release.value());
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.stage = Stage::Idle;
                }
            }
        }
        self.meter.set_envelope(self.level, self.stage);
        [self.level].into()
    }
}

impl AdsrNode {
    /// Per-sample change to cover `distance` in `seconds`, all of it at once if `seconds` is 0
    fn step(&self, distance: f32, seconds: f32) -> f32 {
        if seconds > 0.0 {
            distance / (seconds * self.sample_rate)
        } else {
            distance
        }
    }
}

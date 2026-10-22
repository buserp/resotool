use fundsp::prelude32::{An, EnvelopeIn, Frame, Shared, U1, clamp01, envelope2, lerp, shared, var};

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

pub fn adsr_shared(
    attack: Shared,
    decay: Shared,
    sustain: Shared,
    release: Shared,
) -> An<EnvelopeIn<f32, impl FnMut(f32, &Frame<f32, U1>) -> f32 + Clone, U1, f32>> {
    let a = shared(-1.0);
    let b = shared(-1.0);
    let attack_start = var(&a);
    let release_start = var(&b);
    envelope2(move |time, control| {
        if attack_start.value() < 0.0 && control > 0.0 {
            attack_start.set_value(time);
            release_start.set_value(-1.0);
        } else if release_start.value() < 0.0 && control <= 0.0 {
            release_start.set_value(time);
            attack_start.set_value(-1.0);
        }
        clamp01(if release_start.value() < 0.0 {
            ads(&attack, &decay, &sustain, time - attack_start.value())
        } else {
            releasing(&sustain, &release, time - release_start.value())
        })
    })
}

fn ads(attack: &Shared, decay: &Shared, sustain: &Shared, time: f32) -> f32 {
    if time < attack.value() {
        lerp(0.0, 1.0, time / attack.value())
    } else {
        let decay_time = time - attack.value();
        if decay_time < decay.value() {
            lerp(1.0, sustain.value(), decay_time / decay.value())
        } else {
            sustain.value()
        }
    }
}

fn releasing(sustain: &Shared, release: &Shared, release_time: f32) -> f32 {
    if release_time > release.value() {
        0.0
    } else {
        lerp(sustain.value(), 0.0, release_time / release.value())
    }
}

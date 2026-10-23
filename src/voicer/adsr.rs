use fundsp::prelude32::{An, EnvelopeIn, Frame, Shared, U1, clamp01, envelope2, lerp};

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
    // Stage start times, `None` if inactive. Both `None` = idle/silent
    let mut attack_start: Option<f32> = None;
    let mut release_start: Option<f32> = None;
    // Level at note off, release fades from here
    let mut release_level = 0.0;
    envelope2(move |time, control| {
        match attack_start {
            None if control > 0.0 => {
                // Resume attack from current level by backdating its start (avoids retrigger click)
                let level = release_start.map_or(0.0, |start| {
                    clamp01(releasing(release_level, &release, time - start))
                });
                attack_start = Some(time - attack.value() * level);
                release_start = None;
            }
            Some(start) if control <= 0.0 => {
                // Key up: save current level
                release_level = clamp01(ads(&attack, &decay, &sustain, time - start));
                release_start = Some(time);
                attack_start = None;
            }
            _ => {}
        }
        clamp01(match (attack_start, release_start) {
            (Some(start), _) => ads(&attack, &decay, &sustain, time - start),
            (None, Some(start)) => releasing(release_level, &release, time - start),
            (None, None) => 0.0,
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

fn releasing(level: f32, release: &Shared, release_time: f32) -> f32 {
    if release_time >= release.value() {
        0.0
    } else {
        lerp(level, 0.0, release_time / release.value())
    }
}

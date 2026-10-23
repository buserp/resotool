use fundsp::prelude32::*;
use nice_plug::util::midi_key_to_freq;

mod adsr;

// Make sure to change these together
const NUM_VOICES: usize = 8;
type VoiceSize = U8;

pub struct Processor {
    /// Note event counter, for voice age
    clock: u32,
    voices: Box<[Voice; NUM_VOICES]>,
    wetdry: Shared,
    makeup: Shared,
    // Shared by all voices
    bandwidth: Shared,
    adsr: adsr::Adsr,
    pub audio: Box<dyn AudioUnit>,
}

pub struct ProcessorParams {
    pub wetdry: f32,
    pub makeup: f32,
    pub bandwidth: f32,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

impl Processor {
    pub fn update(&self, params: ProcessorParams) {
        self.wetdry.set(params.wetdry);
        self.makeup.set(params.makeup);
        self.bandwidth.set(params.bandwidth);
        self.adsr.attack.set(params.attack);
        self.adsr.decay.set(params.decay);
        self.adsr.sustain.set(params.sustain);
        self.adsr.release.set(params.release);
    }

    pub fn note_on(&mut self, midi_note: u8) {
        self.clock += 1;
        let voices = &self.voices;
        let index = voices
            .iter()
            // Reuse voice already on this note
            .position(|voice| voice.note == Some(midi_note))
            // Else free voice released longest ago (quietest tail)
            .or_else(|| {
                (0..NUM_VOICES)
                    .filter(|&i| !voices[i].sounding)
                    .min_by_key(|&i| voices[i].last_released)
            })
            // Else steal oldest held voice
            .unwrap_or_else(|| {
                (0..NUM_VOICES)
                    .min_by_key(|&i| voices[i].last_played)
                    .expect("Voices is not empty")
            });
        self.voices[index].note_on(midi_note, self.clock);
    }

    pub fn note_off(&mut self, midi_note: u8) {
        self.clock += 1;
        for voice in self.voices.iter_mut() {
            if voice.sounding && voice.note == Some(midi_note) {
                voice.note_off(self.clock);
            }
        }
    }

    pub fn new() -> Self {
        let voices: [Voice; NUM_VOICES] = std::array::from_fn(|_| Voice::default());
        let wetdry = Shared::new(1.0);
        let makeup = Shared::new(1.0);
        let bandwidth = Shared::new(100.0);
        let adsr = adsr::Adsr::default();
        let filterbank = sumi::<VoiceSize, _, _>(|i| {
            let voice = &voices[i as usize];
            (pass()
                * (var(&voice.trigger)
                    >> adsr::adsr_shared(adsr.clone()))
                | var(&voice.cutoff)
                // The bandpass takes Q rather than bandwidth in Hz
                | ((var(&voice.cutoff) | var(&bandwidth))
                    >> map(|f: &Frame<f32, U2>| f[0] / f[1])))
                >> bandpass()
                // The bandpass peaks at Q, so scale by 1 / Q for unity gain at the center
                * ((var(&bandwidth) | var(&voice.cutoff))
                    >> map(|f: &Frame<f32, U2>| f[0] / f[1]))
        });
        // The filterbank has unity peak gain, so broadband input comes out much quieter
        // Makeup gain brings it back up, and the soft clipper gently shapes it
        let wet = ((split() >> filterbank) * var(&makeup) >> shape(Tanh(1.0))) * var(&wetdry);
        let dry = pass() * var_fn(&wetdry, |wd| 1.0 - wd);
        let audio = Box::new(wet & dry);
        Self {
            audio,
            clock: 0,
            voices: Box::new(voices),
            wetdry,
            makeup,
            bandwidth,
            adsr,
        }
    }
}

struct Voice {
    last_played: u32,
    last_released: u32,
    /// Current note, `None` if never played
    note: Option<u8>,
    sounding: bool,
    trigger: Shared,
    cutoff: Shared,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            last_played: 0,
            last_released: 0,
            note: None,
            sounding: false,
            trigger: Shared::new(0.0),
            cutoff: Shared::new(440.0),
        }
    }
}

impl Voice {
    fn note_on(&mut self, midi_note: u8, last_played: u32) {
        self.trigger.set(1.0);
        self.note = Some(midi_note);
        self.sounding = true;
        self.cutoff.set(midi_key_to_freq(midi_note));
        self.last_played = last_played;
    }

    fn note_off(&mut self, last_released: u32) {
        self.trigger.set(0.0);
        self.sounding = false;
        self.last_released = last_released;
    }
}

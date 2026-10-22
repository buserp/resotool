use nice_plug::midi::Key;
use nice_plug::prelude::*;
use std::sync::Arc;
use voicer::{Voicer, VoicerParams};

mod voicer;

struct Resotool {
    params: Arc<ResotoolParams>,
    voicer: Voicer,
}

#[derive(Params)]
struct ResotoolParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish.
    #[id = "cutoff"]
    pub bandwidth: FloatParam,
    #[id = "attack"]
    pub attack: FloatParam,
    #[id = "decay"]
    pub decay: FloatParam,
    #[id = "sustain"]
    pub sustain: FloatParam,
    #[id = "release"]
    pub release: FloatParam,
    #[id = "wetdry"]
    pub wetdry: FloatParam,
    #[id = "makeup"]
    pub makeup: FloatParam,
}

impl Default for Resotool {
    fn default() -> Self {
        let params = ResotoolParams::default();
        Self {
            params: Arc::new(params),
            voicer: Voicer::new(),
        }
    }
}

fn adsr_param(name: &str, default: f32) -> FloatParam {
    FloatParam::new(
        name,
        default,
        FloatRange::Skewed {
            min: 0.0,
            max: 10.0,
            factor: 0.5,
        },
    )
    .with_unit("seconds")
    .with_step_size(0.01)
    .with_value_to_string(formatters::v2s_f32_rounded(2))
}

impl Default for ResotoolParams {
    fn default() -> Self {
        Self {
            bandwidth: FloatParam::new(
                "Bandwidth",
                10.0,
                FloatRange::Skewed {
                    min: 0.1,
                    max: 15.0,
                    factor: 0.5,
                },
            )
            .with_step_size(0.01)
            .with_value_to_string(formatters::v2s_f32_rounded(2))
            .with_unit("Hz"),
            attack: adsr_param("Attack", 0.5),
            decay: adsr_param("Decay", 0.25),
            sustain: FloatParam::new("Sustain", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_unit("%")
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),
            release: adsr_param("Release", 0.75),
            wetdry: FloatParam::new("Wet/Dry", 0.75, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_unit("%")
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),
            // Stored as linear gain, displayed in decibels
            makeup: FloatParam::new(
                "Makeup",
                util::db_to_gain(24.0),
                FloatRange::gain_range(0.0, 48.0),
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(1))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
        }
    }
}

impl Plugin for Resotool {
    const NAME: &'static str = "Resotool";
    const VENDOR: &'static str = "Paul Buser";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "paul@beepyversion.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(1),
        main_output_channels: NonZeroU32::new(1),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // Plugins without an editor set this to `()`.
    type Editor = ();
    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn activate(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl ActivateContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        self.voicer
            .audio
            .set_sample_rate(_buffer_config.sample_rate.into());
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let voicer_params = VoicerParams {
            wetdry: self.params.wetdry.value(),
            makeup: self.params.makeup.value(),
            bandwidth: self.params.bandwidth.value(),
            attack: self.params.attack.value(),
            decay: self.params.decay.value(),
            sustain: self.params.sustain.value(),
            release: self.params.release.value(),
        };
        self.voicer.update(voicer_params);
        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::NoteOn {
                    key: Key::Number(note),
                    ..
                } => {
                    self.voicer.note_on(note);
                }
                NoteEvent::NoteOff {
                    key: Key::Number(note),
                    ..
                } => {
                    self.voicer.note_off(note);
                }
                _ => (),
            }
        }
        for channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves
            let output_buffer = &mut [0f32; 1];
            for sample in channel_samples {
                self.voicer.audio.tick(&[*sample], output_buffer);
                *sample = output_buffer[0];
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Resotool {
    const CLAP_ID: &'static str = "com.beepyversion.resotool";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("MIDI controllable resonators");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Resotool {
    const VST3_CLASS_ID: [u8; 16] = *b"ResoToolCoolTool";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nice_export_clap!(Resotool);
nice_export_vst3!(Resotool);

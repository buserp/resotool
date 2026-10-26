use crate::ResotoolParams;
use crate::processor::{NUM_VOICES, Stage, VoiceMeter};
use egui::{Color32, Rect, vec2};
use nice_plug::context::gui::GuiContext;
use nice_plug::editor::dpi::LogicalSize;
use nice_plug::prelude::*;
use nice_plug_egui::{Frame, NiceEguiApp, widgets::ParamSlider};
use std::sync::Arc;

mod piano;
mod resonators;

pub const WINDOW_SIZE: LogicalSize<f32> = LogicalSize::new(720.0, 420.0);

// Layout, in logical pixels
const MARGIN: i8 = 12;
const HEADING_GAP: f32 = 6.0;
const CHART_HEIGHT: f32 = 220.0;
const PIANO_HEIGHT: f32 = 36.0;
const CONTROLS_GAP: f32 = 10.0;
const GRID_SPACING: [f32; 2] = [12.0, 6.0];
const SLIDER_WIDTH: f32 = 180.0;

// Frequency axis, log scale
const F_MIN: f32 = 20.0;
const F_MAX: f32 = 20_000.0;

const ATTACK: Color32 = Color32::from_rgb(216, 90, 48);
const SUSTAIN: Color32 = Color32::from_rgb(29, 158, 117);
const RELEASE: Color32 = Color32::from_rgb(127, 119, 221);

pub struct ResotoolEditor {
    params: Arc<ResotoolParams>,
    meters: [Arc<VoiceMeter>; NUM_VOICES],
    gui_ctx: Option<GuiContext>,
}

/// A sounding voice, read from its meter
struct Voice {
    freq: f32,
    level: f32,
    stage: Stage,
}

impl ResotoolEditor {
    pub fn new(params: Arc<ResotoolParams>, meters: [Arc<VoiceMeter>; NUM_VOICES]) -> Self {
        Self {
            params,
            meters,
            gui_ctx: None,
        }
    }
}

impl NiceEguiApp for ResotoolEditor {
    fn build(
        &mut self,
        _egui_ctx: egui::Context,
        gui_ctx: GuiContext,
        _frame: &mut Frame,
    ) -> Result<(), nice_plug_egui::baseview::HandlerError> {
        self.gui_ctx = Some(gui_ctx);
        Ok(())
    }

    fn editor_closed(&mut self) {
        self.gui_ctx = None;
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        let Some(gui_ctx) = &self.gui_ctx else {
            return;
        };
        let setter = gui_ctx.param_setter();
        let voices = self.voices();
        // Repaint while voices sound, one more frame clears the last
        if !voices.is_empty() {
            ui.ctx().request_repaint();
        }

        egui::Frame::new().inner_margin(MARGIN).show(ui, |ui| {
            ui.heading("Resotool");
            ui.add_space(HEADING_GAP);
            let width = ui.available_width();
            let bandwidth = self.params.bandwidth.value();
            resonators::resonators(ui, vec2(width, CHART_HEIGHT), &voices, bandwidth);
            piano::piano(ui, vec2(width, PIANO_HEIGHT), &voices);
            ui.add_space(CONTROLS_GAP);
            self.controls(ui, &setter);
        });
    }
}

impl ResotoolEditor {
    fn voices(&self) -> Vec<Voice> {
        self.meters
            .iter()
            .map(|meter| Voice {
                freq: meter.freq(),
                level: meter.level(),
                stage: meter.stage(),
            })
            .filter(|voice| voice.stage != Stage::Idle)
            .collect()
    }

    fn controls(&self, ui: &mut egui::Ui, setter: &ParamSetter) {
        let params = &self.params;
        egui::Grid::new("controls")
            .num_columns(4)
            .spacing(GRID_SPACING)
            .show(ui, |ui| {
                for row in [
                    [(&params.bandwidth, "Bandwidth"), (&params.attack, "Attack")],
                    [(&params.makeup, "Makeup Gain"), (&params.decay, "Decay")],
                    [(&params.wetdry, "Wet/dry"), (&params.sustain, "Sustain")],
                ] {
                    for (param, label) in row {
                        ui.label(label);
                        ui.add(ParamSlider::for_param(param, setter).with_width(SLIDER_WIDTH));
                    }
                    ui.end_row();
                }
                ui.label("");
                ui.label("");
                ui.label("Release");
                ui.add(ParamSlider::for_param(&params.release, setter).with_width(SLIDER_WIDTH));
                ui.end_row();
            });
    }
}

fn stage_color(stage: Stage) -> Color32 {
    match stage {
        Stage::Attack | Stage::Decay => ATTACK,
        Stage::Release => RELEASE,
        Stage::Sustain | Stage::Idle => SUSTAIN,
    }
}

fn x_of(freq: f32, plot: Rect) -> f32 {
    plot.min.x + (freq / F_MIN).ln() / (F_MAX / F_MIN).ln() * plot.width()
}

fn freq_of(x: f32, plot: Rect) -> f32 {
    F_MIN * (F_MAX / F_MIN).powf((x - plot.min.x) / plot.width())
}

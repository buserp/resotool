use super::{Voice, stage_color, x_of};
use egui::{Rect, Sense, StrokeKind, Vec2};

// Key centers in an octave, in white key widths
// Black keys sit off-center, like on a real piano
const KEY_CENTERS: [f32; 12] = [0.5, 0.9, 1.5, 2.1, 2.5, 3.5, 3.85, 4.5, 5.0, 5.5, 6.15, 6.5];
// Black key size, relative to white
const BLACK_WIDTH: f32 = 0.6;
const BLACK_LENGTH: f32 = 0.65;

// 88 keys of a piano (A0 to C8) as MIDI notes
const FIRST_NOTE: u8 = 21;
const LAST_NOTE: u8 = 108;

/// Piano on the frequency axis, keys lit by sounding voices
pub(super) fn piano(ui: &mut egui::Ui, size: Vec2, voices: &[Voice]) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    let visuals = ui.visuals();
    let white = visuals.extreme_bg_color;
    let black = visuals.widgets.inactive.bg_fill;
    let line = visuals.widgets.noninteractive.bg_stroke;
    // Same horizontal extent as the resonator plot
    let plot = rect;

    let lit = |note: u8| {
        voices
            .iter()
            .find(|voice| freq_note(voice.freq) == note)
            .map(|voice| stage_color(voice.stage))
    };
    // Each octave spans C - ½ to B + ½ semitone, split into 7 equal white keys
    let span = |note: u8| {
        let width = if is_black(note) { BLACK_WIDTH } else { 1.0 };
        let center = KEY_CENTERS[note as usize % 12];
        let x = |pos: f32| {
            let octave_start = (note / 12 * 12) as f32 - 0.5;
            x_of(note_freq(octave_start + pos / 7.0 * 12.0), plot)
        };
        x(center - width / 2.0)..=x(center + width / 2.0)
    };

    for note in (FIRST_NOTE..=LAST_NOTE).filter(|&note| !is_black(note)) {
        let key = Rect::from_x_y_ranges(span(note), rect.y_range());
        painter.rect_filled(key, 0.0, lit(note).unwrap_or(white));
        painter.rect_stroke(key, 0.0, line, StrokeKind::Inside);
    }

    let black_height = rect.height() * BLACK_LENGTH;
    for note in (FIRST_NOTE..=LAST_NOTE).filter(|&note| is_black(note)) {
        let key = Rect::from_x_y_ranges(span(note), rect.min.y..=rect.min.y + black_height);
        painter.rect_filled(key, 0.0, lit(note).unwrap_or(black));
        painter.rect_stroke(key, 0.0, line, StrokeKind::Inside);
    }
}

fn is_black(note: u8) -> bool {
    matches!(note % 12, 1 | 3 | 6 | 8 | 10)
}

fn note_freq(note: f32) -> f32 {
    440.0 * 2f32.powf((note - 69.0) / 12.0)
}

fn freq_note(freq: f32) -> u8 {
    (69.0 + 12.0 * (freq / 440.0).log2())
        .round()
        .clamp(0.0, 127.0) as u8
}

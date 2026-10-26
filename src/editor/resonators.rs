use super::{Voice, freq_of, stage_color, x_of};
use egui::{Align2, FontId, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Vec2, pos2};

// Real peaks are hairline thin on a log axis
// Draw them at least this wide (fraction of center)
const MIN_DISPLAY_WIDTH: f32 = 0.03;

/// Log frequency grid, one bandpass curve per voice scaled by its envelope
pub(super) fn resonators(ui: &mut egui::Ui, size: Vec2, voices: &[Voice], bandwidth: f32) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    let visuals = ui.visuals();
    let grid = visuals.widgets.noninteractive.bg_stroke;
    let text = visuals.weak_text_color();
    painter.rect_filled(rect, 4.0, visuals.extreme_bg_color);

    let plot = Rect::from_min_max(rect.min, pos2(rect.max.x, rect.max.y - 14.0));
    for f in [50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10_000.0] {
        let x = x_of(f, plot);
        painter.line_segment([pos2(x, plot.min.y), pos2(x, plot.max.y)], grid);
        let label = if f >= 1000.0 {
            format!("{}k", f / 1000.0)
        } else {
            format!("{f}")
        };
        painter.text(
            pos2(x + 2.0, rect.max.y - 1.0),
            Align2::LEFT_BOTTOM,
            label,
            FontId::proportional(11.0),
            text,
        );
    }

    for voice in voices {
        let color = stage_color(voice.stage);
        let q = voice.freq / bandwidth.max(voice.freq * MIN_DISPLAY_WIDTH);
        let y_of = |gain: f32| plot.max.y - 0.92 * plot.height() * gain * voice.level;
        let peak = pos2(x_of(voice.freq, plot), y_of(1.0));
        let mut points: Vec<Pos2> = (0..=plot.width() as usize)
            .map(|i| {
                let x = plot.min.x + i as f32;
                pos2(x, y_of(bandpass_gain(freq_of(x, plot), voice.freq, q)))
            })
            .collect();
        // Peaks are a few pixels wide, sample the exact top so none get clipped
        let at = points.partition_point(|point| point.x < peak.x);
        points.insert(at, peak);
        painter.add(Shape::line(points, Stroke::new(1.5, color)));
        painter.circle_filled(peak, 3.5, color);
    }
    painter.rect_stroke(rect, 4.0, grid, StrokeKind::Inside);
}

/// Unity-peak bandpass magnitude at `freq`
fn bandpass_gain(freq: f32, center: f32, q: f32) -> f32 {
    let detune = freq / center - center / freq;
    1.0 / (1.0 + q * q * detune * detune).sqrt()
}

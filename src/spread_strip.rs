//! The spread strip: a thin bar just above the bottom edge of a fuzzy number's center input,
//! spanning center ± spread against the input's range. Flat for a uniform shape; for a bell, its
//! opacity follows the bell's density, fading toward the ends. A wrapping range's spread can cross
//! the seam, and then the strip is two segments, one at each end.

use std::f32::consts::FRAC_PI_2;

use bevy::{
    prelude::*,
    ui::{BackgroundGradient, ColorStop, Gradient, InterpolationColorSpace, LinearGradient},
};
use bevy_immediate::Imm;

use crate::{
    fuzzy::{Distribution, Fuzzy},
    properties::CapsPanel,
};

/// Light enough to show over both the input's fill bar and its background.
const STRIP_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.8);

/// Lifts the strip off the bottom edge, so it spans the input's full width yet clears its rounded
/// corners: 2 px up, a 4 px radius curves in by under a pixel.
const BOTTOM_LIFT: f32 = 2.0;

/// The narrowest a segment is drawn, and the width of the tick shown for no spread.
const TICK_WIDTH: f32 = 2.0;

/// Stops per bell, one every twelfth of the spread; the density between them is drawn as linear.
const BELL_STOPS: usize = 12;

/// The input's range, which the strip spans.
#[derive(Clone, Copy, Debug)]
pub struct StripRange {
    pub start: f32,
    pub end: f32,
    /// Values past one end come round from the other, as for an angle of a full turn.
    pub wrap: bool,
}

/// A visible piece of the strip.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Segment {
    /// Where it starts and how wide it is, as fractions of the range.
    left: f32,
    width: f32,
    /// The part of the spread it shows, 0 at center − spread and 1 at center + spread.
    from: f32,
    to: f32,
}

/// Adds the strip to the number input being built. Its nodes are updated only when it changes.
pub fn spread_strip(ui: &mut Imm<CapsPanel>, fuzzy: Fuzzy, range: StripRange) {
    ui.ch_id("spread_strip")
        .on_spawn_insert(|| {
            (
                Name::new("spread_strip"),
                Node {
                    position_type: PositionType::Absolute,
                    // The full width, so the strip lines up with the fill bar.
                    left: px(0),
                    right: px(0),
                    bottom: px(BOTTOM_LIFT),
                    height: px(2),
                    ..default()
                },
                // Above the text input whichever order the input's children are in.
                ZIndex(1),
                // Scrubbing and clicks go to the input beneath.
                Pickable::IGNORE,
            )
        })
        .add(|ui| {
            for (index, segment) in segments(&fuzzy, &range).into_iter().enumerate() {
                let key = (
                    segment.left.to_bits(),
                    segment.width.to_bits(),
                    segment.from.to_bits(),
                    segment.to.to_bits(),
                    fuzzy.shape,
                );
                ui.ch_id(index)
                    .on_spawn_insert(move || Name::new(format!("segment{index}")))
                    .on_hash_change_insert("segment", &key, || {
                        segment_bundle(segment, fuzzy.shape)
                    });
            }
        });
}

fn segment_bundle(segment: Segment, shape: Distribution) -> impl Bundle {
    let tick = segment.width == 0.0;
    (
        Node {
            position_type: PositionType::Absolute,
            left: percent(segment.left * 100.0),
            width: percent(segment.width * 100.0),
            // A spread too small to see still shows; a tick is centred on its value.
            min_width: px(TICK_WIDTH),
            margin: UiRect::left(px(if tick { -TICK_WIDTH / 2.0 } else { 0.0 })),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        BackgroundGradient(vec![Gradient::Linear(LinearGradient {
            angle: FRAC_PI_2,
            stops: stops(segment, shape),
            color_space: InterpolationColorSpace::Srgba,
        })]),
        Pickable::IGNORE,
    )
}

/// Color stops across a segment, its opacity following the shape's density. A tick is solid.
fn stops(segment: Segment, shape: Distribution) -> Vec<ColorStop> {
    if segment.to <= segment.from {
        return vec![
            ColorStop::new(STRIP_COLOR, percent(0)),
            ColorStop::new(STRIP_COLOR, percent(100)),
        ];
    }
    let mut points = vec![segment.from];
    if shape == Distribution::Bell {
        points.extend(
            (1..BELL_STOPS)
                .map(|i| i as f32 / BELL_STOPS as f32)
                .filter(|&t| t > segment.from && t < segment.to),
        );
    }
    points.push(segment.to);
    let length = segment.to - segment.from;
    points
        .into_iter()
        .map(|t| {
            let alpha = STRIP_COLOR.alpha() * shape.density(t);
            ColorStop::new(
                STRIP_COLOR.with_alpha(alpha),
                percent((t - segment.from) / length * 100.0),
            )
        })
        .collect()
}

/// The pieces of center ± spread that fall within the range: clamped to its ends, or, if it
/// wraps, the overhang brought round to the other end. With no spread, a tick at the center, of
/// no width (the node gives it one).
fn segments(fuzzy: &Fuzzy, range: &StripRange) -> Vec<Segment> {
    let span = range.end - range.start;
    if span <= 0.0 {
        return Vec::new();
    }
    if fuzzy.spread <= 0.0 {
        return vec![Segment {
            left: ((fuzzy.center - range.start) / span).clamp(0.0, 1.0),
            width: 0.0,
            from: 0.5,
            to: 0.5,
        }];
    }
    let (low, high) = (fuzzy.center - fuzzy.spread, fuzzy.center + fuzzy.spread);
    let turns: &[f32] = if range.wrap {
        &[-1.0, 0.0, 1.0]
    } else {
        &[0.0]
    };
    turns
        .iter()
        .filter_map(|&turn| {
            let (low, high) = (low + turn * span, high + turn * span);
            let (start, end) = (low.max(range.start), high.min(range.end));
            (end > start).then(|| Segment {
                left: (start - range.start) / span,
                width: (end - start) / span,
                from: (start - low) / (high - low),
                to: (end - low) / (high - low),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fuzzy(center: f32, spread: f32) -> Fuzzy {
        Fuzzy::new(center, spread, Distribution::Uniform)
    }

    const UNIT: StripRange = StripRange {
        start: 0.0,
        end: 10.0,
        wrap: false,
    };

    const TURN: StripRange = StripRange {
        start: 0.0,
        end: 360.0,
        wrap: true,
    };

    #[test]
    fn inside_the_range() {
        let segments = segments(&fuzzy(5.0, 1.0), &UNIT);
        assert_eq!(
            segments,
            [Segment {
                left: 0.4,
                width: 0.2,
                from: 0.0,
                to: 1.0
            }]
        );
    }

    #[test]
    fn clamped_at_an_end() {
        let segments = segments(&fuzzy(9.0, 2.0), &UNIT);
        assert_eq!(
            segments,
            [Segment {
                left: 0.7,
                width: 0.3,
                from: 0.0,
                to: 0.75
            }]
        );
    }

    #[test]
    fn no_spread_a_tick_at_the_center() {
        let segments = segments(&fuzzy(5.0, 0.0), &UNIT);
        assert_eq!(
            segments,
            [Segment {
                left: 0.5,
                width: 0.0,
                from: 0.5,
                to: 0.5
            }]
        );
        assert_eq!(stops(segments[0], Distribution::Bell)[0].color, STRIP_COLOR);
    }

    #[test]
    fn wrapping_across_the_seam() {
        let segments = segments(&fuzzy(10.0, 30.0), &TURN);
        assert_eq!(segments.len(), 2);
        // The overhang below zero comes round to the top end.
        let top = segments.iter().find(|s| s.left > 0.5).unwrap();
        let bottom = segments.iter().find(|s| s.left == 0.0).unwrap();
        assert!((top.left - 340.0 / 360.0).abs() < 1e-6);
        assert!((top.to - 1.0 / 3.0).abs() < 1e-6);
        assert!((bottom.width - 40.0 / 360.0).abs() < 1e-6);
        assert!((bottom.from - 1.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn bell_stops_fade_to_clear_at_the_ends() {
        let segment = Segment {
            left: 0.0,
            width: 1.0,
            from: 0.0,
            to: 1.0,
        };
        let stops = stops(segment, Distribution::Bell);
        assert_eq!(stops.len(), BELL_STOPS + 1);
        assert_eq!(stops[0].color.alpha(), 0.0);
        assert_eq!(stops[BELL_STOPS / 2].color.alpha(), STRIP_COLOR.alpha());
    }
}

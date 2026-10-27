//! The curve editor in each configuration it supports, each beside its curve as an app might
//! store it. The editor edits a `ScalarCurve`; what the app stores is its own choice, holding only
//! what its constraints allow.

use arbor_day::curve::{
    CurveConstraints, CurveEditorFormat, CurveExtrapolation, CurvePlugin, CurvePoint, CurveSegment, CurveValue,
    DirectionHandle, HandleMode, PinEnds, ScalarCurve, curve_editor, curve_self_update,
};
use bevy::{
    feathers::{
        FeathersPlugins,
        constants::{fonts, size},
        dark_theme::create_dark_theme,
        display::label,
        theme::{ThemeBackgroundColor, ThemedText, UiTheme},
        tokens,
    },
    prelude::*,
    text::FontSourceTemplate,
};
use bevy_remote_driver::RemoteDriverPlugin;

/// The editor column's width, the panel's in arbor_day.
const EDITOR_WIDTH: f32 = 360.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Curve editor".into(),
                    ..default()
                }),
                ..default()
            }),
            FeathersPlugins,
            CurvePlugin,
            RemoteDriverPlugin,
        ))
        .insert_resource(UiTheme(create_dark_theme()))
        .add_systems(Startup, ui.spawn())
        .add_systems(Update, show_stored)
        .run();
}

/// How a row's app stores its curve.
#[derive(Component, Clone, Copy, Debug, Default)]
enum Stored {
    /// `[x, y]` pairs, as arbor_day stores a place along a parent: its constraints allow cubic
    /// segments with auto handles only, so the points are the whole curve.
    #[default]
    Pairs,
    /// CSS's `cubic-bezier(x1, y1, x2, y2)`: its constraints allow one cubic segment between
    /// pinned ends, so its two inner control points are the whole curve.
    CubicBezier,
}

impl Stored {
    /// How the row's editor writes its values: a `cubic-bezier`'s points are its pinned ends, so
    /// it has no point inputs.
    fn format(self) -> CurveEditorFormat {
        CurveEditorFormat {
            hide_point_inputs: matches!(self, Self::CubicBezier),
            ..default()
        }
    }

    fn write(self, curve: &ScalarCurve) -> String {
        match self {
            Self::Pairs => {
                let pairs: Vec<String> = (curve.points.iter())
                    .map(|point| format!("  [{}, {}],", point.x, point.y))
                    .collect();
                format!("[\n{}\n]", pairs.join("\n"))
            }
            Self::CubicBezier => {
                let prepared = curve.prepare();
                let Some(segment) = prepared.segments().first() else {
                    return String::new();
                };
                let [_, a, b, _] = segment.controls;
                // To four places, so a sum of rounded offsets prints as typed.
                let short = |value: f32| (value * 1e4).round() / 1e4;
                format!(
                    "cubic-bezier({}, {}, {}, {})",
                    short(a.x),
                    short(a.y),
                    short(b.x),
                    short(b.y)
                )
            }
        }
    }
}

/// The text showing its row's curve as stored.
#[derive(Component, Clone, Default)]
struct StoredText;

fn ui() -> impl SceneList {
    bsn_list! {
        Camera2d
        --
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(16)),
            row_gap: px(24),
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children [
            @row("auto", "Implicit slopes", auto_curve(), auto_constraints(), Stored::Pairs)
            --
            @row(
                "bezier",
                "One cubic, its handles aligned",
                ease_curve(),
                bezier_constraints(),
                Stored::CubicBezier
            )
        ]
    }
}

/// A row named `name`: an editor captioned `caption` showing `curve`, which it edits within
/// `constraints`, beside the curve as `stored`.
fn row(
    name: &'static str,
    caption: &'static str,
    curve: ScalarCurve,
    constraints: CurveConstraints,
    stored: Stored,
) -> impl Scene {
    bsn! {
        Name({name})
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(24),
        }
        Children [
            Node {
                width: px(EDITOR_WIDTH),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
            }
            Children [
                @label(caption)
                --
                @curve_editor("x", "y")
                #editor
                ~{CurveValue(curve.clone())}
                ~{constraints.clone()}
                ~{stored}
                ~{stored.format()}
                on(curve_self_update)
            ]
            --
            Text("")
            TextFont {
                font: FontSourceTemplate::Handle(fonts::MONO),
                font_size: size::COMPACT_FONT,
            }
            ThemedText
            StoredText
            #stored
        ]
    }
}

/// A curve through a few points, its slopes implied by them.
fn auto_curve() -> ScalarCurve {
    ScalarCurve {
        points: [(0.0, 0.8), (0.4, 0.3), (1.0, 0.6)]
            .into_iter()
            .map(|(x, y)| CurvePoint { x, y, ..default() })
            .collect(),
        extrapolation: CurveExtrapolation::Hold,
    }
}

/// Cubic segments with auto handles, and nothing else, as arbor_day's curves allow.
fn auto_constraints() -> CurveConstraints {
    CurveConstraints {
        segments: vec![CurveSegment::Cubic],
        handles: vec![HandleMode::Auto],
        extrapolation: vec![CurveExtrapolation::Hold],
        ..default()
    }
}

/// CSS's `ease`, `cubic-bezier(0.25, 0.1, 0.25, 1)`.
fn ease_curve() -> ScalarCurve {
    ScalarCurve {
        points: vec![
            CurvePoint {
                right: DirectionHandle::Aligned(Vec2::new(0.25, 0.1)),
                ..default()
            },
            CurvePoint {
                x: 1.0,
                y: 1.0,
                left: DirectionHandle::Aligned(Vec2::new(-0.75, 0.0)),
                ..default()
            },
        ],
        extrapolation: CurveExtrapolation::Hold,
    }
}

/// One cubic segment, its ends pinned, its handles aligned.
fn bezier_constraints() -> CurveConstraints {
    CurveConstraints {
        segments: vec![CurveSegment::Cubic],
        handles: vec![HandleMode::Aligned],
        points: 2..=2,
        pin_ends: PinEnds::XY,
        extrapolation: vec![CurveExtrapolation::Hold],
        ..default()
    }
}

/// Writes each row's curve as its app stores it.
fn show_stored(
    editors: Query<(&CurveValue, &Stored, &ChildOf), Changed<CurveValue>>,
    parents: Query<&ChildOf>,
    children: Query<&Children>,
    mut texts: Query<&mut Text, With<StoredText>>,
) {
    for (curve, stored, column) in &editors {
        let Ok(row) = parents.get(column.parent()) else {
            continue;
        };
        for child in children.iter_descendants(row.parent()) {
            if let Ok(mut text) = texts.get_mut(child) {
                text.0 = stored.write(&curve.0);
            }
        }
    }
}

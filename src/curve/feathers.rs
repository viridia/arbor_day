//! Feathers widgets for curves.

use bevy::{
    feathers::{theme::ThemeBackgroundColor, tokens},
    prelude::*,
};

use super::{plot::CurveStroke, view::CurvePlot};

/// A curve drawn small, with no gestures, as a collapsed row shows it. Insert a
/// [`CurveValue`](super::model::CurveValue) and its
/// [`CurveConstraints`](super::model::CurveConstraints) on the same entity, and write the value
/// when the curve changes.
#[derive(Component, Clone, Default)]
pub struct FeathersCurveThumbnail;

/// A [`FeathersCurveThumbnail`] filling its row's spare width, as tall as a number input.
pub fn curve_thumbnail() -> impl Scene {
    bsn! {
        FeathersCurveThumbnail
        Node {
            flex_grow: 1.0,
            height: px(24),
            border_radius: BorderRadius::all(px(4)),
        }
        ThemeBackgroundColor(tokens::SLIDER_BG)
        Children [
            CurvePlot { inset: 3.0 }
            CurveStroke
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
            }
            Pickable::IGNORE
        ]
    }
}

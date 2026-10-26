//! Scalar curves and their widgets: a number that changes along another, such as a lateral's
//! length along its parent, drawn and edited as a curve.
//!
//! [`ScalarCurve`] is the value a consumer stores; [`CurveConstraints`] says what it may become.
//! A widget shows a copy, [`CurveValue`], and an editor proposes each change as a [`CurveEdit`],
//! which the consumer judges, with [`ScalarCurve::apply`] or rules of its own, before writing the
//! copy back.

mod edit;
mod feathers;
mod gesture;
mod model;
mod plot;
mod view;

use bevy::prelude::*;

pub use edit::CurveEdit;
pub use feathers::{CurveEditorFormat, curve_editor, curve_thumbnail};
pub use model::{
    CurveAxis, CurveConstraints, CurveExtrapolation, CurvePoint, CurveSegment, CurveValue,
    HandleMode, ScalarCurve,
};

/// Draws curve plots and thumbnails, and runs curve editors.
pub struct CurvePlugin;

impl Plugin for CurvePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((plot::PathMaterialPlugin, gesture::plugin, feathers::plugin))
            .add_systems(
                PostUpdate,
                (
                    edit::follow_point_count,
                    view::update_curve_paths.after(bevy::ui::UiSystems::Layout),
                ),
            );
    }
}

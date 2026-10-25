//! Scalar curves and their widgets: a number that changes along another, such as a lateral's
//! length along its parent, drawn and edited as a curve.
//!
//! [`ScalarCurve`] is the value a consumer stores; [`CurveConstraints`] says what it may become.
//! A widget shows a copy, [`CurveValue`], which the consumer writes back after judging each edit
//! with [`ScalarCurve::apply`].

mod feathers;
mod model;
mod plot;
mod view;

use bevy::prelude::*;

pub use feathers::curve_thumbnail;
pub use model::{
    CurveAxis, CurveConstraints, CurveEditOp, CurveExtrapolation, CurvePoint, CurveSegment,
    CurveValue, HandleMode, ScalarCurve,
};

/// Draws curve plots and thumbnails.
pub struct CurvePlugin;

impl Plugin for CurvePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(plot::PathMaterialPlugin).add_systems(
            PostUpdate,
            view::update_curve_paths.after(bevy::ui::UiSystems::Layout),
        );
    }
}

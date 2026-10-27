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
#[allow(
    unused_imports,
    reason = "the host's selection API; arbor_day uses it from undo (C7)"
)]
pub use edit::{
    CurveSelection, CurveSelectionChanged, PointRef, SelectCurve, SelectedPoints, curve_self_update,
};
pub use feathers::{
    CurveEditorFormat, CurveMenuReplace, CurveMenuTarget, curve_editor, curve_menu,
    curve_menu_item, curve_thumbnail,
};
pub use model::{
    CurveAxis, CurveConstraints, CurveExtrapolation, CurvePoint, CurveSegment, CurveValue,
    HandleMode, ScalarCurve,
};

/// Draws curve plots and thumbnails, and runs curve editors.
pub struct CurvePlugin;

impl Plugin for CurvePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((plot::PathMaterialPlugin, gesture::plugin, feathers::plugin))
            .add_observer(edit::select_curve)
            .add_systems(
                PostUpdate,
                (
                    (edit::follow_point_count, edit::notify_selection).chain(),
                    view::update_curve_paths.after(bevy::ui::UiSystems::Layout),
                ),
            );
    }
}

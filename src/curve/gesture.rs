//! Pointer and key handling on a curve's plot: which point the pointer is over, selecting points,
//! and the edits a drag, a double-click or the Delete key proposes.

use bevy::{
    ecs::system::SystemParam,
    input::{ButtonState, keyboard::KeyboardInput},
    input_focus::FocusedInput,
    picking::{
        events::{
            PointerDrag, PointerDragEnd, PointerLeave, PointerMove, PointerPress, PointerRelease,
        },
        hover::PointerCaptureMap,
        pointer::{PointerButton, PointerId},
    },
    prelude::*,
    ui::{ComputedUiRenderTargetInfo, UiGlobalTransform},
};

use super::{
    edit::{CurveEdit, CurveEditPhase, CurveSelection, PointRef, SeenCurve, Sent},
    model::{CurveAxis, CurveConstraints, CurveEditOp, CurveValue, ScalarCurve},
    view::{CurvePlot, CurveView, curve_owner},
};

/// On a [`CurvePlot`]: pointer and key handling. Selecting needs a [`CurveSelection`] on the entity
/// holding the curve; the Delete key needs the plot focusable.
#[derive(Component, Clone, Default)]
#[require(CurveHover, CurveGesture, CurveHitRadius)]
pub struct CurveGestures;

/// How near the pointer must come to a point to be over it, in logical pixels.
#[derive(Component, Clone, Copy, Debug)]
pub struct CurveHitRadius(pub f32);

impl Default for CurveHitRadius {
    fn default() -> Self {
        Self(6.0)
    }
}

/// On a plot with [`CurveGestures`]: the point under the pointer, for styling. It holds still
/// during a drag.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct CurveHover(pub Option<PointRef>);

/// On a plot with [`CurveGestures`]: the interaction in progress.
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub enum CurveGesture {
    #[default]
    Idle,
    /// Moving the selected points of `curve`, from where they were when pressed (`snapshot`) by
    /// the pointer's `offset` since, in the curve's units, each moved point rounded to the
    /// precision of its `axes`, x and y.
    MovePoints {
        curve: Entity,
        snapshot: Vec<(usize, Vec2)>,
        offset: Vec2,
        axes: [CurveAxis; 2],
    },
}

impl CurveGesture {
    /// The move this gesture proposes. Points stay where they were until the pointer moves.
    fn proposed(&self) -> Option<CurveEditOp> {
        let Self::MovePoints {
            snapshot,
            offset,
            axes: [x, y],
            ..
        } = self
        else {
            return None;
        };
        let moves = (snapshot.iter())
            .map(|&(index, at)| {
                if *offset == Vec2::ZERO {
                    return (index, at);
                }
                let to = at + *offset;
                (index, Vec2::new(x.round(to.x), y.round(to.y)))
            })
            .collect();
        Some(CurveEditOp::Move(moves))
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_observer(hover_point)
        .add_observer(leave_plot)
        .add_observer(press_plot)
        .add_observer(drag_points)
        .add_observer(end_drag)
        .add_observer(release_plot)
        .add_observer(cancel_on_removal)
        .add_observer(delete_selected);
}

/// A plot with gestures, and the curve it shows.
struct Plot<'a> {
    /// The entity holding the curve, which is the editor while it shows one curve.
    curve_entity: Entity,
    curve: &'a ScalarCurve,
    constraints: &'a CurveConstraints,
    view: CurveView,
    node: &'a ComputedNode,
    transform: UiGlobalTransform,
    scale: f32,
    radius: f32,
}

impl Plot<'_> {
    /// `position`, a pointer's in the window, in logical pixels from the plot's top left.
    fn local(&self, position: Vec2) -> Option<Vec2> {
        let centred = self
            .node
            .normalize_point(self.transform, position * self.scale)?;
        Some((centred + Vec2::splat(0.5)) * self.node.size() * self.node.inverse_scale_factor())
    }

    /// The nearest point within the hit radius of `local`.
    fn hit(&self, local: Vec2) -> Option<usize> {
        (self.curve.points.iter().enumerate())
            .map(|(index, point)| {
                let pixel = self.view.to_pixels(Vec2::new(point.x, point.y));
                (index, pixel.distance(local))
            })
            .filter(|&(_, distance)| distance <= self.radius)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index)
    }
}

#[derive(SystemParam)]
struct Plots<'w, 's> {
    plots: Query<
        'w,
        's,
        (
            &'static CurvePlot,
            &'static ComputedNode,
            &'static UiGlobalTransform,
            &'static ComputedUiRenderTargetInfo,
            &'static CurveHitRadius,
        ),
        With<CurveGestures>,
    >,
    parents: Query<'w, 's, &'static ChildOf>,
    curves: Query<'w, 's, (&'static CurveValue, &'static CurveConstraints)>,
    ui_scale: Res<'w, UiScale>,
}

impl Plots<'_, '_> {
    fn get(&self, plot: Entity) -> Option<Plot<'_>> {
        let (settings, node, transform, target, radius) = self.plots.get(plot).ok()?;
        let curve_entity = curve_owner(plot, &self.parents, |owner| self.curves.contains(owner))?;
        let (curve, constraints) = self.curves.get(curve_entity).ok()?;
        let size = node.size() * node.inverse_scale_factor();
        Some(Plot {
            curve_entity,
            curve: &curve.0,
            constraints,
            view: CurveView::new(constraints, size, settings.inset),
            node,
            transform: *transform,
            scale: target.scale_factor() / self.ui_scale.0,
            radius: radius.0,
        })
    }
}

fn hover_point(
    event: On<PointerMove>,
    plots: Plots,
    mut hovers: Query<(&CurveGesture, &mut CurveHover)>,
) {
    let Some(plot) = plots.get(event.entity) else {
        return;
    };
    let Ok((CurveGesture::Idle, mut hover)) = hovers.get_mut(event.entity) else {
        return;
    };
    let index = (plot.local(event.pointer.position)).and_then(|local| plot.hit(local));
    let curve = plot.curve_entity;
    hover.set_if_neq(CurveHover(index.map(|index| PointRef { curve, index })));
}

fn leave_plot(event: On<PointerLeave>, mut hovers: Query<(&CurveGesture, &mut CurveHover)>) {
    if let Ok((CurveGesture::Idle, mut hover)) = hovers.get_mut(event.entity) {
        hover.set_if_neq(CurveHover(None));
    }
}

/// A press on a point selects it, or with Shift or Ctrl adds it or takes it away, and starts
/// moving the selection. A press elsewhere clears the selection; a second one inserts a point.
fn press_plot(
    mut press: On<PointerPress>,
    plots: Plots,
    keys: Res<ButtonInput<KeyCode>>,
    mut gestures: Query<&mut CurveGesture>,
    mut selections: Query<&mut CurveSelection>,
    mut seen: Query<&mut SeenCurve>,
    mut capture: ResMut<PointerCaptureMap>,
    mut commands: Commands,
) {
    if press.button != PointerButton::Primary {
        return;
    }
    let Some(plot) = plots.get(press.entity) else {
        return;
    };
    press.propagate(false);
    let Some(local) = plot.local(press.pointer.position) else {
        return;
    };
    let curve = plot.curve_entity;
    let Ok(mut selection) = selections.get_mut(curve) else {
        return;
    };
    let adding = keys.any_pressed([
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight,
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);
    let Some(index) = plot.hit(local) else {
        if press.count == 2 {
            if let Ok(mut seen) = seen.get_mut(curve) {
                seen.sent = Some(Sent::Insert);
            }
            commands.trigger(CurveEdit {
                editor: curve,
                curve,
                op: CurveEditOp::Insert(plot.view.to_curve(local)),
                phase: CurveEditPhase::Finish,
            });
        } else if !adding && !selection.points.is_empty() {
            selection.points.clear();
        }
        return;
    };
    let point = PointRef { curve, index };
    let selected = selection.points.contains(&point);
    if adding && selected {
        selection.points.retain(|other| *other != point);
        return;
    }
    if adding {
        selection.points.push(point);
    } else if !selected {
        selection.points = vec![point];
    }
    let snapshot = (selection.indices(curve).into_iter())
        .filter_map(|index| {
            let point = plot.curve.points.get(index)?;
            Some((index, Vec2::new(point.x, point.y)))
        })
        .collect();
    let gesture = CurveGesture::MovePoints {
        curve,
        snapshot,
        offset: Vec2::ZERO,
        axes: [plot.constraints.x, plot.constraints.y],
    };
    if let Some(op) = gesture.proposed() {
        commands.trigger(CurveEdit {
            editor: curve,
            curve,
            op,
            phase: CurveEditPhase::Start,
        });
    }
    if let Ok(mut current) = gestures.get_mut(press.entity) {
        *current = gesture;
    }
    capture.capture(press.pointer.id, press.entity, press.hit.clone());
}

fn drag_points(
    mut drag: On<PointerDrag>,
    plots: Plots,
    mut gestures: Query<&mut CurveGesture>,
    mut commands: Commands,
) {
    let (Some(plot), Ok(mut gesture)) = (plots.get(drag.entity), gestures.get_mut(drag.entity))
    else {
        return;
    };
    drag.propagate(false);
    let CurveGesture::MovePoints { curve, offset, .. } = gesture.as_mut() else {
        return;
    };
    let curve = *curve;
    *offset = plot.view.delta_to_curve(drag.distance / plots.ui_scale.0);
    if let Some(op) = gesture.proposed() {
        commands.trigger(CurveEdit {
            editor: curve,
            curve,
            op,
            phase: CurveEditPhase::Update,
        });
    }
}

fn end_drag(
    mut drag: On<PointerDragEnd>,
    plots: Plots,
    mut gestures: Query<&mut CurveGesture>,
    mut capture: ResMut<PointerCaptureMap>,
    mut commands: Commands,
) {
    let (Some(plot), Ok(mut gesture)) = (plots.get(drag.entity), gestures.get_mut(drag.entity))
    else {
        return;
    };
    drag.propagate(false);
    if let CurveGesture::MovePoints { offset, .. } = gesture.as_mut() {
        *offset = plot.view.delta_to_curve(drag.distance / plots.ui_scale.0);
    }
    finish(&mut gesture, drag.pointer.id, &mut capture, &mut commands);
}

/// A press that never became a drag ends here; a drag ends at whichever of its end and the release
/// comes first.
fn release_plot(
    mut release: On<PointerRelease>,
    mut gestures: Query<&mut CurveGesture, With<CurveGestures>>,
    mut capture: ResMut<PointerCaptureMap>,
    mut commands: Commands,
) {
    if let Ok(mut gesture) = gestures.get_mut(release.entity) {
        release.propagate(false);
        finish(
            &mut gesture,
            release.pointer.id,
            &mut capture,
            &mut commands,
        );
    }
}

fn finish(
    gesture: &mut CurveGesture,
    pointer: PointerId,
    capture: &mut PointerCaptureMap,
    commands: &mut Commands,
) {
    let ended = std::mem::take(gesture);
    if let (CurveGesture::MovePoints { curve, .. }, Some(op)) = (&ended, ended.proposed()) {
        commands.trigger(CurveEdit {
            editor: *curve,
            curve: *curve,
            op,
            phase: CurveEditPhase::Finish,
        });
        capture.release(pointer);
    }
}

/// A plot despawned mid-drag cancels the drag, so the curve returns to where it started.
fn cancel_on_removal(
    removed: On<Remove<CurveGesture>>,
    gestures: Query<&CurveGesture>,
    mut commands: Commands,
) {
    let Ok(CurveGesture::MovePoints {
        curve, snapshot, ..
    }) = gestures.get(removed.entity)
    else {
        return;
    };
    let edit = CurveEdit {
        editor: *curve,
        curve: *curve,
        op: CurveEditOp::Move(snapshot.clone()),
        phase: CurveEditPhase::Cancel,
    };
    // The editor may be going too.
    commands.queue(move |world: &mut World| {
        if world.get_entity(edit.editor).is_ok() {
            world.trigger(edit);
        }
    });
}

/// Delete or Backspace, on a focused plot, deletes the selection.
fn delete_selected(
    mut input: On<FocusedInput<KeyboardInput>>,
    plots: Plots,
    gestures: Query<&CurveGesture>,
    selections: Query<&CurveSelection>,
    mut seen: Query<&mut SeenCurve>,
    mut commands: Commands,
) {
    let plot_entity = input.focused_entity;
    let key = &input.input;
    if key.state != ButtonState::Pressed
        || !matches!(key.key_code, KeyCode::Delete | KeyCode::Backspace)
    {
        return;
    }
    let (Some(plot), Ok(CurveGesture::Idle)) = (plots.get(plot_entity), gestures.get(plot_entity))
    else {
        return;
    };
    input.propagate(false);
    let curve = plot.curve_entity;
    let indices = selections
        .get(curve)
        .map(|selection| selection.indices(curve))
        .unwrap_or_default();
    if indices.is_empty() {
        return;
    }
    if let Ok(mut seen) = seen.get_mut(curve) {
        seen.sent = Some(Sent::Delete(indices.clone()));
    }
    commands.trigger(CurveEdit {
        editor: curve,
        curve,
        op: CurveEditOp::Delete(indices),
        phase: CurveEditPhase::Finish,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moving(offset: Vec2) -> CurveGesture {
        let mut y = CurveAxis::unit();
        y.precision = 0.1;
        CurveGesture::MovePoints {
            curve: Entity::PLACEHOLDER,
            snapshot: vec![(1, Vec2::new(0.503, 0.47))],
            offset,
            axes: [CurveAxis::unit(), y],
        }
    }

    #[test]
    fn a_move_lands_on_each_axis_precision() {
        let Some(CurveEditOp::Move(moves)) = moving(Vec2::new(0.1, 0.12)).proposed() else {
            panic!("a move");
        };
        let (index, at) = moves[0];
        assert_eq!(index, 1);
        assert!((at.x - 0.6).abs() < 1e-6, "{at}");
        assert!((at.y - 0.6).abs() < 1e-6, "{at}");
    }

    #[test]
    fn a_press_without_a_move_leaves_the_points_where_they_are() {
        let Some(CurveEditOp::Move(moves)) = moving(Vec2::ZERO).proposed() else {
            panic!("a move");
        };
        assert_eq!(moves, vec![(1, Vec2::new(0.503, 0.47))]);
    }
}

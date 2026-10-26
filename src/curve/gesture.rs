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
    /// Selecting the points of `curve` inside the rectangle from `from` to `to`, in the curve's
    /// units, as well as those selected before it began (`base`).
    SelectRect {
        curve: Entity,
        from: Vec2,
        to: Vec2,
        base: Vec<PointRef>,
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
        .add_observer(plot_keys);
}

/// `base` and the points of `curve`, held by entity `owner`, inside the rectangle from `from` to
/// `to`.
fn rect_selection(
    curve: &ScalarCurve,
    owner: Entity,
    from: Vec2,
    to: Vec2,
    base: &[PointRef],
) -> Vec<PointRef> {
    let (min, max) = (from.min(to), from.max(to));
    let mut points = base.to_vec();
    for (index, point) in curve.points.iter().enumerate() {
        let at = Vec2::new(point.x, point.y);
        let point = PointRef {
            curve: owner,
            index,
        };
        if at.cmpge(min).all() && at.cmple(max).all() && !points.contains(&point) {
            points.push(point);
        }
    }
    points
}

/// The point Tab selects among `count`, the `selected` indices in order, or the previous one with
/// Shift (`back`); none past either end, so focus moves on.
fn tab_point(selected: &[usize], count: usize, back: bool) -> Option<usize> {
    if back {
        match selected.first() {
            None => count.checked_sub(1),
            Some(&index) => index.checked_sub(1),
        }
    } else {
        let next = selected.last().map_or(0, |&index| index + 1);
        (next < count).then_some(next)
    }
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
/// moving the selection. A press elsewhere clears the selection, unless adding, and starts a
/// rectangle. A second press inserts a point, or on a point deletes it.
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
    // Clicks in quick succession keep counting, so two quick double-clicks are a fourth click.
    let double = press.count.is_multiple_of(2);
    let Some(index) = plot.hit(local) else {
        if double {
            if let Ok(mut seen) = seen.get_mut(curve) {
                seen.sent = Some(Sent::Insert);
            }
            commands.trigger(CurveEdit {
                editor: curve,
                curve,
                op: CurveEditOp::Insert(plot.view.to_curve(local)),
                phase: CurveEditPhase::Finish,
            });
            return;
        }
        if !adding && !selection.points.is_empty() {
            selection.points.clear();
        }
        let at = plot.view.to_curve(local);
        if let Ok(mut current) = gestures.get_mut(press.entity) {
            *current = CurveGesture::SelectRect {
                curve,
                from: at,
                to: at,
                base: selection.points.clone(),
            };
        }
        capture.capture(press.pointer.id, press.entity, press.hit.clone());
        return;
    };
    if double {
        if let Ok(mut seen) = seen.get_mut(curve) {
            seen.sent = Some(Sent::Delete(vec![index]));
        }
        commands.trigger(CurveEdit {
            editor: curve,
            curve,
            op: CurveEditOp::Delete(vec![index]),
            phase: CurveEditPhase::Finish,
        });
        return;
    }
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
    mut selections: Query<&mut CurveSelection>,
    mut commands: Commands,
) {
    let (Some(plot), Ok(mut gesture)) = (plots.get(drag.entity), gestures.get_mut(drag.entity))
    else {
        return;
    };
    drag.propagate(false);
    let moved = plot.view.delta_to_curve(drag.distance / plots.ui_scale.0);
    let curve = match gesture.as_mut() {
        CurveGesture::Idle => return,
        CurveGesture::SelectRect {
            curve,
            from,
            to,
            base,
        } => {
            *to = *from + moved;
            let points = rect_selection(plot.curve, *curve, *from, *to, base);
            if let Ok(mut selection) = selections.get_mut(*curve) {
                selection.set_if_neq(CurveSelection { points });
            }
            return;
        }
        CurveGesture::MovePoints { curve, offset, .. } => {
            *offset = moved;
            *curve
        }
    };
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
    finish(
        &mut gesture,
        drag.entity,
        drag.pointer.id,
        &mut capture,
        &mut commands,
    );
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
            release.entity,
            release.pointer.id,
            &mut capture,
            &mut commands,
        );
    }
}

/// Ends the gesture on `plot`, and releases the pointer if the plot holds it, which it still does
/// after Escape ended the gesture early.
fn finish(
    gesture: &mut Mut<CurveGesture>,
    plot: Entity,
    pointer: PointerId,
    capture: &mut PointerCaptureMap,
    commands: &mut Commands,
) {
    if **gesture != CurveGesture::Idle {
        let ended = std::mem::take(gesture.as_mut());
        if let (CurveGesture::MovePoints { curve, .. }, Some(op)) = (&ended, ended.proposed()) {
            commands.trigger(CurveEdit {
                editor: *curve,
                curve: *curve,
                op,
                phase: CurveEditPhase::Finish,
            });
        }
    }
    if capture.get(&pointer).is_some_and(|(held, _)| held == plot) {
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

/// Keys on a focused plot. Escape cancels the gesture in progress: a move proposes its snapshot, a
/// rectangle restores the selection it began with. While idle, Delete and Backspace delete the
/// selection; arrows move it by a step of the axis, ten with Shift; Tab selects the next point,
/// with Shift the previous one, and past either end lets focus move on.
fn plot_keys(
    mut input: On<FocusedInput<KeyboardInput>>,
    plots: Plots,
    keys: Res<ButtonInput<KeyCode>>,
    mut gestures: Query<&mut CurveGesture>,
    mut selections: Query<&mut CurveSelection>,
    mut seen: Query<&mut SeenCurve>,
    mut commands: Commands,
) {
    let plot_entity = input.focused_entity;
    let (code, state) = (input.input.key_code, input.input.state);
    if state != ButtonState::Pressed {
        return;
    }
    let (Some(plot), Ok(mut gesture)) = (plots.get(plot_entity), gestures.get_mut(plot_entity))
    else {
        return;
    };
    let curve = plot.curve_entity;
    let Ok(mut selection) = selections.get_mut(curve) else {
        return;
    };
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let selected = selection.indices(curve);
    let idle = *gesture == CurveGesture::Idle;
    let handled = match code {
        KeyCode::Escape if !idle => {
            match std::mem::take(gesture.as_mut()) {
                CurveGesture::MovePoints {
                    curve, snapshot, ..
                } => commands.trigger(CurveEdit {
                    editor: curve,
                    curve,
                    op: CurveEditOp::Move(snapshot),
                    phase: CurveEditPhase::Cancel,
                }),
                CurveGesture::SelectRect { base, .. } => {
                    selection.set_if_neq(CurveSelection { points: base });
                }
                CurveGesture::Idle => {}
            }
            true
        }
        _ if !idle => false,
        KeyCode::Tab => match tab_point(&selected, plot.curve.points.len(), shift) {
            Some(index) => {
                selection.points = vec![PointRef { curve, index }];
                true
            }
            None => false,
        },
        _ if selected.is_empty() => false,
        KeyCode::Delete | KeyCode::Backspace => {
            if let Ok(mut seen) = seen.get_mut(curve) {
                seen.sent = Some(Sent::Delete(selected.clone()));
            }
            commands.trigger(CurveEdit {
                editor: curve,
                curve,
                op: CurveEditOp::Delete(selected),
                phase: CurveEditPhase::Finish,
            });
            true
        }
        KeyCode::ArrowLeft | KeyCode::ArrowRight | KeyCode::ArrowUp | KeyCode::ArrowDown => {
            let direction = match code {
                KeyCode::ArrowLeft => Vec2::NEG_X,
                KeyCode::ArrowRight => Vec2::X,
                KeyCode::ArrowUp => Vec2::Y,
                _ => Vec2::NEG_Y,
            };
            let (x, y) = (&plot.constraints.x, &plot.constraints.y);
            let steps = if shift { 10.0 } else { 1.0 };
            let delta = direction * Vec2::new(x.step, y.step) * steps;
            // Only the coordinate moved is rounded, so the other keeps whatever it held.
            let moves = (selected.iter())
                .filter_map(|&index| {
                    let point = plot.curve.points.get(index)?;
                    let at = Vec2::new(point.x, point.y) + delta;
                    let at = if delta.x != 0.0 {
                        Vec2::new(x.round(at.x), at.y)
                    } else {
                        Vec2::new(at.x, y.round(at.y))
                    };
                    Some((index, at))
                })
                .collect();
            commands.trigger(CurveEdit {
                editor: curve,
                curve,
                op: CurveEditOp::Move(moves),
                phase: CurveEditPhase::Finish,
            });
            true
        }
        _ => false,
    };
    if handled {
        input.propagate(false);
    }
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
    fn a_rectangle_adds_the_points_inside_it_to_its_base() {
        let curve = ScalarCurve {
            points: [(0.0, 0.5), (0.3, 0.6), (0.6, 0.2), (1.0, 0.9)]
                .into_iter()
                .map(|(x, y)| crate::curve::CurvePoint { x, y, ..default() })
                .collect(),
            extrapolation: crate::curve::CurveExtrapolation::Hold,
        };
        let owner = Entity::PLACEHOLDER;
        let at = |index| PointRef {
            curve: owner,
            index,
        };
        // Dragged up and to the left: the corners in either order.
        let inside = rect_selection(&curve, owner, Vec2::new(0.7, 0.1), Vec2::new(0.2, 0.7), &[]);
        assert_eq!(inside, vec![at(1), at(2)]);
        let added = rect_selection(
            &curve,
            owner,
            Vec2::new(0.2, 0.1),
            Vec2::new(0.7, 0.7),
            &[at(2), at(3)],
        );
        assert_eq!(added, vec![at(2), at(3), at(1)]);
    }

    #[test]
    fn tab_steps_through_the_points_and_out_past_either_end() {
        assert_eq!(tab_point(&[], 3, false), Some(0));
        assert_eq!(tab_point(&[], 3, true), Some(2));
        assert_eq!(tab_point(&[0, 1], 3, false), Some(2));
        assert_eq!(tab_point(&[1, 2], 3, true), Some(0));
        assert_eq!(tab_point(&[2], 3, false), None);
        assert_eq!(tab_point(&[0], 3, true), None);
        assert_eq!(tab_point(&[], 0, false), None);
    }

    #[test]
    fn a_press_without_a_move_leaves_the_points_where_they_are() {
        let Some(CurveEditOp::Move(moves)) = moving(Vec2::ZERO).proposed() else {
            panic!("a move");
        };
        assert_eq!(moves, vec![(1, Vec2::new(0.503, 0.47))]);
    }
}

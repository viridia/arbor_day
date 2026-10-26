//! The events between a curve editor and the consumer that owns its curve, and the editor's
//! selection.

use bevy::prelude::*;

use super::model::{CurveEditOp, CurveValue, ScalarCurve};

/// Where an edit stands in the interaction that made it. An edit made in one go, such as an
/// insert, is `Finish`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurveEditPhase {
    Start,
    Update,
    Finish,
    /// The interaction was abandoned: the edit proposes the curve as it was at the start.
    Cancel,
}

/// An edit the editor proposes. The consumer judges it, with [`ScalarCurve::apply`] or rules of
/// its own, and writes the curve's [`CurveValue`] back; the editor never changes the curve itself.
#[allow(
    dead_code,
    reason = "arbor_day's panel shows one curve and judges every phase alike; undo (C7) will read them"
)]
#[derive(EntityEvent, Clone, Debug)]
pub struct CurveEdit {
    #[event_target]
    pub editor: Entity,
    /// The entity holding the edited curve's [`CurveValue`]: the editor itself while it shows one
    /// curve.
    pub curve: Entity,
    pub op: CurveEditOp,
    pub phase: CurveEditPhase,
}

/// A point of a curve, by its curve and index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PointRef {
    pub curve: Entity,
    pub index: usize,
}

/// On an editor: the points selected. The editor keeps it as points are added and removed.
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct CurveSelection {
    pub points: Vec<PointRef>,
}

impl CurveSelection {
    /// The selected points of `curve`, by index, in order.
    pub fn indices(&self, curve: Entity) -> Vec<usize> {
        let mut indices: Vec<usize> = (self.points.iter())
            .filter(|point| point.curve == curve)
            .map(|point| point.index)
            .collect();
        indices.sort_unstable();
        indices
    }

    /// The one point selected, if exactly one is.
    pub fn single(&self) -> Option<PointRef> {
        match self.points.as_slice() {
            [point] => Some(*point),
            _ => None,
        }
    }
}

/// On a curve's entity: the curve as the editor last saw it, and the insert or delete the editor
/// last sent, so a change in the point count shifts the selection past it.
#[derive(Component, Clone, Debug, Default)]
pub(super) struct SeenCurve {
    pub(super) curve: Option<ScalarCurve>,
    pub(super) sent: Option<Sent>,
}

/// An edit that adds or removes points.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Sent {
    Insert,
    Delete(Vec<usize>),
}

/// The selected indices of a curve that changed from `before` to `after`. After the insert the
/// editor sent, the selection is the new point; after its delete, the remaining points shift
/// down past the deleted ones. Any other change of point count trims the selection.
pub(super) fn adjust_indices(
    selected: &[usize],
    before: &ScalarCurve,
    after: &ScalarCurve,
    sent: Option<&Sent>,
) -> Vec<usize> {
    let (old, new) = (before.points.len(), after.points.len());
    match sent {
        Some(Sent::Insert) if new == old + 1 => {
            let inserted = (before.points.iter().zip(&after.points))
                .position(|(a, b)| a.x != b.x || a.y != b.y)
                .unwrap_or(old);
            vec![inserted]
        }
        Some(Sent::Delete(deleted)) if new + deleted.len() == old => (selected.iter())
            .filter(|index| !deleted.contains(index))
            .map(|&index| index - deleted.iter().filter(|&&gone| gone < index).count())
            .collect(),
        _ => selected
            .iter()
            .copied()
            .filter(|&index| index < new)
            .collect(),
    }
}

/// Keeps each editor's selection on the same points as its curve's point count changes.
pub(super) fn follow_point_count(
    mut curves: Query<(Entity, &CurveValue, &mut SeenCurve), Changed<CurveValue>>,
    mut selections: Query<&mut CurveSelection>,
) {
    for (curve, value, mut seen) in &mut curves {
        let sent = seen.sent.take();
        let before = seen.curve.replace(value.0.clone());
        let Some(before) = before else {
            continue;
        };
        if before.points.len() == value.0.points.len() {
            continue;
        }
        // One curve per editor for now: the selection is on the curve's own entity.
        let Ok(mut selection) = selections.get_mut(curve) else {
            continue;
        };
        let adjusted = adjust_indices(&selection.indices(curve), &before, &value.0, sent.as_ref());
        let points = (adjusted.into_iter())
            .map(|index| PointRef { curve, index })
            .collect();
        selection.set_if_neq(CurveSelection { points });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::model::{CurveExtrapolation, CurvePoint};

    fn curve(xs: &[f32]) -> ScalarCurve {
        ScalarCurve {
            points: (xs.iter())
                .map(|&x| CurvePoint {
                    x,
                    y: x,
                    ..default()
                })
                .collect(),
            extrapolation: CurveExtrapolation::Hold,
        }
    }

    #[test]
    fn an_insert_selects_the_new_point() {
        let before = curve(&[0.0, 0.5, 1.0]);
        let after = curve(&[0.0, 0.25, 0.5, 1.0]);
        let adjusted = adjust_indices(&[2], &before, &after, Some(&Sent::Insert));
        assert_eq!(adjusted, vec![1]);
    }

    #[test]
    fn a_delete_shifts_the_points_after_it() {
        let before = curve(&[0.0, 0.25, 0.5, 0.75, 1.0]);
        let after = curve(&[0.0, 0.5, 1.0]);
        let sent = Sent::Delete(vec![1, 3]);
        let adjusted = adjust_indices(&[1, 2, 4], &before, &after, Some(&sent));
        assert_eq!(adjusted, vec![1, 2]);
    }

    #[test]
    fn a_replacement_trims_the_selection() {
        let before = curve(&[0.0, 0.25, 0.5, 1.0]);
        let after = curve(&[0.0, 1.0]);
        assert_eq!(adjust_indices(&[1, 3], &before, &after, None), vec![1]);
        // A delete that removed other points than the editor sent is a replacement too.
        let sent = Sent::Delete(vec![1]);
        assert_eq!(
            adjust_indices(&[1, 3], &before, &after, Some(&sent)),
            vec![1]
        );
    }
}

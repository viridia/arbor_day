//! Carrying on (D30): an axis bearing limbs ends at its last node, and one more limb carries it on.

use super::{
    envelope::distance_to_envelope,
    placement::draw,
    skeleton::{Axis, AxisKind, CARRY_ON_STREAM, Level, OWN_VALUES_STREAM, Ring, Skeleton},
    trace::{end_at, trace},
};
use crate::species::LimbLevel;

/// Carries on each axis of the level before that bears laterals, unless it forked, its copies
/// carrying it on instead: one more limb of the level leaves it at its last lateral's node, along
/// it, at its radius there, from a stream of its own, so the laterals leave its draws alone, its
/// reach the level's times its carry-on reach. It is grown as they are (`trace`), and kept if it
/// is longer than it is thick. The parent is cut at its start either way, so the two tubes meet,
/// or with none the parent ends at its last node.
pub(super) fn carry_on(level: &Level, skeleton: &mut Skeleton) {
    let axes = &mut skeleton.axes;
    for index in 0..axes.len() {
        let parent = &axes[index];
        let bears = |kind| parent.bears(axes, kind);
        if parent.level + 1 != level.number
            || !bears(AxisKind::Lateral)
            || bears(AxisKind::ForkCopy)
        {
            continue;
        }
        let last = parent.scions[parent.scions.len() - 1];
        let (_, at) = axes[last].parent.expect("a lateral grows from a parent");
        let frame = parent.frame_at(at);
        let tip = frame.ring(at);
        let rng = parent.rng.fork(CARRY_ON_STREAM);
        let reach = draw::<LimbLevel>(
            &level.params.reach,
            "reach",
            &mut rng.fork(OWN_VALUES_STREAM),
        ) * level.params.carry_on
            * distance_to_envelope(&level.envelope, frame.point, frame.tangent);
        let start = Ring { at: 0.0, ..tip };
        let kind = AxisKind::CarryOn;
        let mut carrier = Axis::placed(level.number, kind, (index, at), start, reach, rng);
        end_at(&mut axes[index].rings, tip);
        if trace(level, &mut carrier) {
            let id = axes.len();
            axes[index].scions.push(id);
            axes.push(carrier);
        }
    }
}

//! The plant as grown, before meshing: a list of axes, an axis in botany's sense, one stem of any
//! order, the trunk or a limb (`docs/patches.md`, The abstract plant).

use bevy::math::Vec3;

use super::{
    envelope::{ENVELOPE_BISECTIONS, Envelope, inside_envelope},
    foliage::clusters,
    placement::{grow_level, stems},
    trace::{TRACE_STEP, end_at},
};
use crate::{random::Rng, species::Species};

/// Streams forked from an axis's own: its scions take theirs by index from 0, so these take
/// indices no scion reaches.
pub(super) const WANDER_STREAM: u64 = u64::MAX;
/// The values drawn for a limb itself: its angle and reach.
pub(super) const OWN_VALUES_STREAM: u64 = u64::MAX - 1;
/// The layout of an axis's limbs: their count, and each node's turn, limb count and place.
pub(super) const LAYOUT_STREAM: u64 = u64::MAX - 2;
/// The limb carrying an axis on past its last node.
pub(super) const CARRY_ON_STREAM: u64 = u64::MAX - 4;
/// A limb's fork: whether it forks, where and how widely, and its copies.
pub(super) const FORK_STREAM: u64 = u64::MAX - 5;

/// A ring along an axis, as meshed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ring {
    pub center: Vec3,
    /// Unit direction of the axis.
    pub tangent: Vec3,
    /// Unit vector square to the tangent, carried along the axis without twisting (a
    /// rotation-minimizing frame). It sets the ring's roll, and azimuth 0 for scions starting
    /// there.
    pub side: Vec3,
    /// Distance along the axis from its base.
    pub at: f32,
}

/// An axis at one point along it: where its scions start and how they are oriented.
#[derive(Debug, Clone, Copy)]
pub(super) struct Frame {
    pub point: Vec3,
    /// Unit direction of the axis.
    pub tangent: Vec3,
    /// Unit vector square to the tangent, toward azimuth 0.
    pub side: Vec3,
}

/// How an axis came to grow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AxisKind {
    /// Rising from the ground.
    Stem,
    /// Leaving its parent's side at a node.
    Lateral,
    /// Carrying its parent on from the parent's last node, along it (D30).
    CarryOn,
    /// One of the copies carrying a forked parent on (D31).
    ForkCopy,
}

/// One axis as grown, before meshing.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Axis {
    /// 0 for a stem, 1 to 3 for the limb levels.
    pub level: usize,
    pub kind: AxisKind,
    /// At least two, base to tip.
    pub rings: Vec<Ring>,
    pub radii: (f32, f32),
    /// The axis's own random stream, before any draw; its scions' streams fork from it.
    pub rng: Rng,
    /// The axes growing from it, as indices in the plant's: its limbs and the one carrying it on,
    /// or its fork's copies and the limbs before the fork. An axis bearing none bears leaves.
    pub scions: Vec<usize>,
}

impl Axis {
    /// An axis before any scions have grown on it.
    pub fn new(
        level: usize,
        kind: AxisKind,
        rings: Vec<Ring>,
        radii: (f32, f32),
        rng: Rng,
    ) -> Self {
        Self {
            level,
            kind,
            rings,
            radii,
            rng,
            scions: Vec::new(),
        }
    }

    /// Whether any of its scions, among `axes`, is of `kind`.
    pub fn bears(&self, axes: &[Axis], kind: AxisKind) -> bool {
        self.scions.iter().any(|&scion| axes[scion].kind == kind)
    }

    /// Its length, the last ring's distance along it.
    pub fn length(&self) -> f32 {
        self.rings[self.rings.len() - 1].at
    }

    /// The axis at `t` (0 to 1) along it as meshed (`frame_at`).
    pub fn frame(&self, t: f32) -> Frame {
        self.frame_at(t.clamp(0.0, 1.0) * self.length())
    }

    /// The axis `at` metres along it as meshed: on the straight chord between ring centers, so
    /// scions start inside the bark, with that chord's direction.
    pub fn frame_at(&self, at: f32) -> Frame {
        let rings = &self.rings;
        let next = rings
            .partition_point(|ring| ring.at <= at)
            .clamp(1, rings.len() - 1);
        let (a, b) = (rings[next - 1], rings[next]);
        let along = if b.at > a.at {
            (at - a.at) / (b.at - a.at)
        } else {
            0.0
        };
        let tangent = (b.center - a.center).normalize_or(a.tangent);
        let side = a
            .side
            .lerp(b.side, along)
            .reject_from_normalized(tangent)
            .normalize_or(a.side);
        Frame {
            point: a.center.lerp(b.center, along),
            tangent,
            side,
        }
    }

    /// The part of the axis inside the crown, from the first of its points inside to the last, as
    /// fractions of its length; `None` if no point a trace step apart is inside. A limb lies
    /// wholly inside, so only a stem's part is shorter than the stem.
    pub fn inside_part(&self, crown: &Envelope) -> Option<(f32, f32)> {
        let length = self.length();
        let steps = ((length / TRACE_STEP).ceil() as u32).max(1);
        let inside = |t: f32| inside_envelope(crown, self.frame(t).point);
        let at = |step: u32| step as f32 / steps as f32;
        let first = (0..=steps).find(|&step| inside(at(step)))?;
        let last = (first..=steps).rev().find(|&step| inside(at(step)))?;
        // Each end lies between a step inside and its neighbour outside: halve the gap.
        let edge = |mut inner: f32, mut outer: f32| {
            for _ in 0..ENVELOPE_BISECTIONS {
                let middle = (inner + outer) * 0.5;
                if inside(middle) {
                    inner = middle;
                } else {
                    outer = middle;
                }
            }
            inner
        };
        let start = if first > 0 {
            edge(at(first), at(first - 1))
        } else {
            0.0
        };
        let end = if last < steps {
            edge(at(last), at(last + 1))
        } else {
            1.0
        };
        Some((start, end))
    }
}

#[cfg(test)]
impl Axis {
    pub fn origin(&self) -> Vec3 {
        self.rings[0].center
    }

    /// Unit direction at the base.
    pub fn direction(&self) -> Vec3 {
        self.rings[0].tangent
    }
}

/// A leaf cluster `at` metres along the axis `axis` (D27), where its card's base lies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Card {
    pub axis: usize,
    pub at: f32,
}

/// The plant as grown, before meshing.
pub(super) struct Skeleton {
    /// Stems first, then each level's limbs in turn.
    pub axes: Vec<Axis>,
    pub cards: Vec<Card>,
}

/// The stems, their limbs level by level, each level on the axes of the one before, and the leaf
/// clusters (`clusters`). A parent's top, where it ends and its radius there, is where its last
/// limb leaves it (`grow_level`), so it is set once the level after has grown.
pub(super) fn grow(species: &Species) -> Skeleton {
    let mut axes = stems(species);
    for level in 1..=species.grown_levels().len() {
        let mut grown = Vec::new();
        for parent in 0..axes.len() {
            if axes[parent].level + 1 != level {
                continue;
            }
            let forked = axes[parent].bears(&axes, AxisKind::ForkCopy);
            let bearing = grow_level(
                species,
                level,
                &axes[parent],
                forked,
                axes.len(),
                &mut grown,
            );
            let parent = &mut axes[parent];
            if let Some((tip, top)) = bearing.tip {
                end_at(&mut parent.rings, tip);
                parent.radii.1 = top;
            }
            parent.scions.extend(bearing.scions);
        }
        axes.extend(grown);
    }
    let cards = clusters(species, &axes);
    Skeleton { axes, cards }
}

/// The unit vector square to `direction` nearest world up: a limb's upper side. A limb pointing
/// straight up or down has none, and takes its parent's side, made square to it.
pub(super) fn upper_side(direction: Vec3, parent_side: Vec3) -> Vec3 {
    let up = Vec3::Y.reject_from_normalized(direction);
    if up.length_squared() > 1e-6 {
        up.normalize()
    } else {
        parent_side
            .reject_from_normalized(direction)
            .try_normalize()
            .unwrap_or_else(|| direction.any_orthonormal_vector())
    }
}

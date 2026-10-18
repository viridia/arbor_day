//! The plant as grown, before meshing: a list of axes, an axis in botany's sense, one stem of any
//! order, the trunk or a limb (`docs/patches.md`, The abstract plant).

use bevy::math::Vec3;

use super::{
    carry_on::carry_on,
    envelope::{ENVELOPE_BISECTIONS, Envelope, inside_envelope},
    foliage::{axial_fold, clusters},
    fork::fork,
    hint::hint,
    placement::{place_scions, stems},
    trace::{TRACE_STEP, kink_spacing, trace_scions},
    wander::wander,
};
use crate::{
    random::Rng,
    species::{LimbLevel, Species},
};

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

/// A ring along an axis: one at every step of its trace, until its level is hinted (`hint`).
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
    /// The axis's radius there, as grown, before the drawing thickens it (`tube`). The start
    /// ring's is set where the axis is placed, and the taper (`taper_rings`) writes the rest once
    /// the axis is traced.
    pub radius: f32,
}

/// An axis at one point along it: where its scions start and how they are oriented.
#[derive(Debug, Clone, Copy)]
pub(super) struct Frame {
    pub point: Vec3,
    /// Unit direction of the axis.
    pub tangent: Vec3,
    /// Unit vector square to the tangent, toward azimuth 0.
    pub side: Vec3,
    /// The axis's radius there.
    pub radius: f32,
}

impl Frame {
    /// A ring here, `at` metres along the axis: where it is cut, or where an axis carrying it on
    /// starts.
    pub fn ring(&self, at: f32) -> Ring {
        Ring {
            center: self.point,
            tangent: self.tangent,
            side: self.side,
            at,
            radius: self.radius,
        }
    }
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
    /// The axis it grows from, as an index in the plant's, and the distance along that where it
    /// leaves; none for a stem.
    pub parent: Option<(usize, f32)>,
    /// Base to tip: at least two once traced, and only the start ring while placed.
    pub rings: Vec<Ring>,
    /// How far it grows, as placed; the trace stops there or where it would leave the envelope.
    pub reach: f32,
    /// The axis's own random stream, before any draw; its scions' streams fork from it.
    pub rng: Rng,
    /// The axes growing from it, as indices in the plant's: its limbs and the one carrying it on,
    /// or its fork's copies and the limbs before the fork. An axis bearing none bears leaves.
    pub scions: Vec<usize>,
}

impl Axis {
    /// A stem, traced from the ground.
    pub fn stem(rings: Vec<Ring>, rng: Rng) -> Self {
        let reach = rings[rings.len() - 1].at;
        Self {
            level: 0,
            kind: AxisKind::Stem,
            parent: None,
            rings,
            reach,
            rng,
            scions: Vec::new(),
        }
    }

    /// An axis placed on `parent` (an index and a distance along it) and not yet traced: its
    /// start ring, at its own base radius, and how far it is to grow.
    pub fn placed(
        level: usize,
        kind: AxisKind,
        parent: (usize, f32),
        start: Ring,
        reach: f32,
        rng: Rng,
    ) -> Self {
        Self {
            level,
            kind,
            parent: Some(parent),
            rings: vec![start],
            reach,
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

    /// The axis at `t` (0 to 1) along it (`frame_at`).
    pub fn frame(&self, t: f32) -> Frame {
        self.frame_at(t.clamp(0.0, 1.0) * self.length())
    }

    /// The axis `at` metres along it: on the straight chord between ring centers, with that
    /// chord's direction, so once hinted, scions start inside the bark as meshed. A point on a ring
    /// takes the chord after it, so a card at the start of a branch end's straight run lies along
    /// the run.
    pub fn frame_at(&self, at: f32) -> Frame {
        let rings = &self.rings;
        let next = rings
            .partition_point(|ring| ring.at <= at + 1e-5)
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
            radius: a.radius + (b.radius - a.radius) * along,
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

    /// The radii at the base and the tip.
    pub fn radii(&self) -> (f32, f32) {
        (
            self.rings[0].radius,
            self.rings[self.rings.len() - 1].radius,
        )
    }
}

/// Writes the plant's taper on `rings`: `base` at the start, narrowing evenly by `taper` of it
/// over their length (D22).
pub(super) fn taper_rings(rings: &mut [Ring], base: f32, taper: f32) {
    let length = rings[rings.len() - 1].at.max(1e-4);
    for ring in rings {
        ring.radius = base * (1.0 - taper * ring.at / length);
    }
}

/// A leaf card whose base lies `at` metres along the axis `axis` (D27).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Card {
    pub axis: usize,
    pub at: f32,
    /// Unit direction from its base: along the twig, or turned from it in a rosette.
    pub direction: Vec3,
}

/// The plant as grown, before meshing.
pub(super) struct Skeleton {
    /// Stems first, then each level's limbs in turn.
    pub axes: Vec<Axis>,
    pub cards: Vec<Card>,
}

/// One limb level as it grows: its parameters, and what each of its stages reads.
pub(super) struct Level<'a> {
    /// 1 to 3.
    pub number: usize,
    pub params: &'a LimbLevel,
    pub envelope: Envelope,
    /// The plant's taper (D22), 0 to 1.
    pub taper: f32,
    /// The distance between kinks (`kink_spacing`).
    pub spacing: f32,
    /// How far back from its tip a limb runs straight, under its tip's card (D17), whether or not
    /// leaves grow, so turning them off leaves the branches alone.
    pub straight: f32,
}

impl<'a> Level<'a> {
    pub fn new(species: &'a Species, number: usize) -> Self {
        let envelope = Envelope::of(species);
        Self {
            number,
            params: &species.limbs.grown()[number - 1],
            taper: species.taper.clamp(0.0, 1.0),
            spacing: kink_spacing(&envelope),
            straight: axial_fold(species)
                * species.ornaments.tip_leaves.settings.size
                * envelope.radius.max(0.01),
            envelope,
        }
    }
}

/// The stems, hinted (`hint`), their limbs level by level, each level on the axes of the one
/// before, and the leaf clusters (`clusters`). Each level grows in stages, in a fixed order: its
/// laterals are placed (`place_scions`) and traced (`trace_scions`), one more limb carries each
/// parent bearing them on (`carry_on`), the level wanders (`wander`) and is tapered (`taper`), its
/// limbs may fork (`fork`), and it is hinted.
pub(super) fn grow(species: &Species) -> Skeleton {
    let mut skeleton = Skeleton {
        axes: stems(species),
        cards: Vec::new(),
    };
    hint(0, species.detail, &mut skeleton);
    for number in 1..=species.limbs.grown().len() {
        let level = Level::new(species, number);
        place_scions(&level, &mut skeleton);
        trace_scions(&level, &mut skeleton);
        carry_on(&level, &mut skeleton);
        wander(&level, &mut skeleton);
        taper(&level, &mut skeleton);
        fork(&level, &mut skeleton);
        hint(number, species.detail, &mut skeleton);
    }
    skeleton.cards = clusters(species, &skeleton.axes);
    skeleton
}

/// Writes the taper on the level's axes (`taper_rings`), each from the radius its start ring was
/// placed at. A fork's copies come later, and `fork` tapers them as it adds them.
fn taper(level: &Level, skeleton: &mut Skeleton) {
    let axes = skeleton.axes.iter_mut();
    for axis in axes.filter(|axis| axis.level == level.number) {
        let base = axis.rings[0].radius;
        taper_rings(&mut axis.rings, base, level.taper);
    }
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

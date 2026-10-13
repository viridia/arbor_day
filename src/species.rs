//! The species: every parameter that shapes a plant, edited in the panel and read by the generator.

use bevy::{feathers::controls::NumberInputUnits, prelude::*, reflect::Typed};

use std::ops::RangeInclusive;

use crate::{
    fuzzy::{Distribution, Fuzzy},
    properties::{Caption, Precision, ShownCount, Spinbox, ValueRange, Wrap},
    random::Seed,
};

/// Most levels of limbs a species can have.
pub const MAX_LIMB_LEVELS: usize = 3;

/// The species being edited. One resource, so any edit rebuilds the whole plant: most parameters
/// change much of the tree anyway.
#[derive(Resource, Reflect, Clone, PartialEq)]
#[reflect(Resource)]
pub struct Species {
    /// Where the plant's random numbers start: the same seed makes the same plant.
    #[reflect(@Caption("Seed"))]
    pub seed: Seed,
    #[reflect(@Caption("Envelope"))]
    pub envelope: Envelope,
    #[reflect(@Caption("Trunk"))]
    pub trunk: Trunk,
    /// How many of the limb levels grow, from the first. The rest keep their parameters, so a
    /// level hidden and shown again comes back as it was.
    #[reflect(@ValueRange::<i32>(1..=MAX_LIMB_LEVELS as i32))]
    pub limb_levels: i32,
    /// Each level grows on the limbs of the one before it, the first on the trunk.
    #[reflect(@Caption("Limbs"), @ShownCount("limb_levels"))]
    pub limbs: [LimbLevel; MAX_LIMB_LEVELS],
    #[reflect(@Caption("Bark"))]
    pub bark: Bark,
}

impl Species {
    /// The levels that grow.
    pub fn grown_levels(&self) -> &[LimbLevel] {
        let count = (self.limb_levels.max(1) as usize).min(MAX_LIMB_LEVELS);
        &self.limbs[..count]
    }
}

impl Default for Species {
    fn default() -> Self {
        Self {
            seed: Seed(1),
            envelope: Envelope::default(),
            trunk: Trunk::default(),
            limb_levels: 1,
            limbs: [
                LimbLevel::default(),
                LimbLevel {
                    density: 9.0,
                    per_node: 2.0,
                    divergence: degrees(90.0, 20.0),
                    span_middle: Fuzzy::exactly(0.6),
                    span_length: Fuzzy::exactly(0.6),
                    angle: degrees(45.0, 10.0),
                    bend: 0.5,
                    wander: 30.0_f32.to_radians(),
                    reach: Fuzzy::new(0.6, 0.1, Distribution::Bell),
                    radius: 0.5,
                    sides: 3,
                    ..LimbLevel::default()
                },
                LimbLevel {
                    density: 19.0,
                    per_node: 2.0,
                    divergence: degrees(90.0, 20.0),
                    span_middle: Fuzzy::exactly(0.65),
                    span_length: Fuzzy::exactly(0.5),
                    angle: degrees(40.0, 10.0),
                    bend: 0.5,
                    wander: 40.0_f32.to_radians(),
                    reach: Fuzzy::new(0.5, 0.1, Distribution::Bell),
                    radius: 0.5,
                    sides: 3,
                    ..LimbLevel::default()
                },
            ],
            bark: Bark::default(),
        }
    }
}

/// The crown's outline: a surface of revolution about the vertical axis, between the crown base
/// and the top. Limbs reach to it and foliage fills it.
#[derive(Reflect, Clone, PartialEq)]
pub struct Envelope {
    pub profile: Profile,
    /// The top of the crown above the ground, in meters.
    #[reflect(@ValueRange::<f32>(0.5..=30.0), @Precision(1))]
    pub height: f32,
    /// Where the crown starts, as a fraction of its height.
    #[reflect(
        @ValueRange::<f32>(0.0..=0.95),
        @Precision(2),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Crown base"),
    )]
    pub crown_base: f32,
    /// The crown's radius where it is widest, in meters.
    #[reflect(@ValueRange::<f32>(0.1..=15.0), @Precision(1))]
    pub radius: f32,
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            profile: Profile::Ellipsoid,
            height: 6.0,
            crown_base: 0.3,
            radius: 2.5,
        }
    }
}

/// The shape of the crown's outline, from the crown base to the top.
#[derive(Reflect, Clone, Copy, PartialEq, Debug)]
pub enum Profile {
    /// Widest at the base, narrowing to a point.
    Cone,
    /// Widest halfway up, round at both ends.
    Ellipsoid,
    /// Widest at the base, round at the top.
    Hemisphere,
    /// The same width all the way up.
    Column,
    /// Widest 70% of the way up, then narrowing to a point.
    Flame,
}

/// The main stem, from the ground to the top of the plant.
#[derive(Reflect, Clone, PartialEq)]
pub struct Trunk {
    /// Length along the axis, in meters.
    #[reflect(@ValueRange::<f32>(0.5..=20.0), @Precision(1))]
    pub height: f32,
    /// Radius at the base, in meters.
    #[reflect(@ValueRange::<f32>(0.02..=1.0), @Precision(2))]
    pub radius: f32,
    /// How much narrower the top is than the base: 0 keeps the radius, 1 closes to a point.
    #[reflect(
        @ValueRange::<f32>(0.0..=1.0),
        @Precision(2),
        @NumberInputUnits::new_static("ratio_percent"),
    )]
    pub taper: f32,
    /// Sides of each ring.
    #[reflect(@ValueRange::<i32>(3..=8), @Spinbox)]
    pub sides: i32,
    /// How far the axis turns from vertical between the base and the top.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::FRAC_PI_2),
        @NumberInputUnits::new_static("angle_degrees"),
    )]
    pub lean: f32,
}

/// One level of limbs on a parent, the trunk or a limb of the level before, in nodes spaced
/// evenly along a span of the parent. Each limb leaves at an angle from its parent and grows
/// toward the envelope, bending toward an angle from world up and wandering as it goes.
#[derive(Reflect, Clone, PartialEq)]
pub struct LimbLevel {
    /// Places where limbs start, per crown radius of the parent's span, so a long parent bears
    /// more than a short one. Measured against the crown, so a shrub and a tree with the same
    /// settings branch alike. A parent's count is rounded up or down at random, up with the chance
    /// of its fraction.
    #[reflect(@ValueRange::<f32>(0.0..=40.0), @Precision(1), @Caption("Nodes per crown radius"))]
    pub density: f32,
    /// Limbs at each node, evenly spaced around it: 1 for alternate or spiral, 2 for opposite,
    /// more for a whorl. Rounded at random for each node, so 1.5 makes half the nodes opposite.
    #[reflect(@ValueRange::<f32>(1.0..=6.0), @Precision(1), @Caption("Per node"))]
    pub per_node: f32,
    /// The turn about the parent from one node to the next: 137.5° for a spiral, 180° for
    /// alternate, 90° for opposite pairs crossing. Drawn for each node.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::TAU),
        @Wrap,
        @NumberInputUnits::new_static("angle_degrees"),
    )]
    pub divergence: Fuzzy,
    /// The middle of the stretch of the parent bearing nodes, as a fraction of the parent's
    /// length. Drawn for each parent.
    #[reflect(
        @ValueRange::<f32>(0.0..=1.0),
        @Precision(2),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Span middle"),
    )]
    pub span_middle: Fuzzy,
    /// The length of the stretch bearing nodes, as a fraction of the parent's length; it is cut
    /// short where it would pass either end of the parent. Drawn for each parent.
    #[reflect(
        @ValueRange::<f32>(0.0..=1.0),
        @Precision(2),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Span length"),
    )]
    pub span_length: Fuzzy,
    /// The angle from the parent's direction where the limb starts: 0 runs along it, 90° square
    /// to it. Drawn for each limb.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::PI),
        @NumberInputUnits::new_static("angle_degrees"),
    )]
    pub angle: Fuzzy,
    /// The angle from world up the limb bends toward, keeping its compass heading: 0 up, as a
    /// poplar's, 90° level, as an oak's, 180° down, as a weeping tree's.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::PI),
        @NumberInputUnits::new_static("angle_degrees"),
        @Caption("Bend toward"),
    )]
    pub bend_toward: f32,
    /// How much of the angle left to its bend the limb turns through over a length equal to the
    /// crown's radius: 0 keeps it straight. Measured against the crown, so a shrub and a tree
    /// with the same settings bend alike, and against what is left, so every bend target moves
    /// the tip, and longer limbs bend further.
    #[reflect(
        @ValueRange::<f32>(0.0..=0.99),
        @Precision(2),
        @NumberInputUnits::new_static("ratio_percent"),
    )]
    pub bend: f32,
    /// The most the limb turns at random, per metre, so it winds rather than running straight.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::FRAC_PI_2),
        @NumberInputUnits::new_static("angle_degrees"),
        @Caption("Wander per metre"),
    )]
    pub wander: f32,
    /// Length as a fraction of the distance from the limb's start to the envelope along its
    /// starting direction. A limb also stops where it leaves the envelope. Drawn for each limb.
    #[reflect(
        @ValueRange::<f32>(0.0..=1.0),
        @Precision(2),
        @NumberInputUnits::new_static("ratio_percent"),
    )]
    pub reach: Fuzzy,
    /// Radius at the base, as a fraction of the parent's radius where the limb starts. Limbs taper
    /// as the trunk does.
    #[reflect(
        @ValueRange::<f32>(0.05..=1.0),
        @Precision(2),
        @NumberInputUnits::new_static("ratio_percent"),
    )]
    pub radius: f32,
    /// Sides of each ring.
    #[reflect(@ValueRange::<i32>(3..=8), @Spinbox)]
    pub sides: i32,
}

impl Default for LimbLevel {
    fn default() -> Self {
        Self {
            density: 8.0,
            per_node: 1.0,
            divergence: degrees(137.5, 10.0),
            span_middle: Fuzzy::exactly(0.65),
            span_length: Fuzzy::exactly(0.5),
            angle: degrees(60.0, 10.0),
            bend_toward: 90.0_f32.to_radians(),
            bend: 0.4,
            wander: 20.0_f32.to_radians(),
            reach: Fuzzy::new(0.9, 0.1, Distribution::Bell),
            radius: 0.5,
            sides: 4,
        }
    }
}

impl LimbLevel {
    /// The limits of the field `name` (its `ValueRange`), to which values drawn for it are
    /// clamped.
    pub fn limits(name: &str) -> RangeInclusive<f32> {
        Self::type_info()
            .as_struct()
            .ok()
            .and_then(|info| info.field(name))
            .and_then(|field| field.custom_attributes().get::<ValueRange<f32>>())
            .map(|range| range.0.clone())
            .unwrap_or_else(|| panic!("LimbLevel has no field `{name}` with a ValueRange<f32>"))
    }
}

/// A bell around an angle, both in degrees, as the panel shows them.
fn degrees(center: f32, spread: f32) -> Fuzzy {
    Fuzzy::new(center.to_radians(), spread.to_radians(), Distribution::Bell)
}

/// The bark's look, the same on every stem.
#[derive(Reflect, Clone, PartialEq)]
pub struct Bark {
    /// The bark's color, written into the mesh as a vertex color, so trunks of different species
    /// can merge into one mesh in the game.
    pub color: Color,
}

impl Default for Bark {
    fn default() -> Self {
        Self {
            color: Color::srgb(0.42, 0.33, 0.25),
        }
    }
}

impl Default for Trunk {
    fn default() -> Self {
        Self {
            height: 5.0,
            radius: 0.2,
            taper: 0.6,
            sides: 6,
            lean: 15.0_f32.to_radians(),
        }
    }
}

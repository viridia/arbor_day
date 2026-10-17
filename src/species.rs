//! The species: every parameter that shapes a plant, edited in the panel and read by the generator.

use bevy::{
    feathers::controls::NumberInputUnits,
    prelude::*,
    reflect::{
        TypeRegistry, Typed,
        serde::{ReflectSerializeWithRegistry, SerializeWithRegistry, TypedReflectSerializer},
    },
};
use serde::ser::{SerializeSeq, Serializer};

use std::ops::RangeInclusive;

use crate::{
    fuzzy::{Distribution, Fuzzy},
    properties::{Caption, ControlGroup, Flatten, Precision, ShownCount, Spinbox, ValueRange},
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
    #[reflect(@Caption("Seed"), @ControlGroup("Plant"))]
    pub seed: Seed,
    /// The smallest detail drawn, in meters, about a pixel at the game's scale: no stem is drawn
    /// narrower, so twigs still show. Only the drawing is thickened: limbs take their share of
    /// their parents' true radii.
    #[reflect(
        @ValueRange::<f32>(0.002..=0.2),
        @Precision(3),
        @Caption("Detail threshold"),
        @ControlGroup("Plant"),
    )]
    pub detail: f32,
    /// How much narrower each stem and limb is at its tip than at its base, narrowing evenly: 0
    /// keeps the radius, 1 closes to a point.
    #[reflect(
        @ValueRange::<f32>(0.0..=1.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Taper"),
        @ControlGroup("Plant"),
    )]
    pub taper: f32,
    /// The bark's color, written into the mesh as a vertex color, so trunks of different species
    /// can merge into one mesh in the game.
    #[reflect(@Caption("Bark color"), @ControlGroup("Plant"))]
    pub bark_color: Color,
    #[reflect(@Caption("Trunk"))]
    pub trunk: Trunk,
    #[reflect(@Caption("Crown"))]
    pub crown: Crown,
    #[reflect(@Flatten)]
    pub limbs: LimbLevels,
    #[reflect(@Caption("Foliage"))]
    pub ornaments: Ornaments,
}

impl Default for Species {
    fn default() -> Self {
        Self {
            seed: Seed(1),
            detail: 0.02,
            bark_color: Color::srgb(0.42, 0.33, 0.25),
            taper: 0.6,
            trunk: Trunk::default(),
            crown: Crown::default(),
            limbs: LimbLevels::default(),
            ornaments: Ornaments::default(),
        }
    }
}

/// The limb levels, each growing on the limbs of the one before it, the first on the trunk (D15).
/// All three are kept, so a level hidden and shown again comes back as it was. A file holds the
/// shown levels only, as a list.
#[derive(Reflect, Clone, PartialEq)]
#[reflect(SerializeWithRegistry)]
pub struct LimbLevels {
    /// How many of the levels grow, from the first.
    #[reflect(@ValueRange::<i32>(1..=MAX_LIMB_LEVELS as i32))]
    pub shown: i32,
    #[reflect(@Caption("Limbs"), @ShownCount("shown"))]
    pub levels: [LimbLevel; MAX_LIMB_LEVELS],
}

impl LimbLevels {
    /// The levels that grow.
    pub fn grown(&self) -> &[LimbLevel] {
        let count = (self.shown.max(1) as usize).min(MAX_LIMB_LEVELS);
        &self.levels[..count]
    }
}

impl SerializeWithRegistry for LimbLevels {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
        registry: &TypeRegistry,
    ) -> Result<S::Ok, S::Error> {
        let grown = self.grown();
        let mut levels = serializer.serialize_seq(Some(grown.len()))?;
        for level in grown {
            levels.serialize_element(&TypedReflectSerializer::new(level, registry))?;
        }
        levels.end()
    }
}

impl Default for LimbLevels {
    fn default() -> Self {
        Self {
            shown: 1,
            levels: [
                LimbLevel::default(),
                LimbLevel {
                    density: 3.5,
                    per_node: 1.7,
                    facing: Facing::Sides,
                    start: 0.2,
                    angle: degrees(45.0, 10.0),
                    reach: Fuzzy::new(0.6, 0.1, Distribution::Bell),
                    radius: 0.5,
                    sides: 3,
                    bend: Bend {
                        amount: 0.5,
                        ..Bend::default()
                    },
                    wander: Wander::degrees(15.0),
                    ..LimbLevel::default()
                },
                LimbLevel {
                    density: 4.0,
                    per_node: 2.0,
                    facing: Facing::Sides,
                    start: 0.3,
                    angle: degrees(40.0, 10.0),
                    reach: Fuzzy::new(0.5, 0.1, Distribution::Bell),
                    radius: 0.5,
                    sides: 3,
                    bend: Bend {
                        amount: 0.5,
                        ..Bend::default()
                    },
                    wander: Wander::degrees(20.0),
                    ..LimbLevel::default()
                },
            ],
        }
    }
}

/// The crown's envelope: a surface of revolution about the vertical axis, between its base and
/// its top, both measured against the trunk's height, so the crown moves without moving the
/// trunk. Limbs reach to it and foliage fills it.
#[derive(Reflect, Clone, PartialEq)]
pub struct Crown {
    #[reflect(@Caption("Profile"))]
    pub profile: Profile,
    /// The crown's top above the ground, as a multiple of the trunk's height: near 100% for a
    /// tree whose leader runs to the top, more for one whose trunk stops short.
    #[reflect(
        @ValueRange::<f32>(1.0..=2.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Top"),
    )]
    pub top: f32,
    /// Where the crown starts above the ground, as a share of the trunk's height.
    #[reflect(
        @ValueRange::<f32>(0.0..=0.95),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Base"),
    )]
    pub base: f32,
    /// The crown's radius where it is widest, in meters.
    #[reflect(@ValueRange::<f32>(0.1..=15.0), @Precision(1), @Caption("Radius"))]
    pub radius: f32,
}

impl Default for Crown {
    fn default() -> Self {
        Self {
            profile: Profile::Ellipsoid,
            top: 1.25,
            base: 0.4,
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

/// The stems rising from the ground: one for a tree, several for a shrub.
#[derive(Reflect, Clone, PartialEq)]
pub struct Trunk {
    /// Stems rising from the ground, spaced evenly around.
    #[reflect(@ValueRange::<i32>(1..=5), @Spinbox, @Caption("Stems"))]
    pub stems: i32,
    /// The angle from up at which each stem leaves the ground.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::FRAC_PI_3),
        @Precision(1),
        @NumberInputUnits::new_static("angle_degrees"),
        @Caption("Splay"),
    )]
    pub splay: f32,
    /// Each stem's length along its axis in meters, its height if upright: the plant's size, which
    /// the crown is measured against.
    #[reflect(@ValueRange::<f32>(0.1..=30.0), @Precision(2), @Caption("Height"))]
    pub height: f32,
    /// Radius at the base, in meters.
    #[reflect(@ValueRange::<f32>(0.02..=1.0), @Precision(2), @Caption("Radius"))]
    pub radius: f32,
    /// Sides of each ring.
    #[reflect(@ValueRange::<i32>(3..=8), @Spinbox, @Caption("Sides"))]
    pub sides: i32,
    /// How much further from up each stem's axis turns between the base and the top, away from
    /// the middle, but never past level. A tree's compass direction is drawn from the seed, so
    /// neighbouring trees of one species lean different ways.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::FRAC_PI_2),
        @Precision(1),
        @NumberInputUnits::new_static("angle_degrees"),
        @Caption("Lean"),
    )]
    pub lean: f32,
    /// Each stem zigzagging about the course the splay and lean set, as a limb does.
    #[reflect(@Caption("Wander"))]
    pub wander: Wander,
}

/// One level of limbs on a parent, the trunk or a limb of the level before, in nodes along the
/// part of the parent inside the crown. Each limb leaves at an angle from its parent and grows
/// toward the envelope, bending, wandering and forking as its operators say.
#[derive(Reflect, Clone, PartialEq)]
pub struct LimbLevel {
    /// Places where limbs start, per crown radius of the parent inside the crown, so a long parent
    /// bears more than a short one. Measured against the crown, so a shrub and a tree with the
    /// same settings branch alike. A parent's count is rounded up or down at random, up with the
    /// chance of its fraction.
    #[reflect(@ValueRange::<f32>(0.0..=40.0), @Precision(1), @Caption("Nodes per crown radius"), @ControlGroup("Arrangement"))]
    pub density: f32,
    /// Limbs at each node: 1 for a spiral, 2 for opposite pairs, more for a whorl. Rounded at
    /// random for each node, so 1.5 makes half the nodes opposite.
    #[reflect(@ValueRange::<f32>(1.0..=6.0), @Precision(1), @Caption("Per node"), @ControlGroup("Arrangement"))]
    pub per_node: f32,
    /// Which way around the parent its limbs leave.
    #[reflect(@Caption("Facing"), @ControlGroup("Arrangement"))]
    pub facing: Facing,
    /// The most each limb turns about the parent at random, away from where its facing puts it.
    /// Drawn for each limb.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::FRAC_PI_2),
        @Precision(1),
        @NumberInputUnits::new_static("angle_degrees"),
        @Caption("Skew"),
        @ControlGroup("Arrangement"),
    )]
    pub skew: f32,
    /// Where nodes start along the part of the parent inside the crown, as a share of it: 0 from
    /// its base, higher to keep the limbs toward its tip, as a tree's are. Nodes are spread evenly
    /// from there to the tip, where one limb carries the parent on.
    #[reflect(
        @ValueRange::<f32>(0.0..=0.9),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Start"),
        @ControlGroup("Arrangement"),
    )]
    pub start: f32,
    /// The angle from the parent's direction where the limb starts: 0 runs along it, 90° square
    /// to it. Drawn for each limb.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::PI),
        @Precision(1),
        @NumberInputUnits::new_static("angle_degrees"),
        @Caption("Angle"),
        @ControlGroup("Shape"),
    )]
    pub angle: Fuzzy,
    /// Length as a fraction of the distance from the limb's start to the envelope along its
    /// starting direction. A limb also stops where it leaves the envelope. Drawn for each limb.
    #[reflect(
        @ValueRange::<f32>(0.0..=1.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Reach"),
        @ControlGroup("Shape"),
    )]
    pub reach: Fuzzy,
    /// Radius at the base, as a share of the parent's radius where the limb leaves it: 100% is as
    /// thick as the parent there.
    #[reflect(
        @ValueRange::<f32>(0.05..=1.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Radius"),
        @ControlGroup("Shape"),
    )]
    pub radius: f32,
    /// Sides of each ring.
    #[reflect(@ValueRange::<i32>(3..=8), @Spinbox, @Caption("Sides"), @ControlGroup("Shape"))]
    pub sides: i32,
    #[reflect(@Caption("Bend"))]
    pub bend: Bend,
    #[reflect(@Caption("Wander"))]
    pub wander: Wander,
    #[reflect(@Caption("Fork"))]
    pub fork: Fork,
}

impl Default for LimbLevel {
    fn default() -> Self {
        Self {
            density: 8.0,
            per_node: 1.0,
            facing: Facing::Around,
            skew: 20.0_f32.to_radians(),
            start: 0.0,
            angle: degrees(60.0, 10.0),
            reach: Fuzzy::new(0.9, 0.1, Distribution::Bell),
            radius: 0.5,
            sides: 4,
            bend: Bend::default(),
            wander: Wander::degrees(10.0),
            fork: Fork::default(),
        }
    }
}

/// The limits of `T`'s field `name` (its `ValueRange`), to which values drawn for it are clamped.
pub fn limits<T: Typed>(name: &str) -> RangeInclusive<f32> {
    T::type_info()
        .as_struct()
        .ok()
        .and_then(|info| info.field(name))
        .and_then(|field| field.custom_attributes().get::<ValueRange<f32>>())
        .map(|range| range.0.clone())
        .unwrap_or_else(|| {
            let path = T::type_info().type_path();
            panic!("{path} has no field `{name}` with a ValueRange<f32>")
        })
}

/// A limb's course turning, as it grows, toward an angle from world up, keeping its compass
/// heading. Without it the course is straight.
#[derive(Reflect, Clone, PartialEq)]
pub struct Bend {
    /// The angle from world up the limb bends toward: 0 up, as a poplar's, 90° level, as an oak's,
    /// 180° down, as a weeping tree's.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::PI),
        @Precision(1),
        @NumberInputUnits::new_static("angle_degrees"),
        @Caption("Toward"),
    )]
    pub toward: f32,
    /// How much of the angle left to its target the limb turns through over a length equal to
    /// the crown's radius: 0 keeps it straight. Measured against the crown, so a shrub and a tree
    /// with the same settings bend alike, and against what is left, so every target moves the
    /// tip, and longer limbs bend further.
    #[reflect(
        @ValueRange::<f32>(0.0..=0.99),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Amount"),
    )]
    pub amount: f32,
}

impl Default for Bend {
    fn default() -> Self {
        Self {
            toward: 90.0_f32.to_radians(),
            amount: 0.4,
        }
    }
}

/// A stem or limb zigzagging about its course rather than running smooth, its tip where the course
/// ends. Kinks fall about one spacing apart on every level, so long limbs kink several times and
/// short twigs once or not at all.
#[derive(Reflect, Clone, PartialEq)]
pub struct Wander {
    /// The most each stretch points off the course.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::FRAC_PI_4),
        @Precision(1),
        @NumberInputUnits::new_static("angle_degrees"),
        @Caption("Angle"),
    )]
    pub angle: f32,
}

impl Wander {
    /// A wander of `angle` degrees.
    pub fn degrees(angle: f32) -> Self {
        Self {
            angle: angle.to_radians(),
        }
    }
}

/// A limb ending partway along, and copies of it carrying it on from there, of its level, so they
/// reach the envelope and bear limbs as it would.
#[derive(Reflect, Clone, PartialEq)]
pub struct Fork {
    /// The chance that a limb forks.
    #[reflect(
        @ValueRange::<f32>(0.0..=1.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Chance"),
    )]
    pub chance: f32,
    /// Where along a limb it forks, as a share of its length. Drawn for each limb.
    #[reflect(
        @ValueRange::<f32>(0.05..=0.95),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("At"),
    )]
    pub at: Fuzzy,
    /// Copies carrying a forked limb on, spread evenly around its direction.
    #[reflect(@ValueRange::<i32>(2..=4), @Spinbox, @Caption("Copies"))]
    pub copies: i32,
    /// The angle across a fork: each copy turns half of it from the limb's direction. Drawn for
    /// each fork.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::PI),
        @Precision(1),
        @NumberInputUnits::new_static("angle_degrees"),
        @Caption("Angle"),
    )]
    pub angle: Fuzzy,
    /// Where the copies lie around the limb: at 0, two copies spread level, side by side; at 90°,
    /// one above the other.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::FRAC_PI_2),
        @Precision(1),
        @NumberInputUnits::new_static("angle_degrees"),
        @Caption("Roll"),
    )]
    pub roll: f32,
}

impl Default for Fork {
    fn default() -> Self {
        Self {
            chance: 0.0,
            at: Fuzzy::new(0.5, 0.1, Distribution::Bell),
            copies: 2,
            angle: degrees(40.0, 10.0),
            roll: 0.0,
        }
    }
}

/// Which way around its parent a level's limbs leave, the parent's upper side being the one
/// nearest world up. Each limb then turns by up to the level's skew.
#[derive(Reflect, Clone, Copy, PartialEq, Debug)]
pub enum Facing {
    /// Each node where the limbs below crowd it least, so one limb a node spirals by about
    /// 137.5°, pairs cross and whorls alternate, as phyllotaxis does.
    Around,
    /// To the parent's flanks, in a flat spray (amphitony, as in elm and beech): one limb a node
    /// on the less crowded flank, so they alternate, a pair on both.
    Sides,
    /// From the parent's upper side (epitony).
    Up,
    /// From the parent's underside (hypotony).
    Down,
}

/// A bell around an angle, both in degrees, as the panel shows them.
fn degrees(center: f32, spread: f32) -> Fuzzy {
    Fuzzy::new(center.to_radians(), spread.to_radians(), Distribution::Bell)
}

/// What is placed on the grown limbs without changing them: so far the leaves at their tips.
#[derive(Reflect, Clone, PartialEq)]
pub struct Ornaments {
    /// Whether leaves grow; off for a bare plant.
    #[reflect(@Caption("Leaves"))]
    pub leaves: bool,
    #[reflect(@Caption("Tip leaves"))]
    pub tip_leaves: TipLeaves,
}

impl Default for Ornaments {
    fn default() -> Self {
        Self {
            leaves: true,
            tip_leaves: TipLeaves::default(),
        }
    }
}

/// A leaf cluster at the tip of every limb that bears none, so the leaves follow the branch ends:
/// a card facing up, or on a steep twig what `Upright` says.
#[derive(Reflect, Clone, PartialEq)]
pub struct TipLeaves {
    /// What a steep tip bears.
    #[reflect(@Caption("Upright tips"))]
    pub upright: Upright,
    /// Each card's width, as a fraction of the crown's radius.
    #[reflect(
        @ValueRange::<f32>(0.02..=1.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Size"),
    )]
    pub size: f32,
    /// How far each half of a card turns down about its midrib, folding it along the sprig's
    /// stem; negative turns the halves up, cupping it.
    #[reflect(
        @ValueRange::<f32>(-std::f32::consts::FRAC_PI_2..=std::f32::consts::FRAC_PI_2),
        @Precision(1),
        @NumberInputUnits::new_static("angle_degrees"),
        @Caption("Lateral droop"),
    )]
    pub lateral_droop: f32,
    /// How far the far half of a card bends down about the line across its middle; negative
    /// bends it up.
    #[reflect(
        @ValueRange::<f32>(-std::f32::consts::FRAC_PI_2..=std::f32::consts::FRAC_PI_2),
        @Precision(1),
        @NumberInputUnits::new_static("angle_degrees"),
        @Caption("Axial droop"),
    )]
    pub axial_droop: f32,
}

/// What a tip bears where its twig rises 45° or more above level, where one card along it would
/// begin to stand on end.
#[derive(Reflect, Clone, Copy, PartialEq, Debug)]
pub enum Upright {
    /// No leaves, as on shoots still growing.
    Bare,
    /// One card along the twig, as on a level one: crops and shrubs, seen from above.
    Single,
    /// Two cards from the tip, splayed toward level.
    Double,
    /// Three cards from the tip, splayed toward level, so the tip reads from every side.
    Triple,
}

impl Default for TipLeaves {
    fn default() -> Self {
        Self {
            upright: Upright::Triple,
            size: 0.25,
            // Seedling's most common pair, on the shrubs and two hemlocks.
            lateral_droop: (-16.0_f32).to_radians(),
            axial_droop: 17.0_f32.to_radians(),
        }
    }
}

impl Default for Trunk {
    fn default() -> Self {
        Self {
            stems: 1,
            splay: 0.0,
            height: 5.0,
            radius: 0.2,
            sides: 6,
            lean: 15.0_f32.to_radians(),
            wander: Wander::degrees(8.0),
        }
    }
}

//! The species: every parameter that shapes a plant, edited in the panel and read by the generator.

use bevy::{
    feathers::controls::NumberInputUnits,
    prelude::*,
    reflect::{
        ApplyError, PartialReflect, TypeRegistry, Typed,
        serde::{
            DeserializeWithRegistry, ReflectDeserializeWithRegistry, ReflectSerializeWithRegistry,
            SerializeWithRegistry, TypedReflectDeserializer, TypedReflectSerializer,
        },
    },
};
use serde::{
    de::{self, Deserializer, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
};

use std::{any::TypeId, fmt, ops::RangeInclusive};

use crate::{
    fuzzy::{Distribution, Fuzzy, Interval},
    properties::{
        Caption, ControlGroup, Flatten, Icon, Precision, ShownCount, Spinbox, Stripe, ValueRange,
    },
    random::Seed,
    toggle::{OperatorState, Toggle, all_off, serialize_on},
};

/// Most levels of limbs a species can have.
pub const MAX_LIMB_LEVELS: usize = 3;

/// The panel's [`Stripe`] colors, one per part of the plant, from bark brown to light green, root
/// to leaf; the crown, the envelope rather than a part, neutral grey.
const STRIPE_PLANT: Color = Color::srgb(0.42, 0.29, 0.20);
const STRIPE_TRUNK: Color = Color::srgb(0.52, 0.36, 0.22);
const STRIPE_LIMBS: [Color; MAX_LIMB_LEVELS] = [
    Color::srgb(0.56, 0.45, 0.23),
    Color::srgb(0.52, 0.53, 0.25),
    Color::srgb(0.44, 0.60, 0.28),
];
const STRIPE_FOLIAGE: Color = Color::srgb(0.50, 0.76, 0.36);
const STRIPE_CROWN: Color = Color::srgb(0.48, 0.48, 0.48);

/// The species being edited. One resource, so any edit rebuilds the whole plant: most parameters
/// change much of the tree anyway.
#[derive(Resource, Reflect, Clone, PartialEq)]
#[reflect(Resource)]
pub struct Species {
    /// Where the plant's random numbers start: the same seed makes the same plant.
    #[reflect(@Caption("Seed"), @ControlGroup("Plant"), @Stripe(vec![STRIPE_PLANT]))]
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
    /// Dark outlines, as the rest of the game has: the bark drawn again as an inverted hull, and
    /// the leaf texture painted with a stroke around the sprig.
    #[reflect(@Caption("Outline"), @ControlGroup("Plant"))]
    pub outline: bool,
    /// How far the bark's outline stands out from its surface, in meters, as the game reads it.
    #[reflect(
        @ValueRange::<f32>(0.001..=0.05),
        @Precision(3),
        @Caption("Bark outline"),
        @ControlGroup("Plant"),
    )]
    pub bark_outline: f32,
    /// The width of the leaf texture's stroke, in texture pixels.
    #[reflect(
        @ValueRange::<f32>(0.5..=12.0),
        @Precision(1),
        @Caption("Leaf outline"),
        @ControlGroup("Plant"),
    )]
    pub leaf_outline: f32,
    #[reflect(@Caption("Trunk"), @Stripe(vec![STRIPE_TRUNK]))]
    pub trunk: Trunk,
    #[reflect(@Caption("Crown"), @Stripe(vec![STRIPE_CROWN]))]
    pub crown: Crown,
    #[reflect(@Flatten)]
    pub limbs: LimbLevels,
    #[reflect(@Caption("Foliage"), @Stripe(vec![STRIPE_FOLIAGE]))]
    pub ornaments: Ornaments,
}

impl Default for Species {
    fn default() -> Self {
        Self {
            seed: Seed(1),
            detail: 0.02,
            bark_color: Color::srgb(0.42, 0.33, 0.25),
            taper: 0.9,
            outline: true,
            // Panoply's models' width.
            bark_outline: 0.01,
            leaf_outline: 4.0,
            trunk: Trunk::default(),
            crown: Crown::default(),
            limbs: LimbLevels::default(),
            ornaments: Ornaments::default(),
        }
    }
}

impl Species {
    /// The leaf texture's stroke width in pixels, if outlined.
    pub fn leaf_stroke(&self) -> Option<f32> {
        self.outline.then_some(self.leaf_outline)
    }
}

/// The limb levels, each growing on the limbs of the one before it, the first on the trunk (D15).
/// All three are kept, so a level hidden and shown again comes back as it was. A file holds the
/// shown levels only, as a list; read, each lands on its depth's defaults, and the levels after
/// them are their depths' defaults.
#[derive(Reflect, Clone, PartialEq)]
#[reflect(SerializeWithRegistry, DeserializeWithRegistry)]
pub struct LimbLevels {
    /// How many of the levels grow, from the first.
    #[reflect(@ValueRange::<i32>(1..=MAX_LIMB_LEVELS as i32))]
    pub shown: i32,
    #[reflect(@Caption("Limbs"), @ShownCount("shown"), @Stripe(STRIPE_LIMBS.to_vec()))]
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

impl<'de> DeserializeWithRegistry<'de> for LimbLevels {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
        registry: &TypeRegistry,
    ) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(LevelsVisitor { registry })
    }
}

/// Reads a file's list of limb levels, each onto its depth's defaults with every operator off, so
/// an operator the file leaves out is off.
struct LevelsVisitor<'a> {
    registry: &'a TypeRegistry,
}

impl<'de> Visitor<'de> for LevelsVisitor<'_> {
    type Value = LimbLevels;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a list of 1 to {MAX_LIMB_LEVELS} limb levels")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<LimbLevels, A::Error> {
        let registration = self
            .registry
            .get(TypeId::of::<LimbLevel>())
            .ok_or_else(|| de::Error::custom("LimbLevel is not registered"))?;
        let mut levels = Vec::new();
        while let Some(level) =
            seq.next_element_seed(TypedReflectDeserializer::new(registration, self.registry))?
        {
            if levels.len() == MAX_LIMB_LEVELS {
                return Err(de::Error::invalid_length(levels.len() + 1, &self));
            }
            levels.push(level);
        }
        if levels.is_empty() {
            return Err(de::Error::invalid_length(0, &self));
        }
        LimbLevels::from_read(&levels).map_err(de::Error::custom)
    }
}

impl LimbLevels {
    /// The levels read from a file, one to [`MAX_LIMB_LEVELS`], each applied onto its depth's
    /// defaults with every operator off; the levels after them are their depths' defaults.
    pub fn from_read(levels: &[Box<dyn PartialReflect>]) -> Result<Self, ApplyError> {
        let mut limbs = LimbLevels::default();
        for (depth, level) in limbs.levels.iter_mut().zip(levels) {
            all_off(depth);
            depth.try_apply(&**level)?;
        }
        limbs.shown = levels.len() as i32;
        Ok(limbs)
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
                    sag: Toggle::off(Sag::degrees(20.0)),
                    wander: Toggle::off(Wander::degrees(15.0)),
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
                    sag: Toggle::off(Sag::degrees(20.0)),
                    wander: Toggle::off(Wander::degrees(20.0)),
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

/// The stems rising from the ground: one for a tree, several for a shrub. A file holds the
/// operators that are not off.
#[derive(Reflect, Clone, PartialEq)]
#[reflect(SerializeWithRegistry)]
pub struct Trunk {
    /// Its length along its axis in meters, its height if upright, and each stem's length from the
    /// ground to its tip where it forks: the plant's size, which the crown is measured against.
    #[reflect(@ValueRange::<f32>(0.1..=30.0), @Precision(2), @Caption("Height"))]
    pub height: f32,
    /// Radius at the base, in meters.
    #[reflect(@ValueRange::<f32>(0.01..=1.0), @Precision(2), @Caption("Radius"))]
    pub radius: f32,
    /// Sides of each ring.
    #[reflect(@ValueRange::<i32>(3..=8), @Spinbox, @Caption("Sides"))]
    pub sides: i32,
    #[reflect(@Caption("Lean"))]
    pub lean: Toggle<Lean>,
    /// Each stem zigzagging about the course the lean sets, as a limb does.
    #[reflect(@Caption("Wander"))]
    pub wander: Toggle<Wander>,
    #[reflect(@Caption("Flare"))]
    pub flare: Toggle<Flare>,
    /// The trunk splitting into stems, as codominant trunks do (a vase elm, an old oak), or at the
    /// ground into a clump.
    #[reflect(@Caption("Fork"))]
    pub fork: Toggle<Fork>,
}

impl SerializeWithRegistry for Trunk {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
        registry: &TypeRegistry,
    ) -> Result<S::Ok, S::Error> {
        serialize_on(self, serializer, registry)
    }
}

/// One level of limbs on a parent, the trunk or a limb of the level before, in nodes along the
/// part of the parent inside the crown. Each limb leaves at an angle from its parent and grows
/// toward the envelope, sagging, rising, wandering and forking as its operators say. A file holds the
/// operators that are on.
#[derive(Reflect, Clone, PartialEq)]
#[reflect(SerializeWithRegistry)]
pub struct LimbLevel {
    /// Places where limbs start, per crown radius of the parent inside the crown, so a long parent
    /// bears more than a short one. Measured against the crown, so a shrub and a tree with the
    /// same settings branch alike. A parent's count is rounded up or down at random, up with the
    /// chance of its fraction.
    #[reflect(@ValueRange::<f32>(0.0..=40.0), @Precision(1), @Caption("Nodes per crown radius"), @ControlGroup("Arrangement"), @Icon("icons/groups/arrangement.png"))]
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
    /// from there to the end.
    #[reflect(
        @ValueRange::<f32>(0.0..=0.9),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Start"),
        @ControlGroup("Arrangement"),
    )]
    pub start: f32,
    /// Where nodes end along the part of the parent inside the crown, as a share of it: lower to
    /// leave the parent's tip bare, so it leads past its last limbs. Below the start, every node is
    /// at the start.
    #[reflect(
        @ValueRange::<f32>(0.1..=1.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("End"),
        @ControlGroup("Arrangement"),
    )]
    pub end: f32,
    /// The angle from the parent's direction where the limb starts: 0 runs along it, 90° square
    /// to it. Drawn for each limb.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::PI),
        @Precision(1),
        @NumberInputUnits::new_static("angle_degrees"),
        @Caption("Angle"),
        @ControlGroup("Shape"),
        @Icon("icons/groups/shape.png"),
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
    #[reflect(@Caption("Sag"))]
    pub sag: Toggle<Sag>,
    #[reflect(@Caption("Rise"))]
    pub rise: Toggle<Rise>,
    #[reflect(@Caption("Wander"))]
    pub wander: Toggle<Wander>,
    #[reflect(@Caption("Bias"))]
    pub bias: Toggle<Bias>,
    #[reflect(@Caption("Fork"))]
    pub fork: Toggle<Fork>,
}

impl SerializeWithRegistry for LimbLevel {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
        registry: &TypeRegistry,
    ) -> Result<S::Ok, S::Error> {
        serialize_on(self, serializer, registry)
    }
}

impl Default for LimbLevel {
    fn default() -> Self {
        Self {
            density: 8.0,
            per_node: 1.0,
            facing: Facing::Around,
            skew: 20.0_f32.to_radians(),
            start: 0.0,
            end: 0.85,
            angle: degrees(60.0, 10.0),
            reach: Fuzzy::new(0.9, 0.1, Distribution::Bell),
            radius: 0.5,
            sides: 4,
            sag: Toggle::off(Sag::default()),
            rise: Toggle::off(Rise::default()),
            wander: Toggle::off(Wander::degrees(10.0)),
            bias: Toggle::off(Bias::default()),
            fork: Toggle::off(Fork::default()),
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

/// The trunk curving away from up as it rises, toward a compass heading drawn from the seed, so
/// neighbouring trees of one species lean different ways. Without it the stems grow straight.
#[derive(Reflect, Clone, PartialEq)]
#[reflect(@Icon("icons/operators/bend.png"))]
pub struct Lean {
    /// How much further from up the trunk turns between the base and the top, but never past
    /// level; a fork's stems turn by what is left, away from the middle.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::FRAC_PI_2),
        @Precision(1),
        @NumberInputUnits::new_static("angle_degrees"),
        @Caption("Angle"),
    )]
    pub angle: f32,
}

impl Lean {
    /// A lean of `angle` degrees.
    pub fn degrees(angle: f32) -> Self {
        Self {
            angle: angle.to_radians(),
        }
    }
}

impl Default for Lean {
    fn default() -> Self {
        Self::degrees(15.0)
    }
}

/// The stems widening toward the ground, as a mature tree's do where the roots spread, over a few
/// times their base radius.
#[derive(Reflect, Clone, PartialEq)]
#[reflect(@Icon("icons/operators/taper.png"))]
pub struct Flare {
    /// How much wider a stem is at the ground than the taper makes it, as a share of its base
    /// radius: 100% is twice as wide.
    #[reflect(
        @ValueRange::<f32>(0.0..=2.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Width"),
    )]
    pub width: f32,
}

impl Default for Flare {
    fn default() -> Self {
        Self { width: 0.5 }
    }
}

/// A limb's course turning down under its weight as it grows, keeping its compass heading: most
/// where it is level, and not at all where it points straight up or down, as a cantilever bends.
#[derive(Reflect, Clone, PartialEq)]
#[reflect(@Icon("icons/operators/sag.png"))]
pub struct Sag {
    /// How far a level limb turns down over a length equal to the crown's radius, less the nearer
    /// it points up or down. Measured against the crown, so a shrub and a tree with the same
    /// settings sag alike.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::PI),
        @Precision(1),
        @NumberInputUnits::new_static("angle_degrees"),
        @Caption("Amount"),
    )]
    pub amount: f32,
}

impl Sag {
    /// A sag of `amount` degrees.
    pub fn degrees(amount: f32) -> Self {
        Self {
            amount: amount.to_radians(),
        }
    }
}

impl Default for Sag {
    fn default() -> Self {
        Self::degrees(15.0)
    }
}

/// A limb's course turning up toward the light as it grows, keeping its compass heading; an
/// upright limb has no angle left to turn.
#[derive(Reflect, Clone, PartialEq)]
#[reflect(@Icon("icons/operators/rise.png"))]
pub struct Rise {
    /// How much of its angle from up the limb turns through over a length equal to the crown's
    /// radius: 0 keeps it as it is. Measured against the crown, so a shrub and a tree with the
    /// same settings rise alike.
    #[reflect(
        @ValueRange::<f32>(0.0..=0.99),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Amount"),
    )]
    pub amount: f32,
}

impl Default for Rise {
    fn default() -> Self {
        Self { amount: 0.3 }
    }
}

/// A stem or limb zigzagging about its course rather than running smooth, its tip where the course
/// ends. Kinks fall about one spacing apart on every level, so long limbs kink several times and
/// short twigs once or not at all.
#[derive(Reflect, Clone, PartialEq)]
#[reflect(@Icon("icons/operators/wander.png"))]
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

/// Laterals changing along their parent, by their node's place. Size changes their length about
/// the middle node's, which keeps the level's: positive, longer at the base and shorter toward the
/// tip, as a conifer's; negative, the other way, short near a limb's root so its leaves fill the
/// crown's middle. Aim turns them toward the crown's outline, the upper ones toward its top, so
/// their leaves cover the crown's middle from above. Leaves carries the size change to the leaves
/// the laterals bear, so a tall, thin cone keeps its shape.
#[derive(Reflect, Clone, PartialEq)]
#[reflect(@Icon("icons/operators/size-bias.png"))]
pub struct Bias {
    /// How much longer the laterals at the parent's first node are, as a share of the reach, and
    /// shorter at its last, changing evenly between: 60% is 160% at the base and 40% at the tip.
    #[reflect(
        @ValueRange::<f32>(-1.0..=1.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Size"),
    )]
    pub size: f32,
    /// How far the laterals turn from their drawn angle toward a point on the crown's outline, at
    /// their own compass heading: the first node's toward where the crown is widest, the last's
    /// toward its top, those between spread evenly along the outline. 100% points them there.
    /// Meant for the trunk's laterals.
    #[reflect(
        @ValueRange::<f32>(0.0..=1.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Aim"),
    )]
    pub aim: f32,
    /// How much the leaves on the laterals, and on the limbs they bear, follow the laterals'
    /// lengths: at 100%, a lateral half as long as its parent's longest bears leaves at half their
    /// size, the longest's keeping theirs. A parent's own tip takes its last node's. Leaves never
    /// shrink below the tip leaves' smallest.
    #[reflect(
        @ValueRange::<f32>(0.0..=1.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Leaves"),
    )]
    pub leaves: f32,
}

impl Default for Bias {
    fn default() -> Self {
        Self {
            size: 0.0,
            aim: 0.0,
            leaves: 0.0,
        }
    }
}

/// A stem or limb ending partway along, and copies of it carrying it on from there, of its level,
/// so they grow and bear limbs as it would. Forked at its base, its copies take its place.
#[derive(Reflect, Clone, PartialEq)]
#[reflect(@Icon("icons/operators/fork.png"))]
pub struct Fork {
    /// The chance that a stem or limb forks.
    #[reflect(
        @ValueRange::<f32>(0.0..=1.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Chance"),
    )]
    pub chance: f32,
    /// Where along a stem or limb it forks, as a share of its length: at 0, its base. Drawn for
    /// each.
    #[reflect(
        @ValueRange::<f32>(0.0..=0.95),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("At"),
    )]
    pub at: Fuzzy,
    /// Copies carrying a forked stem or limb on, spread evenly around its direction.
    #[reflect(@ValueRange::<i32>(2..=5), @Spinbox, @Caption("Copies"))]
    pub copies: i32,
    /// The angle across a fork: each copy turns half of it from the forked axis's direction.
    /// Drawn for each fork.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::PI),
        @Precision(1),
        @NumberInputUnits::new_static("angle_degrees"),
        @Caption("Angle"),
    )]
    pub angle: Fuzzy,
    /// Where the copies lie around the forked axis: at 0, two copies spread level, side by side; at
    /// 90°, one above the other. Drawn for each fork.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::FRAC_PI_2),
        @Precision(1),
        @NumberInputUnits::new_static("angle_degrees"),
        @Caption("Roll"),
    )]
    pub roll: Fuzzy,
}

impl Default for Fork {
    fn default() -> Self {
        Self {
            chance: 0.5,
            at: Fuzzy::new(0.5, 0.1, Distribution::Bell),
            copies: 2,
            angle: degrees(40.0, 10.0),
            roll: degrees(20.0, 20.0),
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

/// What is placed on the grown limbs without changing them, one slot per kind, added and removed as
/// a level's operators are: so far the leaves at their tips. A file holds the slots that are not
/// off.
#[derive(Reflect, Clone, PartialEq)]
#[reflect(SerializeWithRegistry)]
pub struct Ornaments {
    /// Off for a bare plant.
    #[reflect(@Caption("Tip leaves"))]
    pub tip_leaves: Toggle<TipLeaves>,
}

impl SerializeWithRegistry for Ornaments {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
        registry: &TypeRegistry,
    ) -> Result<S::Ok, S::Error> {
        serialize_on(self, serializer, registry)
    }
}

impl Default for Ornaments {
    fn default() -> Self {
        // On, the one slot a new species has on (D33).
        Self {
            tip_leaves: Toggle {
                state: OperatorState::On,
                settings: TipLeaves::default(),
            },
        }
    }
}

/// A leaf cluster at the tip of every limb that bears none, so the leaves follow the branch ends:
/// a card facing up, or on a steep twig what `Upright` says.
#[derive(Reflect, Clone, PartialEq)]
#[reflect(@Icon("icons/operators/tip-leaves.png"))]
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
    /// The least share of its size a card shrinks to by Bias's leaves: lower closes a conifer's
    /// tip, but a smaller card's outline and texture detail show.
    #[reflect(
        @ValueRange::<f32>(0.25..=1.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Smallest"),
    )]
    pub smallest: f32,
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
    #[reflect(@Caption("Sprig"))]
    pub sprig: Sprig,
    #[reflect(@Caption("Leaf"))]
    pub leaf: Leaf,
}

/// How the leaves of the painted sprig are arranged: a fan at the stem's end, and pairs below it
/// down the stem. Lengths are in leaf lengths, the middle leaf's.
#[derive(Reflect, Clone, PartialEq)]
pub struct Sprig {
    /// Leaves in the fan either side of the middle one, each smaller than the one inside it.
    #[reflect(@ValueRange::<i32>(0..=9), @Spinbox, @Caption("Fan"))]
    pub fan: i32,
    /// The fan's outermost leaves' angle from the stem, the leaves inside it spaced evenly, and the
    /// pairs' angle. Each leaf strays from its angle by up to the spread.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::PI),
        @Precision(1),
        @NumberInputUnits::new_static("angle_degrees"),
        @Caption("Angle"),
    )]
    pub angle: Interval,
    /// The size of the fan's outermost leaves and of the pairs, as a share of the middle leaf's.
    #[reflect(
        @ValueRange::<f32>(0.0..=2.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Outer size"),
    )]
    pub outer_size: f32,
    /// Pairs of leaves down the stem below the fan.
    #[reflect(@ValueRange::<i32>(0..=10), @Spinbox, @Caption("Pairs"))]
    pub pairs: i32,
    /// The distance between pairs, in leaf lengths. Each leaf strays from its place by up to the
    /// spread.
    #[reflect(
        @ValueRange::<f32>(0.0..=1.5),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Spacing"),
    )]
    pub spacing: Interval,
}

impl Default for Sprig {
    fn default() -> Self {
        // Seedling's poplar-green, whose texture the placeholder was.
        Self {
            fan: 2,
            angle: Interval::new(50.0_f32.to_radians(), 9.0_f32.to_radians()),
            outer_size: 0.72,
            pairs: 2,
            spacing: Interval::new(0.5, 0.0),
        }
    }
}

/// One painted leaf: a curve from base to tip for each half, mirrored, cut into segments that
/// may end in teeth, and filled with a gradient. Lengths are in leaf lengths.
#[derive(Reflect, Clone, PartialEq)]
pub struct Leaf {
    /// How far out the curve is pulled near the base.
    #[reflect(
        @ValueRange::<f32>(0.01..=1.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Base width"),
    )]
    pub base_width: f32,
    /// How far up the curve is pulled near the base: higher widens the leaf further from it.
    #[reflect(
        @ValueRange::<f32>(-1.0..=1.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Base taper"),
    )]
    pub base_taper: f32,
    /// How far out the curve is pulled near the tip.
    #[reflect(
        @ValueRange::<f32>(0.01..=1.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Tip width"),
    )]
    pub tip_width: f32,
    /// How far down the curve is pulled near the tip: higher widens the leaf further from it.
    #[reflect(
        @ValueRange::<f32>(-1.0..=1.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Tip taper"),
    )]
    pub tip_taper: f32,
    /// Pieces each half is cut into, each but the last ending in a tooth when serrated.
    #[reflect(@ValueRange::<i32>(1..=12), @Spinbox, @Caption("Segments"))]
    pub segments: i32,
    /// How far each tooth stands out from the edge, and the tip beyond the curve.
    #[reflect(
        @ValueRange::<f32>(0.0..=1.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Serration"),
    )]
    pub serration: f32,
    /// How far each tooth is pushed toward the tip, in segment lengths.
    #[reflect(
        @ValueRange::<f32>(0.0..=2.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Rake"),
    )]
    pub rake: f32,
    /// The most each tooth is moved at random, either way along each axis.
    #[reflect(
        @ValueRange::<f32>(0.0..=0.2),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Jitter"),
    )]
    pub jitter: f32,
    #[reflect(@Caption("Inner color"))]
    pub inner_color: Color,
    #[reflect(@Caption("Outer color"))]
    pub outer_color: Color,
    /// Which way the colors run.
    #[reflect(@Caption("Gradient"))]
    pub gradient: Gradient,
    /// How far each leaf's hue and lightness shift at random.
    #[reflect(
        @ValueRange::<f32>(0.0..=1.0),
        @Precision(0),
        @NumberInputUnits::new_static("ratio_percent"),
        @Caption("Variation"),
    )]
    pub variation: f32,
}

impl Default for Leaf {
    fn default() -> Self {
        // Seedling's poplar-green.
        Self {
            base_width: 0.18,
            base_taper: 0.0,
            tip_width: 0.2,
            tip_taper: 0.41,
            segments: 1,
            serration: 0.0,
            rake: 0.0,
            jitter: 0.0,
            inner_color: Color::srgb_u8(65, 146, 146),
            outer_color: Color::srgb_u8(80, 201, 80),
            gradient: Gradient::Axial,
            variation: 0.18,
        }
    }
}

/// Which way a leaf's colors run.
#[derive(Reflect, Clone, Copy, PartialEq, Debug)]
pub enum Gradient {
    /// The inner color down the midrib, the outer at the edges.
    Lateral,
    /// The inner color at the base, the outer at the tip.
    Axial,
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
            smallest: 0.5,
            // Seedling's most common pair, on the shrubs and two hemlocks.
            lateral_droop: (-16.0_f32).to_radians(),
            axial_droop: 17.0_f32.to_radians(),
            sprig: Sprig::default(),
            leaf: Leaf::default(),
        }
    }
}

impl Default for Trunk {
    fn default() -> Self {
        Self {
            height: 5.0,
            radius: 0.2,
            sides: 6,
            lean: Toggle::off(Lean::default()),
            wander: Toggle::off(Wander::degrees(8.0)),
            flare: Toggle::off(Flare::default()),
            // Turned on, a trunk splits: there is only one to fork.
            fork: Toggle::off(Fork {
                chance: 1.0,
                ..Fork::default()
            }),
        }
    }
}

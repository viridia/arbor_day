//! The species: every parameter that shapes a plant, edited in the panel and read by the generator.

use bevy::{feathers::controls::NumberInputUnits, prelude::*};

use crate::properties::{Caption, Precision, Spinbox, ValueRange, Wrap};

/// The species being edited. One resource, so any edit rebuilds the whole plant: most parameters
/// change much of the tree anyway.
#[derive(Resource, Reflect, Clone, PartialEq, Default)]
#[reflect(Resource)]
pub struct Species {
    #[reflect(@Caption("Envelope"))]
    pub envelope: Envelope,
    #[reflect(@Caption("Trunk"))]
    pub trunk: Trunk,
    #[reflect(@Caption("Limbs"))]
    pub limbs: Limbs,
    #[reflect(@Caption("Bark"))]
    pub bark: Bark,
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
    #[reflect(@ValueRange::<f32>(0.0..=0.95), @Precision(2), @Caption("Crown base"))]
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
    #[reflect(@ValueRange::<f32>(0.0..=1.0), @Precision(2))]
    pub taper: f32,
    /// Sides of each ring.
    #[reflect(@ValueRange::<i32>(3..=8), @Spinbox)]
    pub sides: i32,
    /// How far the axis turns from vertical between the base and the top.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::FRAC_PI_2),
        @NumberInputUnits::new_static("angle_degrees"),
    )]
    pub bend: f32,
}

/// The limbs on the trunk, in nodes spaced evenly along a span of it. Each limb leaves at an angle
/// from world up and runs straight toward the envelope.
#[derive(Reflect, Clone, PartialEq)]
pub struct Limbs {
    /// Places along the trunk where limbs start.
    #[reflect(@ValueRange::<i32>(0..=20), @Spinbox)]
    pub nodes: i32,
    /// Limbs at each node, evenly spaced around it: 1 for alternate or spiral, 2 for opposite,
    /// more for a whorl.
    #[reflect(@ValueRange::<i32>(1..=6), @Spinbox, @Caption("Per node"))]
    pub per_node: i32,
    /// The turn about the trunk from one node to the next: 137.5° for a spiral, 180° for
    /// alternate, 90° for opposite pairs crossing.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::TAU),
        @Wrap,
        @NumberInputUnits::new_static("angle_degrees"),
    )]
    pub divergence: f32,
    /// Where the lowest node sits, as a fraction of the trunk's length.
    #[reflect(@ValueRange::<f32>(0.0..=1.0), @Precision(2), @Caption("Span start"))]
    pub start: f32,
    /// Where the highest node sits, as a fraction of the trunk's length.
    #[reflect(@ValueRange::<f32>(0.0..=1.0), @Precision(2), @Caption("Span end"))]
    pub end: f32,
    /// The angle from world up: 0 points straight up, 90° level, 180° straight down.
    #[reflect(
        @ValueRange::<f32>(0.0..=std::f32::consts::PI),
        @NumberInputUnits::new_static("angle_degrees"),
    )]
    pub elevation: f32,
    /// Length as a fraction of the distance from the limb's start to the envelope along it.
    #[reflect(@ValueRange::<f32>(0.0..=1.0), @Precision(2))]
    pub reach: f32,
    /// Radius at the base, as a fraction of the trunk's radius where the limb starts. Limbs taper
    /// as the trunk does.
    #[reflect(@ValueRange::<f32>(0.05..=1.0), @Precision(2))]
    pub radius: f32,
    /// Sides of each ring.
    #[reflect(@ValueRange::<i32>(3..=8), @Spinbox)]
    pub sides: i32,
}

impl Default for Limbs {
    fn default() -> Self {
        Self {
            nodes: 8,
            per_node: 1,
            divergence: 137.5_f32.to_radians(),
            start: 0.4,
            end: 0.9,
            elevation: 60.0_f32.to_radians(),
            reach: 0.9,
            radius: 0.5,
            sides: 4,
        }
    }
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
            bend: 15.0_f32.to_radians(),
        }
    }
}

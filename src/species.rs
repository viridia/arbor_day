//! The species: every parameter that shapes a plant, edited in the panel and read by the generator.

use bevy::{feathers::controls::NumberInputUnits, prelude::*};

use crate::properties::{Caption, Precision, Spinbox, ValueRange};

/// The species being edited. One resource, so any edit rebuilds the whole plant: most parameters
/// change much of the tree anyway.
#[derive(Resource, Reflect, Clone, PartialEq, Default)]
#[reflect(Resource)]
pub struct Species {
    #[reflect(@Caption("Envelope"))]
    pub envelope: Envelope,
    #[reflect(@Caption("Trunk"))]
    pub trunk: Trunk,
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

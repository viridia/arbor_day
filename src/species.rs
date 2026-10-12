//! The species: every parameter that shapes a plant, edited in the panel and read by the generator.

use bevy::{feathers::controls::NumberInputUnits, prelude::*};

use crate::properties::{Precision, Spinbox, ValueRange};

/// The species being edited. One resource, so any edit rebuilds the whole plant: most parameters
/// change much of the tree anyway.
#[derive(Resource, Reflect, Clone, PartialEq, Default)]
#[reflect(Resource)]
pub struct Species {
    pub trunk: Trunk,
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

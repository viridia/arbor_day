//! A parameter that varies from plant to plant: a designed value, how far a plant may stray from
//! it, and the shape of that straying. Named after the fuzzy number, a value with a peak, bounded
//! support either side, and a shape.

use bevy::prelude::*;

/// A designed value and the most a plant may stray from it, either way.
#[derive(Reflect, Clone, Copy, PartialEq, Debug)]
pub struct Fuzzy {
    pub center: f32,
    pub spread: f32,
    pub shape: Distribution,
}

impl Fuzzy {
    pub fn new(center: f32, spread: f32, shape: Distribution) -> Self {
        Self {
            center,
            spread,
            shape,
        }
    }
}

/// How values fall within a [`Fuzzy`]'s spread.
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Distribution {
    /// Every value in the spread equally likely.
    #[default]
    Uniform,
    /// Values near the center more likely, thinning toward the ends of the spread.
    Bell,
}

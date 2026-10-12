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

impl Distribution {
    /// How likely a value is at `t` across the spread (0 at center − spread, 1 at center + spread),
    /// relative to the most likely value.
    pub fn density(self, t: f32) -> f32 {
        match self {
            Distribution::Uniform => 1.0,
            // The mean of three uniform samples. Three times it follows the Irwin–Hall
            // distribution for three, piecewise quadratic and peaking at 3/4.
            Distribution::Bell => {
                let x = 3.0 * t.clamp(0.0, 1.0);
                let density = if x < 1.0 {
                    x * x / 2.0
                } else if x < 2.0 {
                    (-2.0 * x * x + 6.0 * x - 3.0) / 2.0
                } else {
                    (3.0 - x) * (3.0 - x) / 2.0
                };
                density / 0.75
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bell_density_peaks_at_center_and_vanishes_at_ends() {
        assert_eq!(Distribution::Bell.density(0.5), 1.0);
        assert_eq!(Distribution::Bell.density(0.0), 0.0);
        assert_eq!(Distribution::Bell.density(1.0), 0.0);
        // Continuous where the pieces meet.
        let joint = Distribution::Bell.density(1.0 / 3.0);
        assert!((joint - 2.0 / 3.0).abs() < 1e-6);
        assert!((Distribution::Bell.density(1.0 / 3.0 - 1e-4) - joint).abs() < 1e-3);
    }
}

//! A parameter that varies from plant to plant: a designed value, how far a plant may stray from
//! it, and the shape of that straying. Named after the fuzzy number, a value with a peak, bounded
//! support either side, and a shape.

use bevy::prelude::*;

use crate::random::Rng;

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

    /// A value with no spread.
    pub fn exactly(center: f32) -> Self {
        Self::new(center, 0.0, Distribution::Uniform)
    }

    /// A value within center ± spread. Always draws three numbers, whatever the shape, so
    /// toggling one parameter's shape leaves every later draw from `rng` as it was. A caller
    /// with a wrapping angle reduces it modulo a turn; one with limits clamps it.
    pub fn sample(&self, rng: &mut Rng) -> f32 {
        let draws = [rng.unit(), rng.unit(), rng.unit()];
        let t = match self.shape {
            Distribution::Uniform => draws[0],
            Distribution::Bell => (draws[0] + draws[1] + draws[2]) / 3.0,
        };
        self.center + self.spread * (2.0 * t - 1.0)
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

    #[test]
    fn samples_stay_within_the_spread() {
        let mut rng = Rng::stream(3, "test");
        for shape in [Distribution::Uniform, Distribution::Bell] {
            let fuzzy = Fuzzy::new(2.0, 0.5, shape);
            let samples: Vec<f32> = (0..10_000).map(|_| fuzzy.sample(&mut rng)).collect();
            assert!(samples.iter().all(|s| (1.5..=2.5).contains(s)), "{shape:?}");
            let mean = samples.iter().sum::<f32>() / samples.len() as f32;
            assert!((mean - 2.0).abs() < 0.01, "{shape:?}: {mean}");
        }
    }

    #[test]
    fn bell_samples_crowd_the_center() {
        let mut rng = Rng::stream(4, "test");
        let mut near = |shape| {
            let fuzzy = Fuzzy::new(0.0, 1.0, shape);
            (0..10_000)
                .filter(|_| fuzzy.sample(&mut rng).abs() < 0.5)
                .count()
        };
        // Half of uniform samples lie in the middle half of the spread, and 1 − 2 × 0.75³/6, about
        // 86%, of bell samples do.
        let (uniform, bell) = (near(Distribution::Uniform), near(Distribution::Bell));
        assert!((4_800..5_200).contains(&uniform), "{uniform}");
        assert!((8_400..8_800).contains(&bell), "{bell}");
    }

    #[test]
    fn the_shape_does_not_shift_later_draws() {
        let next_after = |shape| {
            let mut rng = Rng::stream(5, "test");
            Fuzzy::new(1.0, 0.2, shape).sample(&mut rng);
            rng.next_u64()
        };
        assert_eq!(
            next_after(Distribution::Uniform),
            next_after(Distribution::Bell)
        );
    }
}

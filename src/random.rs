//! Random numbers for the generator, specified here rather than taken from a crate, so the same
//! seed makes the same plant on every platform and through every dependency update.

/// A SplitMix64 generator (Steele, Lea and Flood, 2014): a 64-bit counter, each output a mix of
/// the next count.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// The stream named `name` for `seed`. Each subsystem draws from a stream of its own, so a
    /// change to one (more limbs, say) leaves the others' numbers as they were.
    pub fn stream(seed: u64, name: &str) -> Self {
        Self {
            state: mix(seed ^ fnv1a(name)),
        }
    }

    /// The next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        mix(self.state)
    }

    /// A number in [0, 1), from the top 24 bits, so every value it can take is equally likely.
    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }
}

/// SplitMix64's output function.
fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// The 64-bit FNV-1a hash of `name`.
fn fnv1a(name: &str) -> u64 {
    name.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ byte as u64).wrapping_mul(0x0100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_reference_outputs() {
        // SplitMix64 from a state of 0, as in the reference implementation.
        let mut rng = Rng { state: 0 };
        assert_eq!(rng.next_u64(), 0xe220_a839_7b1d_cdaf);
        assert_eq!(rng.next_u64(), 0x6e78_9e6a_a1b9_65f4);
        assert_eq!(rng.next_u64(), 0x06c4_5d18_8009_454f);
    }

    #[test]
    fn streams_are_repeatable_and_distinct() {
        let draws = |seed, name| {
            let mut rng = Rng::stream(seed, name);
            [rng.next_u64(), rng.next_u64()]
        };
        assert_eq!(draws(7, "branching"), draws(7, "branching"));
        assert_ne!(draws(7, "branching"), draws(7, "foliage"));
        assert_ne!(draws(7, "branching"), draws(8, "branching"));
    }

    #[test]
    fn units_lie_in_the_unit_interval() {
        let mut rng = Rng::stream(1, "test");
        let units: Vec<f32> = (0..10_000).map(|_| rng.unit()).collect();
        assert!(units.iter().all(|u| (0.0..1.0).contains(u)));
        let mean = units.iter().sum::<f32>() / units.len() as f32;
        assert!((mean - 0.5).abs() < 0.01, "{mean}");
    }
}

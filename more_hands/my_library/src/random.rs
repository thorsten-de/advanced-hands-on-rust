use rand::{Rng, SeedableRng, prelude::StdRng};
use std::ops::Range;

pub struct RandomNumberGenerator {
    rng: StdRng,
}

impl RandomNumberGenerator {
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_os_rng(),
        }
    }
    pub fn seeded(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn range(&mut self, range: Range<u32>) -> u32 {
        self.rng.random_range(range)
    }
}

impl Default for RandomNumberGenerator {
    fn default() -> Self {
        Self::new()
    }
}

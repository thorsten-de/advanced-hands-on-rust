use rand::{
    Rng, SeedableRng,
    distr::{Distribution, StandardUniform},
    prelude::StdRng,
};
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

    pub fn next<T>(&mut self) -> T
    where
        StandardUniform: Distribution<T>,
    {
        self.rng.random()
    }
}

impl Default for RandomNumberGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_range_bounds() {
        let mut rng = RandomNumberGenerator::new();

        for _ in 0..1000 {
            let n = rng.range(1..10);
            assert!(n >= 1);
            assert!(n < 10);
        }
    }

    #[test]
    fn test_reproducibility() {
        let mut rng = (
            RandomNumberGenerator::seeded(1),
            RandomNumberGenerator::seeded(1),
        );

        (0..1000).for_each(|_| {
            assert_eq!(
                rng.0.range(u32::MIN..u32::MAX),
                rng.1.range(u32::MIN..u32::MAX),
            );
        });
    }

    #[test]
    fn test_next_types() {
        let mut rng = RandomNumberGenerator::new();
        let _: i32 = rng.next();
        let _ = rng.next::<f32>();
    }
}

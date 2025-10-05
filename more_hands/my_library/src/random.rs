use rand::{
    Rng, SeedableRng,
    distr::{
        Distribution, StandardUniform,
        uniform::{SampleRange, SampleUniform},
    },
};
#[cfg(all(not(feature = "xorshift"), not(feature = "pcg")))]
type RngCore = rand::prelude::StdRng;

#[cfg(feature = "xorshift")]
type RngCore = rand_xorshift::XorShiftRng;

#[cfg(feature = "pcg")]
type RngCore = rand_pcg::Pcg64Mcg;

/// `RandomNumberGenerator` holds random number generation state and offers
/// random number generation services to your program.
///
/// `RandomNumberGenerator` defaults to using the [PCG](https://crates.io/crates/rand_pcg) algorithm.
/// You can specify `xorshift` as a feature flag to use it instead.
///
/// By default, `RandomNumberGenerator` requires mutability--- it is shared in Bevy with
/// `ResMut<RandomNumberGenerator`. If you prefer interior mutability instead
/// (and use `Res<RandomNumberGenerator>`), specify the `locking` feature flag.
///
/// ## Example
///
/// ```
/// use my_library::RandomNumberGenerator;
/// let mut my_rng = RandomNumberGenerator::new();
/// let random_number = my_rng.range(1..10);
/// println!("{random_number}")
/// ```
#[derive(bevy::prelude::Resource)]
pub struct RandomNumberGenerator {
    rng: RngCore,
}

impl RandomNumberGenerator {
    pub fn new() -> Self {
        Self {
            rng: RngCore::from_os_rng(),
        }
    }
    pub fn seeded(seed: u64) -> Self {
        Self {
            rng: RngCore::seed_from_u64(seed),
        }
    }

    pub fn range<T>(&mut self, range: impl SampleRange<T>) -> T
    where
        T: SampleUniform + PartialOrd,
    {
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

    #[test]
    fn test_float() {
        let mut rng = RandomNumberGenerator::new();

        for _ in 0..1000 {
            let n: f32 = rng.range(-5000.0..5000.0);
            assert!(n.is_finite());
            assert!(n > -5000.0);
            assert!(n < 5000.0);
        }
    }
}

pub struct RandomPlugin;
impl bevy::prelude::Plugin for RandomPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(RandomNumberGenerator::new());
    }
}

use rand::{
    Rng, SeedableRng,
    distr::{
        Distribution, StandardUniform,
        uniform::{SampleRange, SampleUniform},
    },
};
use std::sync::Mutex;

#[cfg(all(not(feature = "xorshift"), not(feature = "pcg")))]
type RngCore = rand::prelude::StdRng;

#[cfg(feature = "xorshift")]
type RngCore = rand_xorshift::XorShiftRng;

#[cfg(feature = "pcg")]
type RngCore = rand_pcg::Pcg64Mcg;

#[derive(bevy::prelude::Resource)]
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
pub struct RandomNumberGenerator {
    pub rng: Mutex<RngCore>,
}

impl RandomNumberGenerator {
    /// Creates a default `RandomNumberGenerator`, with a randomly
    /// selected starting seed
    pub fn new() -> Self {
        Self {
            rng: Mutex::new(RngCore::from_os_rng()),
        }
    }
    /// Creates a `RandomNumberGenerator` with a specified random seed.
    /// Given the same requests, it will produce the *same results* each time.
    ///
    /// # Arguments
    ///
    /// * `seed` - the random seed to use.
    ///
    /// # Example
    ///
    /// ```
    /// use my_library::RandomNumberGenerator;
    /// let mut rng1 = RandomNumberGenerator::seeded(1);
    /// let mut rng2 = RandomNumberGenerator::seeded(1);
    /// let results: (u32, u32) = ( rng1.next(), rng2.next());
    /// assert_eq!(results.0, results.1);
    /// ```
    pub fn seeded(seed: u64) -> Self {
        Self {
            rng: Mutex::new(RngCore::seed_from_u64(seed)),
        }
    }

    /// Generates a random number within a specified range.
    ///
    /// # Arguments
    ///
    /// * `range` - the range (inclusive or exclusive) within which to
    /// generate a random number
    ///
    /// # Example
    ///
    /// ```
    /// use my_library::RandomNumberGenerator;
    /// let mut rng = RandomNumberGenerator::new();
    /// let one_to_nine = rng.range(1..10);
    /// let one_to_ten = rng.range(1..=10);    ///
    /// ```
    pub fn range<T>(&self, range: impl SampleRange<T>) -> T
    where
        T: SampleUniform + PartialOrd,
    {
        let mut lock = self.rng.lock().unwrap();
        lock.random_range(range)
    }

    /// Generates a new random number of the requested type.
    pub fn next<T>(&self) -> T
    where
        StandardUniform: Distribution<T>,
    {
        let mut lock = self.rng.lock().unwrap();
        lock.random()
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
        let rng = RandomNumberGenerator::new();

        for _ in 0..1000 {
            let n = rng.range(1..10);
            assert!(n >= 1);
            assert!(n < 10);
        }
    }

    #[test]
    fn test_reproducibility() {
        let rng = (
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
        let rng = RandomNumberGenerator::new();
        let _: i32 = rng.next();
        let _ = rng.next::<f32>();
    }

    #[test]
    fn test_float() {
        let rng = RandomNumberGenerator::new();

        for _ in 0..1000 {
            let n: f32 = rng.range(-5000.0..5000.0);
            assert!(n.is_finite());
            assert!(n > -5000.0);
            assert!(n < 5000.0);
        }
    }
}

/// `Random` is a Bevy plugin that inserts a `RandomNumberGenerator`
/// Resource into your application.
///
/// Once you add the plugin (with `App::new().add_plugin("RandomPlugin")`),
/// you can access a random number generator in all systems with
/// `rng: ResMut<RandomNumberGenerator>`
pub struct RandomPlugin;
impl bevy::prelude::Plugin for RandomPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(RandomNumberGenerator::new());
    }
}

//! `my_library` provides a suite of helpers to create games with Bevy.
//!
//! Whats included?
//! ---------------
//!
//! `my_library` includes:
//!
//! - Random number generation facilities
//!
//!
//! Feature flags
//! -------------
//!
//! The following feature flags are supported: `xorshift`, `pcg`, `locking`
//!
//! ### Random number generation
//!
//! - The `locking` feature enables interior mutability inside [`RandomNumberGenerator`],
//!   allowing it to be used as a resource (`Res<RandomNumberGenerator`) rather than
//!   requiring mutability (`ResMut<RandomNumberGenerator`)
//! - You can control which random number generation algorithm is used by specifying one of:
//!     - `xorshift` to use the XorShift algorithm
//!     - `pcg` to use the PCG algorithm

#![warn(missing_docs)]

mod bevy_framework;
pub use bevy_framework::*;

/// [`RandomNumberGenerator`] wraps the `rand` crate. The `rand` crate
/// is re-exported for your convenience
pub use rand;

#[cfg(not(feature = "locking"))]
mod random;
#[cfg(not(feature = "locking"))]
pub use random::*;

#[cfg(feature = "locking")]
mod random_locking;
#[cfg(feature = "locking")]
pub use random_locking::*;

mod bevy_assets;
pub use bevy_assets::*;

/// Wraps the `anyhow`-crate for error handling
pub mod anyhow {
    pub use anyhow::*;
}

/// Wraps the bevy_egui crate;
pub mod egui {
    pub use bevy_egui::*;
}

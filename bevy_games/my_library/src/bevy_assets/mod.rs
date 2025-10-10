//! The `bevy_assets` module implements an *Asset Manager* for Bevy.
//!
//! - lists game assets upfront
//! - tags assets for easy access
//! - provides a single resource to get asset handles
//!
//! This allows to declare all assets in one place to manage them more
//! easily.
mod asset_manager;
pub use asset_manager::AssetManager;

//! This module defines a mini scripting language for animations.

use bevy::platform::collections::HashMap;
use bevy::{log, prelude::*};

/// Actions that can uccor in any given frame.
pub enum AnimationOption {
    /// Do nothing. Freezes the animation.
    None,
    /// Move on with the next frame in sequence.
    NextFrame,
    /// Jump to a numbered frame. Allows for skipping or repeating.
    GoToFrame(usize),
    /// Switch to a different animation
    SwitchToAnimation(String),
    /// Play a sound. Synchronize animation with sound effects
    PlaySound(String),
}

pub struct AnimationFrame {
    /// The index of the SpriteSheet frame to display, from 0 to the number
    /// of images in the sheet.
    sprite_index: usize,
    /// The time in ms to display the frame before executing the options
    delay_ms: u128,
    /// The actions setup for this frame
    action: Vec<AnimationOption>,
}

impl AnimationFrame {
    /// Defines a new Frame for an animation sequence
    pub fn new(sprite_index: usize, delay_ms: u128, action: Vec<AnimationOption>) -> Self {
        Self {
            sprite_index,
            delay_ms,
            action,
        }
    }
}

/// A sequence of animated frames
pub struct PerFrameAnimation {
    /// Frames defining the animation
    pub frames: Vec<AnimationFrame>,
}

impl PerFrameAnimation {
    /// Define a new sequence of animated frames
    pub fn new(frames: Vec<AnimationFrame>) -> Self {
        Self { frames }
    }
}

/// Bevy resource to hold named animation sequences
#[derive(Resource)]
pub struct Animations(HashMap<String, PerFrameAnimation>);

impl Animations {
    /// Creates new animations resource
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Stores an animation sequence under a given tag
    pub fn with_animation<S: ToString>(mut self, tag: S, animation: PerFrameAnimation) -> Self {
        self.0.insert(tag.to_string(), animation);
        self
    }
}

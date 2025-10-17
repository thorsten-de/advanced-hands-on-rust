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

/// Defines a frame that is part of the animated sequence
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

/// A component to attach the animation state machine to the animated entity
#[derive(Component)]
pub struct AnimationCycle {
    /// the tag to refer to this animation sequence
    animation_tag: String,

    /// the frame currently executed in this animation
    current_frame: usize,

    /// The time elapsed since animation was rendered the last time. This keeps
    /// the timer state independently for each executing animation.
    timer: u128,
}

impl AnimationCycle {
    /// Creates a new state machine for a given tag starting at time/frame 0
    pub fn new<S: ToString>(tag: S) -> Self {
        Self {
            animation_tag: tag.to_string(),
            current_frame: 0,
            timer: 0,
        }
    }

    /// Switches an already running animation to a *different* animation sequence
    pub fn switch<S: ToString>(&mut self, new: S) {
        let new = new.to_string();
        if new != self.animation_tag {
            self.animation_tag = new;
            self.current_frame = 0;
            self.timer = 0;
        }
    }
}

/// System that animates frame sequences by using animation data
pub fn cycle_animations(
    animations: Res<Animations>,
    mut animated: Query<(&mut AnimationCycle, &mut Sprite)>, // mutable access to all entities with both AnimationCycle and Sprite components
    time: Res<Time>,
    assets: Res<crate::AssetStore>,
    mut commands: Commands,
    loaded_assets: Res<crate::LoadedAssets>,
) {
    let ms_since_last_call = time.delta().as_millis();

    animated.iter_mut().for_each(|(mut animation, mut sprite)| {
        animation.timer += ms_since_last_call;

        if let Some(cycle) = animations.0.get(&animation.animation_tag) {
            let current_frame = &cycle.frames[animation.current_frame];

            if animation.timer > current_frame.delay_ms {
                animation.timer = 0;
                for action in current_frame.action.iter() {
                    match action {
                        AnimationOption::None => {}
                        AnimationOption::NextFrame => {
                            animation.current_frame += 1;
                        }
                        AnimationOption::SwitchToAnimation(other) => {
                            animation.switch(other);
                        }
                        AnimationOption::GoToFrame(frame) => {
                            animation.current_frame = *frame;
                        }
                        AnimationOption::PlaySound(tag) => {
                            assets.play(tag, &mut commands, &loaded_assets);
                        }
                    }

                    if let Some(texture_atlas) = &mut sprite.texture_atlas {
                        texture_atlas.index = cycle.frames[animation.current_frame].sprite_index;
                    }
                }
            }
        } else {
            log::warn!("Animation Cycle [{}] not found!", animation.animation_tag);
        }
    });
}

/// Spawns an animated sprite
#[macro_export]
macro_rules! spawn_animated_sprite {
    ($assets:expr, $commands:expr, $index:expr, $x:expr, $y:expr, $z:expr, $animation_name:expr, $($component:expr), *) =>
    {
        let Some((img, atlas)) = $assets.get_atlas_handle($index) else {
            panic!()
        };
        $commands.spawn(
            (Sprite::from_atlas_image(
                img.clone(),
                TextureAtlas {
                    layout: atlas.clone(),
                    index: 0,
                }),
            Transform::from_xyz($x, $y, $z),
            AnimationCycle::new($animation_name),
            ))
            $(
                .insert($component)
            )*;
    }
}

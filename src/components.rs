use bevy::prelude::*;

use crate::SpriteImage;

/// The retro camera component
#[derive(Default, Debug, Clone)]
pub struct Camera {
    pub position: UVec2,
    pub size: u32,
}

/// A bundle containing all the components necessary to render a sprite
#[derive(Bundle, Default)]
pub struct SpriteBundle {
    /// The image data of the sprite
    pub image: Handle<SpriteImage>,
    /// The visibility of the sprite
    pub visible: Visible,
    /// The transform of the sprite. Note that for sprites only the translation are used.
    pub transform: Transform,
    /// The global transform of the sprite. Note that for sprites only the translation are used.
    pub global_transform: GlobalTransform,
}

/// Indicates whether or not an object should be rendered
#[derive(Debug, Clone, Copy)]
pub struct Visible(bool);

impl Default for Visible {
    fn default() -> Self {
        Visible(true)
    }
}

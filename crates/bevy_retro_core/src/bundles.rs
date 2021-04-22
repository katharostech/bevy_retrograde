//! Component bundles

use bevy::prelude::*;
use crate::prelude::*;

/// The components necessary to render a sprite
#[derive(Bundle, Default, Clone)]
pub struct SpriteBundle {
    /// Sprite settings
    pub sprite: Sprite,
    /// The image data of the sprite
    pub image: Handle<Image>,
    /// The visibility of the sprite
    pub visible: Visible,
    /// The position of the center of the sprite in world space
    pub position: Position,
    /// The global world position of the sprite
    pub world_position: WorldPosition,
}

/// The components necessary to render a spritesheet
#[derive(Bundle, Default, Clone)]
pub struct SpriteSheetBundle {
    #[bundle]
    /// The sprite bundle
    pub sprite_bundle: SpriteBundle,
    /// The sprite sheet handle
    pub sprite_sheet: Handle<SpriteSheet>,
}

/// The camera bundle
#[derive(Bundle, Default, Debug, Clone)]
pub struct CameraBundle {
    /// The camera config
    pub camera: Camera,

    /// The position of the camera
    ///
    /// The position will refer either to the center of the camera or the top-left corner depending
    /// on the value of the [`Camera.centered`][`Camera::centered`].
    pub position: Position,

    /// The global world position of the sprite
    pub world_position: WorldPosition,
}
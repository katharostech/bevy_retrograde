use crate::*;

/// A bundle containing all the components necessary to render a sprite
#[derive(Bundle, Default, Clone)]
pub struct SpriteBundle {
    // Sprite options
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

/// A bundle containing all the components necessary to render a spritesheet
#[derive(Bundle, Default, Clone)]
pub struct SpriteSheetBundle {
    #[bundle]
    /// The sprite bundle
    pub sprite_bundle: SpriteBundle,
    /// The sprite sheet handle
    pub sprite_sheet: Handle<SpriteSheet>,
}

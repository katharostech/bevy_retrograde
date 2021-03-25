use bevy::asset::AssetPath;

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

impl BundleFromAsset for SpriteBundle {
    fn bundle_from_asset<'a>(
        asset_server: &AssetServer,
        asset_path: bevy::asset::AssetPath<'a>,
    ) -> Self {
        Self {
            image: asset_server.load(asset_path),
            ..Default::default()
        }
    }
}

/// A bundle containing all the components necessary to render a spritesheet
#[derive(Bundle, Default, Clone)]
pub struct SpriteSheetBundle {
    #[bundle]
    /// The sprite bundle
    sprite_bundle: SpriteBundle,
    /// The sprite sheet handle
    sprite_sheet: Handle<SpriteSheet>,
}

impl BundleFromAsset for SpriteSheetBundle {
    fn bundle_from_asset(asset_server: &AssetServer, asset_path: bevy::asset::AssetPath) -> Self {
        let image_asset_path = AssetPath::new_ref(asset_path.path(), Some("image"));

        Self {
            sprite_bundle: SpriteBundle {
                image: asset_server.load(image_asset_path),
                ..Default::default()
            },
            sprite_sheet: asset_server.load(asset_path),
        }
    }
}

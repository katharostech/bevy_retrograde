use bevy::{
    asset::{AssetLoader, LoadContext},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::Deserialize;

/// Settings for a sprite sheet
#[derive(Debug, Clone, TypeUuid)]
#[uuid = "64746631-1afe-4ca6-8398-7c0df62f7813"]
pub struct SpriteSheet {
    pub grid_size: UVec2,
    pub tile_index: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct SpriteSheetMeta {
    pub sprite_sheet: String,
    pub grid_size: [u32; 2],
    #[serde(default)]
    pub tile_index: u32,
}

impl Default for SpriteSheet {
    fn default() -> Self {
        Self {
            grid_size: UVec2::splat(16),
            tile_index: 0,
        }
    }
}

/// An error that occurs when loading a spritesheet file
#[derive(thiserror::Error, Debug)]
pub enum SpriteSheetLoaderError {
    #[error("Error parsing sprite sheet file: {0}")]
    SpriteSheetError(#[from] serde_yaml::Error),
}

/// An spritesheet asset loader
#[derive(Default)]
pub(crate) struct SpriteSheetLoader;

impl AssetLoader for SpriteSheetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        // Create a future for the load function
        Box::pin(async move { Ok(load_spritesheet(bytes, load_context).await?) })
    }

    fn extensions(&self) -> &[&str] {
        &["spritesheet.yml"]
    }
}

async fn load_spritesheet<'a, 'b>(
    _bytes: &'a [u8],
    _load_context: &'a mut LoadContext<'b>,
) -> Result<(), SpriteSheetLoaderError> {
    unimplemented!();
    // let metadata: SpriteSheetMeta = serde_yaml::from_slice(bytes)?;

    // // Get path to the sprite image asset
    // let sprite_sheet = SpriteSheet {
    //     grid_size: metadata.grid_size.into(),
    //     tile_index: metadata.tile_index,
    // };
    // let image_file_path = load_context
    //     .path()
    //     .parent()
    //     .unwrap()
    //     .join(&metadata.sprite_sheet);
    // let image_asset_path = AssetPath::new(image_file_path.clone(), None);
    // let image_handle: Handle<Image> = load_context.get_handle(image_asset_path.clone());

    // load_context
    //     .set_default_asset(LoadedAsset::new(sprite_sheet).with_dependency(image_asset_path));

    // load_context.set_labeled_asset("image", );

    // Ok(())
}

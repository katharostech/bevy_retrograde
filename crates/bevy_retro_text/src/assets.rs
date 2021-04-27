use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};

/// A font asset
#[derive(TypeUuid, Clone, Debug)]
#[uuid = "8dd853b0-f6b7-406a-b1c0-d81abd4137fc"]
pub struct Font(bdf::Font);
bevy_retro_macros::impl_deref!(Font, bdf::Font);

/// An error that occurs when loading an image file
#[derive(thiserror::Error, Debug)]
pub enum FontLoaderError {
    #[error("Error parsing font: {0}")]
    FontError(#[from] bdf::Error),
}

/// An image asset loader
#[derive(Default)]
pub(crate) struct FontLoader;

impl AssetLoader for FontLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        // Create a future for the load function
        Box::pin(async move { Ok(load_image(bytes, load_context).await?) })
    }

    fn extensions(&self) -> &[&str] {
        &["bdf"]
    }
}

async fn load_image<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), FontLoaderError> {
    // Load the font
    let font = bdf::read(bytes)?;

    load_context.set_default_asset(LoadedAsset::new(Font(font)));

    Ok(())
}

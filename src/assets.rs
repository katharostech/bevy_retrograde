use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use image::{io::Reader as ImageReader, RgbaImage};

/// An LDtk map asset
#[derive(TypeUuid)]
#[uuid = "48d2e3c8-2f48-4330-b7fe-fac3e81c60f3"]
pub struct SpriteImage {
    image: RgbaImage,
    size: Size<u32>,
}

/// Add asset types and asset loader to the app builder
pub(crate) fn add_assets(app: &mut AppBuilder) {
    app.add_asset::<SpriteImage>()
        .init_asset_loader::<SpriteLoader>();
}

/// An error that occurs when loading a GLTF file
#[derive(thiserror::Error, Debug)]
pub enum SpriteLoaderError {
    #[error("Error parsing image: {0}")]
    ImageError(#[from] image::ImageError),
}

/// An LDTK map asset loader
#[derive(Default)]
struct SpriteLoader;

impl AssetLoader for SpriteLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        // Create a future for the load function
        Box::pin(async move { Ok(load_sprite(bytes, load_context).await?) })
    }

    fn extensions(&self) -> &[&str] {
        // Register this loader for .ldtk files and for .ldtk.json files. ( .ldtk.json will only
        // work after Bevy 0.5 is released )
        &[
            #[cfg(feature = "gif")]
            "gif",
            #[cfg(feature = "jpeg")]
            "jpeg",
            #[cfg(feature = "jpeg")]
            "jpg",
            #[cfg(feature = "png")]
            "png",
            #[cfg(feature = "tga")]
            "tga",
            #[cfg(feature = "tiff")]
            "tiff",
            #[cfg(feature = "webp")]
            "webp",
            #[cfg(feature = "bmp")]
            "bmp",
        ]
    }
}

async fn load_sprite<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), SpriteLoaderError> {
    // Create a cursor over our bytes to let the image reader `Seek` insdie of them
    let reader = std::io::Cursor::new(bytes);

    // Load the image
    let image = ImageReader::new(reader)
        .with_guessed_format()
        .unwrap() // Unwrap because we know the `&[u8]` will return no IO Error
        .decode()?
        .to_rgba8();
    let (width, height) = image.dimensions();

    load_context.set_default_asset(LoadedAsset::new(SpriteImage {
        size: Size::new(width, height),
        image,
    }));

    Ok(())
}

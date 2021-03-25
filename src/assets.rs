use std::path::Path;

use bevy::{asset::AssetPath, prelude::*};

mod image;
pub use self::image::*;
mod spritesheet;
pub use self::spritesheet::*;

pub trait AssetServerLoadBundle {
    fn load_bundle<B: BundleFromAsset>(&self, path: &str) -> B;
}

pub trait BundleFromAsset {
    fn bundle_from_asset<'a>(asset_server: &AssetServer, asset_path: AssetPath<'a>) -> Self;
}

impl AssetServerLoadBundle for AssetServer {
    fn load_bundle<B: BundleFromAsset>(&self, path: &str) -> B {
        B::bundle_from_asset(self, AssetPath::new_ref(Path::new(path), None))
    }
}

/// Add asset types and asset loader to the app builder
pub(crate) fn add_assets(app: &mut AppBuilder) {
    app.add_asset::<Image>()
        .init_asset_loader::<ImageLoader>()
        .add_asset::<SpriteSheet>()
        .init_asset_loader::<SpriteSheetLoader>();
}

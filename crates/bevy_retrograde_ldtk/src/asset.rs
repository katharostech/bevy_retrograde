use bevy::{
    asset::{AssetLoader, AssetPath, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    render2::texture::Image,
    sprite2::TextureAtlas,
    utils::{BoxedFuture, HashMap},
};

/// An LDtk map asset
#[derive(TypeUuid)]
#[uuid = "abd7b6d9-633f-4322-a8f4-e5f011cae9c6"]
pub struct LdtkMap {
    /// The full project structure for the LDtk map
    pub project: ldtk::Project,
    // /// A mapping of Tileset uids to their texture handles
    pub texture_atlases: HashMap<i32, Handle<TextureAtlas>>,
}

/// Add asset types and asset loader to the app builder
pub(crate) fn add_assets(app: &mut App) {
    app.add_asset::<LdtkMap>()
        .init_asset_loader::<LdtkMapLoader>();
}

/// An error that occurs when loading a GLTF file
#[derive(thiserror::Error, Debug)]
pub enum LdtkMapLoaderError {
    #[error("Could not parese LDtk map file: {0}")]
    ParsingError(#[from] serde_json::Error),
}

/// An LDTK map asset loader
#[derive(Default)]
struct LdtkMapLoader;

impl AssetLoader for LdtkMapLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        // Create a future for the load function
        Box::pin(async move { Ok(load_ldtk(bytes, load_context).await?) })
    }

    fn extensions(&self) -> &[&str] {
        // Register this loader for .ldtk files and for .ldtk.json files. ( .ldtk.json will only
        // work after Bevy 0.5 is released )
        &["ldtk", "ldtk.json"]
    }
}

async fn load_ldtk<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), LdtkMapLoaderError> {
    // Deserialize the LDTK project file
    let project: ldtk::Project = serde_json::from_slice(bytes)?;

    // Create a map asset
    let mut map = LdtkMap {
        project,
        texture_atlases: Default::default(),
    };

    // Create our dependency list
    let mut dependencies = Vec::new();

    // Loop through the tilesets
    for tileset in &map.project.defs.tilesets {
        // Get the path to the tileset image asset
        let file_path = load_context
            .path()
            .parent()
            .unwrap()
            .join(&tileset.rel_path);
        let tileset_image_asset_path = AssetPath::new(file_path.clone(), None);

        // Add asset to our dependencies list to make sure it is loaded by the asset
        // server when our map is.
        dependencies.push(tileset_image_asset_path.clone());

        // Obtain a handle to the tileset image asset
        let handle: Handle<Image> = load_context.get_handle(tileset_image_asset_path.clone());

        // Create a new texture atlas
        let texture_atlas = TextureAtlas::from_grid(
            handle,
            IVec2::new(
                tileset.px_wid / tileset.__c_wid,
                tileset.px_hei / tileset.__c_hei,
            )
            .as_f32(),
            tileset.__c_wid as usize,
            tileset.__c_hei as usize,
        );

        // Add the tileset as a labled asset
        let atlas_uid = tileset.uid;
        let atlas_handle = load_context.set_labeled_asset(
            &format!("atlas_{}", atlas_uid),
            LoadedAsset::new(texture_atlas).with_dependency(tileset_image_asset_path),
        );

        map.texture_atlases.insert(atlas_uid, atlas_handle);
    }

    // Set the loaded map as the default asset for this file
    load_context.set_default_asset(LoadedAsset::new(map).with_dependencies(dependencies));

    Ok(())
}

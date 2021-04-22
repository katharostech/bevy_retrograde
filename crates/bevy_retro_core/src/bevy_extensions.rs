//! Extension traits for Bevy types

use bevy::{asset::*, prelude::*};
use dashmap::DashMap;

use crate::graphics::*;

lazy_static::lazy_static! {
    /// An asset handle cache used by [`AssetServerExt`]
    static ref ASSET_CACHE: DashMap<AssetPathId, HandleUntyped> = DashMap::new();
}

/// Extension functions for the Bevy [`AssetServer`]
pub trait AssetServerExt {
    /// Load an asset and add it to an internal cache, or if it has already been loaded, get the
    /// cached asset handle.
    ///
    /// **This is provided by an extension trait to the Bevy asset server.**
    ///
    /// # Note
    ///
    /// If the asset that has previously been cached is being loaded and it has been manually
    /// removed from the asset store, the handle returned by this function will point to an
    /// un-loaded asset and the asset must be re-loaded with the normal `load` function.
    fn load_cached<'a, T, P>(&self, path: P) -> Handle<T>
    where
        P: Into<AssetPath<'a>>,
        T: Asset;

    /// Remove a handle from the asset cache. It is recommended to do this for any cached assets
    /// that are manually removed to prevent the cached handle being returned for a non-existent
    /// asset.
    ///
    /// **This is provided by an extension trait to the Bevy asset server.**
    fn remove_from_cache<T: Asset>(handle: Handle<T>);
}

impl AssetServerExt for AssetServer {
    fn load_cached<'a, T, P>(&self, path: P) -> Handle<T>
    where
        P: Into<AssetPath<'a>>,
        T: Asset,
    {
        // Get the path and ID of the asset we are to load
        let path = path.into();
        let id = path.get_id();

        // If the asset cache has the asset in it
        if let Some(handle) = ASSET_CACHE.get(&id) {
            // Return the cached asset
            handle.clone().typed()

        // If the asset cache doesn't have the asset
        } else {
            // Load the asset
            let handle = self.load(path);

            // Cache its handle
            ASSET_CACHE.insert(id, handle.clone_untyped());

            // And return the handle
            handle
        }
    }

    fn remove_from_cache<T: Asset>(handle: Handle<T>) {
        ASSET_CACHE.retain(|_, v| v != &handle.clone_untyped());
    }
}

/// Render hook extension to the Bevy [`AppBuilder`]
pub trait AppBuilderRenderHookExt {
    /// Add a new [`RenderHook`] to the Bevy Retro renderer
    fn add_render_hook<T: RenderHook + 'static>(self) -> Self;
}
impl AppBuilderRenderHookExt for &mut AppBuilder {
    fn add_render_hook<T: RenderHook + 'static>(self) -> Self {
        let world = self.world_mut();
        world.resource_scope(|_, mut render_hooks: Mut<RenderHooks>| {
            render_hooks.add_render_hook::<T>();
        });

        self
    }
}

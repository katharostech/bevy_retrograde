use bevy::prelude::*;

/// Re-export of the [`image`] crate
pub use image;

mod renderer;
use renderer::*;
pub use renderer::{RenderHook, RenderHooks};

mod assets;
pub use assets::*;

mod components;
pub use components::*;

mod bundles;
pub use bundles::*;

mod collisions;
pub use collisions::*;

mod shaders;
pub use shaders::*;

mod bevy_extensions;
pub use bevy_extensions::*;

/// The ECS schedule stages that the Bevy retro code is run in
#[derive(Debug, Clone, Copy, StageLabel, Hash, PartialEq, Eq)]
enum RetroStage {
    WorldPositionPropagation,
    Rendering,
}

/// The Core Bevy Retro plugin
#[derive(Default)]
pub struct RetroCorePlugin;

impl Plugin for RetroCorePlugin {
    fn build(&self, app: &mut AppBuilder) {
        add_components(app);
        add_assets(app);

        app.init_resource::<SceneGraph>()
            .init_resource::<RenderHooks>()
            .add_render_hook::<renderer::backend::sprite_hook::SpriteHook>()
            .add_stage_after(
                CoreStage::Last,
                RetroStage::WorldPositionPropagation,
                SystemStage::single_threaded()
                    .with_system(propagate_world_positions_system.system()),
            )
            .add_stage_after(
                RetroStage::WorldPositionPropagation,
                RetroStage::Rendering,
                SystemStage::single_threaded().with_system(get_render_system().exclusive_system()),
            );
    }
}

/// Utility to implement deref for single-element tuple structs
///
/// # Example
///
/// ```rust
/// struct Score(usize);
///
/// impl_deref!(Score, usize);
/// ```
#[macro_export(crate)]
macro_rules! impl_deref {
    ($struct:ident, $target:path) => {
        impl std::ops::Deref for $struct {
            type Target = $target;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $struct {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

/// Utility macro for adding a `#[cfg]` attribute to a batch of items
///
/// # Example
///
/// ```
/// // Only import these libraries for wasm targets
/// cfg_items!(wasm, {
///     use web_sys;
///     use js_sys;
/// });
/// ```
#[macro_export(crate)]
macro_rules! cfg_items {
    ($cfg:meta, {
        $(
            $item:item
        )*
    }) => {
        $(
            #[cfg($cfg)]
            $item
        )*
    };
}

//! The Bevy Retro text rendering plugin

use bevy::{ecs::component::ComponentDescriptor, prelude::*};
use bevy_retro_core::RetroCoreStage;

#[doc(hidden)]
pub mod prelude {
    pub use crate::assets::*;
    pub use crate::components::*;
    pub use crate::RetroTextPlugin;
}

mod assets;

mod components;

mod systems;
pub use systems::rasterize_text_block;
use systems::*;

use prelude::*;

/// The bevy stage the [`RetroTextPlugin`] runs its systems in
#[derive(StageLabel, Debug, Clone, Hash, PartialEq, Eq)]
pub struct RetroTextStage;

/// Text rendering plugin for Bevy Retro
pub struct RetroTextPlugin;

impl Plugin for RetroTextPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            // use sparce storage for marker components
            .register_component(ComponentDescriptor::new::<TextNeedsUpdate>(
                bevy::ecs::component::StorageType::SparseSet,
            ))
            // Add our font asset
            .add_asset::<Font>()
            // Add our font asset loader
            .add_asset_loader(FontLoader)
            // Add our font rendering system
            .add_stage_before(
                RetroCoreStage::Rendering,
                RetroTextStage,
                SystemStage::single(font_rendering.system()),
            );
    }
}

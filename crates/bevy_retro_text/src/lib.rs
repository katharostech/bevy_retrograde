//! The Bevy Retro text rendering plugin

use bevy::{ecs::component::ComponentDescriptor, prelude::*};

mod assets;
pub use assets::*;

mod components;
pub use components::*;

mod systems;
use systems::*;

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
            // FIXME: Add to proper stage to sync with render system properly
            .add_system_to_stage(CoreStage::PreUpdate, font_rendering.system());
    }
}

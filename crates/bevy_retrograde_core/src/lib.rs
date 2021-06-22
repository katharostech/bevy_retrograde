//! Bevy Retrograde core

use bevy::prelude::*;

/// The prelude
#[doc(hidden)]
pub mod prelude {
    pub use crate::assets::*;
    pub use crate::bevy_extensions::*;
    pub use crate::bundles::*;
    pub use crate::components::*;
    pub use crate::shaders::*;
}

/// Re-export of the [`image`] crate
pub use image;

/// Luminance rendering types
pub use luminance;

pub mod assets;
pub mod bevy_extensions;
pub mod bundles;
pub mod components;
pub mod graphics;
pub mod shaders;

mod renderer;

/// The ECS schedule stages that the Bevy Retrograde code is run in
#[derive(Debug, Clone, Copy, StageLabel, Hash, PartialEq, Eq)]
pub enum RetroCoreStage {
    Rendering,
}

use crate::{graphics::*, prelude::*, renderer::*};

/// The Bevy Retrograde Core plugin
#[derive(Default)]
pub struct RetroCorePlugin;

impl Plugin for RetroCorePlugin {
    fn build(&self, app: &mut AppBuilder) {
        add_components(app);
        add_assets(app);

        app.init_resource::<RenderHooks>()
            .add_render_hook::<graphics::hooks::SpriteHook>()
            .add_stage_after(
                CoreStage::Last,
                RetroCoreStage::Rendering,
                SystemStage::single_threaded().with_system(get_render_system().exclusive_system()),
            );
    }
}

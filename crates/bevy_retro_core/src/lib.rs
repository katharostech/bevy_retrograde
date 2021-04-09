use bevy::prelude::*;

/// Re-export of the [`image`] crate
pub use image;

mod renderer;
use renderer::*;

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

/// The ECS schedule stages that the Bevy retro code is run in
#[derive(Debug, Clone, Copy, StageLabel, Hash, PartialEq, Eq)]
struct RetroStage;
#[derive(Debug, Clone, Copy, StageLabel, Hash, PartialEq, Eq, SystemLabel)]
struct PropagateSystem;

/// The Core Bevy Retro plugin
#[derive(Default)]
pub struct RetroCorePlugin;

impl Plugin for RetroCorePlugin {
    fn build(&self, app: &mut AppBuilder) {
        add_components(app);
        add_assets(app);

        app.init_resource::<SceneGraph>()
            .add_stage_after(CoreStage::Last, RetroStage, SystemStage::parallel())
            .add_system_set_to_stage(
                RetroStage,
                SystemSet::new()
                    .with_system(
                        propagate_world_positions_system
                            .system()
                            .label(PropagateSystem),
                    )
                    .with_system(
                        get_render_system()
                            .exclusive_system()
                            .after(PropagateSystem),
                    ),
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

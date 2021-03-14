//! Bevy Retro is an experimental 2D, pixel-perfect renderer for Bevy that can target both web and
//! desktop using OpenGL.

use bevy::{asset::AssetStage, prelude::*};

mod renderer;
pub use renderer::*;

mod assets;
pub use assets::*;

mod components;
pub use components::*;

mod position_propagation;
use position_propagation::*;

#[derive(Debug, Clone, Copy, StageLabel, Hash, PartialEq, Eq)]
pub enum RetroStage {
    WorldPositionPropagation,
    PreRender,
    Render,
}

#[derive(Default)]
pub struct RetroPlugin;

impl Plugin for RetroPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let render_system = renderer::get_render_system();

        add_assets(app);

        app.init_resource::<RetroRenderOptions>()
            .init_resource::<RenderFrame>()
            .init_resource::<SceneGraph>()
            .add_stage_after(
                AssetStage::AssetEvents,
                RetroStage::WorldPositionPropagation,
                SystemStage::parallel(),
            )
            .add_stage_after(
                AssetStage::AssetEvents,
                RetroStage::PreRender,
                SystemStage::parallel(),
            )
            .add_stage_after(
                RetroStage::PreRender,
                RetroStage::Render,
                SystemStage::parallel(),
            )
            .add_system_to_stage(RetroStage::PreRender, propagate_world_positions_system.system())
            .add_system_to_stage(RetroStage::PreRender, pre_render_system.system())
            .add_system_to_stage(RetroStage::Render, render_system.exclusive_system());
    }
}

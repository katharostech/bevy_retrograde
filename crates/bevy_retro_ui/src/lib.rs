use bevy::prelude::*;

use bevy_retro_core::{prelude::AppBuilderRenderHookExt};

mod resources;
use bevy_retro_text::RetroTextStage;
pub use resources::*;

mod render_hook;
use render_hook::UiRenderHook;

pub use raui;

#[derive(StageLabel, Debug, Clone, Hash, PartialEq, Eq)]
struct RetroUiStage;

/// Text rendering plugin for Bevy Retro
pub struct RetroUiPlugin;

impl Plugin for RetroUiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            // Add the UI tree resource
            .init_resource::<UiTree>()
            .init_resource::<UiApplication>()
            .add_render_hook::<UiRenderHook>()
            .add_stage_before(
                RetroTextStage,
                RetroUiStage,
                SystemStage::single(apply_ui_changes.system()),
            );
    }
}

/// Apply changes to the UI tree to the UI application
fn apply_ui_changes(ui_tree: Res<UiTree>, mut app: ResMut<UiApplication>) {
    if ui_tree.is_changed() {
        app.apply(ui_tree.0.clone());
    }
}

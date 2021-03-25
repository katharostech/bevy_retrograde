//! Bevy Retro is a 2D, pixel-perfect renderer for [Bevy] that can target both web and desktop using
//! OpenGL/WebGL.
//!
//! [Bevy]: https://bevyengine.org
//! 
//! # Example
//! 
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_retro::*;
//! 
//! fn main() {
//!     App::build()
//!         .add_plugins(RetroPlugins)
//!         .add_startup_system(setup.system())
//!         .run();
//! }
//! 
//! struct Player;
//! 
//! fn setup(
//!     mut commands: Commands,
//!     asset_server: Res<AssetServer>,
//!     mut scene_graph: ResMut<SceneGraph>,
//! ) {
//!     // Load our sprites
//!     let red_radish_image = asset_server.load("redRadish.png");
//!     let yellow_radish_image = asset_server.load("yellowRadish.png");
//!     let blue_radish_image = asset_server.load("blueRadish.png");
//! 
//!     // Spawn the camera
//!     commands.spawn().insert_bundle(CameraBundle {
//!         camera: Camera {
//!             // Set our camera to have a fixed height and an auto-resized width
//!             size: CameraSize::FixedHeight(100),
//!             background_color: Color::new(0.2, 0.2, 0.2, 1.0),
//!             ..Default::default()
//!         },
//!         position: Position::new(0, 0, 0),
//!         ..Default::default()
//!     });
//! 
//!     // Spawn a red radish
//!     let red_radish = commands
//!         .spawn()
//!         .insert_bundle(SpriteBundle {
//!             image: red_radish_image,
//!             position: Position::new(0, 0, 0),
//!             sprite: Sprite {
//!                 flip_x: true,
//!                 flip_y: false,
//!                 ..Default::default()
//!             },
//!             ..Default::default()
//!         })
//!         // Add our player marker component so we can move it
//!         .insert(Player)
//!         .id();
//! 
//!     // Spawn a yellow radish
//!     let yellow_radish = commands
//!         .spawn()
//!         .insert_bundle(SpriteBundle {
//!             image: yellow_radish_image,
//!             position: Position::new(-20, 0, 0),
//!             sprite: Sprite {
//!                 flip_x: true,
//!                 flip_y: false,
//!                 ..Default::default()
//!             },
//!             ..Default::default()
//!         })
//!         .id();
//! 
//!     // Make the yello radish a child of the red radish
//!     scene_graph
//!         .add_child(red_radish, yellow_radish)
//!         // This could fail if the child is an ancestor of the parent
//!         .unwrap();
//! 
//!     // Spawn a blue radish
//!     commands.spawn().insert_bundle(SpriteBundle {
//!         image: blue_radish_image,
//!         // Set the blue radish back a layer so that he shows up under the other two
//!         position: Position::new(-20, -20, -1),
//!         sprite: Sprite {
//!             flip_x: true,
//!             flip_y: false,
//!             ..Default::default()
//!         },
//!         ..Default::default()
//!     });
//! }
//! ```


use bevy::{asset::AssetStage, prelude::*};

pub use image;

mod renderer;
use renderer::*;

mod assets;
pub use assets::*;

mod components;
pub use components::*;

mod bundles;
pub use bundles::*;

#[derive(Debug, Clone, Copy, StageLabel, Hash, PartialEq, Eq)]
pub enum RetroStage {
    WorldPositionPropagation,
    Render,
}

pub struct RetroPlugins;

impl PluginGroup for RetroPlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group.add(bevy::log::LogPlugin::default());
        group.add(bevy::core::CorePlugin::default());
        group.add(bevy::diagnostic::DiagnosticsPlugin::default());
        group.add(bevy::input::InputPlugin::default());
        group.add(bevy::window::WindowPlugin::default());
        group.add(bevy::asset::AssetPlugin::default());
        group.add(bevy::winit::WinitPlugin::default());
        group.add(RetroPlugin);
    }
}

#[derive(Default)]
pub struct RetroPlugin;

impl Plugin for RetroPlugin {
    fn build(&self, app: &mut AppBuilder) {
        add_assets(app);

        app.init_resource::<SceneGraph>()
            .add_stage_after(
                AssetStage::AssetEvents,
                RetroStage::WorldPositionPropagation,
                SystemStage::parallel(),
            )
            .add_stage_after(
                RetroStage::WorldPositionPropagation,
                RetroStage::Render,
                SystemStage::parallel(),
            )
            .add_system_to_stage(
                RetroStage::WorldPositionPropagation,
                propagate_world_positions_system.system(),
            )
            .add_system_to_stage(RetroStage::Render, get_render_system().exclusive_system());
    }
}

/// Utility to implement deref for single-element tuple structs
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

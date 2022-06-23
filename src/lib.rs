//! Bevy Retrograde is an opinionated plugin pack for the [Bevy] game engine with tools to help you
//! make 2D games!
//!
//! Bevy Retrograde is not specific to pixel-art games, but it does include some features that would
//! be particularly useful for pixel games. The ultimate goal is to act as an extension to Bevy that
//! gives you common tools necessary to make a 2D game such as map loading, physics, UI, save-data,
//! etc. Not all of the features we want to add are implemented yet, but we will be expanding the
//! feature set as we developer our own game with it.
//!
//! [Bevy]: https://bevyengine.org
//!
//! # License
//!
//! Bevy Retrograde LDtk is licensed under the [Katharos License][k_license] which places certain
//! restrictions on what you are allowed to use it for. Please read and understand the terms before
//! using Bevy Retrograde for your project.
//!
//! [k_license]: https://github.com/katharostech/katharos-license
//!
//! # Development Status
//!
//! Bevy Retrograde is in early stages of development. The API is not stable and may change
//! dramatically at any time. We have just made a major update, migrating from Bevy 0.5 and a custom
//!
//! See also [Supported Bevy Version](#supported-bevy-version) below.
//!
//! # Features & Examples
//!
//! Check out our [examples] list to see how to use each Bevy Retrograde feature:
//!
//! - Supports web and desktop out-of-the-box
//! - [LDtk](https://ldtk.io) map loading and rendering using [`bevy_ecs_ldtk`]
//! - An integration with the [RAUI] UI library for building in-game user interfaces and HUD
//! - Physics and collision detection powered by [Rapier] with automatic generation of convex
//!   collision shapes from sprite images
//! - Text rendering of bitmap fonts in the BDF format
//! - A simple but effective sound playing API
//!
//! [examples]:
//! https://github.com/katharostech/bevy_retrograde/tree/master/examples#bevy-retro-examples
//!
//! [Rapier]: https://rapier.rs/
//!
//! # Supported Bevy Version
//!
//! 
//! | bevy | bevy_retrograde |
//! |------|-----------------|
//! | 0.7  | 0.3             |
//! | 0.6  |                 |
//! | 0.5  | 0.1, 0.2        |
//!
//! **`Cargo.toml`:**
//!
//! ```toml
//! bevy = { version = "0.7", default-features = false }
//! bevy_retrograde = "0.3.0"
//! ```

/// Bevy Retrograde default plugins
pub struct RetroPlugins {
    /// Used to calculate the physics scale, if the physics feature is enabled.
    pub pixels_per_meter: f32,
}

impl Default for RetroPlugins {
    fn default() -> Self {
        Self {
            pixels_per_meter: 8.0,
        }
    }
}

impl bevy::app::PluginGroup for RetroPlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        // Add the plugins we need from Bevy
        bevy::DefaultPlugins.build(group);

        #[cfg(feature = "audio")]
        group.add(audio::AudioPlugin);

        #[cfg(feature = "ldtk")]
        group.add(ldtk::LdtkPlugin);

        #[cfg(feature = "text")]
        group.add(text::RetroTextPlugin);

        #[cfg(feature = "physics")]
        group.add(physics::RetroPhysicsPlugin {
            pixels_per_meter: self.pixels_per_meter,
        });

        #[cfg(feature = "ui")]
        group.add(ui::RetroUiPlugin);
    }
}

/// Bevy Retrograde prelude
#[doc(hidden)]
pub mod prelude {
    pub use crate::*;
    pub use bevy_retrograde_macros::impl_deref;

    #[cfg(feature = "audio")]
    pub use bevy_kira_audio::*;

    #[cfg(feature = "ldtk")]
    pub use bevy_ecs_ldtk::*;

    #[cfg(feature = "ui")]
    pub use bevy_retrograde_ui::*;

    #[cfg(feature = "physics")]
    pub use bevy_retrograde_physics::*;
}

#[cfg(feature = "re-export-bevy")]
pub use bevy;

pub use bevy_retrograde_macros::impl_deref;

#[cfg(feature = "audio")]
#[doc(inline)]
pub use bevy_kira_audio as audio;

#[cfg(feature = "physics")]
#[doc(inline)]
pub use bevy_retrograde_physics as physics;

#[cfg(feature = "ldtk")]
pub use bevy_ecs_ldtk as ldtk;

#[cfg(feature = "ui")]
#[doc(inline)]
pub use bevy_retrograde_ui as ui;

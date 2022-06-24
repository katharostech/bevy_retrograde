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
//! dramatically at any time.
//!
//! We have just made a major update. This update removed ~75% of Bevy Retro ( that's good! ) by
//! updating to Bevy 0.7, and:
//!
//!   - Replacing our custom renderer with Bevy's
//!   - Replacing our custom map laoder with [`bevy_ecs_ldtk`]
//!   - Replacing our custom [RAUI] UI renderer with [`bevy_egui`]
//!
//! Now Bevy Retrograde mostly includes some existing libraries and provides small utilities on top
//! such as the 9-patch style UI addtions for egui.
//!
//! Since it's been so long since our last we want to get another release out soon, just to get
//! everything working again on top of the latest crates. We are just wating on a [tilemap rendering
//! fix](https://github.com/StarArawn/bevy_ecs_tilemap/pull/197) to get merged before we publish an
//! `0.3.0` release.
//!
//! After that we plan to re-visit what extra features we might want, such as an easier way to setup
//! to 2D camera, and a save data system, and we will look at polishing our integrations and
//! utilities where appropriate.
//!
//! See also [Supported Bevy Version](#supported-bevy-version) below.
//!
//! [`bevy_ecs_ldtk`]: https://github.com/Trouv/bevy_ecs_ldtk
//! [`bevy_egui`]: https://github.com/mvlabat/bevy_egui
//! [RAUI]: https://raui-labs.github.io/raui/
//!
//! # Features & Examples
//!
//! Check out our [examples] list to see how to use each Bevy Retrograde feature:
//!
//! - Supports web and desktop out-of-the-box
//! - [LDtk](https://ldtk.io) map loading and rendering using [`bevy_ecs_ldtk`].
//! - An integration with the [`egui`] UI library with extra 9-patch style widgets.
//! - Text rendering of bitmap fonts in the BDF format
//! - Physics and collision detection powered by [Rapier] with automatic generation of convex
//!   collision shapes from sprite images.
//! - Sound playing with [`bevy_kira_audio`].
//!
//! [examples]:
//! https://github.com/katharostech/bevy_retrograde/tree/master/examples#bevy-retro-examples
//!
//! [`egui`]: https://github.com/emilk/egui
//!
//! [Rapier]: https://rapier.rs/
//!
//! [`bevy_kira_audio`]: https://github.com/NiklasEi/bevy_kira_audio
//!
//! # Supported Bevy Version
//!
//!
//! | bevy | bevy_retrograde |
//! |------|-----------------|
//! | 0.7  | master ( `0.3` release comming soon! ) |
//! | 0.6  |                 |
//! | 0.5  | 0.1, 0.2        |
//!
//! **`Cargo.toml`:**
//!
//! ```toml
//! [dependencies]
//! bevy = { version = "0.7", default-features = false }
//! bevy_retrograde = { git = "https://github.com/katharostech/bevy_retrograde.git" }
//! ```

mod core;

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

        group.add(core::RetroCorePlugin);
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
    pub use bevy_ecs_ldtk::prelude::*;

    #[cfg(feature = "ui")]
    pub use bevy_retrograde_ui::prelude::*;

    #[cfg(feature = "physics")]
    pub use bevy_retrograde_physics::prelude::*;
}

pub use bevy_retrograde_macros::impl_deref;

#[cfg(feature = "audio")]
pub use bevy_kira_audio as audio;

#[cfg(feature = "physics")]
#[doc(inline)]
pub use bevy_retrograde_physics as physics;

#[cfg(feature = "ldtk")]
pub use bevy_ecs_ldtk as ldtk;

#[cfg(feature = "ui")]
#[doc(inline)]
pub use bevy_retrograde_ui as ui;

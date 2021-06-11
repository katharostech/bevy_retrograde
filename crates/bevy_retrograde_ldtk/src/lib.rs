//! A [Bevy Retrograde][br] plugin for loading [LDtk] tile maps.
//!
//! [ldtk]: https://github.com/deepnight/ldtk
//!
//! [br]: https://github.com/katharostech/bevy_retrograde
//!
//! # Caveats
//!
//! The plugin is in relatively early stages, but it is still rather functional for many basic maps
//!
//! - Many features are not supported yet, including:
//!   - tilesets with spacing in them
//!   - levels in separate files
//!
//! [#1]: https://github.com/katharostech/bevy_ldtk/issues/1
//!
//! If you run into anything that isn't supported that you want to use in your game open an issue or
//! PR to help prioritize what gets implemented.
//!
//! # License
//!
//! Bevy Retrograde LDtk is licensed under the [Katharos License][k_license] which places certain
//! restrictions on what you are allowed to use it for. Please read and understand the terms before
//! using Bevy LDtk for your project.
//!
//! [k_license]: https://github.com/katharostech/katharos-license

use bevy::prelude::*;

mod asset;
mod components;
mod system;

pub use asset::*;
pub use components::*;

use system::add_systems;

/// Bevy plugin that adds support for loading LDtk tile maps
#[derive(Default)]
pub struct LdtkPlugin;

impl Plugin for LdtkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // Add asssets, systems, and graphics pipeline
        add_assets(app);
        add_systems(app);
    }
}

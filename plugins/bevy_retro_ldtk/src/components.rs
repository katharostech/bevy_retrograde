use bevy::prelude::*;
use bevy_retro::{Position, WorldPosition};
use ldtk::LayerInstance;

use crate::asset::LdtkMap;

/// A component bundle for spawning an LDtk map
#[derive(Default, Bundle)]
pub struct LdtkMapBundle {
    /// The handle to a map asset
    pub map: Handle<LdtkMap>,
    /// The transform of the map
    pub position: Position,
    /// The map configuration settings
    pub config: LdtkMapConfig,
    /// The world position
    pub world_position: WorldPosition,
}

/// Component added to spawned map layers
pub struct LdtkMapLayer {
    pub map: Handle<LdtkMap>,
    pub layer_instance: LayerInstance,
}

/// Configuration for how to display the Ldtk map
pub struct LdtkMapConfig {
    /// Whether or not to set the clear color of the screen to match the background color of the
    /// LDtk map.
    pub set_clear_color: bool,
    /// Which level from the LDtk project to display, if there are more than one level.
    pub level: usize,
    /// Whether or not to center the map about it's origin [`Transform`].
    ///
    /// This default value for this is false, in which case the top-left corner of the map will be
    /// the map origin.
    pub center_map: bool,
}

impl Default for LdtkMapConfig {
    fn default() -> Self {
        LdtkMapConfig {
            set_clear_color: false,
            level: 0,
            center_map: false,
        }
    }
}

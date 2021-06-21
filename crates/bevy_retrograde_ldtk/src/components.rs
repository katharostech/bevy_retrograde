use bevy::prelude::*;
use ldtk::LayerInstance;

use crate::asset::LdtkMap;

/// A component bundle for spawning an LDtk map
#[derive(Default, Bundle)]
pub struct LdtkMapBundle {
    /// The handle to a map asset
    pub map: Handle<LdtkMap>,
    /// The transform of the map
    pub transform: Transform,
    /// The world position
    pub global_transform: GlobalTransform,
}

/// Component added to spawned map layers
pub struct LdtkMapLayer {
    pub map: Handle<LdtkMap>,
    pub level_identifier: String,
    pub layer_instance: LayerInstance,
}

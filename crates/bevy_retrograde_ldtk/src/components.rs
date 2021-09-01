use bevy::prelude::*;

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

/// Component added to each tile sprite spawned when loading the map
pub struct LdtkMapTile {
    pub map: Handle<LdtkMap>,
    pub level_uid: i32,
    pub layer_instance_index: usize,
}

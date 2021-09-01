use asset::LdtkMap;
use bevy::sprite2::{PipelinedSpriteSheetBundle, TextureAtlasSprite};

use crate::*;

/// Add the Ldtk map systems to the app builder
pub(crate) fn add_systems(app: &mut App) {
    app.add_system(process_ldtk_maps)
        .add_system(hot_reload_maps);
}

struct LdtkMapHasLoaded;

/// This system spawns the map layers for every unloaded entity with an LDtk map
fn process_ldtk_maps(
    mut commands: Commands,
    mut new_maps: Query<(Entity, &Handle<LdtkMap>), Without<LdtkMapHasLoaded>>,
    map_assets: Res<Assets<LdtkMap>>,
) {
    // Loop through all of the maps
    for (map_ent, map_handle) in new_maps.iter_mut() {
        // Get the map asset, if available
        if let Some(map) = map_assets.get(map_handle) {
            let project = &map.project;

            // Loop through the levels in the map
            for level in &project.levels {
                let level_offset = Vec3::new(level.world_x as f32, -level.world_y as f32, 0.0);

                // Loop through the layers in the selected level
                for (z, layer) in level.layer_instances.as_ref().unwrap().iter().enumerate() {
                    let layer_offset = level_offset
                        + Vec3::new(
                            layer.__px_total_offset_x as f32,
                            -layer.__px_total_offset_y as f32,
                            0.0,
                        );
                    // Get the texture atlas for this layer
                    let texture_atlas = if let Some(tileset_uid) = layer.__tileset_def_uid {
                        map.texture_atlases.get(&tileset_uid).unwrap()

                    // Skip layers without a tileset
                    } else {
                        continue;
                    };

                    // Get the tiles for this layer, either from the auto-tiles or the grid tiles,
                    // based on which is present
                    let tiles = if !layer.auto_layer_tiles.is_empty() {
                        &layer.auto_layer_tiles
                    } else if !layer.grid_tiles.is_empty() {
                        &layer.grid_tiles
                    } else {
                        // Skip the layer if there are no tiles for it
                        continue;
                    };

                    // For every tile in the layer
                    for (i, tile) in tiles.iter().enumerate() {
                        let tile_position = layer_offset
                            + IVec2::new(tile.px[0], -tile.px[1])
                                .as_f32()
                                .extend(z as f32 + 0.001 * i as f32);

                        // Spawn the tile
                        let tile_ent = commands
                            .spawn_bundle(PipelinedSpriteSheetBundle {
                                texture_atlas: texture_atlas.clone(),
                                sprite: TextureAtlasSprite {
                                    flip_x: tile.f.x,
                                    flip_y: tile.f.y,
                                    index: tile.t as u32,
                                    visible: layer.visible,
                                    ..Default::default()
                                },
                                transform: Transform {
                                    translation: tile_position,
                                    // Grow the tile size very slightly in order to prevent
                                    // lines between the tiles when rendering
                                    scale: Vec2::splat(
                                        1.0 + 2.0 / layer.__grid_size as f32 * 0.002,
                                    )
                                    .extend(1.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(LdtkMapTile {
                                map: map_handle.clone(),
                                level_uid: level.uid,
                                layer_instance_index: z,
                            })
                            .id();

                        // Add the tile as a child of the map
                        commands.entity(map_ent).push_children(&[tile_ent]);
                    }
                }

                // Mark the map as having been loaded so that we don't process it again
                commands.entity(map_ent).insert(LdtkMapHasLoaded);
            }
        }
    }
}

type MapEvent = AssetEvent<LdtkMap>;

/// This system watches for changes to map assets and makes sure that the map is reloaded upon
/// changes.
fn hot_reload_maps(
    mut commands: Commands,
    mut event_reader: EventReader<MapEvent>,
    tiles: Query<(Entity, &LdtkMapTile)>,
    maps: Query<(Entity, &Handle<LdtkMap>)>,
) {
    for event in event_reader.iter() {
        match event {
            // When the map asset has been modified
            AssetEvent::Modified { handle } => {
                // Loop through all the layers in the world, find the ones that are for this map and remove them
                for (
                    layer_ent,
                    LdtkMapTile {
                        map: map_handle, ..
                    },
                ) in tiles.iter()
                {
                    if map_handle == handle {
                        commands.entity(layer_ent).despawn();
                    }
                }

                // Then remove the `LdtkMapHasLoaded` component from the map so that it will be
                // reloaded by the `process_ldtk_maps` system.
                for (map_ent, map_handle) in maps.iter() {
                    if map_handle == handle {
                        commands.entity(map_ent).remove::<LdtkMapHasLoaded>();
                    }
                }
            }
            _ => (),
        }
    }
}

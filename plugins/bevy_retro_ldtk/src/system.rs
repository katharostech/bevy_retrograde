use asset::LdtkMap;
use bevy::utils::HashMap;
use bevy_retro::{
    image::{
        self,
        imageops::{self, flip_horizontal_in_place, flip_vertical_in_place},
        GenericImage, GenericImageView,
    },
    Camera, Color, Image, Position, SceneGraph, Sprite, SpriteBundle, Visible,
};

use crate::*;

/// Add the Ldtk map systems to the app builder
pub(crate) fn add_systems(app: &mut AppBuilder) {
    app.add_system(process_ldtk_maps.system())
        .add_system(hot_reload_maps.system());
}

struct LdtkMapHasLoaded;

/// This system spawns the map layers for every unloaded entity with an LDtk map
fn process_ldtk_maps(
    mut commands: Commands,
    mut cameras: Query<&mut Camera>,
    mut new_maps: Query<(Entity, &Handle<LdtkMap>, &LdtkMapConfig), Without<LdtkMapHasLoaded>>,
    map_assets: Res<Assets<LdtkMap>>,
    mut image_assets: ResMut<Assets<Image>>,
    mut scene_graph: ResMut<SceneGraph>,
) {
    // Loop through all of the maps
    for (map_ent, map_handle, config) in new_maps.iter_mut() {
        // Get the map asset, if available
        if let Some(map) = map_assets.get(map_handle) {
            let project = &map.project;

            // Create a hasmap mapping tileset def uid's to the tileset definition and it's texture handle
            let mut tilesets = HashMap::default();

            // Load all the tilesets
            for (tileset_name, image_handle) in &map.tile_sets {
                // Get the tileset info
                let tileset_info = project
                    .defs
                    .tilesets
                    .iter()
                    .filter(|x| &x.identifier == tileset_name)
                    .next()
                    .expect("Could not find tilset inside of map data");

                if image_assets.get(image_handle).is_some() {
                    // Insert it into the tileset map
                    tilesets.insert(tileset_info.uid, image_handle);
                } else {
                    // Wait for tilemap to load
                    return;
                }
            }

            // Get the level that we are to display
            let level = map.project.levels.get(config.level as usize).unwrap();

            // If the camera background color isn't set, set it
            if config.set_clear_color {
                for mut camera in cameras.iter_mut() {
                    let decoded = hex::decode(
                        level
                            .bg_color
                            .as_ref()
                            .unwrap_or(&map.project.default_level_bg_color)
                            .strip_prefix("#")
                            .expect("Invalid background color"),
                    )
                    .expect("Invalid background color");

                    camera.background_color =
                        Color::from_rgba8(decoded[0], decoded[1], decoded[2], 1);
                }
            }

            // Loop through the layers in the selected level
            for (z, layer) in level
                .layer_instances
                .as_ref()
                .unwrap()
                .iter()
                .rev() // Reverse the layer order so that the bottom layer is first
                .enumerate()
            {
                // Get the information for the tileset associated to this layer
                let tileset_handle = if let Some(uid) = layer.__tileset_def_uid {
                    tilesets.get(&uid).expect("Missing tileset").clone()

                // Skip this layer if there is no tileset texture for it
                } else {
                    continue;
                };
                // This unwrap is OK because we checked above that the asset was loaded
                let tileset_image = image_assets.get(tileset_handle).unwrap();

                // Get a list of all the tiles in the layer
                let tiles = if !layer.auto_layer_tiles.is_empty() {
                    &layer.auto_layer_tiles
                } else if !layer.grid_tiles.is_empty() {
                    &layer.grid_tiles
                } else {
                    // Skip the layer if there are no tiles for it
                    continue;
                };

                // Create the layer image
                let width = (layer.__c_wid * layer.__grid_size) as u32;
                let height = (layer.__c_hei * layer.__grid_size) as u32;
                let mut layer_image = image::RgbaImage::new(width, height);

                // For every tile in the layer
                for tile in tiles {
                    // Get a view of the tilesheet image referenced by the tile

                    // TODO: [perf] we only technically need to copy this image if it is flipped,
                    // but right now we are doing it no matter what for ease
                    let mut tile_src = tileset_image
                        .view(
                            tile.src[0] as u32,
                            tile.src[1] as u32,
                            layer.__grid_size as u32,
                            layer.__grid_size as u32,
                        )
                        .to_image();

                    if tile.f.x {
                        flip_horizontal_in_place(&mut tile_src);
                    }
                    if tile.f.y {
                        flip_vertical_in_place(&mut tile_src);
                    }

                    // Get a sub-image for the spot that the tile is supposed to go
                    let mut tile_target = layer_image.sub_image(
                        tile.px[0] as u32,
                        tile.px[1] as u32,
                        layer.__grid_size as u32,
                        layer.__grid_size as u32,
                    );

                    // Overlay the tile on top of the layer
                    imageops::overlay(&mut tile_target, &tile_src, 0, 0);
                }

                // If the layer opacity is not 100%, adjust the transparency accordingly
                if layer.__opacity != 1.0 {
                    for pixel in layer_image.pixels_mut() {
                        pixel[3] = (layer.__opacity * 255.0 * (pixel[3] as f32 / 255.0)) as u8;
                    }
                }

                // Spawn the layer
                let layer_ent = commands
                    .spawn()
                    .insert_bundle(SpriteBundle {
                        image: image_assets.add(Image::from(layer_image)),
                        // Each layer is 2 units higher than the one before it
                        sprite: Sprite {
                            centered: config.center_map,
                            ..Default::default()
                        },
                        visible: Visible(layer.visible),
                        position: Position::new(0, 0, z as i32 * 2),
                        ..Default::default()
                    })
                    .insert(LdtkMapLayer {
                        map: map_handle.clone(),
                        layer_instance: layer.clone(),
                    })
                    .id();

                scene_graph.add_child(map_ent, layer_ent).unwrap();
            }

            // Mark the map as having been loaded so that we don't process it again
            commands.entity(map_ent).insert(LdtkMapHasLoaded);
        }
    }
}

type MapEvent = AssetEvent<LdtkMap>;

/// This system watches for changes to map assets and makes sure that the map is reloaded upon
/// changes.
fn hot_reload_maps(
    mut commands: Commands,
    mut events: EventReader<MapEvent>,
    layers: Query<(Entity, &LdtkMapLayer, &Handle<Image>)>,
    maps: Query<(Entity, &Handle<LdtkMap>), With<LdtkMapConfig>>,
    mut image_assets: ResMut<Assets<Image>>,
) {
    for event in events.iter() {
        match event {
            // When the map asset has been modified
            AssetEvent::Modified { handle } => {
                // Loop through all the layers in the world, find the ones that are for this map and remove them
                for (layer_ent, LdtkMapLayer { map, .. }, image_handle) in layers.iter() {
                    if map == handle {
                        // Despawn the layer
                        commands.entity(layer_ent).despawn();
                        // Remove the layer image
                        image_assets.remove(image_handle);
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

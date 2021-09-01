//! Physics in Bevy Retrograde currently just leverages the heron crate for almost
//! everything. The only difference is the use of the `TesselatedCollider` component that can be
//! used to create a convex hull collision shape from a sprite image.

use bevy::{
    prelude::*,
    render2::{
        camera::{DepthCalculation, OrthographicCameraBundle, OrthographicProjection, ScalingMode},
        texture::Image,
    },
    sprite2::{PipelinedSpriteBundle, TextureAtlas, TextureAtlasSprite},
};
use bevy_retrograde::prelude::*;

use image::GenericImageView;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde Physics Map".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .add_startup_system(setup.system())
        .add_system(update_map_collisions.system())
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    asset_server.watch_for_changes().unwrap();

    commands.insert_resource(Gravity::from(Vec3::new(0., -9.8 * 16., 0.)));

    // Spawn the camera
    const CAMERA_HEIGHT: f32 = 160.0;
    commands.spawn_bundle(OrthographicCameraBundle {
        orthographic_projection: OrthographicProjection {
            scale: CAMERA_HEIGHT / 2.0,
            scaling_mode: ScalingMode::FixedVertical,
            depth_calculation: DepthCalculation::ZDifference,
            ..Default::default()
        },
        ..OrthographicCameraBundle::new_2d()
    });

    // Spawn the map
    commands.spawn().insert_bundle(LdtkMapBundle {
        map: asset_server.load("maps/physicsDemoMap.ldtk"),
        ..Default::default()
    });

    let radish_images = [
        asset_server.load("redRadish.png"),
        asset_server.load("blueRadish.png"),
        asset_server.load("yellowRadish.png"),
    ];

    // Spawn bouncy radishes
    for y in 0..=2 {
        for x in -10..=10 {
            let sprite_image = radish_images[((x as i32).abs() % 3) as usize].clone();
            commands
                .spawn_bundle(PipelinedSpriteBundle {
                    texture: sprite_image.clone(),
                    transform: Transform::from_xyz(x as f32 * 12., 80. + y as f32 * 20., 0.),
                    ..Default::default()
                })
                .insert(TesselatedCollider {
                    texture: sprite_image,
                    tesselator_config: TesselatedColliderConfig {
                        // We want the collision shape for the player to be highly accurate
                        vertice_separation: 0.,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                // The player is also a dynamic body
                .insert(RigidBody::Dynamic)
                // Disable friction and bounciness
                .insert(PhysicMaterial {
                    friction: 0.2,
                    restitution: 1.,
                    ..Default::default()
                })
                // Set the player speed to 0 initially
                .insert(Velocity::from_linear(Vec3::default()))
                .insert(Player);
        }
    }
}

struct MapTileLoaded;

/// This system will go through each layer in spawned maps and generate a collision shape for each tile
fn update_map_collisions(
    mut commands: Commands,
    map_tiles: Query<
        (Entity, &Handle<TextureAtlas>, &TextureAtlasSprite),
        (Without<MapTileLoaded>, With<LdtkMapTile>),
    >,
    texture_atlas_assets: Res<Assets<TextureAtlas>>,
    image_assets: Res<Assets<Image>>,
) {
    for (tile_ent, atlas_handle, sprite) in map_tiles.iter() {
        // Get the texture atlas
        let atlas = if let Some(atlas) = texture_atlas_assets.get(atlas_handle) {
            atlas
        } else {
            continue;
        };
        // Get the texture atlas image
        let image = if let Some(image) = image_assets.get(&atlas.texture) {
            image
        } else {
            continue;
        };

        let mut tile_commands = commands.entity(tile_ent);

        // Create an image reference to the raw texture data
        let image_ref = image::flat::FlatSamples {
            samples: image.data.as_slice(),
            layout: image::flat::SampleLayout::row_major_packed(
                4,
                image.texture_descriptor.size.width,
                image.texture_descriptor.size.height,
            ),
            color_hint: Some(image::ColorType::Rgba8),
        };
        let image_view = image_ref.as_view().unwrap();

        // Get the section of the texture atlas for this tile in the map
        let rect = atlas.textures.get(sprite.index as usize).unwrap();

        // Get the portion of the atlas image for this tile
        let sub_img = image_view.view(
            rect.min.x as u32,
            rect.min.y as u32,
            rect.width() as u32,
            rect.height() as u32,
        );

        // Copy the portion of the image for this tile into a new image buffer
        let mut image_buffer = image::ImageBuffer::new(sub_img.width(), sub_img.height());
        for y in 0..sub_img.height() {
            for x in 0..sub_img.width() {
                let p = sub_img.get_pixel(x, y);
                image_buffer.put_pixel(x, y, p);
            }
        }

        // Try to generate a convex collision mesh from the tile image buffer
        let mesh = create_convex_collider(
            image::DynamicImage::ImageRgba8(image_buffer),
            &TesselatedColliderConfig {
                // The maximum accuracy for collision mesh generation
                vertice_separation: 0.,
                ..Default::default()
            },
        );

        // If mesh generation was successful ( wouldn't be for empty tiles, etc. )
        if let Some(mesh) = mesh {
            // Spawn a collider as a child of the map layer
            tile_commands.with_children(|tile| {
                tile.spawn_bundle((mesh, Transform::default(), GlobalTransform::default()));
            });
        }

        tile_commands
            // Make tile a static body
            .insert(RigidBody::Static)
            // Mark as loaded
            .insert(MapTileLoaded);
    }
}

struct Player;

//! Physics in Bevy Retrograde leverages the Rapier crate for almost everything. The only difference
//! is the use of the `TesselatedCollider` component that can be used to create a convex hull
//! collision shape from a sprite image.

use bevy::{prelude::*, sprite::SpriteBundle};
use bevy_retrograde::prelude::*;

use serde::Deserialize;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde Physics Map".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins::default())
        .add_startup_system(setup)
        .add_system(update_map_collisions)
        .insert_resource(LevelSelection::Index(0))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    asset_server.watch_for_changes().unwrap();

    // Spawn the camera
    commands.spawn_bundle(RetroCameraBundle::fixed_height(160.0));

    // Spawn the map
    commands.spawn().insert_bundle(LdtkWorldBundle {
        ldtk_handle: asset_server.load("maps/physicsDemoMap.ldtk"),
        transform: Transform::from_translation(Vec3::new(-130., -75., -1.)),
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
                .spawn_bundle(SpriteBundle {
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
                // And he's bouncy
                .insert(Restitution::coefficient(0.8))
                .insert(Player);
        }
    }
}

/// This is the struct for the LDtk tile metadata.
///
/// You set this metadata using RON syntax in the LDtk GUI.
#[derive(Deserialize)]
struct TileCollisionMetadata {
    colliders: Vec<ColliderMeta>,
}

#[derive(Deserialize)]
#[serde(rename = "Collider")]
struct ColliderMeta {
    #[serde(default)]
    position: Vec2,
    #[serde(default)]
    rotation: f32,
    shape: ColliderShapeMeta,
}

#[derive(Deserialize)]
enum ColliderShapeMeta {
    Rect { size: Vec2 },
    Circle { diameter: f32 },
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct TileCollisionLoaded;

/// This system will go through each layer in spawned maps and generate a collision shape for each tile
fn update_map_collisions(
    mut commands: Commands,
    map_tiles: Query<(Entity, &TileMetadata), Without<TileCollisionLoaded>>,
) {
    for (entity, metadata) in map_tiles.iter() {
        let metadata: &TileMetadata = metadata;
        let entity: Entity = entity;

        let metadata = ron::de::from_str::<TileCollisionMetadata>(&metadata.data).unwrap();
        let colliders = metadata
            .colliders
            .iter()
            .map(|collider| match collider.shape {
                ColliderShapeMeta::Rect { size } => (
                    collider.position,
                    collider.rotation,
                    Collider::cuboid(size.x / 2.0, size.y / 2.0),
                ),
                ColliderShapeMeta::Circle { diameter } => (
                    collider.position,
                    collider.rotation,
                    Collider::ball(diameter / 2.0),
                ),
            })
            .collect::<Vec<_>>();

        let collider = Collider::compound(colliders);

        commands
            .entity(entity)
            .insert(TileCollisionLoaded)
            .with_children(|children| {
                children
                    .spawn()
                    .insert(RigidBody::Fixed)
                    .insert(collider)
                    .insert_bundle(TransformBundle::default());
            });
    }
}

#[derive(Component)]
struct Player;

use bevy::{core::FixedTimestep, prelude::*};
use bevy_retrograde::prelude::*;

// Create a stage label that will be used for our game logic stage
#[derive(StageLabel, Debug, Eq, Hash, PartialEq, Clone)]
struct GameStage;

struct RadishImages {
    collided: Handle<Image>,
    uncollided: Handle<Image>,
}

impl FromWorld for RadishImages {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        RadishImages {
            collided: asset_server.load("redRadish.png"),
            uncollided: asset_server.load("blueRadish.png"),
        }
    }
}

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde Collisions".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .init_resource::<RadishImages>()
        .add_startup_system(setup.system())
        .add_stage(
            GameStage,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(0.05))
                .with_system(move_player.system())
                .with_system(collision_detection.system())
                .with_system(animate_sprite.system()),
        )
        .run();
}

struct Player;
struct SpriteAnimFrame(usize);

fn setup(
    mut commands: Commands,
    radish_images: Res<RadishImages>,
    mut sprite_sheet_assets: ResMut<Assets<SpriteSheet>>,
    asset_server: Res<AssetServer>,
) {
    // Spawn the camera
    commands.spawn().insert_bundle(CameraBundle {
        camera: Camera {
            size: CameraSize::FixedHeight(100),
            background_color: Color::new(0.2, 0.2, 0.2, 1.0),
            ..Default::default()
        },
        ..Default::default()
    });

    // Spawn a radish for the player
    commands
        .spawn()
        .insert_bundle(SpriteSheetBundle {
            sprite_bundle: SpriteBundle {
                image: asset_server.load("yellowRadishSheet.png"),
                position: Position::new(0, 0, 3),
                ..Default::default()
            },
            sprite_sheet: sprite_sheet_assets.add(SpriteSheet {
                grid_size: UVec2::splat(16),
                tile_index: 0,
            }),
        })
        .insert(SpriteAnimFrame(0))
        .insert(Player);

    // Spawn some radishes that just sit there
    for (x, y) in &[(-20, 0), (-20, -20), (20, 20), (20, 0)] {
        commands.spawn().insert_bundle(SpriteBundle {
            image: radish_images.uncollided.clone(),
            position: Position::new(*x, *y, 0),
            ..Default::default()
        });
    }
}

fn move_player(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Position, With<Player>>) {
    for mut pos in query.iter_mut() {
        const SPEED: i32 = 1;

        let mut direction = IVec3::new(0, 0, 0);

        if keyboard_input.pressed(KeyCode::Left) {
            direction += IVec3::new(-SPEED, 0, 0);
        }

        if keyboard_input.pressed(KeyCode::Right) {
            direction += IVec3::new(SPEED, 0, 0);
        }

        if keyboard_input.pressed(KeyCode::Up) {
            direction += IVec3::new(0, -SPEED, 0);
        }

        if keyboard_input.pressed(KeyCode::Down) {
            direction += IVec3::new(0, SPEED, 0);
        }

        if direction != IVec3::new(0, 0, 0) {
            **pos += direction;
        }
    }
}

fn collision_detection(
    // We need mutable access to the world positions so that we can sync transforms
    mut world_positions: WorldPositionsQuery,
    mut players: Query<(Entity, &Handle<Image>, &Sprite, &Handle<SpriteSheet>), With<Player>>,
    mut radishes: Query<(Entity, &mut Handle<Image>, &Sprite), Without<Player>>,
    mut scene_graph: ResMut<SceneGraph>,
    image_assets: Res<Assets<Image>>,
    sprite_sheet_assets: Res<Assets<SpriteSheet>>,
    radish_images: Res<RadishImages>,
) {
    // Make sure collision positions are synchronized
    world_positions.sync_world_positions(&mut scene_graph);

    // Loop over the players
    for (player, player_image, player_sprite, player_sprite_sheet) in players.iter_mut() {
        // Get the collision image of the player
        let player_image = if let Some(col) = image_assets.get(player_image) {
            col
        } else {
            continue;
        };
        // Get the spritesheet of the player
        let player_sprite_sheet = if let Some(col) = sprite_sheet_assets.get(player_sprite_sheet) {
            col
        } else {
            continue;
        };

        for (radish, mut radish_image, radish_sprite) in radishes.iter_mut() {
            // Get collision image for the other radish
            let other_radish_image = if let Some(col) = image_assets.get(radish_image.clone()) {
                col
            } else {
                continue;
            };

            // Check for collisions
            if pixels_collide_with_pixels(
                PixelColliderInfo {
                    image: player_image,
                    sprite: player_sprite,
                    // We need to grab the world position of the player from the
                    // `WorldPositionsQuery` query. Also, because the `WorldPositionsQuery` takes
                    // mutable borrow to the world position, we have to clone it to avoid mutably
                    // borrowing it twice when we get the position of the radish immediately below.
                    world_position: &world_positions
                        .get_world_position_mut(player)
                        .unwrap()
                        .clone(),
                    sprite_sheet: Some(player_sprite_sheet),
                },
                PixelColliderInfo {
                    image: other_radish_image,
                    sprite: radish_sprite,
                    world_position: &world_positions.get_world_position_mut(radish).unwrap(),

                    sprite_sheet: None,
                },
            ) {
                // Set the radish image to the collided image if we are running into him
                *radish_image = radish_images.collided.clone();
            } else {
                // Set the radish image to the uncollided image if we are not running into him
                *radish_image = radish_images.uncollided.clone();
            }
        }
    }
}

fn animate_sprite(
    // Keep track if the frame number
    mut frame: Local<u8>,
    mut query: Query<(&Handle<SpriteSheet>, &mut SpriteAnimFrame), With<Handle<Image>>>,
    mut sprite_sheet_assets: ResMut<Assets<SpriteSheet>>,
) {
    // Increment frame number
    *frame = frame.wrapping_add(1);

    let frames = [4, 5, 6, 7];

    // Play the next animation frame every 10 frames
    if *frame % 10 == 0 {
        *frame = 0;
        for (sprite_sheet_handle, mut frame) in query.iter_mut() {
            if let Some(sprite_sheet) = sprite_sheet_assets.get_mut(sprite_sheet_handle) {
                frame.0 = frame.0.wrapping_add(1);
                sprite_sheet.tile_index = frames[frame.0 % frames.len()];
            }
        }
    }
}

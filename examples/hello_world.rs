use bevy::{core::FixedTimestep, prelude::*};
use bevy_retro::*;

// Create a stage label that will be used for our game logic stage
#[derive(StageLabel, Debug, Eq, Hash, PartialEq, Clone)]
struct GameStage;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retro Hello World".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .add_startup_system(setup.system())
        .add_stage(
            GameStage,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(0.015))
                .with_system(move_player.system()),
        )
        .run();
}

struct Player;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scene_graph: ResMut<SceneGraph>,
) {
    // Load our sprites
    let red_radish_image = asset_server.load("redRadish.png");
    let yellow_radish_image = asset_server.load("yellowRadish.png");
    let blue_radish_image = asset_server.load("blueRadish.png");

    // Spawn the camera
    commands.spawn().insert_bundle(CameraBundle {
        camera: Camera {
            // Set our camera to have a fixed height and an auto-resized width
            size: CameraSize::FixedHeight(100),
            background_color: Color::new(0.2, 0.2, 0.2, 1.0),
            ..Default::default()
        },
        ..Default::default()
    });

    // Spawn a red radish
    let red_radish = commands
        .spawn()
        .insert_bundle(SpriteBundle {
            image: red_radish_image,
            position: Position::new(0, 0, 0),
            ..Default::default()
        })
        // Add our player marker component so we can move it
        .insert(Player)
        .id();

    // Spawn a yellow radish
    let yellow_radish = commands
        .spawn()
        .insert_bundle(SpriteBundle {
            image: yellow_radish_image,
            position: Position::new(-20, 0, 0),
            sprite: Sprite {
                // Make him upside down 🙃
                flip_y: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    // Make the yellow radish a child of the red radish
    scene_graph
        .add_child(red_radish, yellow_radish)
        // This could fail if the child is an ancestor of the parent
        .unwrap();

    // Spawn a blue radish
    commands.spawn().insert_bundle(SpriteBundle {
        image: blue_radish_image,
        // Set the blue radish back a layer so that he shows up under the other two
        position: Position::new(-20, -20, -1),
        sprite: Sprite {
            flip_x: true,
            flip_y: false,
            ..Default::default()
        },
        ..Default::default()
    });
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

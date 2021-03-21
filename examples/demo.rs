use std::time::Duration;

use bevy::prelude::*;
use bevy_retro::*;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retro Demo".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .add_startup_system(setup.system())
        .add_system(move_sensei.system())
        .run();
}

// Marker component for the sensei
struct Sensei;
struct Student;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scene_graph: ResMut<SceneGraph>,
) {
    let sensei_image = asset_server.load("sensei2.gitignore.png");
    let guy_image = asset_server.load("guy.gitignore.png");

    let sensei = commands.spawn(()).current_entity().unwrap();
    let sensei_node = scene_graph.add_node(sensei);
    commands
        .with_bundle(SpriteBundle {
            image: sensei_image,
            scene_node: sensei_node,
            position: Position::new(0, 0, 0),
            sprite_flip: SpriteFlip { x: true, y: false },
            ..Default::default()
        })
        // Add our sensei marker component
        .with(Sensei);

    let guy = commands.spawn(()).current_entity().unwrap();
    let guy_node = scene_graph.add_node(guy);

    // And add the sprite components to the guy
    commands
        .with_bundle(SpriteBundle {
            image: guy_image,
            scene_node: guy_node,
            // The guy follows a little behind the sensei
            position: Position::new(-20, 4, 1),
            ..Default::default()
        })
        .with(Student);

    // Spawn the camera
    let camera = commands.spawn(()).current_entity().unwrap();
    let camera_node = scene_graph.add_node(camera);

    commands.with_bundle(CameraBundle {
        scene_node: camera_node,
        camera: Camera {
            size: CameraSize::FixedHeight(100),
            background_color: Color::new(0.1, 0.1, 0.2, 1.0),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn move_sensei(
    time: Res<Time>,
    mut timer: Local<Timer>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Position, With<Sensei>>,
) {
    timer.set_duration(Duration::from_millis(40));
    timer.set_repeating(true);

    timer.tick(time.delta());

    if timer.finished() {
        for mut pos in query.iter_mut() {
            const SPEED: i32 = 1;

            let mut direction = IVec3::new(0, 0, 0);

            if keyboard_input.pressed(KeyCode::A) {
                direction += IVec3::new(-SPEED, 0, 0);
            }

            if keyboard_input.pressed(KeyCode::D) {
                direction += IVec3::new(SPEED, 0, 0);
            }

            if keyboard_input.pressed(KeyCode::W) {
                direction += IVec3::new(0, -SPEED, 0);
            }

            if keyboard_input.pressed(KeyCode::S) {
                direction += IVec3::new(0, SPEED, 0);
            }

            **pos += direction;
        }
    }
}

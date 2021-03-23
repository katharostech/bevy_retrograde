use std::time::Duration;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_retro::*;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retro Demo".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        // Optional diagnostics
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
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
    let barrel_image = asset_server.load("barrel.gitignore.png");

    let sensei = commands
        .spawn()
        .insert_bundle(SpriteBundle {
            image: sensei_image,
            position: Position::new(0, 0, 1),
            sprite: Sprite {
                flip_x: true,
                flip_y: false,
                ..Default::default()
            },
            ..Default::default()
        })
        // Add our sensei marker component
        .insert(Sensei)
        .id();

    // And add the sprite components to the guy
    let guy = commands
        .spawn()
        .insert_bundle(SpriteBundle {
            image: guy_image,
            // The guy follows a little behind the sensei
            position: Position::new(-40, 0, 0),
            ..Default::default()
        })
        .insert(Student)
        .id();

    // Add guy as a faithful student ( child ) of the sensei
    scene_graph.add_child(sensei, guy).unwrap();

    // And add the sprite components to the guy
    let barrel = commands
        .spawn()
        .insert_bundle(SpriteBundle {
            image: barrel_image,
            // The guy follows a little behind the sensei
            position: Position::new(-20, 0, 0),
            ..Default::default()
        })
        .insert(Student)
        .id();

    // Add the barrel as a child of guy
    scene_graph.add_child(guy, barrel).unwrap();

    // Spawn the camera
    commands.spawn().insert_bundle(CameraBundle {
        camera: Camera {
            size: CameraSize::FixedHeight(100),
            background_color: Color::new(0.1, 0.1, 0.2, 1.0),
            pixel_aspect_ratio: 4.0 / 3.0,
            ..Default::default()
        },
        position: Position::new(0, 0, 0),
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

            if direction != IVec3::new(0, 0, 0) {
                **pos += direction;
            }
        }
    }
}

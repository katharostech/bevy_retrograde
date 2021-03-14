use std::time::Duration;

use bevy::prelude::*;
use bevy_retro::*;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retro Demo".into(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(RetroPlugin)
        .add_startup_system(setup.system())
        .add_system(move_sensei.system())
        .run();
}

struct Sensei;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load our sprite image
    let sensei = asset_server.load("sensei2.gitignore.png");
    let guy = asset_server.load("guy.gitignore.png");

    // Setup the scene
    commands
        // Spawn the camera
        .spawn(CameraBundle {
            camera: Camera {
                size: CameraSize::FixedHeight(100),
                ..Default::default()
            },
            ..Default::default()
        })
        // and the sprite
        .spawn(SpriteBundle {
            image: guy,
            position: Position(IVec3::new(0, 0, 0)),
            ..Default::default()
        })
        // and another
        .spawn(SpriteBundle {
            image: sensei,
            position: Position(IVec3::new(5, 5, 1)),
            ..Default::default()
        })
        .with(Sensei);
}

fn move_sensei(
    time: Res<Time>,
    mut timer: Local<Timer>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Handle<SpriteImage>, &mut Position), With<Sensei>>,
) {
    timer.set_duration(Duration::from_millis(40));
    timer.set_repeating(true);

    timer.tick(time.delta());

    if timer.finished() {
        for (_, mut pos) in query.iter_mut() {
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

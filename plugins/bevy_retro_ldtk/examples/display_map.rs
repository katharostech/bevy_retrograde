use std::{path::PathBuf, time::Duration};

use bevy::prelude::*;
use bevy_retro::*;
use bevy_retro_ldtk::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(LdtkPlugin)
        .add_plugin(RetroPlugin)
        .add_startup_system(setup.system())
        .add_system(move_camera.system())
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scene_graph: ResMut<SceneGraph>,
) {
    // Enable hot reload
    asset_server.watch_for_changes().unwrap();

    // Spawn the camera
    commands.spawn(CameraBundle {
        camera: Camera {
            size: CameraSize::FixedHeight(180),
            ..Default::default()
        },
        ..Default::default()
    });

    // Spawn the map
    let map_ent = commands.spawn(()).current_entity().unwrap();
    let map_node = scene_graph.add_node(map_ent);

    commands.with_bundle(LdtkMapBundle {
        map: asset_server.load(PathBuf::from(
            &std::env::args().nth(1).unwrap_or("map1.ldtk".into()),
        )),
        scene_node: map_node,
        config: LdtkMapConfig {
            set_clear_color: true,
            scale: 3.0,
            level: std::env::args()
                .nth(2)
                .map(|x| x.parse().unwrap())
                .unwrap_or(0),
            center_map: true,
        },
        ..Default::default()
    });
}

fn move_camera(
    time: Res<Time>,
    mut timer: Local<Timer>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Position, With<Camera>>,
) {
    timer.set_duration(Duration::from_millis(10));
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

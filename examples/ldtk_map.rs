use bevy::{core::FixedTimestep, prelude::*};
use bevy_retrograde::prelude::*;

#[derive(StageLabel, Debug, Clone, Hash, Eq, PartialEq)]
struct GameStage;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde LDtk Map".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .add_startup_system(setup.system())
        .add_stage(
            GameStage,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(0.012))
                .with_system(move_camera.system()),
        )
        .add_system(set_background_color.system())
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Enable hot reload
    asset_server.watch_for_changes().unwrap();

    // Spawn the camera
    commands.spawn().insert_bundle(CameraBundle {
        camera: Camera {
            size: CameraSize::FixedHeight(180),
            ..Default::default()
        },
        ..Default::default()
    });

    // Spawn the map
    commands.spawn().insert_bundle(LdtkMapBundle {
        map: asset_server.load("maps/map.ldtk"),
        // We offset the map a little to move it more to the center of the screen, because maps are
        // spawned with (0, 0) as the top-left corner of the map
        position: Position::new(-200, -100, 0),
        ..Default::default()
    });
}

/// This system moves the camera so you can look around the map
fn move_camera(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Position, With<Camera>>) {
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

        **pos += direction;
    }
}

/// This system sets the camera background color to the background color of the maps first level
fn set_background_color(
    mut cameras: Query<&mut Camera>,
    maps: Query<&Handle<LdtkMap>>,
    ldtk_map_assets: Res<Assets<LdtkMap>>,
) {
    // If the camera background color isn't set, set it. We also only read the clear
    // color of the first level for now.
    for map_handle in maps.iter() {
        if let Some(map) = ldtk_map_assets.get(map_handle) {
            let level = map.project.levels.get(0).unwrap();

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

                camera.background_color = Color::from_rgba8(decoded[0], decoded[1], decoded[2], 1);
            }
        }
    }
}

use bevy::{
    core_pipeline::ClearColor,
    prelude::*,
    render2::{
        camera::{
            Camera, DepthCalculation, OrthographicCameraBundle, OrthographicProjection, ScalingMode,
        },
        color::Color,
    },
};
use bevy_retrograde::prelude::*;

#[derive(StageLabel, Debug, Clone, Hash, Eq, PartialEq)]
struct GameStage;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde LDtk Map".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .add_startup_system(setup)
        .add_system(set_background_color)
        .add_system(move_camera)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Enable hot reload
    asset_server.watch_for_changes().unwrap();

    // Spawn the camera
    const CAMERA_HEIGHT: f32 = 200.0;
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
    let map = asset_server.load("maps/map.ldtk");
    commands.spawn_bundle(LdtkMapBundle {
        map: map.clone(),
        // We offset the map a little to move it more to the center of the screen, because maps are
        // spawned with (0, 0) as the top-left corner of the map
        transform: Transform::from_xyz(-175., 100., 0.),
        ..Default::default()
    });
}

/// This system moves the camera so you can look around the map
fn move_camera(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
) {
    for mut transform in query.iter_mut() {
        let speed: f32 = 60.0 * time.delta_seconds();

        let mut direction = Vec3::new(0., 0., 0.);

        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            direction += Vec3::new(-speed, 0., 0.);
        }

        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            direction += Vec3::new(speed, 0., 0.);
        }

        if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
            direction += Vec3::new(0., speed, 0.);
        }

        if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
            direction += Vec3::new(0., -speed, 0.);
        }

        transform.translation += direction;
    }
}

/// This system sets the camera background color to the background color of the maps first level
fn set_background_color(
    mut commands: Commands,
    maps: Query<&Handle<LdtkMap>>,
    ldtk_map_assets: Res<Assets<LdtkMap>>,
) {
    // If the camera background color isn't set, set it. We also only read the clear
    // color of the first level for now.
    for map_handle in maps.iter() {
        if let Some(map) = ldtk_map_assets.get(map_handle) {
            let level = map.project.levels.get(0).unwrap();

            let decoded = hex::decode(
                level
                    .bg_color
                    .as_ref()
                    .unwrap_or(&map.project.default_level_bg_color)
                    .strip_prefix("#")
                    .expect("Invalid background color"),
            )
            .expect("Invalid background color");

            commands.insert_resource(ClearColor(Color::Rgba {
                red: decoded[0] as f32 / 255.0,
                green: decoded[1] as f32 / 255.0,
                blue: decoded[2] as f32 / 255.0,
                alpha: 1.0,
            }));
        }
    }
}

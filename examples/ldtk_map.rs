use bevy::{prelude::*, asset::ChangeWatcher};
use bevy_retrograde::prelude::*;

#[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
struct GameStage;

fn main() {
    App::new()
        .add_plugins(RetroPlugins::default().set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Retrograde LDtk Map".into(),
                ..Default::default()
            }),
            ..Default::default()
        }).set(AssetPlugin {
            watch_for_changes: ChangeWatcher::with_delay(std::time::Duration::from_secs_f32(0.1)),
            ..Default::default()
        }).set(ImagePlugin::default_nearest())
        )
        .add_systems(Startup, setup)
        .add_systems(Update, move_camera)
        .insert_resource(LevelSelection::Index(0))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {

    // Spawn the camera
    commands.spawn(RetroCameraBundle::fixed_height(200.0));

    // Spawn the map
    let map = asset_server.load("maps/map.ldtk");
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: map,
        // We offset the map a little to move it more to the center of the screen, because maps are
        // spawned with (0, 0) as the top-left corner of the map
        transform: Transform::from_xyz(-175., -100., 0.),
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

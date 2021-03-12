use bevy::prelude::*;
use bevy_retro::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(RetroPlugin)
        .add_startup_system(setup.system())
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load our sprite image
    let image = asset_server.load("guy.gitignore.png");

    // Setup the scene
    commands
        // Spawn the camera
        .spawn((Camera {
            size: 400,
            ..Default::default()
        },))
        // and the sprite
        .spawn(SpriteBundle {
            image,
            ..Default::default()
        });
}

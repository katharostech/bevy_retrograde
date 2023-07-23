use bevy::prelude::*;
use bevy_retrograde::prelude::*;

fn main() {
    App::new()
        .add_plugins(RetroPlugins::default().set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Retrograde Audio".into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_systems(Startup, setup)
        .run();
}

fn setup(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    // Load the sound data
    let sound = asset_server.load("blink.ogg");

    // Play the sound
    audio.play(sound);

    // Load the music
    let music = asset_server.load("music.ogg");

    // Play it on loop
    audio.play(music).looped();
}

use bevy::prelude::*;
use bevy_retrograde::prelude::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde Audio".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins::default())
        .add_startup_system(setup)
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
    audio.play_looped(music);
}

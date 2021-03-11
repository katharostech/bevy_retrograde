use bevy::prelude::*;
use bevy_retro::RetroPlugin;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(RetroPlugin)
        .run();
}

# bevy_retro

[![Crates.io](https://img.shields.io/crates/v/bevy_retro.svg)](https://crates.io/crates/bevy_retro)
[![Docs.rs](https://docs.rs/bevy_retro/badge.svg)](https://docs.rs/bevy_retro)
[![Katharos License](https://img.shields.io/badge/License-Katharos-blue)](https://github.com/katharostech/katharos-license)

Bevy Retro is a 2D, pixel-perfect renderer for [Bevy] that can target both web and desktop using
OpenGL/WebGL.

[Bevy]: https://bevyengine.org

## Example

```rust
use bevy::prelude::*;
use bevy_retro::*;

fn main() {
    App::build()
        .add_plugins(RetroPlugins)
        .add_startup_system(setup.system())
        .run();
}

struct Player;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scene_graph: ResMut<SceneGraph>,
) {
    // Load our sprites
    let red_radish_image = asset_server.load("redRadish.png");
    let yellow_radish_image = asset_server.load("yellowRadish.png");
    let blue_radish_image = asset_server.load("blueRadish.png");

    // Spawn the camera
    commands.spawn().insert_bundle(CameraBundle {
        camera: Camera {
            // Set our camera to have a fixed height and an auto-resized width
            size: CameraSize::FixedHeight(100),
            background_color: Color::new(0.2, 0.2, 0.2, 1.0),
            ..Default::default()
        },
        position: Position::new(0, 0, 0),
        ..Default::default()
    });

    // Spawn a red radish
    let red_radish = commands
        .spawn()
        .insert_bundle(SpriteBundle {
            image: red_radish_image,
            position: Position::new(0, 0, 0),
            sprite: Sprite {
                flip_x: true,
                flip_y: false,
                ..Default::default()
            },
            ..Default::default()
        })
        // Add our player marker component so we can move it
        .insert(Player)
        .id();

    // Spawn a yellow radish
    let yellow_radish = commands
        .spawn()
        .insert_bundle(SpriteBundle {
            image: yellow_radish_image,
            position: Position::new(-20, 0, 0),
            sprite: Sprite {
                flip_x: true,
                flip_y: false,
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    // Make the yello radish a child of the red radish
    scene_graph
        .add_child(red_radish, yellow_radish)
        // This could fail if the child is an ancestor of the parent
        .unwrap();

    // Spawn a blue radish
    commands.spawn().insert_bundle(SpriteBundle {
        image: blue_radish_image,
        // Set the blue radish back a layer so that he shows up under the other two
        position: Position::new(-20, -20, -1),
        sprite: Sprite {
            flip_x: true,
            flip_y: false,
            ..Default::default()
        },
        ..Default::default()
    });
}
```

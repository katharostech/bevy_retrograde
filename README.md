# bevy_retro

[![Crates.io](https://img.shields.io/crates/v/bevy_retro.svg)](https://crates.io/crates/bevy_retro)
[![Docs.rs](https://docs.rs/bevy_retro/badge.svg)](https://docs.rs/bevy_retro)
[![lines of code](https://tokei.rs/b1/github/katharostech/bevy_retro?category=code)](https://github.com/katharostech/bevy_retro)
[![Katharos License](https://img.shields.io/badge/License-Katharos-blue)](https://github.com/katharostech/katharos-license)

[bounty bros game screenshot](./doc/bounty_bros.png)

_Screenshot of [Bounty Bros.](https://katharostech.com/post/bounty-bros-on-web) game_

Bevy Retro is a 2D, pixel-perfect renderer for [Bevy] that can target both web and desktop using
OpenGL/WebGL.

[Bevy]: https://bevyengine.org

Bevy Retro is focused on providing an easy and ergonomic way to write 2D, pixel-perfect games.
Compared to the out-of-the-box Bevy setup, it has no concept of 3D, and sprites don't even have
rotations, scales, or floating point positions. All coordinates are based on real pixel
positions.

Bevy Retro replaces almost all of the out-of-the-box Bevy components and Bundles that you would
normally use ( `Transform`, `Camera2DBundle`, etc. ) and comes with its own `Position`,
`Camera`, `Image`, `Sprite`, etc. components and bundles. Bevy Retro tries to provide a focused
2D-centric experience on top of Bevy that helps take out some of the pitfalls and makes it
easier to think about your game when all you need is 2D.

## Development Status

Bevy Retro is in very early stages of development, but should still be somewhat usable.
Potentially drastic breaking changes are a large possibility, though. Bevy Retro's design will
mature as we use it to work on an actual game and we find out what works and what doesn't.

Bevy Retro will most likely track Bevy master as it changes, but we may also be able to make
Bevy Retro releases for each Bevy release.

## Features

- Supports web and desktop out-of-the-box
- Integer pixel coordinates
- Supports sprites and sprite sheets
- A super-simple hierarchy system
- Scaled pixel-perfect rendering with three camera modes: fixed width, fixed height, and
  letter-boxed
- An [LDtk](https://ldtk.io) map loading [plugin](./plugins/bevy_retro_ldtk)
- Pixel-perfect collision detection
- Custom shaders for post-processing, including a built-in CRT shader


## Examples

Check out the [examples] folder for more examples, but here's a quick look at using Bevy Retro:

[examples]: https://github.com/katharostech/bevy_retro/tree/master/examples

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

    // Make the yellow radish a child of the red radish
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

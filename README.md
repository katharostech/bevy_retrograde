# Bevy Retrograde

[![Crates.io](https://img.shields.io/crates/v/bevy_retrograde.svg)](https://crates.io/crates/bevy_retrograde)
[![Docs.rs](https://docs.rs/bevy_retrograde/badge.svg)](https://docs.rs/bevy_retrograde)
[![Build Status](https://github.com/katharostech/bevy_retrograde/actions/workflows/rust.yaml/badge.svg)](https://github.com/katharostech/bevy_retrograde/actions/workflows/rust.yaml)
[![lines of code](https://tokei.rs/b1/github/katharostech/bevy_retrograde?category=code)](https://github.com/katharostech/bevy_retrograde)
[![Katharos License](https://img.shields.io/badge/License-Katharos-blue)](https://github.com/katharostech/katharos-license)

<div align="center">
    <em>( Screenshot of <a href="https://katharostech.com/post/bounty-bros-on-web">Bounty Bros.</a> game made with Bevy Retrograde and <a href="https://github.com/katharostech/skipngo">Skip'n Go</a> )</em>
</div>

![bounty bros game screenshot](./doc/bounty_bros.png)

[skipngo]:  https://github.com/katharostech/skipngo

Bevy Retrograde is a 2D, pixel-perfect renderer for [Bevy][__link0] that can target both web and desktop using OpenGL/WebGL.

Bevy Retrograde is focused on providing an easy and ergonomic way to write 2D, pixel-perfect games. Compared to the out-of-the-box Bevy setup, you do not have to work with a 3D scene to create 2D games. Sprites and their coordinates are based on pixel positions in a retro-resolution scene.

Bevy Retrograde replaces almost all of the out-of-the-box Bevy components and Bundles that you would normally use ( `Transform`, `Camera2DBundle`, etc. ) and comes with its own `Position`, `Camera`, `Image`, `Sprite`, etc. components and bundles. Bevy Retrograde tries to provide a focused 2D-centric experience on top of Bevy that helps take out some of the pitfalls and makes it easier to think about your game when all you need is 2D.

We want to provide a batteries-included plugin that comes with almost everything you need to make a 2D pixel game with Bevy including, collisions, sound, saving data, etc. While adding these features we will try to maintain full web compatibility, but it can’t be guaranteed that all features will be feasible to implement for web.

These extra features will be included as optional cargo features that can be disabled if not needed and, where applicable, may be packaged as separate Rust crates that can be used even if you don’t want to use the rest of Bevy Retrograde.


## License

Bevy Retrograde LDtk is licensed under the [Katharos License][__link1] which places certain restrictions on what you are allowed to use it for. Please read and understand the terms before using Bevy Retrograde for your project.


## Development Status

Bevy Retrograde is in early stages of development. The API is not stable and may change dramatically at any time. Planned possible changes include:

 - Switching to using Bevy’s built-in renderer for desktop/mobile and [`bevy_webgl2`][__link2] for web instead of using our own OpenGL based renderer. This will potentially make Bevy Retrograde more compatible with the larger Bevy ecosystem instead of it creating an island of plugins that only work on Bevy Retro. We will probably wait for the [second iteration][__link3] of the Bevy rendererer to attempt this.

See also [Supported Bevy Version](#supported-bevy-version) below.


## Features & Examples

Check out our [examples][__link4] list to see how to use each Bevy Retrograde feature:

 - Supports web and desktop out-of-the-box
 - Sprites and sprite sheets
 - Scaled pixel-perfect rendering with three camera modes: fixed width, fixed height, and letter-boxed
 - Sprites are pixel-perfectly aligned by default but can be set to non-perfect on a per-sprite basis
 - [LDtk][__link5] map loading and rendering
 - An integration with the [RAUI][__link6] UI library for building in-game user interfaces and HUD
 - Pixel-perfect collision detection
 - Text rendering of BDF fonts
 - Custom shaders for post-processing, including a built-in CRT shader
 - Render hooks allowing you to drop down into raw [Luminance][__link7] calls for custom rendering


## Supported Bevy Version

Bevy Retrograde currently works on the latest Bevy release and *may* support Bevy master as well. Bevy Retrograde will try to follow the latest Bevy release, but if there are features introduced in Bevy master that we need, we may require Bevy master for a time until the next Bevy release.

When depending on the `bevy` crate, you must be sure to set `default-features` to `false` in your `Cargo.toml` so that the rendering types in `bevy` don’t conflict with the ones in `bevy_retrograde`.

**`Cargo.toml`:**


```toml
bevy = { version = "0.5", default-features = false }
bevy_retrograde = "0.1.0"
```


## Sample

Here’s a quick sample of what using Bevy Retrograde looks like:

**`main.rs`:**


```rust
use bevy::prelude::*;
use bevy_retrograde::prelude::*;

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
        ..Default::default()
    });

    // Spawn a red radish
    let red_radish = commands
        .spawn_bundle(SpriteBundle {
            image: red_radish_image,
            transform: Transform::from_xyz(0., 0., 0.),
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
        .spawn_bundle(SpriteBundle {
            image: yellow_radish_image,
            transform: Transform::from_xyz(-20., 0., 0.),
            sprite: Sprite {
                // Flip the sprite upside down 🙃
                flip_y: true,
                // By setting a sprite to be non-pixel-perfect you can get smoother movement
                // for things like characters, like they did in Shovel Knight®.
                pixel_perfect: false,
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    // Make the yellow radish a child of the red radish
    commands.entity(red_radish).push_children(&[yellow_radish]);

    // Spawn a blue radish
    commands.spawn().insert_bundle(SpriteBundle {
        image: blue_radish_image,
        // Set the blue radish back a layer so that he shows up under the other two
        transform: Transform::from_xyz(-20., -20., -1.),
        sprite: Sprite {
            flip_x: true,
            flip_y: false,
            ..Default::default()
        },
        ..Default::default()
    });
}
```



 [__link0]: https://bevyengine.org
 [__link1]: https://github.com/katharostech/katharos-license
 [__link2]: https://github.com/mrk-its/bevy_webgl2
 [__link3]: https://github.com/bevyengine/bevy/discussions/2351
 [__link4]: https://github.com/katharostech/bevy_retrograde/tree/master/examples#bevy-retro-examples
 [__link5]: https://ldtk.io
 [__link6]: https://raui-labs.github.io/raui/
 [__link7]: https://github.com/phaazon/luminance-rs


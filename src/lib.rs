//! Bevy Retrograde is a 2D, pixel-perfect renderer for [Bevy] that can target both web and desktop
//! using OpenGL/WebGL.
//!
//! [Bevy]: https://bevyengine.org
//!
//! Bevy Retrograde is focused on providing an easy and ergonomic way to write 2D, pixel-perfect
//! games. Compared to the out-of-the-box Bevy setup, you do not have to work with a 3D scene to
//! create 2D games. Sprites and their coordinates are based on pixel positions in a
//! retro-resolution scene.
//!
//! Bevy Retrograde replaces almost all of the out-of-the-box Bevy components and Bundles that you
//! would normally use ( `Transform`, `Camera2DBundle`, etc. ) and comes with its own `Position`,
//! `Camera`, `Image`, `Sprite`, etc. components and bundles. Bevy Retrograde tries to provide a
//! focused 2D-centric experience on top of Bevy that helps take out some of the pitfalls and makes
//! it easier to think about your game when all you need is 2D.
//!
//! We want to provide a batteries-included plugin that comes with almost everything you need to
//! make a 2D pixel game with Bevy including, collisions, sound, saving data, etc. While adding
//! these features we will try to maintain full web compatibility, but it can't be guaranteed that
//! all features will be feasible to implement for web.
//!
//! These extra features will be included as optional cargo features that can be disabled if not
//! needed and, where applicable, may be packaged as separate Rust crates that can be used even if
//! you don't want to use the rest of Bevy Retrograde.
//!
//! # License
//!
//! Bevy Retrograde LDtk is licensed under the [Katharos License][k_license] which places certain
//! restrictions on what you are allowed to use it for. Please read and understand the terms before
//! using Bevy Retrograde for your project.
//!
//! [k_license]: https://github.com/katharostech/katharos-license
//!
//! # Development Status
//!
//! Bevy Retrograde is in early stages of development. The API is not stable and may change
//! dramatically at any time. Planned possible changes include:
//!
//! - Switching to using Bevy's built-in renderer for desktop/mobile and [`bevy_webgl2`] for web
//!   instead of using our own OpenGL based renderer. This will potentially make Bevy Retrograde
//!   more compatible with the larger Bevy ecosystem instead of it creating an island of plugins
//!   that only work on Bevy Retro. We will probably wait for the [second iteration][bevy_renderer2]
//!   of the Bevy rendererer to attempt this.
//!
//! [`bevy_webgl2`]: https://github.com/mrk-its/bevy_webgl2
//!
//! [bevy_renderer2]: https://github.com/bevyengine/bevy/discussions/2351
//!
//! See also [Supported Bevy Version](#supported-bevy-version) below.
//!
//! # Features & Examples
//!
//! Check out our [examples] list to see how to use each Bevy Retrograde feature:
//!
//! - Supports web and desktop out-of-the-box
//! - Sprites and sprite sheets
//! - Scaled pixel-perfect rendering with three camera modes: fixed width, fixed height, and
//!   letter-boxed
//! - Sprites are pixel-perfectly aligned by default but can be set to non-perfect on a per-sprite
//!   basis
//! - [LDtk](https://ldtk.io) map loading and rendering
//! - An integration with the [RAUI] UI library for building in-game user interfaces and HUD
//! - Physics and collision detection powered by [Heron] and [Rapier] with automatic generation of
//!   convex collision shapes from sprite images
//! - Text rendering of BDF fonts
//! - Custom shaders for post-processing, including a built-in CRT shader
//! - Render hooks allowing you to drop down into raw [Luminance] calls for custom rendering
//!
//! [examples]:
//! https://github.com/katharostech/bevy_retrograde/tree/master/examples#bevy-retro-examples
//!
//! [luminance]: https://github.com/phaazon/luminance-rs
//!
//! [RAUI]: https://raui-labs.github.io/raui/
//!
//! [Heron]: https://github.com/jcornaz/heron
//!
//! [Rapier]: https://rapier.rs/
//!
//! # Supported Bevy Version
//!
//! Bevy Retrograde currently works on the latest Bevy release and _may_ support Bevy master as
//! well. Bevy Retrograde will try to follow the latest Bevy release, but if there are features
//! introduced in Bevy master that we need, we may require Bevy master for a time until the next
//! Bevy release.
//!
//! When depending on the `bevy` crate, you must be sure to set `default-features` to `false` in
//! your `Cargo.toml` so that the rendering types in `bevy` don't conflict with the ones in
//! `bevy_retrograde`.
//!
//! **`Cargo.toml`:**
//!
//! ```toml
//! # Be sure to turn off the default features of Bevy to avoid conflicts with the
//! # Bevy Retrograde renderer types.
//! bevy = { version = "0.5", default-features = false }
//! bevy_retrograde = "0.1.0"
//! ```
//! # Sample
//!
//! Here's a quick sample of what using Bevy Retrograde looks like:
//!
//! **`main.rs`:**
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_retrograde::prelude::*;
//!
//! fn main() {
//!     App::build()
//!         .add_plugins(RetroPlugins)
//!         .add_startup_system(setup.system())
//!         .run();
//! }
//!
//! struct Player;
//!
//! fn setup(
//!     mut commands: Commands,
//!     asset_server: Res<AssetServer>,
//! ) {
//!     // Load our sprites
//!     let red_radish_image = asset_server.load("redRadish.png");
//!     let yellow_radish_image = asset_server.load("yellowRadish.png");
//!     let blue_radish_image = asset_server.load("blueRadish.png");
//!
//!     // Spawn the camera
//!     commands.spawn().insert_bundle(CameraBundle {
//!         camera: Camera {
//!             // Set our camera to have a fixed height and an auto-resized width
//!             size: CameraSize::FixedHeight(100),
//!             background_color: Color::new(0.2, 0.2, 0.2, 1.0),
//!             ..Default::default()
//!         },
//!         ..Default::default()
//!     });
//!
//!     // Spawn a red radish
//!     let red_radish = commands
//!         .spawn_bundle(SpriteBundle {
//!             image: red_radish_image,
//!             transform: Transform::from_xyz(0., 0., 0.),
//!             sprite: Sprite {
//!                 flip_x: true,
//!                 flip_y: false,
//!                 ..Default::default()
//!             },
//!             ..Default::default()
//!         })
//!         // Add our player marker component so we can move it
//!         .insert(Player)
//!         .id();
//!
//!     // Spawn a yellow radish
//!     let yellow_radish = commands
//!         .spawn_bundle(SpriteBundle {
//!             image: yellow_radish_image,
//!             transform: Transform::from_xyz(-20., 0., 0.),
//!             sprite: Sprite {
//!                 // Flip the sprite upside down 🙃
//!                 flip_y: true,
//!                 // By setting a sprite to be non-pixel-perfect you can get smoother movement
//!                 // for things like characters, like they did in Shovel Knight®.
//!                 pixel_perfect: false,
//!                 ..Default::default()
//!             },
//!             ..Default::default()
//!         })
//!         .id();
//!
//!     // Make the yellow radish a child of the red radish
//!     commands.entity(red_radish).push_children(&[yellow_radish]);
//!
//!     // Spawn a blue radish
//!     commands.spawn().insert_bundle(SpriteBundle {
//!         image: blue_radish_image,
//!         // Set the blue radish back a layer so that he shows up under the other two
//!         transform: Transform::from_xyz(-20., -20., -1.),
//!         sprite: Sprite {
//!             flip_x: true,
//!             flip_y: false,
//!             ..Default::default()
//!         },
//!         ..Default::default()
//!     });
//! }
//! ```

/// The Bevy Retrograde default plugins
pub struct RetroPlugins;

impl bevy::app::PluginGroup for RetroPlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        // Add the plugins we need from Bevy
        group.add(bevy::log::LogPlugin::default());
        group.add(bevy::core::CorePlugin::default());
        group.add(bevy::diagnostic::DiagnosticsPlugin::default());
        group.add(bevy::input::InputPlugin::default());
        group.add(bevy::window::WindowPlugin::default());
        group.add(bevy::asset::AssetPlugin::default());
        group.add(bevy::winit::WinitPlugin::default());
        group.add(bevy::scene::ScenePlugin::default());
        group.add(bevy::transform::TransformPlugin::default());

        group.add(core::RetroCorePlugin);

        #[cfg(feature = "audio")]
        group.add(audio::RetroAudioPlugin);

        #[cfg(feature = "ldtk")]
        group.add(ldtk::LdtkPlugin);

        #[cfg(feature = "text")]
        group.add(text::RetroTextPlugin);

        #[cfg(feature = "physics")]
        group.add(physics::RetroPhysicsPlugin);

        #[cfg(feature = "ui")]
        group.add(ui::RetroUiPlugin);
    }
}

/// The Bevy Retrograde prelude
#[doc(hidden)]
pub mod prelude {
    pub use crate::*;
    pub use bevy_retrograde_core::prelude::*;
    pub use bevy_retrograde_macros::impl_deref;

    #[cfg(feature = "audio")]
    pub use bevy_retrograde_audio::*;

    #[cfg(feature = "text")]
    pub use bevy_retrograde_text::prelude::*;

    #[cfg(feature = "ldtk")]
    pub use bevy_retrograde_ldtk::*;

    #[cfg(feature = "ui")]
    pub use bevy_retrograde_ui::*;

    #[cfg(feature = "physics")]
    pub use bevy_retrograde_physics::*;
}

#[doc(inline)]
pub use bevy_retrograde_core as core;

#[cfg(feature = "re-export-bevy")]
pub use bevy;

pub use bevy_retrograde_macros::impl_deref;

#[cfg(feature = "audio")]
#[doc(inline)]
pub use bevy_retrograde_audio as audio;

#[cfg(feature = "text")]
#[doc(inline)]
pub use bevy_retrograde_text as text;

#[cfg(feature = "physics")]
#[doc(inline)]
pub use bevy_retrograde_physics as physics;

#[cfg(feature = "ldtk")]
pub use bevy_retrograde_ldtk as ldtk;

#[cfg(feature = "ui")]
#[doc(inline)]
pub use bevy_retrograde_ui as ui;

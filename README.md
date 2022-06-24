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

Bevy Retrograde is an opinionated plugin pack for the [Bevy][__link0] game engine with tools to help you make 2D games!

Bevy Retrograde is not specific to pixel-art games, but it does include some features that would be particularly useful for pixel games. The ultimate goal is to act as an extension to Bevy that gives you common tools necessary to make a 2D game such as map loading, physics, UI, save-data, etc. Not all of the features we want to add are implemented yet, but we will be expanding the feature set as we developer our own game with it.


## License

Bevy Retrograde LDtk is licensed under the [Katharos License][__link1] which places certain restrictions on what you are allowed to use it for. Please read and understand the terms before using Bevy Retrograde for your project.


## Development Status

Bevy Retrograde is in early stages of development. The API is not stable and may change dramatically at any time.

We have just made a major update. This update removed ~75% of Bevy Retro ( that’s good! ) by updating to Bevy 0.7, and:

 - Replacing our custom renderer with Bevy’s
 - Replacing our custom map laoder with [`bevy_ecs_ldtk`][__link2]
 - Replacing our custom [RAUI][__link3] UI renderer with [`bevy_egui`][__link4]

Now Bevy Retrograde mostly includes some existing libraries and provides small utilities on top such as the 9-patch style UI addtions for egui.

Since it’s been so long since our last we want to get another release out soon, just to get everything working again on top of the latest crates. We are just wating on a [tilemap rendering fix][__link5] to get merged before we publish an `0.3.0` release.

After that we plan to re-visit what extra features we might want, such as an easier way to setup to 2D camera, and a save data system, and we will look at polishing our integrations and utilities where appropriate.

See also [Supported Bevy Version](#supported-bevy-version) below.


## Features & Examples

Check out our [examples][__link6] list to see how to use each Bevy Retrograde feature:

 - Supports web and desktop out-of-the-box
 - [LDtk][__link7] map loading and rendering using [`bevy_ecs_ldtk`][__link8].
 - An integration with the [`egui`][__link9] UI library with extra 9-patch style widgets.
 - Text rendering of bitmap fonts in the BDF format
 - Physics and collision detection powered by [Rapier][__link10] with automatic generation of convex collision shapes from sprite images.
 - Sound playing with [`bevy_kira_audio`][__link11].


## Supported Bevy Version

| bevy | bevy_retrograde |
| --- | --- |
| 0.7 | master ( `0.3` release comming soon! ) |
| 0.6 |  |
| 0.5 | 0.1, 0.2 |

**`Cargo.toml`:**


```toml
[dependencies]
bevy = { version = "0.7", default-features = false }
 # 0.3.0 Release is comming soon!
bevy_retrograde = { git = "https://github.com/katharostech/bevy_retrograde.git" }
```



 [__link0]: https://bevyengine.org
 [__link1]: https://github.com/katharostech/katharos-license
 [__link10]: https://rapier.rs/
 [__link11]: https://github.com/NiklasEi/bevy_kira_audio
 [__link2]: https://github.com/Trouv/bevy_ecs_ldtk
 [__link3]: https://raui-labs.github.io/raui/
 [__link4]: https://github.com/mvlabat/bevy_egui
 [__link5]: https://github.com/StarArawn/bevy_ecs_tilemap/pull/197
 [__link6]: https://github.com/katharostech/bevy_retrograde/tree/master/examples#bevy-retro-examples
 [__link7]: https://ldtk.io
 [__link8]: https://github.com/Trouv/bevy_ecs_ldtk
 [__link9]: https://github.com/emilk/egui


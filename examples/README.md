# Bevy Retro Examples

## [hello_world]

A good intro into Bevy Retro that also shows how to use the hierarchy system.

![hello_world](./screenshots/hello_world.gif)

[hello_world]: ./hello_world.rs

## [spritesheet]

An example of how to use animated sprite sheets.

![spritesheet](./screenshots/spritesheet.gif)

[spritesheet]: ./spritesheet.rs

## [collisions]

An example demonstrating how to detect pixel-perfect collisions between sprites

![collisions](./screenshots/collisions.gif)

[collisions]: ./collisions.rs

## [text]

An example showing how to render text using BDF font files.

> **Note:** This example shows how to render text, _without_ using the UI system, by creating text entities. This doesn't allow you to do any sort of layout other than positioning the text in the scene like you would any sprite. See the UI example below to see how to use the UI system to render text.

![text](./screenshots/text.png)

[text]: ./text.rs

## [ldtk_map]

An example showing you how to load and display an LDtk map file.

![ldtk map](./screenshots/ldtk_map.png)

[ldtk_map]: ./ldtk_map.rs

## [ui]

An example demonstrating the [RAUI] UI integration. It shows how to create UI elements that can resize with the screen and how to create theme-able buttons and interact with the ECS world from the UI.

[RAUI]: https://raui-labs.github.io/raui/

![ui](./screenshots/ui.gif)

[ui]: ./ui.rs

## [audio]

An example demonstrating how to play sounds and play music on loop.

[audio]: ./audio.rs

## [post_processing]

An example demonstrating how to add post-processing, using either the built-in CRT or your own custom shaders.

![post_processing](./screenshots/post_processing.png)

[post_processing]: ./post_processing.rs

## [custom_rendering]

An advanced example that shows how to do fully custom rendering of your own objects. This utilizes
raw calls to the [Luminance] graphics API allowing you to render _any_ kind of object, even 3D if you wanted to.

![custom_rendering](./screenshots/custom_rendering.gif)

[Luminance]: https://github.com/phaazon/luminance-rs
[custom_rendering]: ./custom_rendering.rs

## [radishmark]

A bunnymark style benchmark that also demonstrates how to use the UI system to render frames-per-second diagnostics.

![radishmark](./screenshots/radishmark.gif)

[radishmark]: ./radishmark.rs

## TODO

Examples that we haven't made that we might make later:

- Character controller

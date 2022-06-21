# Bevy Retrograde Examples

## Running Examples

We use the [just] for automating our development tasks and the project `justfile` includes tasks for
running the examples for web or native:

```bash
# Run native example
just run-example audio # or any other example name

# Run web example
just run-example-web collisions

# If you are running a native example you can also just use cargo
# ( this will not work for web examples )
cargo run --example ui
```

When running the web examples it will try to use [`basic-http-server`] to host the example on port
<http://localhost:4000>. You can install [`basic-http-server`] or you can modify the justfile to use
whatever your favorite development http server is.

[just]: https://github.com/casey/just
[`basic-http-server`]: https://github.com/brson/basic-http-server

## Full List

### [hello_world]

A good intro into Bevy Retrograde that also shows how to use the hierarchy system.

![hello_world](./screenshots/hello_world.gif)

[hello_world]: ./hello_world.rs

### [spritesheet]

An example of how to use animated sprite sheets.

![spritesheet](./screenshots/spritesheet.gif)

[spritesheet]: ./spritesheet.rs

### [physics_character]

An example demonstrating how to use the physics system to create collision boxes from sprites and
how to do simple character movement.

![physics_character](./screenshots/physics_character.gif)

[physics_character]: ./physics_character.rs

### [ldtk_map]

An example showing you how to load and display an LDtk map file.

![ldtk map](./screenshots/ldtk_map.png)

[ldtk_map]: ./ldtk_map.rs

### [physics_map]

An example demonstrating how to create collision shapes for an LDtk map.

![physics_map](./screenshots/physics_map.gif)

[physics_map]: ./physics_map.rs

### [ui]

An example demonstrating the [Egui] UI integration, and the Bevy Retrograde UI components for 9-patch style UI's.

[raui]: https://github.com/emilk/egui

![ui](./screenshots/ui.gif)

[ui]: ./ui.rs

### [audio]

An example demonstrating how to play sounds and play music on loop.

[audio]: ./audio.rs

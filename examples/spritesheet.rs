use bevy::prelude::*;
use bevy_retro::*;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retro Demo".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .add_startup_system(setup.system())
        .add_system(animate_sprite.system())
        .run();
}

/// Just helps us keep track of which frame we're on for our sprite
struct SpriteAnimFrame(usize);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load our sprite images
    let doggo_image = asset_server.load("doggo.gitignore.png");

    commands
        // Spawn sprite
        .spawn(SpriteBundle {
            image: doggo_image,
            ..Default::default()
        })
        .with(SpriteSheet {
            grid_size: 16,
            tile_index: 0,
        })
        .with(Timer::from_seconds(0.12, true))
        .with(SpriteAnimFrame(0))
        // Spawn camera
        .spawn(CameraBundle {
            camera: Camera {
                size: CameraSize::FixedHeight(40),
                background_color: Color::new(0.2, 0.2, 0.2, 1.0),
                ..Default::default()
            },
            ..Default::default()
        });
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&mut Timer, &mut SpriteSheet, &mut SpriteAnimFrame), With<Handle<Image>>>,
) {
    let frames = [136u32, 137, 138, 139];

    for (mut timer, mut spritesheet, mut frame) in query.iter_mut() {
        timer.tick(time.delta());

        if timer.finished() {
            frame.0 = frame.0.wrapping_add(1);
            spritesheet.tile_index = *frames.iter().cycle().nth(frame.0).unwrap();
        }
    }
}

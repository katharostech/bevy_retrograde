use bevy::{core::FixedTimestep, prelude::*};
use bevy_retro::*;

#[derive(StageLabel, Debug, Eq, Hash, PartialEq, Clone)]
struct GameStage;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retro Demo".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .add_startup_system(setup.system())
        .add_stage(
            GameStage,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(0.015))
                .with_system(animate_sprite.system()),
        )
        .run();
}

/// This component helps us keep track of which frame we're on for our sprite
struct SpriteAnimFrame(usize);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn sprite
    commands
        .spawn()
        .insert_bundle(
            asset_server.load_bundle::<SpriteSheetBundle>("redRadish.bundle.yml"),
        )
        .insert(Timer::from_seconds(0.12, true))
        .insert(SpriteAnimFrame(0));

    // Spawn camera
    commands.spawn().insert_bundle(CameraBundle {
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
    mut query: Query<(&mut Timer, &Handle<SpriteSheet>, &mut SpriteAnimFrame), With<Handle<Image>>>,
    mut sprite_sheet_assets: ResMut<Assets<SpriteSheet>>,
) {
    let frames = [136u32, 137, 138, 139];

    for (mut timer, sprite_sheet_handle, mut frame) in query.iter_mut() {
        timer.tick(time.delta());

        if timer.finished() {
            if let Some(sprite_sheet) = sprite_sheet_assets.get_mut(sprite_sheet_handle) {
                frame.0 = frame.0.wrapping_add(1);
                sprite_sheet.tile_index = frames[frame.0 % frames.len()];
            }
        }
    }
}

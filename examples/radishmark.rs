use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_retro::prelude::*;
use rand::{thread_rng, Rng};

struct RadishCounter {
    pub count: u128,
}

struct Radish {
    velocity: IVec2,
}

struct RadishImage([Handle<Image>; 3]);

impl FromWorld for RadishImage {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        RadishImage([
            asset_server.load("redRadish.png"),
            asset_server.load("yellowRadish.png"),
            asset_server.load("blueRadish.png"),
        ])
    }
}

const GAME_WIDTH: i32 = 300;
const GAME_HEIGHT: i32 = 300;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retro RadishMark".to_string(),
            width: 600.,
            height: 600.,
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .insert_resource(RadishCounter { count: 0 })
        .init_resource::<RadishImage>()
        .add_startup_system(setup.system())
        .add_system(mouse_handler.system())
        .add_system(movement_system.system())
        .add_system(collision_system.system())
        .add_system(counter_system.system())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn().insert_bundle(CameraBundle {
        camera: Camera {
            size: CameraSize::LetterBoxed {
                width: GAME_WIDTH as u32,
                height: GAME_HEIGHT as u32,
            },
            ..Default::default()
        },
        ..Default::default()
    });
}

fn mouse_handler(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    radish: Res<RadishImage>,
    mut counter: ResMut<RadishCounter>,
) {
    let mut rng = thread_rng();
    if mouse_button_input.pressed(MouseButton::Left) {
        for count in 0..10 {
            counter.count += 1;
            let bird_z = (counter.count + count) as i32 % 1024;
            commands
                .spawn()
                .insert_bundle(SpriteBundle {
                    image: radish.0[rng.gen_range(0..3)].clone(),
                    position: Position::new(
                        rng.gen_range(0..=10),
                        rng.gen_range(0..=10),
                        bird_z as i32,
                    ),
                    ..Default::default()
                })
                .insert(Radish {
                    velocity: IVec2::new(rng.gen_range(1..=2), rng.gen_range(1..=2)),
                });
        }
    }
}

fn movement_system(mut radishes: Query<(&Radish, &mut Position)>) {
    for (radish, mut transform) in radishes.iter_mut() {
        transform.x += radish.velocity.x;
        transform.y += radish.velocity.y;
    }
}

fn collision_system(mut bird_query: Query<(&mut Radish, &Position)>) {
    for (mut radish, pos) in bird_query.iter_mut() {
        if pos.x.abs() > GAME_WIDTH / 2 {
            radish.velocity.x *= -1;
        }
        if pos.y.abs() > GAME_HEIGHT / 2 {
            radish.velocity.y *= -1;
        }
    }
}

fn counter_system(mut idx: Local<u32>, counter: Res<RadishCounter>) {
    *idx = idx.wrapping_add(1);

    if *idx % 100 == 0 {
        info!("radish_count               : {}", counter.count);
    }
}

use std::time::Duration;

use bevy::prelude::*;
use bevy_retro::*;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retro Demo".into(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(RetroPlugin)
        .add_startup_system(setup.system())
        .add_system(move_sensei.system())
        .run();
}

// Marker component for the sensei
struct Sensei;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scene_graph: ResMut<SceneGraph>,
) {
    // Spawn the camera
    commands.spawn(CameraBundle {
        camera: Camera {
            size: CameraSize::FixedHeight(100),
            ..Default::default()
        },
        ..Default::default()
    });

    // Load our sprite images
    let sensei_image = asset_server.load("sensei2.gitignore.png");
    let guy_image = asset_server.load("guy.gitignore.png");
    let barrel_image: Handle<Image> = asset_server.load("barrel.gitignore.png");

    // Create sensei entity
    let sensei = commands.spawn(()).current_entity().unwrap();
    // Add the sensei entity to the scene graph
    let sensei_node = scene_graph.add_node(sensei);
    // And add the sprite components
    commands
        .with_bundle(SpriteBundle {
            image: sensei_image,
            scene_node: sensei_node,
            position: Position::new(0, 0, 2),
            ..Default::default()
        })
        // Add our sensei marker component
        .with(Sensei);

    // Create the guy ( student ) entity
    let guy = commands.spawn(()).current_entity().unwrap();
    // Add the guy entity to the scene graph
    let guy_node = scene_graph.add_node(guy);

    // The guy is a great student and follows the sensei everywhere, i.e, add the guy node as a
    // child of the sensei node
    scene_graph.add_child(sensei_node, guy_node);

    // And add the sprite components to the guy
    commands.with_bundle(SpriteBundle {
        image: guy_image,
        scene_node: guy_node,
        // The guy follows a little behind the sensei
        position: Position::new(-20, 4, -1),
        ..Default::default()
    });

    // Create barrel entity
    let barrel = commands.spawn(()).current_entity().unwrap();
    // Add the barrel entity to the scene graph
    let barrel_node = scene_graph.add_node(barrel);

    // Make the sensei a child of the barrel: no allegorical wisdom, just go with it
    scene_graph.add_child(barrel_node, sensei_node);

    // And add the sprite components
    commands
        .with_bundle(SpriteBundle {
            image: barrel_image,
            scene_node: barrel_node,
            ..Default::default()
        });
}

fn move_sensei(
    time: Res<Time>,
    mut timer: Local<Timer>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Position, With<Sensei>>,
) {
    timer.set_duration(Duration::from_millis(40));
    timer.set_repeating(true);

    timer.tick(time.delta());

    if timer.finished() {
        for mut pos in query.iter_mut() {
            const SPEED: i32 = 1;

            let mut direction = IVec3::new(0, 0, 0);

            if keyboard_input.pressed(KeyCode::A) {
                direction += IVec3::new(-SPEED, 0, 0);
            }

            if keyboard_input.pressed(KeyCode::D) {
                direction += IVec3::new(SPEED, 0, 0);
            }

            if keyboard_input.pressed(KeyCode::W) {
                direction += IVec3::new(0, -SPEED, 0);
            }

            if keyboard_input.pressed(KeyCode::S) {
                direction += IVec3::new(0, SPEED, 0);
            }

            **pos += direction;
        }
    }
}

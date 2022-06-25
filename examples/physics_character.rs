use bevy::prelude::*;
use bevy_retrograde::prelude::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde Physics Character".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins::default())
        .add_startup_system(setup)
        .add_system(move_player)
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..Default::default()
        })
        .run();
}

#[derive(Component)]
struct Player;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn the camera
    commands.spawn_bundle(RetroCameraBundle::fixed_height(150.0));

    // Load our images
    let block = asset_server.load("block.png");
    let triangle = asset_server.load("triangle.png");
    let red_radish = asset_server.load("redRadish.png");

    // Spawn a collider block that will just sit there and be an obstacle
    commands
        // First we spawn a sprite bundle like normal
        .spawn_bundle(SpriteBundle {
            texture: block.clone(),
            ..Default::default()
        })
        // Then we add a tesselated collider component. This will create a convex collision shape
        // from the provided image automatically.
        .insert(TesselatedCollider {
            // We want to use the same block we use for the visual for the collider shape
            texture: block.clone(),
            ..Default::default()
        })
        // Make it a static body
        .insert(RigidBody::Fixed);

    // Spawn a couple more blocks at different positions
    commands
        .spawn_bundle(SpriteBundle {
            texture: block.clone(),
            transform: Transform::from_xyz(200., 24., 0.),
            ..Default::default()
        })
        .insert(TesselatedCollider {
            texture: block.clone(),
            ..Default::default()
        })
        .insert(RigidBody::Fixed);

    commands
        .spawn_bundle(SpriteBundle {
            texture: block.clone(),
            transform: Transform::from_xyz(-200., 24., 0.),
            ..Default::default()
        })
        .insert(TesselatedCollider {
            texture: block.clone(),
            ..Default::default()
        })
        .insert(RigidBody::Fixed);

    // Spawn a triangle obstacle
    commands
        .spawn_bundle(SpriteBundle {
            texture: triangle.clone(),
            transform: Transform::from_xyz(-50., 60., 0.),
            ..Default::default()
        })
        .insert(RigidBody::Fixed)
        .insert(TesselatedCollider {
            texture: triangle,
            // For this obstacle we provide a custom configuration for the tesselator
            tesselator_config: TesselatedColliderConfig {
                // This vertice separation value sets the closest that any two tesselated vertices
                // are allowed to be to each-other. In other words, the higher this value, the less
                // acurate your collision box will be, but less compute expensive the collisions
                // will be.
                //
                // By setting the separation to 0., the collision shape should be as close as
                // possible to the actual pixel shape.
                //
                // The default value is 10.
                vertice_separation: 30.,
                ..Default::default()
            },
            ..Default::default()
        });

    // Spawn the player
    commands
        .spawn_bundle(SpriteBundle {
            texture: red_radish.clone(),
            transform: Transform::from_xyz(0., 50., 0.),
            ..Default::default()
        })
        .insert(TesselatedCollider {
            texture: red_radish.clone(),
            tesselator_config: TesselatedColliderConfig {
                // We want the collision shape for the player to be highly accurate
                vertice_separation: 0.,
                ..Default::default()
            },
            ..Default::default()
        })
        // The player is also a dynamic body with rotations locked
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED)
        // Set the player speed to 0 initially
        .insert(Velocity::linear(Vec2::default()))
        .insert(Player);
}

/// Set's the player speed based on input from the keyboard arrow keys
fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
    time: Res<Time>,
) {
    for mut velocity in query.iter_mut() {
        let speed: f32 = 10.0 * time.delta().as_millis() as f32;

        let mut direction = Vec2::new(0., 0.);

        if keyboard_input.pressed(KeyCode::Left) {
            direction += Vec2::new(-1.0, 0.);
        }

        if keyboard_input.pressed(KeyCode::Right) {
            direction += Vec2::new(1.0, 0.);
        }

        if keyboard_input.pressed(KeyCode::Up) {
            direction += Vec2::new(0., 1.0);
        }

        if keyboard_input.pressed(KeyCode::Down) {
            direction += Vec2::new(0., -1.0);
        }

        if direction.length() != 0.0 {
            direction = direction.normalize() * speed;
        }

        *velocity = Velocity::linear(direction);
    }
}

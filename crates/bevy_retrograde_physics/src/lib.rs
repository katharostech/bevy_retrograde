//! Bevy Retrograde physics plugin
//!
//! This is a re-export of [`bevy_rapier2d`] with some of our own utilities added.

use bevy::prelude::*;
use bevy::render::texture::Image;
use density_mesh_core::prelude::GenerateDensityMeshSettings;
use density_mesh_core::prelude::PointsSeparation;

pub use bevy_rapier2d;
use bevy_rapier2d::prelude::*;

#[doc(hidden)]
pub mod prelude {
    pub use crate::{
        CollisionEventExt, RetroPhysicsPlugin, TesselatedCollider, TesselatedColliderConfig,
    };
    pub use bevy_rapier2d::prelude::*;
}

/// Physics plugin for Bevy Retrograde
pub struct RetroPhysicsPlugin {
    /// Used to calculate the physics scale.
    pub pixels_per_meter: f32,
}

impl Default for RetroPhysicsPlugin {
    fn default() -> Self {
        Self {
            pixels_per_meter: 8.0,
        }
    }
}

impl Plugin for RetroPhysicsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<RapierPhysicsPlugin<NoUserData>>() {
            app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
        }

        #[cfg(feature = "debug")]
        app.add_plugin(RapierDebugRenderPlugin::default());

        app.add_systems(PostUpdate, generate_colliders);
    }
}

/// Helper methods on [`bevy_rapier2d::CollisionEvent`]
pub trait CollisionEventExt {
    fn entities(&self) -> (Entity, Entity);
    fn is_started(&self) -> bool;
    fn is_stopped(&self) -> bool;
}

impl CollisionEventExt for CollisionEvent {
    /// Get the entities involved in the collision
    fn entities(&self) -> (Entity, Entity) {
        match self {
            CollisionEvent::Started(ent1, ent2, _) | CollisionEvent::Stopped(ent1, ent2, _) => {
                (*ent1, *ent2)
            }
        }
    }

    /// Whether or not the contact has just started
    fn is_started(&self) -> bool {
        match self {
            CollisionEvent::Started(_, _, _) => true,
            CollisionEvent::Stopped(_, _, _) => false,
        }
    }

    /// Whether or not the contact has just stopped
    fn is_stopped(&self) -> bool {
        !self.is_started()
    }
}

/// Create a convex hull [`CollisionShape`] from a sprite image based on it's alpha channel
///
/// Returns [`None`] if a mesh for the given image could not be generated
pub fn create_convex_collider_from_image(
    image: DynamicImage,
    tesselator_config: &TesselatedColliderConfig,
) -> Option<Collider> {
    use density_mesh_core::prelude::DensityMeshGenerator;
    use density_mesh_image::settings::GenerateDensityImageSettings;
    let width = image.width();
    let height = image.height();
    let density_map = density_mesh_image::generate_densitymap_from_image(
        image,
        &GenerateDensityImageSettings {
            density_source: density_mesh_image::settings::ImageDensitySource::Alpha,
            scale: 1,
        },
    )
    .ok()?;

    let mut density_mesh_generator = DensityMeshGenerator::new(
        vec![],
        density_map,
        GenerateDensityMeshSettings {
            extrude_size: if tesselator_config.extrusion != 0.0 {
                Some(tesselator_config.extrusion)
            } else {
                None
            },
            points_separation: PointsSeparation::Constant(tesselator_config.vertice_separation),
            ..Default::default()
        },
    );

    density_mesh_generator.process_wait().ok()?;

    let density_mesh = density_mesh_generator.into_mesh()?;

    let points = density_mesh
        .points
        .iter()
        .map(|point| {
            Vec2::new(
                (point.x - width as f32 / 2.0) + 0.5,
                -(point.y - height as f32 / 2.0) - 0.5,
            )
        })
        .collect::<Vec<_>>();

    if tesselator_config.vertice_radius == 0.0 {
        Collider::convex_hull(&points)
    } else {
        Collider::round_convex_hull(&points, tesselator_config.vertice_radius)
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct TesselatedColliderHasLoaded;

use image::DynamicImage;
use image::GenericImageView;
use image::ImageBuffer;

/// Sprite collision tesselator config
#[derive(Debug, Clone)]
pub struct TesselatedColliderConfig {
    /// The minimum separation between generated vertices. This is, in effect, controls the
    /// "resolution" of the mesh, with a value of 0 meaning that vertices may be placed on each
    /// individual pixel, producing the maximum accuracy convex collision shape.
    ///
    /// **Default:** `0.0`
    pub vertice_separation: f32,
    /// The distance to extrude the generated mesh. Adding an extrusion can prevent panics from
    /// being caused when you try to tesselate a collision shape that is only one pixel high.
    ///
    /// When a collision shape is only one pixel high, only two vertices will be created, which is a
    /// mesh with no interior and therefore no convex "shape". This causes panics when such a shape
    /// comes in contact with another one.
    ///
    /// Adding a small extrusion will make sure that even a 2 vertice mesh will get extruded to a 4
    /// vertice mesh that has an interior and will collide properly.
    ///
    /// **Default:** `0.1`
    pub extrusion: f32,
    /// Vertices will be generated in the center of pixels. This means that, without any vertice
    /// radius and no extrusion, the sprite collision will overlap with objects in comes in contact
    /// with by half a pixel. By setting the vertice radius to 0.5, an extra half-pixel buffer will
    /// be added making the collision appear as expected. This can be tweaked in combination with
    /// the extrusion value to control the buffer around the generated sprite mesh.
    ///
    /// **Default:** `0.4`
    pub vertice_radius: f32,
}

impl Default for TesselatedColliderConfig {
    fn default() -> Self {
        Self {
            vertice_separation: 10.,
            extrusion: 0.1,
            vertice_radius: 0.4,
        }
    }
}

/// A component used to automatically add a [`CollisionShape`] to an entity that is generated
/// automatically by tesselating [`Image`] collision shape based on it's alpha channel
#[derive(Default, Component)]
pub struct TesselatedCollider {
    pub texture: Handle<Image>,
    pub tesselator_config: TesselatedColliderConfig,
}

fn generate_colliders(
    mut commands: Commands,
    pending_colliders: Query<(Entity, &TesselatedCollider), Without<TesselatedColliderHasLoaded>>,
    image_assets: Res<Assets<Image>>,
) {
    // TODO: Hot reload collision shape changes
    for (ent, tesselated_collider) in pending_colliders.iter() {
        // Get the collider image
        let image = if let Some(image) = image_assets.get(&tesselated_collider.texture) {
            image
        } else {
            continue;
        };

        let shape = create_convex_collider_from_image(
            DynamicImage::ImageRgba8(
                ImageBuffer::from_vec(
                    image.texture_descriptor.size.width,
                    image.texture_descriptor.size.height,
                    image.data.clone(),
                )
                .unwrap(),
            ),
            &tesselated_collider.tesselator_config,
        )
        .expect("Could not generate collision shape from image");

        commands
            .entity(ent)
            .insert(shape)
            .insert(TesselatedColliderHasLoaded);
    }
}

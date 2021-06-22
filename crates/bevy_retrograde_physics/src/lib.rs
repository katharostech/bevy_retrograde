//! The Bevy Retrograde text rendering plugin

use bevy::{ecs::component::ComponentDescriptor, prelude::*};
use bevy_retrograde_core::prelude::Image;
pub use heron::*;

#[doc(hidden)]
pub mod prelude {
    pub use crate::RetroPhysicsPlugin;
}

/// Physics plugin for Bevy Retrograde
pub struct RetroPhysicsPlugin;

impl Plugin for RetroPhysicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(PhysicsPlugin::default())
            .register_component(ComponentDescriptor::new::<TesselatedColliderHasLoaded>(
                bevy::ecs::component::StorageType::SparseSet,
            ))
            .add_system_to_stage(CoreStage::PostUpdate, generate_colliders.system());
    }
}

/// Create a convex hull [`ColliderShape`] from a sprite image based on it's alpha channel
pub fn create_convex_collider(
    image: &Image,
    tesselator_config: &TesselatedColliderConfig,
) -> CollisionShape {
    use density_mesh_core::prelude::DensityMeshGenerator;
    use density_mesh_image::settings::GenerateDensityImageSettings;
    use image::DynamicImage;

    let e = "Could not tesselate image to produce collider shape";

    let density_map = density_mesh_image::generate_densitymap_from_image(
        DynamicImage::ImageRgba8(image.0.clone()),
        &GenerateDensityImageSettings {
            density_source: density_mesh_image::settings::ImageDensitySource::Alpha,
            scale: 1,
        },
    )
    .expect(e);

    let mut density_mesh_generator =
        DensityMeshGenerator::new(vec![], density_map, tesselator_config.clone());

    density_mesh_generator.process_wait().expect(e);

    let density_mesh = density_mesh_generator.into_mesh().expect(e);

    let points = density_mesh
        .points
        .iter()
        .map(|point| {
            Vec3::new(
                (point.x - image.width() as f32 / 2.0) + 0.5,
                (point.y - image.height() as f32 / 2.0) + 0.5,
                0.,
            )
        })
        .collect::<Vec<_>>();

    CollisionShape::ConvexHull {
        points,
        border_radius: Some(0.5),
    }
}

struct TesselatedColliderHasLoaded;

pub use density_mesh_core::mesh::points_separation::PointsSeparation;
pub use density_mesh_core::mesh::settings::GenerateDensityMeshSettings as TesselatedColliderConfig;

/// A component used to automatically add a [`ColliderBundle`] to an entity that is generated
/// automatically by tesselating [`Image`] collision shape based on it's alpha channel
#[derive(Default)]
pub struct TesselatedCollider {
    pub image: Handle<Image>,
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
        let image = if let Some(image) = image_assets.get(&tesselated_collider.image) {
            image
        } else {
            continue;
        };

        let shape = create_convex_collider(&image, &tesselated_collider.tesselator_config);

        commands
            .entity(ent)
            .insert(shape)
            .insert(TesselatedColliderHasLoaded);
    }
}

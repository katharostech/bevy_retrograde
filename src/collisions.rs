use bevy::{ecs::system::SystemParam, prelude::*};
use fixedbitset::FixedBitSet;

use crate::*;

#[derive(SystemParam)]
pub struct PixelCollisions<'a> {
    ignore_update_warnings: Local<'a, bool>,
    has_synced_transforms: Local<'a, bool>,
    /// The queries that we use for detecting pixel collisions
    ///
    /// There are two queries we need, which conflict with each-other and are therefore in a query set:
    ///
    /// - the first is the query needed for doing the world position propagaiton
    /// - the second is the query that is used to get the world positions when checking for collisions
    queries: QuerySet<(
        Query<'a, (Entity, &'static mut Position, &'static mut WorldPosition)>,
        Query<'a, (Option<&'static Sprite>, &'static WorldPosition)>,
    )>,
    scene_graph: ResMut<'a, SceneGraph>,
}

impl<'a> PixelCollisions<'a> {
    pub fn sync_positions(&mut self) {
        propagate_world_positions(&mut self.scene_graph, &mut self.queries.q0_mut());
        *self.has_synced_transforms = true;
    }

    pub fn ignore_warnings(&mut self) {
        *self.ignore_update_warnings = true;
    }

    pub fn collides_with(
        &self,
        ent1: Entity,
        col_image1: &Image,
        ent2: Entity,
        col_image2: &Image,
    ) -> bool {
        // Warn about potentiallly un-synced positions
        if !*self.has_synced_transforms && !*self.ignore_update_warnings {
            warn!(
                "Running `PixelCollisions::collides_with` without first syncing the \
                positions with `PixelCollisions::sync_positions` may lead to inacurate results. \
                You can silence this warning by first running `PixelCollisions::ignore_warnings`."
            );
        }

        // Get image infos from world
        let image1 = if let Ok((sprite, pos)) = self.queries.q1().get(ent1) {
            ImageData {
                image: col_image1,
                sprite: sprite.cloned().unwrap_or_default(),
                pos: pos,
            }
        } else {
            return false;
        };
        let image2 = if let Ok((sprite, pos)) = self.queries.q1().get(ent2) {
            ImageData {
                image: col_image2,
                sprite: sprite.cloned().unwrap_or_default(),
                pos: pos,
            }
        } else {
            return false;
        };

        let image1_bounds = image1.get_image_bounds();
        let image2_bounds = image2.get_image_bounds();

        // Check whether or not bounding boxes collide
        if image1_bounds.collides_with(image2_bounds) {
            // Get the bounds of both images
            let bounds = ImageBounds::union(image1_bounds, image2_bounds);
            let bounds_pixel_count = bounds.width * bounds.height;

            // Create a bitset that represents the ocupation of each pixel within the bounds of
            // their collision.
            let mut bitset = FixedBitSet::with_capacity(bounds_pixel_count as usize);

            // Fill the bitset with the pixels from image 1
            let x_offset = image1_bounds.min.x - bounds.min.x;
            let y_offset = image1_bounds.min.y - bounds.min.y;
            for x in 0..image1.image.rgba_image.dimensions().0 as i32 {
                let translated_x = x + x_offset;
                for y in 0..image1.image.rgba_image.dimensions().1 as i32 {
                    let translated_y = y + y_offset;

                    let image_index = y * image1_bounds.width + x;
                    let translated_index = translated_y * bounds.width + translated_x;

                    bitset.set(
                        translated_index as usize,
                        image1.image.collision.contains(image_index as usize),
                    );
                }
            }

            // Loop through all the pixels in image 2 and see if any of them are also in image 1
            let x_offset = image2_bounds.min.x - bounds.min.x;
            let y_offset = image2_bounds.min.y - bounds.min.y;
            for x in 0..image1.image.rgba_image.dimensions().0 as i32 {
                let translated_x = x + x_offset;
                for y in 0..image1.image.rgba_image.dimensions().1 as i32 {
                    let translated_y = y + y_offset;

                    let image_index = y * image1_bounds.width + x;
                    let translated_index = translated_y * bounds.width + translated_x;

                    if image1.image.collision.contains(image_index as usize)
                        && bitset.contains(translated_index as usize)
                    {
                        return true;
                    }
                }
            }

            // If none of those pixels collided, return false
            false
        } else {
            false
        }
    }
}

struct ImageData<'a> {
    image: &'a Image,
    sprite: Sprite,
    pos: &'a WorldPosition,
}

impl<'a> ImageData<'a> {
    fn get_image_bounds(&self) -> ImageBounds {
        let (image_width, image_height) = self.image.rgba_image.dimensions();
        let (image_width, image_height) = (image_width as i32, image_height as i32);
        let min = if self.sprite.centered {
            IVec2::new(
                self.pos.x - image_width as i32 / 2,
                self.pos.y - image_height as i32 / 2,
            )
        } else {
            IVec2::new(self.pos.x, self.pos.y)
        };
        let max = if self.sprite.centered {
            IVec2::new(
                self.pos.x + image_width as i32 / 2 + image_height % 2,
                self.pos.y + image_height as i32 / 2 + image_height % 2,
            )
        } else {
            IVec2::new(self.pos.x + image_width, self.pos.y + image_height)
        };

        ImageBounds::new(min + self.sprite.offset, max + self.sprite.offset)
    }
}

#[derive(Debug, Clone, Copy)]
struct ImageBounds {
    min: IVec2,
    max: IVec2,
    width: i32,
    height: i32,
}

impl ImageBounds {
    fn new(min: IVec2, max: IVec2) -> Self {
        Self {
            min,
            max,
            width: max.x - min.x,
            height: max.y - min.y,
        }
    }

    // Create an image bounds that spans both image bounds
    fn union(a: Self, b: Self) -> Self {
        let min_x = if a.min.x < b.min.x { a.min.x } else { b.min.x };
        let min_y = if a.min.y < b.min.y { a.min.y } else { b.min.y };
        let max_x = if a.max.x > b.max.x { a.max.x } else { b.max.x };
        let max_y = if a.max.y > b.max.y { a.max.y } else { b.max.y };

        Self::new(IVec2::new(min_x, min_y), IVec2::new(max_x, max_y))
    }

    fn collides_with<'a>(self, other: ImageBounds) -> bool {
        self.min.x < other.max.x
            && self.min.y < other.max.y
            && self.max.x > other.min.x
            && self.max.y > other.min.y
    }
}

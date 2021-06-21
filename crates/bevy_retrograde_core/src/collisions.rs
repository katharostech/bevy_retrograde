//! Collision detection utilities

use euclid::default::{Box2D, Point2D, Vector2D};

use crate::prelude::*;
use bevy::prelude::*;

/// Information needed to detect pixel collisions using [`pixels_collide_with_pixels`]
#[derive(Clone, Copy)]
pub struct PixelColliderInfo<'a> {
    pub image: &'a Image,
    pub world_position: &'a Vec3,
    pub sprite: &'a Sprite,
    pub sprite_sheet: Option<&'a SpriteSheet>,
}

impl<'a> PixelColliderInfo<'a> {
    fn _get_bounds(&self) -> Box2D<f32> {
        let (image_width, image_height) = if let Some(sheet) = self.sprite_sheet {
            (sheet.grid_size.x, sheet.grid_size.y)
        } else {
            self.image.dimensions()
        };
        let (image_width, image_height) = (image_width as f32, image_height as f32);
        let min = Point2D::new(self.world_position.x, self.world_position.y);
        let max = Point2D::new(
            self.world_position.x + image_width,
            self.world_position.y + image_height,
        );

        let bounds = Box2D::new(min, max);

        let bounds = if self.sprite.centered {
            bounds.translate(Vector2D::new(-image_width / 2., -image_height / 2.))
        } else {
            bounds
        };

        bounds.translate(Vector2D::new(self.sprite.offset.x, self.sprite.offset.y))
    }
}

/// Get whether or not the pixels in `a` collide with the pixels in `b`
#[allow(clippy::many_single_char_names)]
pub fn pixels_collide_with_pixels(_a: PixelColliderInfo, _b: PixelColliderInfo) -> bool {
    bevy::log::warn!(
        "`pixels_collide_with_pixels` is being re-implemented and will \
        always return `false`."
    );
    false
}

/// A bounding box, used to detect collitions with [`pixels_collide_with_bounding_box`]
pub struct BoundingBox {
    pub min: IVec2,
    pub max: IVec2,
}

impl From<BoundingBox> for Box2D<i32> {
    fn from(bounding_box: BoundingBox) -> Self {
        Box2D::new(
            Point2D::new(bounding_box.min.x, bounding_box.min.y),
            Point2D::new(bounding_box.max.x, bounding_box.max.y),
        )
    }
}

/// Get whether or not the pixels in `a` collide with the bounding box `b`
pub fn pixels_collide_with_bounding_box(_a: PixelColliderInfo, _b: BoundingBox) -> bool {
    bevy::log::warn!(
        "`pixels_collide_bounding_box` is being re-implemented and will \
        always return `false`."
    );
    false
}

use euclid::default::{Box2D, Point2D, Vector2D};
use image::GenericImageView;

use crate::*;

/// Information needed to detect pixel collisions using [`pixels_collid_with`].
#[derive(Clone, Copy)]
pub struct PixelColliderInfo<'a> {
    pub image: &'a Image,
    pub world_position: &'a IVec3,
    pub sprite: &'a Sprite,
    pub sprite_sheet: Option<&'a SpriteSheet>,
}

impl<'a> PixelColliderInfo<'a> {
    fn get_bounds(&self) -> Box2D<i32> {
        let (image_width, image_height) = if let Some(sheet) = self.sprite_sheet {
            (sheet.grid_size.x, sheet.grid_size.y)
        } else {
            self.image.dimensions()
        };
        let (image_width, image_height) = (image_width as i32, image_height as i32);
        let min = Point2D::new(self.world_position.x, self.world_position.y);
        let max = Point2D::new(
            self.world_position.x + image_width,
            self.world_position.y + image_height,
        );

        let bounds = Box2D::new(min, max);

        let bounds = if self.sprite.centered {
            bounds.translate(Vector2D::new(-image_width / 2, -image_height / 2))
        } else {
            bounds
        };

        bounds.translate(Vector2D::new(self.sprite.offset.x, self.sprite.offset.y))
    }
}

/// Get whether or not the pixels in `a` collide with the pixels in `b`
pub fn pixels_collide_with_pixels(a: PixelColliderInfo, b: PixelColliderInfo) -> bool {
    let a_bounds = a.get_bounds();
    let b_bounds = b.get_bounds();
    let a_offset = if let Some(sheet) = a.sprite_sheet {
        let (width, _) = a.image.dimensions();
        let width_tiles = width / sheet.grid_size.x;
        let y = sheet.tile_index / width_tiles * sheet.grid_size.y;
        let x = (sheet.tile_index % width_tiles) * sheet.grid_size.x;

        IVec2::new(x as i32, y as i32)
    } else {
        IVec2::ZERO
    };

    let b_offset = if let Some(sheet) = b.sprite_sheet {
        let (w, _) = a.image.dimensions();
        let y = sheet.tile_index / w * sheet.grid_size.y;
        let x = sheet.tile_index % w * sheet.grid_size.x;

        IVec2::new(x as i32, y as i32)
    } else {
        IVec2::ZERO
    };

    // Check whether or not bounding boxes collide
    if a_bounds.intersects(&b_bounds) {
        // Get the bounds of the sprites' intersection
        let intersection = a_bounds.intersection_unchecked(&b_bounds);
        let (width, height) = (intersection.width(), intersection.height());

        // Create a view of the image intersection for `a`
        let a_view = a.image.view(
            (intersection.min.x - a_bounds.min.x + a_offset.x) as u32,
            (intersection.min.y - a_bounds.min.y + a_offset.y) as u32,
            width as u32,
            height as u32,
        );
        // Create a view of the image intersection for `b`
        let b_view = b.image.view(
            (intersection.min.x - b_bounds.min.x + b_offset.x) as u32,
            (intersection.min.y - b_bounds.min.y + b_offset.y) as u32,
            width as u32,
            height as u32,
        );

        // Zip the pixels of both images and loop through them
        for ((_, _, a_pix), (_, _, b_pix)) in a_view.pixels().zip(b_view.pixels()) {
            // If both pixels are non-transparent, return true
            if a_pix[3] > 0 && b_pix[3] > 0 {
                return true;
            }
        }

        // If none of those pixels collided, return false
        false
    } else {
        false
    }
}

/// A bounding box defined by a min point and a max point
pub struct BoundingBox {
    pub min: IVec2,
    pub max: IVec2,
}

impl Into<Box2D<i32>> for BoundingBox {
    fn into(self) -> Box2D<i32> {
        Box2D::new(
            Point2D::new(self.min.x, self.min.y),
            Point2D::new(self.max.x, self.max.y),
        )
    }
}

/// Get whether or not the pixels in `a` collide with the pixels in `b`
pub fn pixels_collide_with_bounding_box(a: PixelColliderInfo, b: BoundingBox) -> bool {
    let a_bounds = a.get_bounds();
    let b_bounds: Box2D<i32> = b.into();
    let a_offset = if let Some(sheet) = a.sprite_sheet {
        let (width, _) = a.image.dimensions();
        let width_tiles = width / sheet.grid_size.x;
        let y = sheet.tile_index / width_tiles * sheet.grid_size.y;
        let x = (sheet.tile_index % width_tiles) * sheet.grid_size.x;

        IVec2::new(x as i32, y as i32)
    } else {
        IVec2::ZERO
    };

    // Check whether or not bounding boxes collide
    if a_bounds.intersects(&b_bounds) {
        // Get the bounds of the sprites' intersection
        let intersection = a_bounds.intersection_unchecked(&b_bounds);
        let (width, height) = (intersection.width(), intersection.height());

        // Create a view of the image intersection for `a`
        let a_view = a.image.view(
            (intersection.min.x - a_bounds.min.x + a_offset.x) as u32,
            (intersection.min.y - a_bounds.min.y + a_offset.y) as u32,
            width as u32,
            height as u32,
        );

        // Loop through every pixel in the intersection zone of a and see if any of the pixels are
        // non-transparent
        for (_, _, a_pix) in a_view.pixels() {
            if a_pix[3] > 0 {
                return true;
            }
        }

        // If none of those pixels collided, return false
        false
    } else {
        false
    }
}

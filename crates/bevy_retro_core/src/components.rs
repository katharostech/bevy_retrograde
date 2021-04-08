use bevy::{prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};

use crate::*;

mod position;
pub use position::*;

pub(crate) fn add_components(app: &mut AppBuilder) {
    app.register_type::<Camera>()
        .register_type::<Color>()
        .register_type::<CameraSize>()
        .register_type::<Position>()
        .register_type::<WorldPosition>()
        .register_type::<Sprite>()
        .register_type::<SpriteSheet>();
}

/// The retro camera bundle
#[derive(Bundle, Default, Debug, Clone)]
pub struct CameraBundle {
    /// The camera config
    pub camera: Camera,

    /// The position of the center of the camera
    ///
    /// If the width or height of the camera is an even number, the center pixel will be the pixel
    /// to the top-left of the true center.
    pub position: Position,

    /// The global world position of the sprite
    pub world_position: WorldPosition,
}

/// An 8-bit RGBA color
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Reflect)]
#[reflect_value(Serialize, Deserialize, PartialEq, Component)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 1.,
        }
    }
}

/// The camera component
#[derive(Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Camera {
    /// The size of the camera along the fixed axis, which is by default the vertical axis
    pub size: CameraSize,
    /// Whether the camera should be centered about it's position. Defaults to `true`. If `false`
    /// the top-left corner of the camrera will be at its [`Position`].
    pub centered: bool,
    /// The background color of the camera
    ///
    /// This is the color that will be scene in the viewport when there are no sprites in the game
    /// area.
    pub background_color: Color,
    /// The color of the letter box
    ///
    /// The letter box is only visible when the camera size is set to [`LetterBoxed`][CameraSize::LetterBoxed].
    pub letterbox_color: Color,
    /// The aspect ratio of the pxiels when rendered through this camera
    pub pixel_aspect_ratio: f32,
    /// Additional shader code that will be added to the camera rendering that can be used for
    /// post-processing
    ///
    /// TODO: Example
    pub custom_shader: Option<String>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            size: Default::default(),
            centered: true,
            background_color: Color::default(),
            letterbox_color: Color::default(),
            pixel_aspect_ratio: 1.0,
            custom_shader: None,
        }
    }
}

/// The size of the 2D camera
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[reflect_value(PartialEq, Serialize, Deserialize)]
pub enum CameraSize {
    /// Fix the camera height in pixels and make the the width scale to whatever the window/screen
    /// size is.
    FixedHeight(u32),
    /// Fix the camera width in pixels and make the the height scale to whatever the window/screen
    /// size is.
    FixedWidth(u32),
    /// Fix the camera width and height in pixels and fill the empty space with the camera
    /// background color.
    LetterBoxed { width: u32, height: u32 },
}

impl Default for CameraSize {
    fn default() -> Self {
        Self::FixedHeight(200)
    }
}

impl Camera {
    /// Get the size in game pixels ( retro-sized, not screen pixels ) of the camera view
    pub fn get_target_size(&self, window: &bevy::window::Window) -> UVec2 {
        let window_width = window.width();
        let window_height = window.height();
        let aspect_ratio = window_width / window_height;
        match self.size {
            CameraSize::FixedHeight(height) => UVec2::new(
                (aspect_ratio * height as f32 / self.pixel_aspect_ratio).floor() as u32,
                height,
            ),
            CameraSize::FixedWidth(width) => UVec2::new(
                width,
                (width as f32 / aspect_ratio * self.pixel_aspect_ratio).floor() as u32,
            ),
            CameraSize::LetterBoxed { width, height } => UVec2::new(width, height),
        }
    }
}

/// Sprite options
#[derive(Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Sprite {
    /// Whether or not the sprite is centered about its position
    pub centered: bool,
    /// Flip the sprite on x
    pub flip_x: bool,
    /// Flip the sprite on y
    pub flip_y: bool,
    /// A visual offset for the sprite
    pub offset: IVec2,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            centered: true,
            flip_x: false,
            flip_y: false,
            offset: IVec2::default(),
        }
    }
}

/// Settings for a sprite sheet
#[derive(Debug, Clone, TypeUuid, Reflect)]
#[uuid = "64746631-1afe-4ca6-8398-7c0df62f7813"]
#[reflect(Component)]
pub struct SpriteSheet {
    pub grid_size: UVec2,
    pub tile_index: u32,
}

impl Default for SpriteSheet {
    fn default() -> Self {
        Self {
            grid_size: UVec2::splat(16),
            tile_index: 0,
        }
    }
}

/// Indicates whether or not an object should be rendered
#[derive(Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Visible(pub bool);
impl_deref!(Visible, bool);

impl Default for Visible {
    fn default() -> Self {
        Visible(true)
    }
}

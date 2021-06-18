//! ECS components

use bevy::{prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};

mod position;
pub use position::*;

pub(crate) fn add_components(app: &mut AppBuilder) {
    app.register_type::<Camera>()
        .register_type::<Color>()
        .register_type::<CameraSize>()
        .register_type::<Position>()
        .register_type::<WorldPosition>()
        .register_type::<Sprite>()
        .register_type::<SpriteSheet>()
        .register_type::<Visible>();
}

/// A floating point RGBA color
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
    /// Whether the camera should be centered about it's position. Defaults to `true`. If set to
    /// false `false`, the top-left corner of the camera will be at its [`Position`].
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
    /// This must be a [OpenGL ES Shading Language 1.0][essl1] string.
    ///
    /// [essl1]: https://www.khronos.org/registry/OpenGL/specs/es/2.0/GLSL_ES_Specification_1.00.pdf
    ///
    /// ```ignore
    /// // Spawn the camera
    /// commands.spawn().insert_bundle(CameraBundle {
    ///     camera: Camera {
    ///         // Set our camera to have a fixed height and an auto-resized width
    ///         size: CameraSize::FixedHeight(100),
    ///         background_color: Color::new(0.2, 0.2, 0.2, 1.0),
    ///         custom_shader: Some(
    ///             CrtShader {
    ///                 // You can configure shader options here
    ///                 ..Default::default()
    ///             }
    ///             .get_shader(),
    ///         ),
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// });
    /// ```
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
                // The width must be an even number to keep the alignment with non-pixel-perfect
                // sprites working ( for some reason I have not yet fully understood )
                {
                    let x = (aspect_ratio * height as f32 / self.pixel_aspect_ratio).floor() as u32;
                    if x % 2 != 0 {
                        x - 1
                    } else {
                        x
                    }
                },
                height,
            ),
            CameraSize::FixedWidth(width) => UVec2::new(width, {
                // The width must be an even number to keep the alignment with non-pixel-perfect
                // sprites working ( for some reason I have not yet fully understood )
                let y = (width as f32 / aspect_ratio * self.pixel_aspect_ratio).floor() as u32;
                if y % 2 != 0 {
                    y - 1
                } else {
                    y
                }
            }),
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
    pub offset: Vec2,
    /// Whether or not to constrain the sprite rendering to perfect pixel alignment with the
    /// virtual, low resolution of the camera
    pub pixel_perfect: bool,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            centered: true,
            flip_x: false,
            flip_y: false,
            offset: Vec2::default(),
            pixel_perfect: true,
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
bevy_retrograde_macros::impl_deref!(Visible, bool);

impl Default for Visible {
    fn default() -> Self {
        Visible(true)
    }
}

use bevy::prelude::*;

use crate::SpriteImage;

macro_rules! impl_deref {
    ($struct:ident, $target:path) => {
        impl std::ops::Deref for $struct {
            type Target = $target;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $struct {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

/// The retro camera bundle
#[derive(Bundle, Default, Debug, Clone)]
pub struct CameraBundle {
    /// The position of the center of the camera
    ///
    /// If the width or height of the camera is an even number, the center pixel will be the pixel
    /// to the top-left of the true center.
    pub position: Position,
    /// The camera config
    pub camera: Camera,
}

/// An 8-bit RGBA color
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
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
#[derive(Debug, Clone)]
pub struct Camera {
    /// The size of the camera along the fixed axis, which is by default the vertical axis
    pub size: CameraSize,
    /// Whether or not the camera is active
    ///
    /// If multiple cameras are active at the same time a blank screen will be displayed until only
    /// one camera is active.
    pub active: bool,
    /// The background color of the camera
    ///
    /// This is only visible if the camera size is `Fixed`, in which case it is the color of the
    /// letter-box.
    pub background_color: Color,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            size: Default::default(),
            active: true,
            background_color: Color::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CameraSize {
    /// Fix the camera height in pixels and make the the width scale to whatever the window/screen
    /// size is.
    FixedHeight(u32),
    /// Fix the camera width in pixels and make the the height scale to whatever the window/screen
    /// size is.
    FixedWidth(u32),
    /// Fix the camera width and height in pixels and fill the empty space with the camera
    /// background color.
    Fixed { width: u32, height: u32 },
}

impl Default for CameraSize {
    fn default() -> Self {
        Self::FixedHeight(200)
    }
}

#[derive(Default, Debug, Clone)]
/// The position of a 2D object in the world
pub struct Position(pub IVec3);
impl_deref!(Position, IVec3);

/// A bundle containing all the components necessary to render a sprite
#[derive(Bundle, Default)]
pub struct SpriteBundle {
    /// The image data of the sprite
    pub image: Handle<SpriteImage>,
    /// The visibility of the sprite
    pub visible: Visible,
    /// The position of the center of the sprite in world space
    pub position: Position,
}

/// Indicates whether or not an object should be rendered
#[derive(Debug, Clone, Copy)]
pub struct Visible(pub bool);
impl_deref!(Visible, bool);

impl Default for Visible {
    fn default() -> Self {
        Visible(true)
    }
}

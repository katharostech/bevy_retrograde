//! Bevy Retrograde UI plugin
//!
//! Primarily a wrapper around [`bevy_egui`] with extra utilties for 9-patch sytle UI and bitmap
//! font rendering.

use bevy::{asset::AssetPath, prelude::*};

pub mod bdf;

/// UI plugin for Bevy Retrograde built on [`bevy_egui`]
pub struct RetroUiPlugin;

impl Plugin for RetroUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .add_asset::<RetroFont>()
            .add_asset_loader(RetroFontLoader::default())
            .add_system(font_texture_update);
    }
}

pub mod bordered_frame;
pub mod fonts;
pub mod retro_button;
pub mod retro_label;

#[doc(hidden)]
pub mod prelude {
    pub use crate::{
        bordered_frame::*, fonts::*, retro_button::*, retro_label::*, BorderImage, RetroEguiUiExt,
    };
    pub use bevy_egui::*;
}

use prelude::*;

/// Extra functions on top of [`egui::Ui`] for retro widgets
pub trait RetroEguiUiExt {
    fn retro_label(self, text: &str, font: &Handle<RetroFont>) -> egui::Response;
}

impl RetroEguiUiExt for &mut egui::Ui {
    fn retro_label(self, text: &str, font: &Handle<RetroFont>) -> egui::Response {
        RetroLabel::new(text, font).show(self)
    }
}

/// A 9-patch style border image that can be used, for exmaple, with [`RetroFrame`] to render a
/// bordered frame
///
/// # Example
///
/// You can easily load your border images into a resource using the [`UiBorderImage::load_from_world()`].
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_retrograde_ui::prelude::*;
/// struct UiTheme {
///     panel_bg: BorderImage,
///     button_up_bg: BorderImage,
///     button_down_bg: BorderImage,
/// }
///
/// impl FromWorld for UiTheme {
///     fn from_world(world: &mut World) -> Self {
///         Self {
///             panel_bg: BorderImage::load_from_world(
///                 world,
///                 "ui/panel.png",
///                 UVec2::new(48, 48),
///                 Rect::all(8.0),
///             ),
///             button_up_bg: BorderImage::load_from_world(
///                 world,
///                 "ui/button-up.png",
///                 UVec2::new(32, 16),
///                 Rect::all(8.0),
///             ),
///             button_down_bg: BorderImage::load_from_world(
///                 world,
///                 "ui/button-down.png",
///                 UVec2::new(32, 16),
///                 Rect::all(8.0),
///             ),
///         }
///     }
/// }
/// ```
pub struct BorderImage {
    /// This is the handle to the Bevy image, which keeps the texture from being garbage collected.
    pub handle: Handle<Image>,
    /// This is the egui texture ID for the image.
    pub egui_texture: egui::TextureId,
    /// This is the size of the frame
    pub texture_border_size: Rect<f32>,
    /// This is the size of the texture in pixels
    pub texture_size: UVec2,
}

impl BorderImage {
    /// Load a border image from the Bevy world
    pub fn load_from_world<'a, P: Into<AssetPath<'a>>>(
        world: &mut World,
        path: P,
        image_size: UVec2,
        border_size: Rect<f32>,
    ) -> Self {
        let world = world.cell();
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let mut ctx = world.get_resource_mut::<EguiContext>().unwrap();

        let handle = asset_server.load(path);

        Self {
            egui_texture: ctx.add_image(handle.clone()),
            handle,
            texture_border_size: border_size,
            texture_size: image_size,
        }
    }
}

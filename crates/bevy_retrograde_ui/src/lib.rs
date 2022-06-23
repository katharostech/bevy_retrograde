//! Bevy Retrograde UI plugin

use bevy::prelude::*;
pub use bevy_egui::*;
pub use egui_extras;

pub mod bdf;

/// UI plugin for Bevy Retrograde built on [bevy_egui]
pub struct RetroUiPlugin;

impl Plugin for RetroUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_egui::EguiPlugin)
            .add_asset::<RetroFont>()
            .add_asset_loader(RetroFontLoader::default())
            .add_system(font_texture_update);
    }
}

pub use bordered_frame::*;
pub mod bordered_frame;

pub use retro_label::*;
pub mod retro_label;

pub use retro_button::*;
pub mod retro_button;

pub use fonts::*;
pub mod fonts;

pub trait RetroEguiUiExt {
    fn retro_label(self, text: &str, font: &Handle<RetroFont>) -> egui::Response;
}

impl RetroEguiUiExt for &mut egui::Ui {
    fn retro_label(self, text: &str, font: &Handle<RetroFont>) -> egui::Response {
        RetroLabel::new(text, font).show(self)
    }
}

/// A 9-patch style border image that can be used, for exmaple, with [`RetroFrame`] to render a
/// bordered frame.
///
/// # Example
///
/// You can easily load your border images into a resource using the [`UiBorderImage::load_from_world()`].
///
/// ```
/// struct UiTheme {
///     panel_bg: UiBorderImage,
///     button_up_bg: UiBorderImage,
///     button_down_bg: UiBorderImage,
/// }
///
/// impl FromWorld for UiTheme {
///     fn from_world(world: &mut World) -> Self {
///         Self {
///             panel_bg: UiBorderImage::load_from_world(
///                 world,
///                 "ui/panel.png",
///                 UVec2::new(48, 48),
///                 Rect::all(8.0),
///             ),
///             button_up_bg: UiBorderImage::load_from_world(
///                 world,
///                 "ui/button-up.png",
///                 UVec2::new(32, 16),
///                 Rect::all(8.0),
///             ),
///             button_down_bg: UiBorderImage::load_from_world(
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

//! Bevy Retrograde [epaint] integration
//!
//! The target use-case is easy drawing of primitives such as lines, circles, text, etc. for use in
//! debug rendering and visualization.
//!
//! [epaint]: https://docs.rs/epaint

use bevy::prelude::*;

use bevy_retrograde_core::prelude::AppBuilderRenderHookExt;

mod render_hook;
pub use epaint::emath::*;
pub use epaint::*;
use render_hook::EpaintRenderHook;

/// Epaint plugin prelude
pub mod prelude {
    pub use crate::ShapeBundle;
    pub use epaint::Shape;
}

/// Text rendering plugin for Bevy Retrograde
pub struct RetroEpaintPlugin;

impl Plugin for RetroEpaintPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_render_hook::<EpaintRenderHook>().insert_resource(
            // TODO: Make pixels per pont configurable
            epaint::text::Fonts::from_definitions(1., Default::default()),
        );
    }
}

/// Bundle for rendering an [`epaint`] shape
#[derive(Bundle, Debug, Clone)]
pub struct ShapeBundle {
    pub shape: Shape,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for ShapeBundle {
    fn default() -> Self {
        Self {
            shape: Shape::Noop,
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

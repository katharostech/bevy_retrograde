use bevy::prelude::*;
use bevy_retro_core::prelude::*;

use crate::prelude::*;

/// Marker component indicating that a text entity needs to be updated but hasn't yet because it's
/// assets are not loaded.
pub(crate) struct TextNeedsUpdate;

#[derive(Bundle, Default, Debug, Clone)]
pub struct TextBundle {
    pub font: Handle<Font>,
    pub text: Text,
    pub sprite: Sprite,
    pub visible: Visible,
    pub position: Position,
    pub world_position: WorldPosition,
}

/// The text inside a text entity or text block
#[derive(Debug, Clone)]
pub struct Text {
    pub text: String,
    pub color: Color,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: String::new(),
            color: Color::new(1., 1., 1., 1.),
        }
    }
}

/// The configuration for a text block
#[derive(Debug, Clone)]
pub struct TextBlock {
    pub max_width: u32,
}

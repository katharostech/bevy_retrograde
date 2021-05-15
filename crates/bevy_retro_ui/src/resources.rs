use raui::prelude::WidgetNode;

/// This resource contains Bevy Retro's UI widget tree
#[derive(Debug, Clone, Default)]
pub struct UiTree(pub WidgetNode);

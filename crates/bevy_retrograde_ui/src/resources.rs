use raui::prelude::WidgetNode;

/// This resource contains Bevy Retrograde's UI widget tree
#[derive(Debug, Clone, Default)]
pub struct UiTree(pub WidgetNode);

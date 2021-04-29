use raui::prelude::{Application, WidgetNode};

/// This resource contains Bevy Retro's UI widget tree
#[derive(Debug, Clone, Default)]
pub struct UiTree(pub WidgetNode);

/// the engine that powers all UI interactions and rendering
pub(crate) struct UiApplication(pub Application);
bevy_retro_macros::impl_deref!(UiApplication, Application);

impl Default for UiApplication {
    fn default() -> Self {
        let mut app = Application::new();
        app.setup(raui::core::widget::setup);

        Self(app)
    }
}

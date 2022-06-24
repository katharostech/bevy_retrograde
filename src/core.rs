use bevy::prelude::*;

pub(crate) struct RetroCorePlugin;

impl Plugin for RetroCorePlugin {
    #[cfg_attr(not(target_arch = "wasm32"), allow(unused))]
    fn build(&self, app: &mut App) {
        #[cfg(target_arch = "wasm32")]
        app.add_system(update_canvas_size);
    }
}

/// System that makes sure the WASM canvas size matches the size of the screen
#[cfg(target_arch = "wasm32")]
pub fn update_canvas_size(mut windows: ResMut<Windows>) {
    // Get the browser window size
    let browser_window = web_sys::window().unwrap();
    let window_width = browser_window.inner_width().unwrap().as_f64().unwrap();
    let window_height = browser_window.inner_height().unwrap().as_f64().unwrap();

    let window = windows.get_primary_mut().unwrap();

    // Set the canvas to the browser size
    window.set_resolution(window_width as f32, window_height as f32);
}

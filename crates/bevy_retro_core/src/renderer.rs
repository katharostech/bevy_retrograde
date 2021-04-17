use bevy::{
    app::{Events, ManualEventReader},
    prelude::*,
    utils::HashMap,
    window::WindowCreated,
    winit::WinitWindows,
};

use crate::cfg_items;

pub(crate) mod luminance_renderer;
pub(crate) mod starc;

use self::luminance_renderer::LuminanceRenderer;

cfg_items!(wasm, {
    mod luminance_web_sys;
    use luminance_web_sys::WebSysWebGL2Surface;
    use std::sync::Arc;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    type Surface = WebSysWebGL2Surface;

    #[wasm_bindgen]
    #[derive(Clone, Debug, Default)]
    pub struct BrowserResizeHandle(Arc<parking_lot::Mutex<Option<(u32, u32)>>>);

    #[wasm_bindgen]
    impl BrowserResizeHandle {
        #[wasm_bindgen]
        pub fn set_new_size(&self, width: u32, height: u32) {
            *self.0.lock() = Some((width, height));
        }
    }
});

#[cfg(not(wasm))]
type Surface = luminance_surfman::SurfmanSurface;

pub(crate) fn get_render_system() -> impl FnMut(&mut World) {
    let mut renderer = RetroRenderer::default();

    move |world| {
        renderer.update(world);
    }
}

#[derive(Default)]
struct RetroRenderer {
    renderers: HashMap<bevy::window::WindowId, LuminanceRenderer>,
    window_created_event_reader: ManualEventReader<WindowCreated>,

    #[cfg(wasm)]
    pub browser_resize_handles: HashMap<bevy::window::WindowId, BrowserResizeHandle>,
    /// These event handlers are held here to keep them from getting dropped so that they can be
    /// called from JavaScript when the browser is resized.
    #[cfg(wasm)]
    pub _browser_resize_event_handlers: HashMap<bevy::window::WindowId, Closure<dyn FnMut()>>,

    #[cfg(not(wasm))]
    pub window_resized_event_reader: ManualEventReader<bevy::window::WindowResized>,
}

/// # Safety
/// FIXME: This is not really safe to `Sync` or `Send`, but we need to make the
/// [`bevy::IntoExclusiveSystem`] trait happy with `RetroRenderer` so this is our temporary
/// workaround.
unsafe impl Sync for RetroRenderer {}
unsafe impl Send for RetroRenderer {}

impl RetroRenderer {
    /// Handle window creation
    #[tracing::instrument(skip(self, world))]
    fn handle_window_create_events(&mut self, world: &mut World) {
        // Get all the windows in the world
        let windows = world.get_resource::<Windows>().unwrap();
        let window_created_events = world.get_resource::<Events<WindowCreated>>().unwrap();

        // Loop through each window creation event
        for window_created_event in self
            .window_created_event_reader
            .iter(&window_created_events)
        {
            // Get the window that was created
            let window_id = window_created_event.id;
            let window = windows
                .get(window_id)
                .expect("Received window created event for non-existent window.");
            let winit_windows = world.get_resource::<bevy::winit::WinitWindows>().unwrap();
            let winit_window = winit_windows.get_window(window.id()).unwrap();

            #[cfg(not(wasm))]
            let surface =
                Surface::from_winit_window(winit_window, luminance_surfman::ShaderVersion::Gles1)
                    .unwrap();

            #[cfg(wasm)]
            let surface = {
                use winit::platform::web::WindowExtWebSys;

                // Get the browser window size
                let browser_window = web_sys::window().unwrap();
                let window_width = browser_window.inner_width().unwrap().as_f64().unwrap();
                let window_height = browser_window.inner_height().unwrap().as_f64().unwrap();

                // Set the canvas to the browser size
                winit_window.set_inner_size(winit::dpi::Size::Logical(winit::dpi::LogicalSize {
                    width: window_width,
                    height: window_height,
                }));

                let canvas = winit_window.canvas();

                // Setup browser resize callback
                let browser_resize_handle = self
                    .browser_resize_handles
                    .entry(window.id())
                    .or_default()
                    .clone();

                let resize_listener = Closure::wrap(Box::new(move || {
                    let browser_window = web_sys::window().unwrap();
                    let window_width = browser_window.inner_width().unwrap().as_f64().unwrap();
                    let window_height = browser_window.inner_height().unwrap().as_f64().unwrap();

                    browser_resize_handle.set_new_size(window_width as u32, window_height as u32);
                })
                    as Box<dyn FnMut() + 'static>);

                browser_window
                    .add_event_listener_with_callback(
                        "resize",
                        resize_listener.as_ref().unchecked_ref(),
                    )
                    .expect("Could not add browser resize event listener");

                // Store the browser event listener so that it doesn't get dropped and so that it can be called
                // by the event handler when the browser resizes.
                self._browser_resize_event_handlers
                    .insert(window_id, resize_listener);

                // Set the browser title
                browser_window.document().unwrap().set_title(window.title());

                // Get the Luminance surface
                WebSysWebGL2Surface::from_canvas(canvas).expect("Could not create graphics surface")
            };

            self.renderers
                .insert(window.id(), LuminanceRenderer::init(window_id, surface));
        }
    }

    #[cfg(not(wasm))]
    #[tracing::instrument(skip(self, world))]
    fn handle_native_window_resize(&mut self, world: &mut World) {
        let window_resized_events = world
            .get_resource::<Events<bevy::window::WindowResized>>()
            .unwrap();

        // for every window resize event
        for event in self
            .window_resized_event_reader
            .iter(&window_resized_events)
        {
            let renderer = self.renderers.get_mut(&event.id).unwrap();
            renderer
                .surface
                .set_size([event.width as u32, event.height as u32])
                .unwrap();
        }
    }

    #[cfg(wasm)]
    #[tracing::instrument(skip(self, world))]
    fn handle_browser_resize(&mut self, world: &mut World) {
        use winit::dpi::{PhysicalSize, Size};
        let winit_windows = world.get_resource::<bevy::winit::WinitWindows>().unwrap();

        for (window_id, resize_handle) in &mut self.browser_resize_handles {
            if let Some((width, height)) = resize_handle.0.lock().take() {
                let winit_window = winit_windows.get_window(*window_id).unwrap();
                winit_window.set_inner_size(Size::Physical(PhysicalSize { width, height }));
            }
        }
    }

    #[tracing::instrument(skip(self, world))]
    fn update(&mut self, world: &mut World) {
        self.handle_window_create_events(world);

        #[cfg(not(wasm))]
        self.handle_native_window_resize(world);

        #[cfg(wasm)]
        self.handle_browser_resize(world);

        for renderer in self.renderers.values_mut() {
            renderer.update(world);
        }
    }
}

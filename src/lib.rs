//! Bevy Retro is an experimental 2D, pixel-perfect renderer for Bevy that can target both web and
//! desktop using OpenGL.

use bevy::{app::ManualEventReader, asset::AssetStage, prelude::*, window::WindowCreated};

#[cfg(not(wasm))]
use bevy::{app::Events, utils::HashMap};
#[cfg(not(wasm))]
use glutin::{ContextBuilder, NotCurrent, RawContext};

#[cfg(unix)]
use glutin::platform::unix::{RawContextExt, WindowExtUnix};

#[cfg(windows)]
use glutin::platform::windows::{RawContextExt, WindowExtUnix};

use glow::HasContext;

#[derive(Debug, Clone, Copy, StageLabel, Hash, PartialEq, Eq)]
pub enum RetroStage {
    Render,
}

#[derive(Default)]
pub struct RetroPlugin;

impl Plugin for RetroPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let render_system = get_render_system();

        app.add_stage_after(
            AssetStage::AssetEvents,
            RetroStage::Render,
            SystemStage::parallel(),
        )
        .add_system_to_stage(RetroStage::Render, render_system.exclusive_system());
    }
}

fn get_render_system() -> impl FnMut(&mut World) {
    let mut renderer = RetroRenderer::default();

    move |world| {
        renderer.update(world);
    }
}

#[derive(Default)]
struct RetroRenderer {
    #[cfg(not(wasm))]
    pub window_contexts: HashMap<bevy::window::WindowId, RawContext<NotCurrent>>,
    pub window_created_event_reader: ManualEventReader<WindowCreated>,
}

impl RetroRenderer {
    #[cfg(not(wasm))]
    fn handle_window_create_events(&mut self, world: &mut World) {
        let world = world.cell();
        let windows = world.get_resource::<Windows>().unwrap();
        let window_created_events = world.get_resource::<Events<WindowCreated>>().unwrap();
        for window_created_event in self
            .window_created_event_reader
            .iter(&window_created_events)
        {
            let window = windows
                .get(window_created_event.id)
                .expect("Received window created event for non-existent window.");
            let winit_windows = world.get_resource::<bevy::winit::WinitWindows>().unwrap();
            let winit_window = winit_windows.get_window(window.id()).unwrap();

            unsafe {
                let context_wrapper = ContextBuilder::new()
                    .build_raw_x11_context(
                        winit_window
                            .xlib_xconnection()
                            .expect("TODO: Support non-x11 windows"),
                        winit_window
                            .xlib_window()
                            .expect("TODO: Support non-x11 windows"),
                    )
                    .expect("TODO: handle error");

                self.window_contexts
                    .insert(window_created_event.id, context_wrapper);
            }
        }
    }

    fn update(&mut self, world: &mut World) {
        #[cfg(not(wasm))]
        self.handle_window_create_events(world);

        for window in world.get_resource::<Windows>().unwrap().iter() {
            unsafe {
                #[cfg(not(wasm))]
                let (gl, context) = {
                    let context = self.window_contexts.remove(&window.id()).unwrap();
                    let context = context.make_current().unwrap();

                    (
                        glow::Context::from_loader_function(|s| {
                            context.get_proc_address(s) as *const _
                        }),
                        context,
                    )
                };

                #[cfg(wasm)]
                let gl = {
                    use wasm_bindgen::JsCast;
                    use winit::platform::web::WindowExtWebSys;

                    let winit_windows = world.get_resource::<bevy::winit::WinitWindows>().unwrap();
                    let winit_window = winit_windows.get_window(window.id()).unwrap();

                    let webgl2_context = winit_window
                        .canvas()
                        .get_context("webgl2")
                        .unwrap()
                        .unwrap()
                        .dyn_into::<web_sys::WebGl2RenderingContext>()
                        .unwrap();

                    glow::Context::from_webgl2_context(webgl2_context)
                };

                gl.clear_color(0.1, 0.1, 0.2, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT);

                #[cfg(not(wasm))]
                {
                    context.swap_buffers().unwrap();

                    self.window_contexts
                        .insert(window.id(), context.treat_as_not_current());
                }
            }
        }
    }
}

use bevy::{
    app::{Events, ManualEventReader},
    prelude::*,
    utils::HashMap,
    window::{WindowCreated, WindowResized},
    winit::WinitWindows,
};

pub(crate) mod luminance_renderer;

#[cfg(wasm)]
mod js;

#[cfg(not(wasm))]
use luminance_surfman::SurfmanSurface;

#[cfg(wasm)]
use luminance_web_sys::WebSysWebGL2Surface;
#[cfg(wasm)]
use std::sync::Arc;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg(not(wasm))]
type Surface = SurfmanSurface;
#[cfg(wasm)]
type Surface = WebSysWebGL2Surface;

use image::{
    imageops::{flip_horizontal_in_place, flip_vertical_in_place},
    RgbaImage,
};

use crate::{Camera, CameraSize, Color, Image, SpriteFlip, SpriteSheet, Visible, WorldPosition};

use self::luminance_renderer::LuminanceRenderer;

pub(crate) trait Renderer {
    fn init(surface: Surface) -> Self;
    fn update(&mut self, render_options: &RetroRenderOptions, render_image: &RenderFrame);
}

#[derive(Clone, Debug)]
pub struct RetroRenderOptions {
    pub pixel_aspect_raio: f32,
}

impl Default for RetroRenderOptions {
    fn default() -> Self {
        Self {
            pixel_aspect_raio: 1.0,
        }
    }
}

/// The information that is sent to the renderer every frame so that it can render the frame
#[derive(Default, Debug, Clone)]
pub(crate) struct RenderFrame {
    image: RgbaImage,
    background_color: Color,
}

/// This system is the system that takes all of the sprites in the scene and produces the final RGBA
/// image that is rendered.
pub(crate) fn pre_render_system(
    sprites: Query<(
        &Handle<Image>,
        &SpriteFlip,
        Option<&Handle<SpriteSheet>>,
        &Visible,
        &WorldPosition,
    )>,
    cameras: Query<(&Camera, &WorldPosition)>,
    sprite_image_assets: Res<Assets<Image>>,
    sprite_sheet_assets: Res<Assets<SpriteSheet>>,
    windows: Res<Windows>,
    winit_windows: Res<WinitWindows>,
    mut render_frame: ResMut<RenderFrame>,
) {
    use image::*;

    // Get the camera
    let mut camera_iter = cameras.iter();
    let (camera, camera_pos) = if let Some(camera_components) = camera_iter.next() {
        camera_components
    } else {
        return;
    };
    if camera_iter.next().is_some() {
        panic!("Only one Retro camera is supported");
    }

    // Get the current window
    let window = if let Some(window) = windows.get_primary() {
        window
    } else {
        return;
    };
    let winit_window = winit_windows.get_window(window.id()).unwrap();
    // Get the window's aspect ratio
    let aspect_ratio =
        winit_window.inner_size().width as f32 / winit_window.inner_size().height as f32;

    // Get the camera height and width
    let camera_height = if let CameraSize::FixedHeight(height) = camera.size {
        height
    } else {
        panic!("Only fixed height camera size is support, open an issue and I'll fix it!");
    };
    let camera_width = (camera_height as f32 * aspect_ratio).floor() as u32;

    // Get the offset of the center of the camera in camera space
    let camera_center_offset_x = (camera_width as f32 / 2.0).floor() as i32;
    let camera_center_offset_y = (camera_height as f32 / 2.0).floor() as i32;

    // Create the render image
    let render_image =

    // If the camera is the same size as the current render image
    if render_frame.image.width() == camera_width && render_frame.image.height() == camera_height {
        // Use the image
        &mut render_frame.image

    // Otherwise
    } else {
        // Create a new image
        render_frame.image = RgbaImage::new(camera_width, camera_height);
        // Use the image
        &mut render_frame.image
    };

    // Clear the image
    for pixel in render_image.pixels_mut() {
        *pixel = Rgba([
            (255.0 * camera.background_color.r as f32).round() as u8,
            (255.0 * camera.background_color.g as f32).round() as u8,
            (255.0 * camera.background_color.b as f32).round() as u8,
            (255.0 * camera.background_color.a as f32).round() as u8,
        ])
    }

    // Sort sprites by their Z index
    let mut sprites = sprites.iter().collect::<Vec<_>>();
    sprites.sort_by(|(_, _, _, _, pos1), (_, _, _, _, pos2)| pos1.z.cmp(&pos2.z));

    // Add sprites to the render image
    for (sprite_handle, sprite_flip, sprite_sheet, visible, sprite_pos) in sprites {
        // Skip invisible sprites
        if !**visible {
            return;
        }

        if let Some(sprite) = sprite_image_assets.get(sprite_handle) {
            let (sheet_width, sheet_height) = sprite.image.dimensions();
            let mut sprite_image = if let Some(sprite_sheet_handle) = sprite_sheet {
                if let Some(sprite_sheet) = sprite_sheet_assets.get(sprite_sheet_handle) {
                    let grid_size = sprite_sheet.grid_size;
                    let tile_index = sprite_sheet.tile_index;
                    let width_tiles = sheet_width / grid_size;

                    let tile_y = (tile_index as f32 / width_tiles as f32).floor() as u32;
                    let tile_x = tile_index - tile_y * width_tiles;

                    sprite
                        .image
                        .view(tile_x * grid_size, tile_y * grid_size, grid_size, grid_size)
                        .to_image()
                } else {
                    continue;
                }
            } else {
                sprite
                    .image
                    .view(0, 0, sheet_width, sheet_height)
                    .to_image()
            };

            let (width, height) = sprite_image.dimensions();

            if sprite_flip.x {
                flip_horizontal_in_place(&mut sprite_image);
            }
            if sprite_flip.y {
                flip_vertical_in_place(&mut sprite_image);
            }

            // Get the offset to the center of the sprite
            let sprite_center_offset_x = (width as f32 / 2.0).floor() as i32;
            let sprite_center_offset_y = (height as f32 / 2.0).floor() as i32;

            // Get the sprite position in camera space
            let sprite_camera_space_x = sprite_pos.x - camera_pos.x;
            let sprite_camera_space_y = sprite_pos.y - camera_pos.y;

            // Get the sprite position in image space
            let sprite_image_space_x = camera_center_offset_x + sprite_camera_space_x;
            let sprite_image_space_y = camera_center_offset_y + sprite_camera_space_y;

            // If the width or height were an odd number, then the `floor()`-ing of the center
            // offset will have chopped off a pixel, so we add that to the height or width here
            let extra_pixel_x = if width % 2 != 0 { 1 } else { 0 };
            let extra_pixel_y = if height % 2 != 0 { 1 } else { 0 };

            // Get the min and max x and y screen position of the sprite in image space
            let sprite_image_space_min_x = (sprite_image_space_x - sprite_center_offset_x)
                .clamp(0, camera_width as i32) as u32;
            let sprite_image_space_max_x =
                (sprite_image_space_x + sprite_center_offset_x + extra_pixel_x)
                    .clamp(0, camera_width as i32) as u32;
            let sprite_image_space_min_y = (sprite_image_space_y - sprite_center_offset_y)
                .clamp(0, camera_height as i32) as u32;
            let sprite_image_space_max_y =
                (sprite_image_space_y + sprite_center_offset_y + extra_pixel_y)
                    .clamp(0, camera_height as i32) as u32;

            // Calculate height and width of the visible portion of the sprite
            let sprite_visible_width = sprite_image_space_max_x - sprite_image_space_min_x;
            let sprite_visible_height = sprite_image_space_max_y - sprite_image_space_min_y;

            // Cull the sprite if it's clamped width or height is 0
            if sprite_visible_width == 0 || sprite_visible_height == 0 {
                continue;
            }

            // If the sprite has clipped at the top-left edge of the screen, set the offset of the
            // sprite view according to how much has been cut off
            let sprite_image_view_offset_x = if sprite_image_space_min_x == 0 {
                ((sprite_image_space_x as i32 - sprite_center_offset_x)
                    - sprite_image_space_min_x as i32)
                    .abs() as u32
            } else {
                0
            };
            let sprite_image_view_offset_y = if sprite_image_space_min_y == 0 {
                ((sprite_image_space_y as i32 - sprite_center_offset_y)
                    - sprite_image_space_min_y as i32)
                    .abs() as u32
            } else {
                0
            };

            // Get a view into the visible portion ov the sprite image
            let sprite_image_view = &sprite_image.view(
                sprite_image_view_offset_x,
                sprite_image_view_offset_y,
                sprite_visible_width,
                sprite_visible_height,
            );

            // Get a sub-image of the camera for where our sprite is to be rendered
            let mut render_sub_image = render_image.sub_image(
                sprite_image_space_min_x,
                sprite_image_space_min_y,
                sprite_visible_width,
                sprite_visible_height,
            );

            // Copy the sprite image onto our render image
            imageops::overlay(&mut render_sub_image, sprite_image_view, 0, 0)
        }
    }
}

pub(crate) fn get_render_system() -> impl FnMut(&mut World) {
    let mut renderer = RetroRenderer::default();

    move |world| {
        renderer.update(world);
    }
}

#[cfg(wasm)]
#[wasm_bindgen]
#[derive(Clone, Debug, Default)]
pub struct BrowserResizeHandle(Arc<parking_lot::Mutex<Option<(u32, u32)>>>);

#[cfg(wasm)]
#[wasm_bindgen]
impl BrowserResizeHandle {
    #[wasm_bindgen]
    pub fn set_new_size(&self, width: u32, height: u32) {
        *self.0.lock() = Some((width, height));
    }
}

#[derive(Default)]
struct RetroRenderer {
    pub renderers: HashMap<bevy::window::WindowId, LuminanceRenderer>,
    pub window_created_event_reader: ManualEventReader<WindowCreated>,
    #[cfg(wasm)]
    pub browser_resize_handles: HashMap<bevy::window::WindowId, BrowserResizeHandle>,
    #[cfg(not(wasm))]
    pub window_resized_event_reader: ManualEventReader<WindowResized>,
}

/// # Safety
/// FIXME: This is not really safe to `Sync` or `Send`, but we need to make the
/// [`bevy::IntoExclusiveSystem`] trait happy with `RetroRenderer` so this is our temporary
/// workaround.
unsafe impl Sync for RetroRenderer {}
unsafe impl Send for RetroRenderer {}

impl RetroRenderer {
    /// Handle window creation
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
            let window = windows
                .get(window_created_event.id)
                .expect("Received window created event for non-existent window.");
            let winit_windows = world.get_resource::<bevy::winit::WinitWindows>().unwrap();
            let winit_window = winit_windows.get_window(window.id()).unwrap();

            #[cfg(not(wasm))]
            let surface = SurfmanSurface::from_winit_window(winit_window).unwrap();

            #[cfg(wasm)]
            let surface = {
                use winit::platform::web::WindowExtWebSys;

                let winit_windows = world.get_resource::<bevy::winit::WinitWindows>().unwrap();
                let winit_window = winit_windows.get_window(window.id()).unwrap();

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
                let browser_resize_handle =
                    self.browser_resize_handles.entry(window.id()).or_default();
                js::setup_canvas_resize_callback(browser_resize_handle.clone());

                // Get the Luminance surface
                WebSysWebGL2Surface::from_canvas(canvas).expect("Could not create graphics surface")
            };

            self.renderers
                .insert(window.id(), LuminanceRenderer::init(surface));
        }
    }

    #[cfg(not(wasm))]
    fn handle_native_window_resize(&mut self, world: &mut World) {
        let window_resized_events = world.get_resource::<Events<WindowResized>>().unwrap();

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
    fn handle_browser_resize(&mut self, world: &mut World) {
        use winit::dpi::{PhysicalSize, Size};
        let winit_windows = world.get_resource::<bevy::winit::WinitWindows>().unwrap();

        for (window_id, resize_handle) in &self.browser_resize_handles {
            if let Some((width, height)) = *resize_handle.0.lock() {
                let winit_window = winit_windows.get_window(*window_id).unwrap();
                winit_window.set_inner_size(Size::Physical(PhysicalSize { width, height }));
            }
        }
    }

    fn update(&mut self, world: &mut World) {
        self.handle_window_create_events(world);

        #[cfg(not(wasm))]
        self.handle_native_window_resize(world);
        #[cfg(wasm)]
        self.handle_browser_resize(world);

        let render_options = world.get_resource::<RetroRenderOptions>().unwrap();
        let render_frame = world.get_resource::<RenderFrame>().unwrap();

        for window in world.get_resource::<Windows>().unwrap().iter() {
            let renderer = self.renderers.get_mut(&window.id()).unwrap();

            renderer.update(render_options, render_frame);

            #[cfg(not(wasm))]
            renderer.surface.swap_buffers().unwrap();
        }
    }
}

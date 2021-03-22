use glutin::{
    dpi::PhysicalSize, platform::unix::WindowExtUnix, ContextBuilder, PossiblyCurrent, RawContext,
};
use luminance::{context::GraphicsContext, framebuffer::Framebuffer, texture::Dim2};
use luminance_gl::GL33;
use winit::window::Window;

pub struct GlutinSurface {
    gl: GL33,
    context: RawContext<PossiblyCurrent>,
    size: [u32; 2],
}

unsafe impl GraphicsContext for GlutinSurface {
    type Backend = GL33;

    fn backend(&mut self) -> &mut Self::Backend {
        &mut self.gl
    }
}

impl GlutinSurface {
    /// Create a surface from a winit window
    ///
    /// > ⚠️ **Warning:** Because glutin will not have access to the window event loop you will need
    /// > to manualy call [`set_size`] on the surface when the window is resized.
    pub fn from_winit_window(window: &Window) -> Self {
        let builder = ContextBuilder::new();

        // Create the raw context
        #[cfg(unix)]
        let context = {
            use glutin::platform::unix::RawContextExt;

            unsafe {
                // TODO: Support wayland and xcb
                builder
                    .build_raw_x11_context(
                        window.xlib_xconnection().unwrap(),
                        window.xlib_window().unwrap(),
                    )
                    .unwrap()
            }
        };

        // Create the raw context
        #[cfg(windows)]
        let context = {
            use glutin::platform::windows::RawContextExt;
        };

        let context = unsafe { context.make_current().unwrap() };

        // Get a pointer to the OpenGL functions
        gl::load_with(|s| context.get_proc_address(s) as *const _);

        let gl = GL33::new().unwrap();

        GlutinSurface {
            gl,
            context,
            size: [100; 2],
        }
    }

    /// Get the back buffer
    pub fn back_buffer(&mut self) -> Framebuffer<GL33, Dim2, (), ()> {
        Framebuffer::back_buffer(self, self.size).unwrap()
    }

    /// Swap the front and back buffers
    pub fn swap_buffers(&mut self) {
        self.context.swap_buffers().unwrap();
    }

    /// Set the size of the surface
    pub fn set_size(&mut self, size: [u32; 2]) {
        self.size = size;
        self.context
            .resize(PhysicalSize::new(self.size[0], self.size[1]))
    }
}

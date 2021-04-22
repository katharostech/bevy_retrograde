//! Graphics types and utilities

use bevy::prelude::*;
use luminance::{self, texture::Dim2};
use luminance_glow::Glow;

pub(crate) mod hooks;

pub use crate::renderer::Surface;
/// A [`luminance`] framebuffer using Bevy Retro's backend
pub type Framebuffer<D, CS, DS> = luminance::framebuffer::Framebuffer<Glow, D, CS, DS>;
/// A [`luminance`] program using Bevy Retro's backend
pub type Program<Sem, Out, Uni> = luminance::shader::Program<Glow, Sem, Out, Uni>;
/// A [`luminance`] tesselator using Bevy Retro's backend
pub type Tess<V, I = (), W = (), S = luminance::tess::Interleaved> =
    luminance::tess::Tess<Glow, V, I, W, S>;
/// A [`luminance`] texturre using Bevy Retro's backend
pub type Texture<D, P> = luminance::texture::Texture<Glow, D, P>;

#[cfg(not(wasm))]
/// A [`luminance`] that is used as the render target for the Bevy Retro scene at the low-res camera
/// resolution
pub type SceneFramebuffer = Framebuffer<Dim2, luminance::pixel::RGBA32F, ()>;
#[cfg(wasm)]
pub type SceneFramebuffer = Framebuffer<Dim2, luminance::pixel::RGBA8UI, ()>;

/// A trait that allows you hook custom functionality into the Bevy Retro renderer
///
/// By implementing [`RenderHook`] you are able to use the raw [`luminance`] API to do fully custom
/// rendering of any kind of object along-side of the built-in Bevy Retro rendering for sprites,
/// text, UI, etc.
///
/// Render hook can be added during the creation of the Bevy app using
/// [`add_render_hook`][`crate::bevy_extensions::AppBuilderRenderHookExt::add_render_hook`] or
/// during the game by using the [`RenderHooks`] resource.
///
/// Currently render hooks are able to render only to the low-resolution framebuffer that is
/// configured at the resolution of the Bevy Retro camera, but in the future you will be able to
/// render at the full resolution of the user's screen if desired, allowing you to selectively break
/// out of the pixel-perfect, retro rendering.
pub trait RenderHook {
    /// Function called upon window creation to initialize the render hook
    fn init(window_id: bevy::window::WindowId, surface: &mut Surface) -> Box<dyn RenderHook>
    where
        Self: Sized;

    /// This function is called before rendering to the retro-resolution framebuffer and is expected
    /// to return a vector of [`RenderHookRenderableHandle`]'s, one for each item that will be
    /// rendered by this hook. The [`RenderHookRenderableHandle`] indicates the depth of the object
    /// in the scene and whether or not it is transparent.
    #[allow(unused_variables)]
    fn prepare_low_res(
        &mut self,
        world: &mut World,
        surface: &mut Surface,
    ) -> Vec<RenderHookRenderableHandle> {
        vec![]
    }

    /// This function is called after [`prepare_low_res`][`RenderHook::prepare_low_res`] is called, possibly multiple times, once
    /// for every batch of renderables that are grouped after depth sorting with all the other
    /// renderables produced by other render hookds. It is passed a framebuffer and a list of
    /// renderables that should be rendered by this hook in this pass.
    #[allow(unused_variables)]
    fn render_low_res(
        &mut self,
        world: &mut World,
        surface: &mut Surface,
        target_framebuffer: &SceneFramebuffer,
        renderables: &[RenderHookRenderableHandle],
    ) {
    }

    // TODO: Add high-res render hook
}
/// Represents a renderable object that can be depth-sorted with other renderables
///
/// The `depth` and `is_transparent` fields are used to sort the renderable objects before rendering
/// and the `identifier` field is used by the [`RenderHook`] that created the handle to identify the
/// renderable that this handle refers to.
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct RenderHookRenderableHandle {
    pub identifier: usize,
    pub is_transparent: bool,
    pub depth: i32,
}

// Sort non-transparent before transparent, and lower depth before higher depth
impl std::cmp::Ord for RenderHookRenderableHandle {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self == other {
            std::cmp::Ordering::Equal
        } else if self.is_transparent && !other.is_transparent {
            std::cmp::Ordering::Greater
        } else if !self.is_transparent && other.is_transparent {
            std::cmp::Ordering::Less
        } else {
            self.depth.cmp(&other.depth)
        }
    }
}

impl PartialOrd for RenderHookRenderableHandle {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Trait that must be implemented for render hook initialization functions
type RenderHookInitFn =
    dyn Fn(bevy::window::WindowId, &mut Surface) -> Box<dyn RenderHook> + Sync + Send + 'static;

/// Bevy resource that can be used to add [`RenderHook`]s to the Bevy Retro renderer
#[derive(Default)]
pub struct RenderHooks {
    pub(crate) new_hooks: Vec<Box<RenderHookInitFn>>,
}

impl RenderHooks {
    /// Add a new [`RenderHook`] to the Bevy Retro renderer
    pub fn add_render_hook<T: RenderHook + 'static>(&mut self) {
        self.new_hooks
            .push(Box::new(T::init) as Box<RenderHookInitFn>);
    }
}

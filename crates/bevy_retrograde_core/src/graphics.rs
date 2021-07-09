//! Graphics types and utilities

use bevy::{prelude::*, utils::HashMap};
use luminance::{self, pixel::NormRGBA8UI, texture::Dim2};
use luminance_glow::Glow;

pub(crate) mod hooks;

use crate::prelude::{Camera, CameraTargetSizes, Image};
pub use crate::renderer::Surface;

mod starc;
pub use starc::*;

/// A [`luminance`] framebuffer using Bevy Retrograde's backend
pub type Framebuffer<D, CS, DS> = luminance::framebuffer::Framebuffer<Glow, D, CS, DS>;
/// A [`luminance`] program using Bevy Retrograde's backend
pub type Program<Sem, Out, Uni> = luminance::shader::Program<Glow, Sem, Out, Uni>;
/// A [`luminance`] tesselator using Bevy Retrograde's backend
pub type Tess<V, I = (), W = (), S = luminance::tess::Interleaved> =
    luminance::tess::Tess<Glow, V, I, W, S>;
/// A [`luminance`] texturre using Bevy Retrograde's backend
pub type Texture<D, P> = luminance::texture::Texture<Glow, D, P>;
/// A cache mapping [`Handle<Image>`]s to luminance textures uploaded to the GPU
///
/// This can be used to get the GPU texture so that you can set a uniform to the value of the
/// texture based on the associated image handle.
pub type TextureCache = HashMap<Handle<Image>, Texture<Dim2, NormRGBA8UI>>;

#[cfg(not(wasm))]
/// A [`luminance`] that is used as the render target for the Bevy Retrograde scene at the low-res camera
/// resolution
pub type SceneFramebuffer = Framebuffer<Dim2, luminance::pixel::RGBA32F, ()>;
#[cfg(wasm)]
pub type SceneFramebuffer = Framebuffer<Dim2, luminance::pixel::RGBA8UI, ()>;

/// A trait that allows you hook custom functionality into the Bevy Retrograde renderer
///
/// By implementing [`RenderHook`] you are able to use the raw [`luminance`] API to do fully custom
/// rendering of any kind of object along-side of the built-in Bevy Retrograde rendering for sprites,
/// text, UI, etc.
///
/// Render hook can be added during the creation of the Bevy app using
/// [`add_render_hook`][`crate::bevy_extensions::AppBuilderRenderHookExt::add_render_hook`] or
/// during the game by using the [`RenderHooks`] resource.
///
/// Currently render hooks are able to render only to the low-resolution framebuffer that is
/// configured at the resolution of the Bevy Retrograde camera, but in the future you will be able to
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
    fn prepare(
        &mut self,
        world: &mut World,
        surface: &mut Surface,
        texture_cache: &mut TextureCache,
        frame_context: &FrameContext,
    ) -> Vec<RenderHookRenderableHandle> {
        vec![]
    }

    /// This function is called after [`prepare_low_res`][`RenderHook::prepare`] is called, possibly multiple times, once
    /// for every batch of renderables that are grouped after depth sorting with all the other
    /// renderables produced by other render hookds. It is passed a framebuffer and a list of
    /// renderables that should be rendered by this hook in this pass.
    #[allow(unused_variables)]
    fn render(
        &mut self,
        world: &mut World,
        surface: &mut Surface,
        texture_cache: &mut TextureCache,
        frame_context: &FrameContext,
        target_framebuffer: &SceneFramebuffer,
        renderables: &[RenderHookRenderableHandle],
    ) {
    }
}

#[derive(Debug, Clone)]
pub struct FrameContext {
    pub camera: Camera,
    pub camera_pos: Vec3,
    pub target_sizes: CameraTargetSizes,
}

/// Represents a renderable object that can be depth-sorted with other renderables
///
/// The `depth` and `is_transparent` fields are used to sort the renderable objects before rendering
/// and the `identifier` field is used by the [`RenderHook`] that created the handle to identify the
/// renderable that this handle refers to.
///
/// The optional entity can be used to break ties in sort order when depths and transparency are
/// equal
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct RenderHookRenderableHandle {
    /// Identifier used to by the render hook to uniquely tie this handle to a specific renderable
    /// that it knows about
    pub identifier: usize,
    /// Whether or not this renderable is transparent
    pub is_transparent: bool,
    /// The z depth of this renderable in the scene
    pub depth: f32,
    /// An optional entity to tie to this renderable that will be used to break ties in depth and
    /// transparency when sorting
    pub entity: Option<Entity>,
}

impl std::cmp::Eq for RenderHookRenderableHandle {}

// Sort non-transparent before transparent, and lower depth before higher depth
impl std::cmp::Ord for RenderHookRenderableHandle {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        if self == other {
            Ordering::Equal
        // First, sort by transparency
        } else if self.is_transparent && !other.is_transparent {
            Ordering::Greater
        } else if !self.is_transparent && other.is_transparent {
            Ordering::Less
        // If their transparency is the same
        } else {
            // Compare depths
            let depth_cmp = self.depth.partial_cmp(&other.depth);

            // Break ties of depth by sorting by the entity id if given
            if (self.depth - other.depth).abs() < f32::EPSILON {
                if self.entity == other.entity {
                    Ordering::Equal
                } else if self.entity.is_none() && other.entity.is_some() {
                    Ordering::Less
                } else if self.entity.is_some() && other.entity.is_none() {
                    Ordering::Greater
                } else {
                    self.entity.unwrap().cmp(&other.entity.unwrap())
                }
            } else {
                // If the depths can be ordered ( i.e. neither is not-a-number )
                if let Some(depth_cmp) = depth_cmp {
                    // Just return the depth ordering
                    depth_cmp
                } else {
                    // Default to "less" in the case of not-a-numbers
                    Ordering::Less
                }
            }
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

/// Bevy resource that can be used to add [`RenderHook`]s to the Bevy Retrograde renderer
#[derive(Default)]
pub struct RenderHooks {
    pub(crate) new_hooks: Vec<Box<RenderHookInitFn>>,
}

impl RenderHooks {
    /// Add a new [`RenderHook`] to the Bevy Retrograde renderer
    pub fn add_render_hook<T: RenderHook + 'static>(&mut self) {
        self.new_hooks
            .push(Box::new(T::init) as Box<RenderHookInitFn>);
    }
}

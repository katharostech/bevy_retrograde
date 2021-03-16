use bevy::{prelude::*, reflect::TypeUuid};

use petgraph::{graph::NodeIndex, stable_graph::StableGraph, Directed};

use crate::Image;

macro_rules! impl_deref {
    ($struct:ident, $target:path) => {
        impl std::ops::Deref for $struct {
            type Target = $target;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $struct {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

/// The retro camera bundle
#[derive(Bundle, Default, Debug, Clone)]
pub struct CameraBundle {
    /// The camera config
    pub camera: Camera,

    /// The position of the center of the camera
    ///
    /// If the width or height of the camera is an even number, the center pixel will be the pixel
    /// to the top-left of the true center.
    pub position: Position,

    /// The corresponding scene node for the camera
    pub scene_node: SceneNode,

    /// The global world position of the sprite
    pub world_position: WorldPosition,
}

/// An 8-bit RGBA color
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 1.,
        }
    }
}

/// The camera component
#[derive(Debug, Clone)]
pub struct Camera {
    /// The size of the camera along the fixed axis, which is by default the vertical axis
    pub size: CameraSize,
    /// Whether or not the camera is active
    ///
    /// If multiple cameras are active at the same time a blank screen will be displayed until only
    /// one camera is active.
    pub active: bool,
    /// The background color of the camera
    ///
    /// This is only visible if the camera size is `Fixed`, in which case it is the color of the
    /// letter-box.
    pub background_color: Color,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            size: Default::default(),
            active: true,
            background_color: Color::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CameraSize {
    /// Fix the camera height in pixels and make the the width scale to whatever the window/screen
    /// size is.
    FixedHeight(u32),
    /// Fix the camera width in pixels and make the the height scale to whatever the window/screen
    /// size is.
    FixedWidth(u32),
    /// Fix the camera width and height in pixels and fill the empty space with the camera
    /// background color.
    Fixed { width: u32, height: u32 },
}

impl Default for CameraSize {
    fn default() -> Self {
        Self::FixedHeight(200)
    }
}

#[derive(Debug, Clone, Copy)]
/// The position of a 2D object in the world
pub struct Position {
    /// The actual position
    pub(crate) pos: IVec3,
    // TODO: Maybe bevy's change detection is good enough to handle this
    /// Whether or not this position has changed since it was last propagated to the global
    /// transform
    pub(crate) dirty: bool,
}

impl Position {
    // Create a new position
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self {
            pos: IVec3::new(x, y, z),
            dirty: true,
        }
    }
}

impl From<IVec3> for Position {
    fn from(pos: IVec3) -> Self {
        Self { pos, dirty: true }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            dirty: true,
        }
    }
}

impl std::ops::Deref for Position {
    type Target = IVec3;

    fn deref(&self) -> &Self::Target {
        &self.pos
    }
}

impl std::ops::DerefMut for Position {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dirty = true;
        &mut self.pos
    }
}

/// A bundle containing all the components necessary to render a sprite
#[derive(Bundle, Default)]
pub struct SpriteBundle {
    /// The image data of the sprite
    pub image: Handle<Image>,
    /// The corresponding scene node for the sprite
    pub scene_node: SceneNode,
    /// The visibility of the sprite
    pub visible: Visible,
    /// The position of the center of the sprite in world space
    pub position: Position,
    /// The global world position of the sprite
    pub world_position: WorldPosition,
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "64746631-1afe-4ca6-8398-7c0df62f7813"]
pub struct SpriteSheet {
    pub grid_size: u32,
    pub tile_index: u32,
}

impl Default for SpriteSheet {
    fn default() -> Self {
        Self {
            grid_size: 16,
            tile_index: 0,
        }
    }
}

/// The graph containing the hierarchy structure of the scene
#[derive(Debug, Clone)]
pub struct SceneGraph(pub(crate) StableGraph<Entity, (), Directed>);

impl Default for SceneGraph {
    fn default() -> Self {
        Self(StableGraph::new())
    }
}

impl SceneGraph {
    pub fn add_node(&mut self, entity: Entity) -> SceneNode {
        SceneNode(self.0.add_node(entity))
    }

    pub fn add_child(&mut self, parent: SceneNode, child: SceneNode) {
        self.0.update_edge(parent.0, child.0, ());
    }

    pub fn remove_child(&mut self, parent: SceneNode, child: SceneNode) {
        if let Some(edge) = self.0.find_edge(parent.0, child.0) {
            self.0.remove_edge(edge);
        }
    }
}

/// An element in the scene
#[derive(Debug, Clone, Copy)]
pub struct SceneNode(pub(crate) NodeIndex);

impl Default for SceneNode {
    fn default() -> Self {
        use rand::prelude::*;
        let mut rng = thread_rng();
        Self(NodeIndex::new(rng.gen()))
    }
}

/// The global position in the world
///
/// Can only be considered up-to-date with the actual sprite world position if `dirty == false`
/// for this sprite and all of it's parents
#[derive(Debug, Clone, Default, Copy)]
pub struct WorldPosition(pub IVec3);
impl_deref!(WorldPosition, IVec3);

/// Indicates whether or not an object should be rendered
#[derive(Debug, Clone, Copy)]
pub struct Visible(pub bool);
impl_deref!(Visible, bool);

impl Default for Visible {
    fn default() -> Self {
        Visible(true)
    }
}

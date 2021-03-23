use bevy::{prelude::*, reflect::TypeUuid, utils::HashMap};

use petgraph::{
    algo::{has_path_connecting, DfsSpace},
    graph::NodeIndex,
    stable_graph::StableGraph,
    visit::{GraphBase, Visitable},
    Directed,
};

use crate::{impl_deref, Image};

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
    /// Whether the camera should be centered about it's position. Defaults to `true`. If `false`
    /// the top-left corner of the camrera will be at its [`Position`].
    pub centered: bool,
    /// The background color of the camera
    ///
    /// This is the color that will be scene in the viewport when there are no sprites in the game
    /// area.
    pub background_color: Color,
    /// The color of the letter box
    ///
    /// The letter box is only visible when the camera size is set to [`Fixed`][CameraSize::Fixed].
    pub letterbox_color: Color,
    /// The aspect ratio of the pxiels when rendered through this camera
    pub pixel_aspect_ratio: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            size: Default::default(),
            centered: true,
            background_color: Color::default(),
            letterbox_color: Color::default(),
            pixel_aspect_ratio: 1.0,
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
    // Sprite options
    pub sprite: Sprite,
    /// The image data of the sprite
    pub image: Handle<Image>,
    /// The visibility of the sprite
    pub visible: Visible,
    /// The position of the center of the sprite in world space
    pub position: Position,
    /// The global world position of the sprite
    pub world_position: WorldPosition,
}

/// Sprite options
#[derive(Debug, Clone)]
pub struct Sprite {
    /// Whether or not the sprite is centered about its position
    pub centered: bool,
    /// Flip the sprite on x
    pub flip_x: bool,
    /// Flip the sprite on y
    pub flip_y: bool,
    /// A visual offset for the sprite
    pub offset: IVec2,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            centered: true,
            flip_x: false,
            flip_y: false,
            offset: IVec2::default(),
        }
    }
}

/// Settings for a sprite sheet
#[derive(Debug, Clone, TypeUuid)]
#[uuid = "64746631-1afe-4ca6-8398-7c0df62f7813"]
pub struct SpriteSheet {
    pub grid_size: UVec2,
    pub tile_index: u32,
}

impl Default for SpriteSheet {
    fn default() -> Self {
        Self {
            grid_size: UVec2::splat(16),
            tile_index: 0,
        }
    }
}

type GraphType = StableGraph<Entity, (), Directed>;

/// The graph containing the hierarchy structure of the scene
#[derive(Debug, Clone)]
pub struct SceneGraph {
    /// A mapping of [`Entities`] to their scene [`NodeIndex`]s
    pub(crate) entity_map: HashMap<Entity, NodeIndex>,
    /// The scene graph
    pub(crate) graph: GraphType,
    /// Used internally to cache graph traversals
    dfs_space: DfsSpace<<GraphType as GraphBase>::NodeId, <GraphType as Visitable>::Map>,
}

impl Default for SceneGraph {
    fn default() -> Self {
        Self {
            entity_map: Default::default(),
            graph: Default::default(),
            dfs_space: Default::default(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GraphError {
    #[error("Operation would result in a cycle")]
    WouldCauseCycle,
}

impl SceneGraph {
    /// # Errors
    /// This function will return an error when `child` is an ancestor of `parent`
    pub fn add_child(&mut self, parent: Entity, child: Entity) -> Result<(), GraphError> {
        let graph = &mut self.graph;
        let parent_node = self
            .entity_map
            .entry(parent)
            .or_insert_with(|| graph.add_node(parent))
            .clone();

        let child_node = self
            .entity_map
            .entry(child)
            .or_insert_with(|| graph.add_node(child))
            .clone();

        // Check for cycles
        if has_path_connecting(&*graph, child_node, parent_node, Some(&mut self.dfs_space)) {
            return Err(GraphError::WouldCauseCycle);
        }

        graph.update_edge(parent_node, child_node, ());

        Ok(())
    }

    pub fn remove_child(&mut self, parent: Entity, child: Entity) {
        let graph = &mut self.graph;

        let parent_node = self
            .entity_map
            .entry(parent)
            .or_insert_with(|| graph.add_node(parent))
            .clone();

        let child_node = self
            .entity_map
            .entry(child)
            .or_insert_with(|| graph.add_node(child))
            .clone();

        if let Some(edge) = graph.find_edge(parent_node, child_node) {
            self.graph.remove_edge(edge);
        }
    }
}

/// The global position in the world
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

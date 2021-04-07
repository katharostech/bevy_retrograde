use bevy::{ecs::query::QueryEntityError, prelude::*, utils::HashMap};
use petgraph::{
    algo::{has_path_connecting, DfsSpace},
    graph::NodeIndex,
    stable_graph::StableGraph,
    visit::{GraphBase, Visitable},
    Directed, Direction,
};

use crate::*;

/// A query that can be used to synchronize the [`WorldPosition`] components of all the entities in
/// the world
pub type WorldPositionsQuery<'a> =
    Query<'a, (Entity, &'static mut Position, &'static mut WorldPosition)>;

/// Trait implemented for [`WorldPositionsQuery`] that adds convenience functions for
/// getting/synchronizing world positions
pub trait WorldPositionsQueryTrait<'a, 'b> {
    fn sync_world_positions(self, scene_graph: &mut SceneGraph);
    fn get_world_position_mut(
        self,
        entity: Entity,
    ) -> Result<Mut<'b, WorldPosition>, QueryEntityError>;
    fn get_local_position_mut(self, entity: Entity) -> Result<Mut<'b, Position>, QueryEntityError>;
}

impl<'a, 'b> WorldPositionsQueryTrait<'a, 'b> for &'b mut WorldPositionsQuery<'a> {
    fn sync_world_positions(self, scene_graph: &mut SceneGraph) {
        propagate_world_positions(scene_graph, self);
    }
    fn get_world_position_mut(
        self,
        entity: Entity,
    ) -> Result<Mut<'b, WorldPosition>, QueryEntityError> {
        Ok(self.get_mut(entity)?.2)
    }
    fn get_local_position_mut(self, entity: Entity) -> Result<Mut<'b, Position>, QueryEntityError> {
        Ok(self.get_mut(entity)?.1)
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

type GraphType = StableGraph<Entity, (), Directed>;

/// The graph containing the hierarchy structure of the scene
#[derive(Debug, Clone)]
pub struct SceneGraph {
    /// A mapping of [`Entity`]'s to their scene [`NodeIndex`]s
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

/// An error that can occur while modifying the scene graph
#[derive(thiserror::Error, Debug)]
pub enum GraphError {
    /// The operation would create a cycle in the scene graph, which is not allowed
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

pub(crate) use systems::*;
mod systems {
    use super::*;

    pub(crate) fn propagate_world_positions_system(
        mut scene_graph: ResMut<SceneGraph>,
        mut query: Query<(Entity, &mut Position, &mut WorldPosition)>,
    ) {
        propagate_world_positions(&mut *scene_graph, &mut query);
    }

    pub(crate) fn propagate_world_positions(
        mut scene_graph: &mut SceneGraph,
        query: &mut Query<(Entity, &mut Position, &mut WorldPosition)>,
    ) {
        // Propagate all graph nodes
        for root_node in scene_graph
            .graph
            .externals(Direction::Incoming)
            .into_iter()
            .collect::<Vec<_>>()
        {
            propagate(root_node, &mut scene_graph, query, None, false);
        }

        // Handle all entities that have not been added to the graph
        for (_, mut pos, mut world_pos) in query
            .iter_mut()
            .filter(|(ent, _, _)| !scene_graph.entity_map.contains_key(ent))
        {
            if pos.dirty {
                **world_pos = **pos;

                pos.dirty = false;
            }
        }
    }

    fn propagate(
        node: NodeIndex,
        scene_graph: &mut SceneGraph,
        query: &mut Query<(Entity, &mut Position, &mut WorldPosition)>,
        parent_world_position: Option<WorldPosition>,
        tree_dirty: bool,
    ) {
        let mut tree_dirty = tree_dirty;

        // Unwrap parent world position
        let parent_world_position = parent_world_position.unwrap_or_default();

        // Handle this node's transform
        let world_pos = {
            // Get the node entity and it's position and world position
            let node_entity = scene_graph.graph[node];
            match query.get_mut(node_entity) {
                Ok((_, mut node_pos, mut world_pos)) => {
                    // If the node's transform has changed since we last saw it
                    if node_pos.dirty || tree_dirty {
                        tree_dirty = true;

                        // Propagate it's global transform
                        **world_pos = *parent_world_position + **node_pos;

                        node_pos.dirty = false;
                    }

                    world_pos.clone()
                }
                Err(e) => match e {
                    QueryEntityError::NoSuchEntity => {
                        // This entity no longer exists so remove it from the scene graph
                        scene_graph.graph.remove_node(node);
                        return;
                    }
                    QueryEntityError::QueryDoesNotMatch => {
                        panic!("Invalid behavior for transform propagate system");
                    }
                },
            }
        };

        // Propagate child nodes
        for child_node in scene_graph
            .graph
            .neighbors(node)
            .into_iter()
            .collect::<Vec<_>>()
        {
            propagate(child_node, scene_graph, query, Some(world_pos), tree_dirty);
        }
    }
}

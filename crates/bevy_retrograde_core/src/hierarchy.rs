//! Scene hierarchy system

use bevy::{prelude::*, utils::HashMap};
use petgraph::{
    algo::{has_path_connecting, DfsSpace},
    graph::NodeIndex,
    stable_graph::StableGraph,
    visit::{GraphBase, Visitable},
    Directed, Direction,
};

use crate::components::*;

/// The petgraph type used for the [`SceneGraph`]
type GraphType = StableGraph<Entity, (), Directed>;

/// Bevy resource containing the hierarchy structure of the scene
///
/// The [`SceneGraph`] is used on combination with the [`Position`] components of each entity in the
/// Bevy world to calculate the effective [`WorldPosition`] of each entity. This [`WorldPosition`]
/// can then be used when doing collision detection or rendering.
///
/// The [`SceneGraph`] is used to add and remove children to entities, allowing you to build the
/// hierarchical structure of the scene.
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

/// An error that can occur while modifying the [`SceneGraph`]
#[derive(thiserror::Error, Debug)]
pub enum GraphError {
    /// The operation would create a cycle in the scene graph, which is not allowed
    #[error("Operation would result in a cycle")]
    WouldCauseCycle,
}

impl SceneGraph {
    /// Make the `child` entity a child of the `parent` entity
    ///
    /// # Errors
    /// This function will return an error when `child` is an ancestor of `parent`
    pub fn add_child(&mut self, parent: Entity, child: Entity) -> Result<(), GraphError> {
        let graph = &mut self.graph;
        let parent_node = *self
            .entity_map
            .entry(parent)
            .or_insert_with(|| graph.add_node(parent));

        let child_node = *self
            .entity_map
            .entry(child)
            .or_insert_with(|| graph.add_node(child));

        // Check for cycles
        if has_path_connecting(&*graph, child_node, parent_node, Some(&mut self.dfs_space)) {
            return Err(GraphError::WouldCauseCycle);
        }

        graph.update_edge(parent_node, child_node, ());

        Ok(())
    }

    /// Make the `child` entity not a child of the `parent` entity
    ///
    /// This function will do nothing if `child` is not a child of `parent.
    pub fn remove_child(&mut self, parent: Entity, child: Entity) {
        let graph = &mut self.graph;

        let parent_node = *self
            .entity_map
            .entry(parent)
            .or_insert_with(|| graph.add_node(parent));

        let child_node = *self
            .entity_map
            .entry(child)
            .or_insert_with(|| graph.add_node(child));

        if let Some(edge) = graph.find_edge(parent_node, child_node) {
            self.graph.remove_edge(edge);
        }
    }
}

pub(crate) use systems::*;
mod systems {
    use bevy::ecs::query::QueryEntityError;

    use super::*;

    /// Bevy system to propagate world positions
    pub(crate) fn propagate_world_positions_system(
        mut scene_graph: ResMut<SceneGraph>,
        mut query: Query<(Entity, &mut Position, &mut WorldPosition)>,
    ) {
        propagate_world_positions(&mut *scene_graph, &mut query);
    }

    /// Function to propagate world positions, used by the [`propagate_world_positions_system`]
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

                    *world_pos
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

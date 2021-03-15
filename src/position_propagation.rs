use bevy::{prelude::*, utils::HashSet};
use petgraph::{graph::NodeIndex, stable_graph::StableGraph, Directed, Direction};

use crate::*;

pub fn propagate_world_positions_system(
    scene_graph: Res<SceneGraph>,
    mut query: Query<(Entity, &mut Position, &SceneNode, &mut WorldPosition)>,
) {
    propagate_world_positions(&*scene_graph, &mut query);
}

pub(crate) fn propagate_world_positions(
    scene_graph: &SceneGraph,
    query: &mut Query<(Entity, &mut Position, &SceneNode, &mut WorldPosition)>,
) {
    let graph = &scene_graph.0;
    let mut visited = HashSet::default();

    // Propagate all graph nodes
    for root_node in graph
        .externals(Direction::Incoming)
        .into_iter()
        .collect::<Vec<_>>()
    {
        propagate(root_node, graph, query, &mut visited, None);
    }

    // Handle all entities that have not been added to the graph
    for (_, mut pos, _, mut world_pos) in query
        .iter_mut()
        .filter(|(ent, _, _, _)| !visited.contains(ent))
    {
        if pos.dirty {
            **world_pos = **pos;

            pos.dirty = false;
        }
    }
}

fn propagate(
    node: NodeIndex,
    graph: &StableGraph<Entity, (), Directed>,
    query: &mut Query<(Entity, &mut Position, &SceneNode, &mut WorldPosition)>,
    visited: &mut HashSet<Entity>,
    parent_world_position: Option<WorldPosition>,
) {
    // Unwrap parent world position
    let parent_world_position = parent_world_position.unwrap_or_default();

    // Handle this node's transform
    let world_pos = {
        // Get the node entity and it's position and world position
        let node_entity = graph[node];
        let (ent, mut node_pos, _, mut world_pos) = query.get_mut(node_entity).unwrap();

        // If the node's transform has changed since we last saw it
        if node_pos.dirty {
            // Propagate it's global transform
            **world_pos = *parent_world_position + **node_pos;

            node_pos.dirty = false;
        }

        visited.insert(ent);

        world_pos.clone()
    };

    // Propagate child nodes
    for child_node in graph.neighbors(node).into_iter().collect::<Vec<_>>() {
        propagate(child_node, graph, query, visited, Some(world_pos));
    }
}

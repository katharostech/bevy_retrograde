use bevy::prelude::*;
use petgraph::{graph::NodeIndex, stable_graph::StableGraph, Directed, Direction};

use crate::*;

pub fn propagate_world_positions_system(
    scene_graph: Res<SceneGraph>,
    mut query: Query<(&Position, &SceneNode, &mut WorldPosition)>,
) {
    propagate_world_positions(&*scene_graph, &mut query);
}

pub(crate) fn propagate_world_positions(
    scene_graph: &SceneGraph,
    query: &mut Query<(&Position, &SceneNode, &mut WorldPosition)>,
) {
    let graph = &scene_graph.0;

    for root_node in graph
        .externals(Direction::Incoming)
        .into_iter()
        .collect::<Vec<_>>()
    {
        propagate(root_node, graph, query, None);
    }
}

fn propagate(
    node: NodeIndex,
    graph: &StableGraph<Entity, (), Directed>,
    query: &mut Query<(&Position, &SceneNode, &mut WorldPosition)>,
    parent_world_position: Option<WorldPosition>,
) {
    // Unwrap parent world position
    let parent_world_position = parent_world_position.unwrap_or_default();

    // Handle this node's transform
    let world_pos = {
        // Get the node entity and it's position and world position
        let node_entity = graph[node];
        let (node_pos, _, mut world_pos) = query.get_mut(node_entity).unwrap();

        // If the node's transform has changed since we last saw it
        if node_pos.dirty {
            // Propagate it's global transform
            **world_pos = *parent_world_position + **node_pos;
        }

        world_pos.clone()
    };

    // Propagate child nodes
    for child_node in graph.neighbors(node).into_iter().collect::<Vec<_>>() {
        propagate(child_node, graph, query, Some(world_pos));
    }
}

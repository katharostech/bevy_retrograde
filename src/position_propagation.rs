use bevy::{ecs::query::QueryEntityError, prelude::*};
use petgraph::{graph::NodeIndex, Direction};

use crate::*;

pub fn propagate_world_positions_system(
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

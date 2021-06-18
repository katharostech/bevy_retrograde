use bevy::{ecs::query::QueryEntityError, prelude::*};
use serde::{Deserialize, Serialize};

use crate::hierarchy::*;

use bevy_retrograde_macros::impl_deref;

/// A query that can be used to synchronize the [`WorldPosition`] components of all the entities in
/// the world
pub type WorldPositionsQuery<'a> =
    Query<'a, (Entity, &'static mut Position, &'static mut WorldPosition)>;

/// Trait implemented for [`WorldPositionsQuery`] that adds convenience functions for
/// getting/synchronizing world positions
pub trait WorldPositionsQueryTrait<'a, 'b> {
    /// Synchronize the world positions of all objects in the scene graph
    ///
    ///
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect_value(PartialEq, Serialize, Deserialize, Component)]
#[serde(default)]
/// The position of a 2D object in the world
pub struct Position {
    /// The actual position
    pub(crate) pos: Vec3,
    // TODO: Maybe bevy's change detection is good enough to handle this
    /// Whether or not this position has changed since it was last propagated to the global
    /// transform
    pub(crate) dirty: bool,
}

impl Position {
    // Create a new position
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            pos: Vec3::new(x, y, z),
            dirty: true,
        }
    }
}

impl From<Vec3> for Position {
    fn from(pos: Vec3) -> Self {
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
    type Target = Vec3;

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

/// The global position in the world
#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect_value(PartialEq, Serialize, Deserialize, Component)]
pub struct WorldPosition(#[serde(default)] pub Vec3);
impl_deref!(WorldPosition, Vec3);

use crate::archetypes::arche::{Archetype, ArchetypeInner};
use crate::component::Bundle;
use crate::entities::entity::{Entity, EntityLocation};
use crate::entities::entity_set::EntitySet;
use std::collections::BTreeMap;
use std::panic::Location;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

pub struct World {
    pub archetypes: BTreeMap<u32, Archetype>,
    pub entities: EntitySet,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldError {
    ArchetypeNotFound,
    TooManyEntitiesInWorld,
    TooManyEntitiesInArchetype,
}

impl World {
    pub fn new(entity_size: u32) -> Self {
        World {
            archetypes: BTreeMap::new(),
            entities: EntitySet::new(entity_size),
        }
    }
    pub fn add_archetype<B: Bundle>(&mut self, size: usize) {
        let inner = ArchetypeInner::new(B::component_info(), size);
        self.archetypes
            .insert(B::archetype_id(), Archetype(Arc::new(inner)));
    }
    /// Call this function when you need to resize an Archetype.
    pub fn take_archetype<B: Bundle>(&mut self) -> Option<Archetype> {
        self.archetypes.remove(&B::archetype_id())
    }
    pub fn push_archetype<B: Bundle>(&mut self, archetype: Archetype) {
        self.archetypes.insert(B::archetype_id(), archetype);
    }

    pub fn add_entity<B: Bundle>(&self, bundle: B) -> Result<(Entity, EntityLocation), WorldError> {
        if !self.entities.entities_left() {
            return Err(WorldError::TooManyEntitiesInWorld);
        }
        let archetype = self
            .archetypes
            .get(&B::archetype_id())
            .ok_or(WorldError::ArchetypeNotFound)?;
        if !archetype.entities_left() {
            return Err(WorldError::TooManyEntitiesInArchetype);
        }
        let entity = self.entities.alloc();
        let data = unsafe { archetype.add_entity(entity.id, bundle.into_component_ptrs().iter()) };
        self.entities.push_location(
            &entity,
            EntityLocation {
                archetype: B::archetype_id(),
                index: Arc::new(AtomicU32::new(data)),
            },
        );
        let location = self.entities.get_location(entity.id).unwrap();
        Ok((entity, location))
    }
}

use crate::archetypes::arche::{Archetype, ArchetypeInner};
use crate::component::Bundle;
use crate::entities::entity::{Entity, EntityLocation};
use crate::entities::entity_set::{EntitySet, EntitySetInner};
use std::collections::BTreeMap;

use std::sync::{atomic, Arc};

#[derive(Clone, Debug)]
pub struct World {
    pub archetypes: BTreeMap<u32, Archetype>,
    pub entities: EntitySet,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldError {
    /// The Archetype was not found. It could be currently being reallocated
    ArchetypeNotFound,
    /// The Entity Set was too small to allocate the entity.
    TooManyEntitiesInWorld,
    /// The Archetype needs to be reallocated
    TooManyEntitiesInArchetype,
    /// The reference to the world is now invalid. You will need to go get a new reference to the world
    EntitySetLocked,
}

impl World {
    pub fn new(entity_size: u32) -> Self {
        World {
            archetypes: BTreeMap::new(),
            entities: EntitySet(Arc::new(EntitySetInner::new(entity_size))),
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
    /// Increases the amount of entities in the world copying the old information.
    ///
    /// The old Arc is still valid, however, will no accept updates or return data.
    ///
    /// It is also marked at locked preventing updates to it
    pub fn increase_entities(&mut self, increase: Option<u32>) -> Result<(), WorldError> {
        self.entities
            .0
            .locked
            .store(true, atomic::Ordering::Relaxed);
        let inner = self
            .entities
            .0
            .reallocate(increase.unwrap_or(self.entities.0.entities.len() as u32));
        self.entities = EntitySet(Arc::new(inner));
        Ok(())
    }
    pub fn remove_entity(&mut self, entity: Entity) {
        let option = self.entities.free(entity);
        if let Some(index) = option {
            let x = self.archetypes.get(&index.archetype).unwrap();
            if x.remove(index.index).is_err() {
                panic!("Tried to remove an entity that was not in the archetype");
            }
        } else {
            panic!("Tried to remove an entity that was not in the world");
        }
    }
    pub fn add_entity<B: Bundle>(&self, bundle: B) -> Result<(Entity, EntityLocation), WorldError> {
        if self.entities.is_locked() {
            return Err(WorldError::EntitySetLocked);
        }
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
        let data = archetype.add_entity(entity.id, bundle.into_component_ptrs().iter());
        self.entities.push_location(
            &entity,
            EntityLocation {
                archetype: B::archetype_id(),
                index: data,
            },
        );
        let location = self.entities.get_location(entity.id).unwrap();
        Ok((entity, location))
    }
}

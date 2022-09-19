use crate::archetypes::arche::{Archetype, ArchetypeInner};
use crate::component::Bundle;
use crate::entities::entity::{Entity, EntityLocation};
use crate::entities::entity_set::{EntitySet, EntitySetInner};
use std::collections::BTreeMap;

use std::sync::{atomic, Arc};
use std::sync::atomic::{AtomicU32, Ordering};

/// The World is the central access point to the data in ECS environment.
#[derive(Clone, Debug)]
pub struct World {
    archetypes: BTreeMap<u32, Archetype>,
    entities: EntitySet,
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
    /// Create a new World pre-allocated with the given number of Entities.
    pub fn new(entity_size: u32) -> Self {
        World {
            archetypes: BTreeMap::new(),
            entities: EntitySet(Arc::new(EntitySetInner::new(entity_size))),
        }
    }
    /// Adds a new Archetype to the World based on the given Type
    ///
    /// # Arguments
    /// * `size` - The number of Entities to allocate for the Archetype.
    pub fn add_archetype<B: Bundle>(&mut self, size: usize) {
        let inner = ArchetypeInner::new(B::component_info(), size);
        self.archetypes
            .insert(B::archetype_id(), Archetype(Arc::new(inner)));
    }
    /// Call this function when you need to resize an Archetype.
    pub fn take_archetype<B: Bundle>(&mut self) -> Option<Archetype> {
        self.archetypes.remove(&B::archetype_id())
    }

    pub fn get_archetype<B: Bundle>(&self) -> Option<&Archetype> {
        self.archetypes.get(&B::archetype_id())
    }
    /// Pushes an Archetype to the World.
    ///
    /// This us done after the Archetype has been reallocated.
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
    pub fn get_entities(&self) -> &EntitySet {
        &self.entities
    }
    /// Remove an entity from the world.
    /// # Arguments
    /// * `entity` - The entity to remove.
    pub fn remove_entity<E: Into<u32>>(&mut self, entity: E) {
        let option = self.entities.free(entity);
        if let Some((index, arch)) = option {
            let x = self.archetypes.get(&arch).unwrap();
            if x.remove(index).is_err() {
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
                archetype: Arc::new(AtomicU32::new(B::archetype_id())),
                index: Arc::new(AtomicU32::new(data)),
            },
        );
        let location = self.entities.get_location(entity.id).unwrap();
        Ok((entity, location))
    }
}

use std::mem;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicIsize, AtomicU32, AtomicUsize, Ordering};
use std::sync::RwLock;
use crate::archetypes::arche::EntityData;
use crate::entities::entity::{Entity, EntityLocation, EntityMeta};
pub struct EntitySet {
    // Active Entities
    pub(crate) entities: Box<[RwLock<EntityMeta>]>,
    // The next available entity ID.
    pub(crate) length: AtomicU32,
    // Entities before the length that are free
    pub(crate) free_list: RwLock<Vec<usize>>,
}

impl EntitySet {
    pub fn new(capacity: u32) -> Self {
        let mut entities = Vec::with_capacity(capacity as usize);
        for _ in 0..capacity {
            entities.push(RwLock::new(EntityMeta::default()));
        }
        Self {
            entities: entities.into_boxed_slice(),
            length: AtomicU32::new(0),
            free_list: RwLock::new(Vec::new()),
        }
    }
    pub fn entities_left(&self) -> bool {
        self.entities.len() >= (self.length.load(Ordering::Relaxed) as usize + 1)
    }
    pub fn alloc(&self) -> Entity {
        if !self.entities_left() {
            panic!("Too many entities in the world!");
        }
        let i = self.length.fetch_add(1, Ordering::Relaxed);
        let guard = self.entities[i as usize].read().unwrap();
        let entity = Entity {
            generation: guard.generation,
            id: i,
        };
        entity
    }
    pub fn push_location(&self, entity: &Entity, location: EntityLocation) {
        let i = entity.id as usize;
        let mut guard = self.entities[i].write().unwrap();
        guard.location = location;
    }
    pub fn free(&self, entity: Entity) {
        let i = entity.id as usize;
        let mut guard = self.entities[i].write().unwrap();
        guard.location = EntityLocation::default();
        guard.generation = NonZeroU32::new(guard.generation.get() + 1).unwrap();
        self.free_list.write().unwrap().push(i);
    }
}
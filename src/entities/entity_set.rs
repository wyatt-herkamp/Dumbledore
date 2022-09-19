use crate::entities::entity::{Entity, EntityLocation, EntityMeta};
use std::mem;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use lockfree::queue::Queue;

#[derive(Debug, Clone)]
pub struct EntitySet(pub Arc<EntitySetInner>);

#[derive(Debug)]
pub struct EntitySetInner {
    // Active Entities
    pub(crate) entities: Box<[EntityMeta]>,
    // The next available entity ID.
    pub(crate) length: AtomicU32,
    // Entities before the length that are free
    pub(crate) free_list: Queue<usize>,

    pub(crate) locked: AtomicBool,
}

impl EntitySetInner {
    pub fn new(capacity: u32) -> Self {
        let mut entities = Vec::with_capacity(capacity as usize);
        for _ in 0..capacity {
            entities.push(EntityMeta::default());
        }
        Self {
            entities: entities.into_boxed_slice(),
            length: AtomicU32::new(0),
            free_list: Queue::new(),
            locked: Default::default(),
        }
    }

    pub fn reallocate(&self, increase: u32) -> Self {
        let new_capacity = self.entities.len() as u32 + increase;
        let mut new_entities = Vec::with_capacity(new_capacity as usize);
        for i in self.entities.iter() {
            new_entities.push(i.clone());
        }
        for _ in self.entities.len()..new_capacity as usize {
            new_entities.push(EntityMeta::default());
        }
        let free_list = Queue::new();
        for x in self.free_list.pop_iter() {
            free_list.push(x);
        }
        Self {
            entities: new_entities.into_boxed_slice(),
            length: AtomicU32::new(self.length.load(Ordering::Relaxed)),
            free_list,
            locked: Default::default(),
        }
    }
}

impl EntitySet {
    pub fn is_locked(&self) -> bool {
        self.0.locked.load(Ordering::Relaxed)
    }
    pub fn entities_left(&self) -> bool {
        self.0.entities.len() >= (self.0.length.load(Ordering::Relaxed) as usize + 1)
    }
    pub fn alloc(&self) -> Entity {
        if !self.entities_left() {
            panic!("Too many entities in the world!");
        }
        let id = if let Some(pop) = self.0.free_list.pop() {
            pop
        } else {
            self.0.length.fetch_add(1, Ordering::Relaxed) as usize
        };
        let mut guard = &self.0.entities[id as usize];
        guard.in_use.store(true, Ordering::Relaxed);
        Entity {
            generation: NonZeroU32::try_from(guard.generation.load(Ordering::Relaxed)).unwrap(),
            id: id as u32,
        }
    }
    pub fn push_location(&self, entity: &Entity, location: EntityLocation) {
        if self.is_locked() {
            panic!("EntitySet is locked!");
        }
        let i = entity.id as usize;
        let mut guard = &self.0.entities[i];
        guard.location.index.store(location.index.load(Ordering::Relaxed), Ordering::Relaxed);
        guard.location.archetype.store(location.archetype.load(Ordering::Relaxed), Ordering::Relaxed);
    }
    pub fn free<E: Into<u32>>(&self, entity: E) -> Option<(u32, u32)> {
        if self.is_locked() {
            panic!("EntitySet is locked!");
        }
        let i = entity.into() as usize;
        let guard = &self.0.entities[i];
        let index = guard.location.index.load(Ordering::Relaxed);
        let archetype = guard.location.archetype.load(Ordering::Relaxed);
        guard.location.archetype.store(0, Ordering::Relaxed);
        guard.location.index.store(0, Ordering::Relaxed);
        guard.generation.fetch_add(1, Ordering::Relaxed);
        guard.in_use.store(false, Ordering::Relaxed);
        self.0.free_list.push(i as usize);
        Some((index, archetype))
    }
    pub fn get_location(&self, entity: u32) -> Option<EntityLocation> {
        if self.is_locked() {
            panic!("EntitySet is locked!");
        }
        if entity as usize >= self.0.entities.len() {
            return None;
        }
        let guard = &self.0.entities[entity as usize];
        Some(guard.location.clone())
    }
    pub fn get_entity(&self, entity: u32) -> Option<(Entity, EntityLocation)> {
        if self.is_locked() {
            panic!("EntitySet is locked!");
        }
        if entity as usize >= self.0.entities.len() {
            return None;
        }
        let guard = &self.0.entities[entity as usize];
        if guard.in_use.load(Ordering::Relaxed) {
            Some((Entity {
                generation: guard.generation.load(Ordering::Relaxed).try_into().unwrap(),
                id: entity,
            }, guard.location.clone()))
        } else {
            None
        }
    }
}

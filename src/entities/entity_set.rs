use crate::entities::entity::{Entity, EntityLocation, EntityMeta};
use std::mem;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct EntitySet(pub Arc<EntitySetInner>);

#[derive(Debug)]
pub struct EntitySetInner {
    // Active Entities
    pub(crate) entities: Box<[Mutex<EntityMeta>]>,
    // The next available entity ID.
    pub(crate) length: AtomicU32,
    // Entities before the length that are free
    pub(crate) free_list: Mutex<Vec<usize>>,

    pub(crate) locked: AtomicBool,
}

impl EntitySetInner {
    pub fn new(capacity: u32) -> Self {
        let mut entities = Vec::with_capacity(capacity as usize);
        for _ in 0..capacity {
            entities.push(Mutex::new(EntityMeta::default()));
        }
        Self {
            entities: entities.into_boxed_slice(),
            length: AtomicU32::new(0),
            free_list: Mutex::new(Vec::new()),
            locked: Default::default(),
        }
    }

    pub fn reallocate(&self, increase: u32) -> Self {
        let new_capacity = self.entities.len() as u32 + increase;
        let mut new_entities = Vec::with_capacity(new_capacity as usize);
        for i in self.entities.iter() {
            new_entities.push(Mutex::new(i.lock().unwrap().clone()));
        }
        for _ in self.entities.len()..new_capacity as usize {
            new_entities.push(Mutex::new(EntityMeta::default()));
        }
        Self {
            entities: new_entities.into_boxed_slice(),
            length: AtomicU32::new(self.length.load(Ordering::Relaxed)),
            free_list: Mutex::new(self.free_list.lock().unwrap().clone()),
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
        let id = if let Some(pop) = self.0.free_list.lock().unwrap().pop() {
            pop
        } else {
            self.0.length.fetch_add(1, Ordering::Relaxed) as usize
        };
        let guard = self.0.entities[id as usize].lock().unwrap();

        Entity {
            generation: guard.generation,
            id: id as u32,
        }
    }
    pub fn push_location(&self, entity: &Entity, location: EntityLocation) {
        if self.is_locked() {
            panic!("EntitySet is locked!");
        }
        let i = entity.id as usize;
        let mut guard = self.0.entities[i].lock().unwrap();
        guard.location = location;
    }
    pub fn free(&self, entity: Entity) -> Option<EntityLocation> {
        if self.is_locked() {
            panic!("EntitySet is locked!");
        }
        let i = entity.id as usize;
        let mut guard = self.0.entities[i].lock().unwrap();
        let old_location = mem::take(&mut guard.location);
        guard.generation = NonZeroU32::new(guard.generation.get() + 1).unwrap();
        self.0.free_list.lock().unwrap().push(i);
        Some(old_location)
    }
    pub fn get_location(&self, entity: u32) -> Option<EntityLocation> {
        if self.is_locked() {
            panic!("EntitySet is locked!");
        }
        if entity as usize >= self.0.entities.len() {
            return None;
        }
        let guard = self.0.entities[entity as usize].lock().unwrap();
        Some(guard.location.clone())
    }
}

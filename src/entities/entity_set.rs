use crate::archetypes::arche::EntityData;
use crate::entities::entity::{Entity, EntityLocation, EntityMeta};
use std::mem;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct EntitySet(pub Arc<EntitySetInner>);

#[derive(Debug)]
pub struct EntitySetInner {
    // Active Entities
    pub(crate) entities: Box<[RwLock<EntityMeta>]>,
    // The next available entity ID.
    pub(crate) length: AtomicU32,
    // Entities before the length that are free
    pub(crate) free_list: RwLock<Vec<usize>>,

    pub(crate) locked: AtomicBool,
}

impl EntitySetInner{
    pub fn new(capacity: u32) -> Self {
        let mut entities = Vec::with_capacity(capacity as usize);
        for _ in 0..capacity {
            entities.push(RwLock::new(EntityMeta::default()));
        }
        Self {
            entities: entities.into_boxed_slice(),
            length: AtomicU32::new(0),
            free_list: RwLock::new(Vec::new()),
            locked: Default::default()
        }
    }

    pub fn reallocate(&self, increase: u32)->Self{
        let new_capacity = self.entities.len() as u32 + increase;
        let mut new_entities = Vec::with_capacity(new_capacity as usize);
        for i in self.entities.iter(){
            new_entities.push(RwLock::new(i.read().unwrap().clone()));
        }
        for _ in self.entities.len()..new_capacity as usize {
            new_entities.push(RwLock::new(EntityMeta::default()));
        }
        Self{
            entities: new_entities.into_boxed_slice(),
            length: AtomicU32::new(self.length.load(Ordering::Relaxed)),
            free_list: RwLock::new(self.free_list.read().unwrap().clone()),
            locked: Default::default()
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
        let i = self.0.length.fetch_add(1, Ordering::Relaxed);
        let guard = self.0.entities[i as usize].read().unwrap();
        let entity = Entity {
            generation: guard.generation,
            id: i,
        };
        entity
    }
    pub fn push_location(&self, entity: &Entity, location: EntityLocation) {
        let i = entity.id as usize;
        let mut guard = self.0.entities[i].write().unwrap();
        guard.location = location;
    }
    pub fn free(&self, entity: Entity) {
        let i = entity.id as usize;
        let mut guard = self.0.entities[i].write().unwrap();
        guard.location = EntityLocation::default();
        guard.generation = NonZeroU32::new(guard.generation.get() + 1).unwrap();
        self.0.free_list.write().unwrap().push(i);
    }
    pub fn get_location(&self, entity: u32) -> Option<EntityLocation> {
        if entity as usize >= self.0.entities.len() {
            return None;
        }
        let guard = self.0.entities[entity as usize].read().unwrap();
        Some(guard.location.clone())
    }
}

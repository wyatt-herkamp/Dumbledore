use std::mem;
use std::sync::atomic::{AtomicIsize, AtomicU32, AtomicUsize};
use crate::entities::entity::{Entity, EntityMeta};

pub struct EntitySet {
    // Active Entities
    pub(crate) entities: Box<[EntityMeta]>,
    // The next available entity ID.
    pub(crate) length: u32,
    // Entities before the length that are free
    pub(crate) free_list: Vec<usize>,
}

impl EntitySet {
    pub fn new(init_size: usize) -> Self {
        Self {
            entities: vec![EntityMeta::default(); init_size].into_boxed_slice(),
            length: 0,
            free_list: Vec::new(),
        }
    }
    pub fn alloc(&mut self) -> (&mut EntityMeta, u32) {
        if let Some(value) = self.free_list.pop() {
            (&mut self.entities[value as usize], value as u32)
        } else {
            self.length += 1;
            let id = self.length;
            if id >= self.entities.len() as u32 {
                self.increase_size(self.entities.len());
            }
            (&mut self.entities[id as usize], id)
        }
    }
    pub fn increase_size(&mut self, size: usize) {
        let mut new_entities = vec![EntityMeta::default(); size + self.entities.len()].into_boxed_slice();
        for (i, entity) in self.entities.iter_mut().enumerate() {
            mem::swap(&mut new_entities[i], entity);
        }
    }
    pub fn get(&self, id: u32) -> Option<&EntityMeta> {
        if self.free_list.contains(&(id as usize)) {
            return None;
        }
        if id >= self.entities.len() as u32 {
            return None;
        }
        Some(&self.entities[id as usize])
    }
    pub fn dealloc(&mut self, entity: Entity) {
        self.free_list.push(entity.id as usize);
        self.entities[entity.id as usize] = EntityMeta::default();
    }
}
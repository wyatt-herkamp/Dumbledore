use std::mem;
use std::sync::atomic::{AtomicIsize, AtomicU32, AtomicUsize};
use std::sync::RwLock;
use crate::archetypes::arche::EntityData;
use crate::entities::entity::{Entity, EntityMeta};

pub struct EntitySet {
    // Active Entities
    pub(crate) entities: Box<[RwLock<EntityData>]>,
    // The next available entity ID.
    pub(crate) length: AtomicU32,
    // Entities before the length that are free
    pub(crate) free_list: RwLock<Vec<usize>>,
}

impl EntitySet {

}
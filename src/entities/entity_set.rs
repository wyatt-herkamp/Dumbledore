use std::sync::atomic::AtomicIsize;
use crate::entities::entity::EntityMeta;

pub struct EntitySet {
    pub(crate) entities: Vec<EntityMeta>,
    pub(crate) length: usize,
    pub(crate) free_list: Vec<usize>,
    pub(crate) free_cursor: AtomicIsize,
}

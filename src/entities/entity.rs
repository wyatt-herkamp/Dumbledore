use std::num::NonZeroU32;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entity {
    pub(crate) generation: NonZeroU32,
    pub id: u32,
}

#[derive(Clone, Debug)]
pub struct EntityMeta {
    pub(crate) generation: NonZeroU32,
    pub location: EntityLocation,
}

impl Default for EntityMeta {
    fn default() -> Self {
        EntityMeta {
            generation: NonZeroU32::new(1).unwrap(),
            location: EntityLocation::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EntityLocation {
    // Archetype ID - Will rarely change.
    pub archetype: u32,
    // Index in the archetype - Could change whenever an entity is moved.
    pub index: Arc<AtomicU32>,
}
impl EntityLocation {
    pub fn get_index(&self) -> u32 {
        self.index.load(std::sync::atomic::Ordering::Relaxed)
    }
}
impl Default for EntityLocation {
    fn default() -> Self {
        EntityLocation {
            archetype: 0,
            index: Arc::new(AtomicU32::new(0)),
        }
    }
}

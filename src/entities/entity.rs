use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entity {
    pub(crate) generation: NonZeroU32,
    pub id: u32,
}
impl From<u32> for Entity {
    fn from(id: u32) -> Self {
        Entity {
            generation: NonZeroU32::new(1).unwrap(),
            id,
        }
    }
}
impl Into<u32> for Entity{
    fn into(self) -> u32 {
        self.id
    }
}
#[derive(Clone, Debug)]
pub struct EntityMeta {
    pub(crate) generation: Arc<AtomicU32>,
    pub in_use:  Arc<AtomicBool>,
    pub location:  EntityLocation,
}

impl Default for EntityMeta {
    fn default() -> Self {
        EntityMeta {
            generation: AtomicU32::new(1).into(),
            in_use: AtomicBool::new(false).into(),
            location: EntityLocation::default().into(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct EntityLocation {
    // Archetype ID - Will rarely change.
    pub archetype:  Arc<AtomicU32>,
    // Index in the archetype - Could change whenever an entity is moved.
    pub index: Arc<AtomicU32>,
}
impl EntityLocation{
    pub fn index(&self) -> u32 {
        self.index.load(Ordering::Relaxed)
    }
}
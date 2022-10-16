use std::num::NonZeroU32;

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
impl Into<u32> for Entity {
    fn into(self) -> u32 {
        self.id
    }
}
#[derive(Clone, Debug)]
pub struct EntityMeta {
    pub(crate) generation: NonZeroU32,
    pub in_use: bool,
    pub location: EntityLocation,
}

impl Default for EntityMeta {
    fn default() -> Self {
        EntityMeta {
            generation: NonZeroU32::new(1).unwrap(),
            in_use: false,
            location: EntityLocation::default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct EntityLocation {
    // Archetype ID - Will rarely change.
    pub archetype: u32,
    // Index in the archetype - Could change whenever an entity is moved.
    pub index: u32,
}

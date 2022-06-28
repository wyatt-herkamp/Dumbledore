use std::num::NonZeroU32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entity {
    pub(crate) generation: NonZeroU32,
    pub(crate) id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityLocation {
    // Archetype ID
    pub archetype: u32,
    // Index in the archetype
    pub index: u32,
}

impl Default for EntityLocation {
    fn default() -> Self {
        EntityLocation {
            archetype: 0,
            index: 0,
        }
    }
}
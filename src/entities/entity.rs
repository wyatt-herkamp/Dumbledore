use std::num::NonZeroU32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entity {
    pub(crate) generation: NonZeroU32,
    id: u32,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityMeta {
    pub(crate) generation: NonZeroU32,
    pub location: EntityLocation,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityLocation {
    pub archetype: u32,
    pub index: u32,
}
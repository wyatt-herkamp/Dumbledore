use std::collections::BTreeMap;
use std::sync::Arc;
use crate::archetypes::arche::{Archetype, ArchetypeInner};
use crate::component::Bundle;
use crate::entities::entity_set::EntitySet;

pub struct World {
    pub archetypes: BTreeMap<u32, Archetype>,
    pub entities: EntitySet,
}

impl World {
    pub fn add_archetype<B: Bundle>(&mut self, size: usize) {
        let inner = ArchetypeInner::new(B::component_info(), size);
        self.archetypes.insert(B::archetype_id(), Archetype(Arc::new(inner)));
    }
    /// Call this function when you need to resize an Archetype.
    pub fn take_archetype<B: Bundle>(&mut self) -> Option<Archetype> {
        self.archetypes.remove(&B::archetype_id())
    }
    pub fn push_archetype<B: Bundle>(&mut self, archetype: Archetype) {
        self.archetypes.insert(B::archetype_id(), archetype);
    }
}


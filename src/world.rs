use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use crate::archetypes::arche::Archetype;
use crate::component::Bundle;
use crate::entities::entity_set::EntitySet;

#[derive(Clone)]
pub struct World {
    pub inner: Arc<WorldInner>,
    pub archetype_start: u32,
}


impl World {
    pub fn new() -> Self {
        World {
            inner: Arc::new(WorldInner {
                entity_set: RwLock::new(EntitySet::new(32)),
                archetype_sets: RwLock::new(BTreeMap::new()),
            }),
            archetype_start: 32,
        }
    }
    /// Registers a Archetype. This is not required. It just allows you to preset a size for a type
    pub fn register_archetype<T: Bundle>(&self, init_size: usize) {
        let archetype = Archetype::new(T::component_info().into_vec(), init_size);
        let mut guard = self.inner.archetype_sets.write().unwrap();
        guard.insert(T::archetype_id(), archetype);
    }
    /// Creates a new Entity.
    pub fn create_entity<B: Bundle>(&self, bundle: B) -> u32 {
        let mut guard = self.inner.entity_set.write().unwrap();
        let (meta, id) = guard.alloc();
        let mut guard = self.inner.archetype_sets.write().unwrap();
        let archetype = guard.get_mut(&B::archetype_id());
       let archetype= if archetype.is_none(){
            let archetype1 = Archetype::new(B::component_info().into_vec(), self.archetype_start as usize);
            guard.insert(B::archetype_id(), archetype1);
            guard.get_mut(&B::archetype_id()).unwrap()
        }else{
            archetype.unwrap()
        };
        let id = unsafe {
            archetype.add_entity(id, bundle.into_component_ptrs().iter())
        };
        meta.location.archetype = B::archetype_id();
        meta.location.index = id;
        id
    }
}

pub struct WorldInner {
    pub entity_set: RwLock<EntitySet>,
    pub archetype_sets: RwLock<BTreeMap<u32, Archetype>>,
}
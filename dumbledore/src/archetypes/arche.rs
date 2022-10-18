use std::alloc::{alloc, dealloc, Layout};

use std::fmt::Debug;
use std::{mem, ptr};

use crate::archetypes::ComponentInfo;
use crate::component::{Bundle, ComponentLookup};

use crate::sets::TypeIdSet;
use std::sync::atomic::{AtomicPtr, AtomicU32, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

/// Contains a Slice of AtomicU8s being the RwLock status of each component within the entity.
///
/// Contains a pointer to the actual component data. Data is offset by the size of the component.
#[derive(Debug)]
pub(crate) struct EntityData {
    pub(crate) inner_ptrs: AtomicPtr<u8>,
    // The EntityID
    pub(crate) entity_id: AtomicU32,
    // If true the entire entity is locked.
    pub(crate) locked: AtomicU8,
    /// Component Data Locks
    pub(crate) anti_racey_bytes: Box<[Arc<AtomicU8>]>,
}

impl EntityData {
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed) == 2
    }
    pub fn is_locking(&self) -> bool {
        self.locked.load(Ordering::Relaxed) == 1
    }
    pub fn is_unlocked(&self) -> bool {
        self.locked.load(Ordering::Relaxed) == 0
    }
    pub fn mark_locking(&self) {
        self.locked.store(1, Ordering::Relaxed);
    }
    /// Only Will Lock if all components are unlocked.
    pub fn try_mark_locked(&self) -> bool {
        for x in self.anti_racey_bytes.iter() {
            if x.load(Ordering::Relaxed) != 0 {
                return false;
            }
        }
        self.locked.store(2, Ordering::Relaxed);
        true
    }
    pub fn mark_unlocked(&self) {
        self.locked.store(0, Ordering::Relaxed);
    }
}

///
/// This is a Wrapper around a Arc<ArchetypeInner>
///
///
/// Memory is layout as follows:
/// ```no_lang
/// | -------------------------- Entity A ------------------------- |
/// | --- Component A --- | -- Component B -- | --- Component C --- |
/// | -------------------------- Entity B ------------------------- |
/// | --- Component A --- | -- Component B -- | --- Component C --- |
/// | -------------------------- Entity C ------------------------- |
/// | --- Component A --- | -- Component B -- | --- Component C --- |
/// | ------------------------------------------------------------- |
/// ```
///
///
#[derive(Debug, Clone)]
pub struct Archetype(pub(crate) Arc<ArchetypeInner>);

impl Archetype {
    pub fn entities_left(&self) -> bool {
        self.0.entity_data.len() >= (self.0.entities_len.load(Ordering::Relaxed) as usize + 1)
    }
    /// Attempts to release the internal ArchetypeInner then increase the size.
    ///
    /// Should be called after World::take_archetype(). This allows the World to stop trying to access the Archetype.
    ///
    /// # Returns
    /// Ok(Self) if the internal arc was able to be released.
    /// Err(Self) if the internal arc was unable to be released.
    pub fn resize(self, increase_by: Option<usize>) -> Result<Self, Self> {
        let mut safe = true;
        for x in self.0.entity_data.iter() {
            x.mark_locking();
            if !x.try_mark_locked() {
                safe = false;
            }
        }
        if !safe {
            // The entity needs to be completely locked before it can be resized.
            return Err(self);
        }
        let result = Arc::try_unwrap(self.0);
        match result {
            Ok(value) => {
                let i = increase_by.unwrap_or(value.entity_data.len());
                let inner = ArchetypeInner::new_from_old(value, i);
                for x in inner.entity_data.iter() {
                    x.mark_unlocked();
                }
                Ok(Archetype(Arc::new(inner)))
            }
            Err(error) => Err(Archetype(error)),
        }
    }
    /// Adds an Entity to the Archetype.
    ///
    /// # Returns
    /// The index for the Entity in the Archetype.
    pub fn add_entity<Data: Bundle>(&self, entity_id: u32, comps: Data) -> u32 {
        let mut result = self.0.free_list.lock().unwrap();
        let id = if let Some(pop) = result.pop() {
            drop(result);
            pop
        } else {
            drop(result);

            self.0.entities_len.fetch_add(1, Ordering::Relaxed)
        };
        let data = &self.0.entity_data[id as usize];
        data.entity_id.store(entity_id, Ordering::Relaxed);
        let ptr = data.inner_ptrs.load(Ordering::Relaxed);
        unsafe {
            comps.put_self(|data, info| {
                let (offset, _index) = *self
                    .0
                    .component_offsets
                    .get(&info.id)
                    .ok_or_else(|| {
                        panic!(
                            "Tried to add a component to an archetype that does not contain it {:?}",
                            info
                        )
                    })
                    .unwrap();
                let x = ptr.add(offset as usize);
                ptr::copy(data, x, info.layout.size());
            });
        }

        id
    }
    /// Returns Err(()) if the entity is locked. However, this does mark the entity as locking so data can not be read anymore
    #[allow(clippy::result_unit_err)]
    pub fn remove(&self, index: u32) -> Result<(), ()> {
        let data = &self.0.entity_data[index as usize];
        data.mark_locking();
        if !data.try_mark_locked() {
            return Err(());
        }
        let ptr = data.inner_ptrs.load(Ordering::Relaxed);

        for comp in self.0.components.iter() {
            let (offset, _) = *self.0.component_offsets.get(&comp.id).unwrap();
            unsafe {
                let x1 = ptr.add(offset as usize);
                (comp.drop)(x1);
            }
        }
        data.entity_id.store(0, Ordering::Relaxed);
        let mut result = self.0.free_list.lock().unwrap();
        result.push(index);
        data.mark_unlocked();
        Ok(())
    }

    /// Returns a Mutable reference to the Component within the Entity.
    ///
    /// # Returns
    /// Ok(Option<MutComponentRef>) if was component is unlocked.
    /// Err(()) if the component is locked.
    #[allow(clippy::result_unit_err)]
    pub fn get_comp_mut<'comp, T: ComponentLookup<'comp>>(
        &self,
        entity_index: u32,
    ) -> Result<Option<T::MutResponse>, ()> {
        let inner = &self.0;

        if entity_index >= inner.entities_len.load(Ordering::Relaxed) {
            return Ok(None);
        }
        let data = &inner.entity_data[entity_index as usize];
        if !data.is_unlocked() {
            return Err(());
        }
        let ptr = data.inner_ptrs.load(Ordering::Relaxed);

        let data = unsafe {
            T::return_mut(|typ| {
                if let Some((offset, index)) = self.0.component_offsets.get(typ) {
                    let anti_race_byte = &data.anti_racey_bytes[*index as usize];
                    let v = anti_race_byte.compare_exchange(
                        0,
                        255,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    );
                    if v.is_ok() {
                        Some((anti_race_byte.clone(), ptr.add(*offset)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        };

        Ok(data)
    }

    /// Returns a reference to the Component within the Entity.
    ///
    /// # Returns
    /// Ok(Option<MutComponentRef>) if was component is unlocked.
    /// Err(()) if the component is locked.
    #[allow(clippy::result_unit_err)]
    pub fn get_comp<'comp, T: ComponentLookup<'comp>>(
        &self,
        entity_index: u32,
    ) -> Result<Option<T::RefResponse>, ()> {
        let inner = &self.0;

        if entity_index >= inner.entities_len.load(Ordering::Relaxed) {
            return Ok(None);
        }
        let data = &inner.entity_data[entity_index as usize];
        if !data.is_unlocked() {
            return Err(());
        }
        let data = unsafe {
            T::return_ref(|typ| {
                if let Some((offset, index)) = self.0.component_offsets.get(typ) {
                    let anti_racey_byte = &data.anti_racey_bytes[*index as usize];
                    let v = anti_racey_byte.fetch_add(1, Ordering::Relaxed);
                    if v < 254 {
                        Some((
                            anti_racey_byte.clone(),
                            data.inner_ptrs.load(Ordering::Relaxed).add(*offset),
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        };

        Ok(data)
    }
}

#[derive(Debug)]
pub struct ArchetypeInner {
    /// The component types in this archetype.
    pub(crate) component_offsets: TypeIdSet<(usize, u32)>,

    pub(crate) components: Box<[ComponentInfo]>,
    /// The data for each entity
    pub(crate) entity_data: Box<[EntityData]>,
    /// The number of entities in this archetype.
    pub(crate) entities_len: AtomicU32,

    pub(crate) home_ptr: AtomicPtr<u8>,

    pub(crate) free_list: Mutex<Vec<u32>>,
    pub(crate) max_size: usize,
}

impl ArchetypeInner {
    pub(crate) fn new(mut components: Vec<ComponentInfo>, entity_start_size: usize) -> Self {
        components.sort_unstable_by_key(|c| c.id);
        let total_size = components.iter().map(|c| c.layout.size()).sum::<usize>();
        let ptr = unsafe {
            let layout = Layout::from_size_align_unchecked(total_size * entity_start_size, 8);
            alloc(layout)
        };
        let mut data = Vec::with_capacity(entity_start_size);
        for entity_index in 0..entity_start_size {
            unsafe {
                data.push(EntityData {
                    inner_ptrs: AtomicPtr::new(ptr.add(total_size * entity_index)),
                    entity_id: AtomicU32::new(0),
                    locked: AtomicU8::new(0),
                    anti_racey_bytes: components
                        .iter()
                        .map(|_| Arc::new(AtomicU8::new(0)))
                        .collect(),
                });
            }
        }

        let mut offset = 0;
        let map = components.iter().enumerate().map(|(index, v)| {
            let my_offset = offset;
            offset += v.layout.size();
            (v.id, (my_offset, index as u32))
        });
        //
        Self {
            component_offsets: TypeIdSet::new(map),
            components: components.into_boxed_slice(),
            entity_data: data.into_boxed_slice(),
            entities_len: AtomicU32::new(0),
            home_ptr: AtomicPtr::new(ptr),
            free_list: Mutex::new(Vec::with_capacity(1)),
            max_size: entity_start_size,
        }
    }
    /// Clones the data from the old archetype into the new one.
    /// Does not deallocate the old one. That will be done by Drop once it goes out of scope
    pub fn new_from_old(mut old: ArchetypeInner, size_increase: usize) -> Self {
        let mut old_entities = mem::take(&mut old.entity_data);
        let total_size = old
            .components
            .iter()
            .map(|c| c.layout.size())
            .sum::<usize>();
        let new_size = old_entities.len() + size_increase;
        let ptr = unsafe { alloc(Layout::from_size_align_unchecked(total_size * new_size, 8)) };
        let mut new_data = Vec::with_capacity(new_size);
        for (index, data) in old_entities.iter_mut().enumerate() {
            unsafe {
                let new_pointer = ptr.add(total_size * index);
                ptr::copy_nonoverlapping(
                    data.inner_ptrs.load(Ordering::Relaxed),
                    new_pointer,
                    total_size,
                );
                new_data.push(EntityData {
                    inner_ptrs: AtomicPtr::new(new_pointer),
                    entity_id: mem::take(&mut data.entity_id),
                    locked: AtomicU8::new(0),
                    anti_racey_bytes: mem::take(&mut data.anti_racey_bytes),
                });
            }
        }
        for i in old_entities.len()..new_size {
            unsafe {
                let new_pointer = ptr.add(total_size * i);
                new_data.push(EntityData {
                    inner_ptrs: AtomicPtr::new(new_pointer),
                    entity_id: AtomicU32::new(0),
                    locked: AtomicU8::new(0),
                    anti_racey_bytes: old
                        .components
                        .iter()
                        .map(|_| Arc::new(AtomicU8::new(0)))
                        .collect(),
                });
            }
        }

        let mutex = mem::take(&mut old.free_list);
        let mut result = mutex.lock().unwrap();
        result.shrink_to(1);
        drop(result);
        Self {
            component_offsets: old.component_offsets.clone(),
            components: old.components.clone(),
            entity_data: new_data.into_boxed_slice(),
            entities_len: mem::take(&mut old.entities_len),
            home_ptr: AtomicPtr::new(ptr),
            free_list: mutex,
            max_size: new_size,
        }
    }
}

impl Drop for ArchetypeInner {
    fn drop(&mut self) {
        let total_size = self
            .components
            .iter()
            .map(|c| c.layout.size())
            .sum::<usize>();
        let entities_len = self.entities_len.load(Ordering::Relaxed);

        for (index, data) in self.entity_data.iter_mut().enumerate() {
            if index >= entities_len as usize {
                break;
            }
            for (comp, (_ty, (offset, _))) in
                self.components.iter().zip(self.component_offsets.0.iter())
            {
                unsafe {
                    let ptr = data.inner_ptrs.load(Ordering::Relaxed).add(*offset);
                    (comp.drop)(ptr);
                }
            }
        }
        unsafe {
            let layout = Layout::from_size_align_unchecked(total_size * self.max_size, 8);
            dealloc(self.home_ptr.load(Ordering::Relaxed), layout);
        }
    }
}

use std::alloc::{alloc, dealloc, Layout};
use std::any::TypeId;

use std::cell::UnsafeCell;
use std::collections::BTreeSet;
use std::fmt::{Debug, Formatter};
use std::{io, mem};

use crate::archetypes::ComponentInfo;
use crate::component::Component;
use crate::component_ref::{ComponentRef, MutComponentRef};
use crate::sets::TypeIdSet;
use std::ptr::{copy_nonoverlapping, NonNull};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use std::sync::Arc;

/// Contains a Slice of AtomicU8s being the RwLock status of each component within the entity.
///
/// Contains a pointer to the actual component data. Data is offset by the size of the component.
#[derive(Debug)]
pub(crate) struct EntityData {
    pub(crate) inner_ptrs: NonNull<u8>,
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
    pub unsafe fn add_entity<'data, Data>(&self, entity_id: u32, data: Data) -> u32
    where
        Data: Iterator<Item = &'data (ComponentInfo, NonNull<u8>)>,
    {
        let id = self.0.entities_len.fetch_add(1, Ordering::Relaxed);
        for (ty, raw_pointer) in data {
            let data = &self.0.entity_data[id as usize];
            data.entity_id.store(entity_id, Ordering::Relaxed);
            let (offset, _index) = *self
                .0
                .component_offsets
                .get(&ty.id)
                .ok_or_else(|| {
                    panic!(
                        "Tried to add a component to an archetype that does not contain it {:?}",
                        ty
                    )
                })
                .unwrap();

            let x = data.inner_ptrs.as_ptr().add(offset as usize);
            copy_nonoverlapping(raw_pointer.as_ptr(), x, ty.layout.size());
        }
        id
    }
    /// Returns a Mutable reference to the Component within the Entity.
    ///
    /// # Returns
    /// Ok(Option<MutComponentRef>) if was component is unlocked.
    /// Err(()) if the component is locked.
    pub fn get_comp_mut<T: Component>(
        &self,
        entity_index: u32,
    ) -> Result<Option<MutComponentRef<T>>, ()> {
        let inner = &self.0;

        if entity_index >= inner.entities_len.load(Ordering::Relaxed) {
            return Ok(None);
        }
        let comp_offset = inner.component_offsets.get(&TypeId::of::<T>());
        if let Some((comp_offset, index)) = comp_offset {
            let data = &inner.entity_data[entity_index as usize];
            if !data.is_unlocked() {
                return Err(());
            }
            let anti_racey_byte = &data.anti_racey_bytes[*index as usize];
            let i = anti_racey_byte.load(Ordering::Acquire);
            if i > 0 {
                return Err(());
            } else {
                let ptr = unsafe { &mut *data.inner_ptrs.as_ptr().add(*comp_offset).cast::<T>() };
                anti_racey_byte.store(255, Ordering::Relaxed);
                Ok(Some(MutComponentRef {
                    component: ptr,
                    ref_count: Arc::clone(anti_racey_byte),
                }))
            }
        } else {
            Ok(None)
        }
    }

    /// Returns a reference to the Component within the Entity.
    ///
    /// # Returns
    /// Ok(Option<MutComponentRef>) if was component is unlocked.
    /// Err(()) if the component is locked.
    pub fn get_comp<T: Component>(&self, entity_index: u32) -> Result<Option<ComponentRef<T>>, ()> {
        let inner = &self.0;

        if entity_index >= inner.entities_len.load(Ordering::Relaxed) {
            return Ok(None);
        }
        let comp_offset = inner.component_offsets.get(&TypeId::of::<T>());
        if let Some((comp_offset, index)) = comp_offset {
            let data = &inner.entity_data[entity_index as usize];
            if !data.is_unlocked() {
                return Err(());
            }
            let anti_racey_byte = &data.anti_racey_bytes[*index as usize];
            let i = anti_racey_byte.load(Ordering::Acquire);
            /// If it is at 254 it has too many readers. 255 it is being written to.
            if i < 254 {
                let ptr = unsafe { &mut *data.inner_ptrs.as_ptr().add(*comp_offset).cast::<T>() };
                anti_racey_byte.store(i + 1, Ordering::Relaxed);

                Ok(Some(ComponentRef {
                    component: ptr,
                    ref_count: anti_racey_byte.clone(),
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
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

    pub(crate) home_ptr: NonNull<u8>,
}

impl ArchetypeInner {
    pub(crate) fn new(mut components: Vec<ComponentInfo>, entity_start_size: usize) -> Self {
        components.sort_unstable_by_key(|c| c.id);
        let total_size = components.iter().map(|c| c.layout.size()).sum::<usize>();
        let data = if entity_start_size > 0 {
            let ptr = unsafe {
                alloc(Layout::from_size_align_unchecked(
                    total_size * entity_start_size,
                    8,
                ))
            };
            let mut data = Vec::with_capacity(entity_start_size);
            for entity_index in 0..entity_start_size {
                unsafe {
                    data.push(EntityData {
                        inner_ptrs: NonNull::new_unchecked(ptr.add(total_size * entity_index)),
                        entity_id: AtomicU32::new(0),
                        locked: AtomicU8::new(0),
                        anti_racey_bytes: components
                            .iter()
                            .map(|_| Arc::new(AtomicU8::new(0)))
                            .collect(),
                    });
                }
            }
            (data, NonNull::new(ptr).unwrap())
        } else {
            (
                Vec::new(),
                NonNull::new(std::ptr::null_mut::<u8>()).unwrap(),
            )
        };
        let mut offset = 0;
        let map = components.iter().enumerate().map(|(index, v)| {
            let my_offset = offset;
            offset += v.layout.size();
            (v.id.clone(), (my_offset, index as u32))
        });
        //
        Self {
            component_offsets: TypeIdSet::new(map),
            components: components.into_boxed_slice(),
            entity_data: data.0.into_boxed_slice(),
            entities_len: AtomicU32::new(0),
            home_ptr: data.1,
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
                copy_nonoverlapping(data.inner_ptrs.as_ptr(), new_pointer, total_size);
                new_data.push(EntityData {
                    inner_ptrs: NonNull::new_unchecked(new_pointer),
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
                    inner_ptrs: NonNull::new_unchecked(new_pointer),
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

        Self {
            component_offsets: old.component_offsets.clone(),
            components: old.components.clone(),
            entity_data: new_data.into_boxed_slice(),
            entities_len: mem::take(&mut old.entities_len),
            home_ptr: NonNull::new(ptr).unwrap(),
        }
    }
}

impl Drop for ArchetypeInner {
    fn drop(&mut self) {
        let i = self
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
                    let ptr = data.inner_ptrs.as_ptr().add(*offset);
                    (comp.drop)(ptr);
                }
            }
        }
        unsafe {
            dealloc(
                self.home_ptr.as_ptr(),
                Layout::from_size_align_unchecked(i * self.entity_data.iter_mut().len(), 8),
            );
        }
    }
}

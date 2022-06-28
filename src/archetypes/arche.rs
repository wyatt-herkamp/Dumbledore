use std::alloc::{alloc, dealloc, Layout};
use std::any::{TypeId};


use std::{io, mem};
use std::collections::BTreeSet;
use std::fmt::{Debug, Formatter};


use std::ptr::{copy_nonoverlapping, NonNull};
use std::sync::{Arc};
use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};
use crate::archetypes::ComponentInfo;
use crate::component::Component;
use crate::component_ref::{ComponentRef, MutComponentRef};
use crate::sets::TypeIdSet;


///
///
///
/// Memory allocation for Components.
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
/// A Slice of AtomicU8s is used to represent the RWLock status of each component within the entity.
///
/// Data will be sorted by the TypeID
///
/// mutability is required to be able to add a new entity because it could result in a reallocation of the memory.
pub struct Archetype {
    /// The component types in this archetype.
    pub(crate) component_offsets: TypeIdSet<(usize, u32)>,
    pub(crate) components: Vec<ComponentInfo>,
    /// Index --> Entity ID within the data array
    pub(crate) entities: Box<[u32]>,
    /// The data for each entity
    pub(crate) data: Box<[EntityData]>,
    /// The number of entities in this archetype.
    pub(crate) entities_len: AtomicU32,

    pub(crate) home_ptr: NonNull<u8>,
}


impl Debug for Archetype {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Archetype {{  components: {:?}, entities: {:?}, data: {:?}, entities_len: {:?} }}", self.components, self.entities, self.data.len(), self.entities_len)
    }
}

impl Archetype {
    pub unsafe fn add_entity<'data, Data>(&mut self, entity_id: u32, data: Data) -> u32
        where Data: Iterator<Item=&'data (ComponentInfo, NonNull<u8>)> {
        let id = self.entities_len.fetch_add(1, Ordering::Relaxed);
        self.entities[id as usize] = entity_id;
        for (ty, raw_pointer) in data {
            let data = &self.data[id as usize];
            let (offset, _index) = *self.component_offsets.get(&ty.id).ok_or_else(|| {
                panic!("Tried to add a component to an archetype that does not contain it {:?}", ty)
            }).unwrap();

            let x = data.inner_ptrs.as_ptr().add(offset as usize);
            copy_nonoverlapping(raw_pointer.as_ptr(), x, ty.layout.size());
        }
        id
    }
    pub fn get_comp_mut<T: Component>(&self, entity_index: u32) -> Result<Option<MutComponentRef<T>>, io::Error> {
        let inner = self;

        if entity_index >= inner.entities_len.load(Ordering::Relaxed) {
            return Ok(None);
        }
        let comp_offset = inner.component_offsets.get(&TypeId::of::<T>());
        if let Some((comp_offset, index)) = comp_offset {
            let data = &inner.data[entity_index as usize];

            let anti_racey_byte = &data.anti_racey_bytes[*index as usize];
            let i = anti_racey_byte.load(Ordering::Acquire);
            if i > 0 {
                Err(io::Error::new(io::ErrorKind::Other, "Component is already locked"))
            } else {
                let ptr = unsafe {
                    &mut *data.inner_ptrs.as_ptr().add(*comp_offset).cast::<T>()
                };
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


    pub fn get_comp<T: Component>(&self, entity_index: u32) -> Result<Option<ComponentRef<T>>, io::Error> {
        let inner = self;

        if entity_index >= inner.entities_len.load(Ordering::Relaxed) {
            return Ok(None);
        }
        let comp_offset = inner.component_offsets.get(&TypeId::of::<T>());
        if let Some((comp_offset, index)) = comp_offset {
            let data = &inner.data[entity_index as usize];
            let anti_racey_byte = &data.anti_racey_bytes[*index as usize];
            let i = anti_racey_byte.load(Ordering::Acquire);
            /// If it is at 254 it has too many readers. 255 it is being written to.
            if i < 254 {
                let ptr = unsafe {
                    &mut *data.inner_ptrs.as_ptr().add(*comp_offset).cast::<T>()
                };
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


impl Archetype {
    pub(crate) fn new(mut components: Vec<ComponentInfo>, entity_start_size: usize) -> Self {
        if entity_start_size % 2 != 0 {
            panic!("entity_start_size must be a multiple of 2");
        }
        components.sort_unstable_by_key(|c| c.id);
        let total_size = components.iter().map(|c| c.layout.size()).sum::<usize>();
        let data = if entity_start_size > 0 {
            let ptr = unsafe {
                alloc(Layout::from_size_align_unchecked(total_size * entity_start_size, 8))
            };
            let mut data = Vec::with_capacity(entity_start_size);
            for entity_index in 0..entity_start_size {
                unsafe {
                    data.push(EntityData {
                        inner_ptrs: NonNull::new_unchecked(ptr.add(total_size * entity_index)),
                        anti_racey_bytes: components.iter().map(|_| Arc::new(AtomicU8::new(0))).collect(),
                    });
                }
            }
            (data, NonNull::new(ptr).unwrap())
        } else {
            (Vec::new(), NonNull::new(std::ptr::null_mut::<u8>()).unwrap())
        };
        let mut offset = 0;
        let map = components.iter().enumerate().map(|(index, v)| {
            let my_offset = offset;
            offset += v.layout.size();
            (v.id.clone(), (my_offset, index as u32, ))
        });
        //
        Self {
            component_offsets: TypeIdSet::new(map),
            components,
            entities: vec![0; entity_start_size as usize].into_boxed_slice(),
            data: data.0.into_boxed_slice(),
            entities_len: AtomicU32::new(0),
            home_ptr: data.1,
        }
    }
    pub fn requires_reallocation(&self) -> bool {
        self.entities.len() == self.entities_len.load(Ordering::Relaxed) as usize
    }

    pub fn grow(&mut self, extend_by: u32) -> Result<(), ()> {
        if extend_by % 2 != 0 {
            panic!("entity_start_size must be a multiple of 2");
        }
        //TODO ensure that all entities are locked before growing.
        let new_size = self.entities.len() + extend_by as usize;
        let mut new_entities = vec![0; new_size].into_boxed_slice();
        self.entities.iter().enumerate().for_each(|(index, v)| {
            new_entities[index] = *v;
        });
        self.entities = new_entities;

        let mut new_data = Vec::with_capacity(new_size);
        let comp_size = self.components.iter().map(|c| c.layout.size()).sum::<usize>();
        let ptr = unsafe {
            alloc(Layout::from_size_align_unchecked(comp_size * new_size, 8))
        };

        for (index, old) in self.data.iter_mut().enumerate() {
            let data = mem::take(&mut old.anti_racey_bytes);
            unsafe {
                let new_index = ptr.add(comp_size * index);
                copy_nonoverlapping(old.inner_ptrs.as_ptr(), new_index.cast(), comp_size);
                new_data.push(EntityData {
                    inner_ptrs: NonNull::new_unchecked(new_index),
                    anti_racey_bytes: data,
                });
            }
        }
        for index in self.data.iter_mut().len()..new_size {
            unsafe {
                new_data.push(EntityData {
                    inner_ptrs: NonNull::new_unchecked(ptr.add(comp_size * index)),
                    anti_racey_bytes: self.components.iter().map(|_| Arc::new(AtomicU8::new(0))).collect(),
                });
            }
        }
        unsafe{
            dealloc(self.home_ptr.as_ptr(), Layout::from_size_align_unchecked(comp_size * self.entities.len(), 8));
        }
        self.home_ptr = NonNull::new(ptr).unwrap();

        self.data = new_data.into_boxed_slice();
        Ok(())
    }
}

impl Drop for Archetype {
    fn drop(&mut self) {
        let i = self.components.iter().map(|c| c.layout.size()).sum::<usize>();
        let entities_len = self.entities_len.load(Ordering::Relaxed);

        for (index, data) in self.data.iter_mut().enumerate() {
            if index >= entities_len as usize {
                break;
            }
            for (comp, (_ty, (offset, _))) in self.components.iter().zip(self.component_offsets.0.iter()) {
                unsafe {
                    let ptr = data.inner_ptrs.as_ptr().add(*offset);
                    (comp.drop)(ptr);
                }
            }
        }
        unsafe {
            dealloc(self.home_ptr.as_ptr(), Layout::from_size_align_unchecked(i * self.data.iter_mut().len(), 8));
        }
    }
}


/// Contains a Slice of AtomicU8s being the RwLock status of each component within the entity.
///
/// Contains a pointer to the actual component data. Data is offset by the size of the component.
#[derive(Debug)]
pub struct EntityData {
    pub(crate) inner_ptrs: NonNull<u8>,

    pub(crate) anti_racey_bytes: Box<[Arc<AtomicU8>]>,
}

#[cfg(test)]
pub mod test {
    use std::ptr::NonNull;
    use crate::archetypes::arche::Archetype;
    use crate::archetypes::ComponentInfo;


    #[derive(Debug)]
    pub struct TestComponent {
        pub value: u32,
        pub string: String,
    }

    #[test]
    pub fn test() {
        let info = vec![ComponentInfo::new::<TestComponent>(), ComponentInfo::new::<String>(), ComponentInfo::new::<u32>()];
        let mut archetype = Archetype::new(info, 32);
        let component = &mut TestComponent {
            value: 1,
            string: "".to_string()
        } as *mut TestComponent;
        for _i in 0..32 {
            let i = unsafe {
                let mut y = 10u32;

                let x = (ComponentInfo::new::<String>(), NonNull::new_unchecked((&mut "Test".to_string() as *mut String) as *mut u8));
                let x1 = (ComponentInfo::new::<TestComponent>(), NonNull::new_unchecked(component as *mut u8));
                let ptr = &mut y as *mut u32;
                let x2 = (ComponentInfo::new::<u32>(), NonNull::new_unchecked(ptr.cast()));
                let mut entity = vec![x, x1, x2];
                entity.sort_by_key(|c| c.0.id);
                archetype.add_entity(2, entity.iter())
            };
            let mut result = archetype.get_comp_mut::<String>(i).unwrap().unwrap();
            println!("{:?}", result.as_mut());
            let result = archetype.get_comp::<TestComponent>(i).unwrap().unwrap();
            println!("{:?}", result.as_ref());
            let result = archetype.get_comp::<u32>(i).unwrap().unwrap();
            println!("{:?}", result.as_ref());
        }
        archetype.grow(32).unwrap();

        for _i in 0..32 {
            let i = unsafe {
                let mut y = 10u32;

                let x = (ComponentInfo::new::<String>(), NonNull::new_unchecked((&mut "Test".to_string() as *mut String) as *mut u8));
                let x1 = (ComponentInfo::new::<TestComponent>(), NonNull::new_unchecked(component as *mut u8));
                let ptr = &mut y as *mut u32;
                let x2 = (ComponentInfo::new::<u32>(), NonNull::new_unchecked(ptr.cast()));
                let mut entity = vec![x, x1, x2];
                entity.sort_by_key(|c| c.0.id);
                archetype.add_entity(2, entity.iter())
            };
            let mut result = archetype.get_comp_mut::<String>(i).unwrap().unwrap();
            println!("{:?}", result.as_ref());
            drop(result);
            let result = archetype.get_comp::<String>(i).unwrap().unwrap();
            println!("{:?}", result.as_ref());
            let result = archetype.get_comp_mut::<TestComponent>(i).unwrap().unwrap();
            println!("{:?}", result.as_ref());
            result.component.value = 55;
            result.component.string = "Test".to_string();
            drop(result);

            let result = archetype.get_comp::<u32>(i).unwrap().unwrap();
            println!("{:?}", result.as_ref());
        }
        println!("{:?}", archetype);
        drop(archetype);
        println!("Hey");
    }
}
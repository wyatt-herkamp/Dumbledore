use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ptr::NonNull;
use crate::archetypes::ComponentInfo;

pub trait Component: Send + Sync + 'static {
    fn component_info() -> ComponentInfo where Self: Sized {
        ComponentInfo::new::<Self>()
    }
}

impl<T: Send + Sync + 'static> Component for T {}

/// A Trait that can be converted into a Archetype.
pub trait Bundle {
    fn into_component_ptrs( self) -> Box<[(ComponentInfo, NonNull<u8>)]>
        where Self: Sized;
    /// Should be ordered Largest to Smallest
    fn component_info() -> Box<[ComponentInfo]>
        where Self: Sized;
    /// Should be a unique identifier for this Bundle.
    /// this can be used to tell the World to query a specific Archetype
    fn archetype_id() -> u32
        where Self: Sized;
}

impl<C: Component> Bundle for C {
    fn into_component_ptrs(mut self) -> Box<[(ComponentInfo, NonNull<u8>)]> where Self: Sized {
        let c = &mut self as *mut C;
        Box::new([(C::component_info(), NonNull::new(c as *mut u8).unwrap())])
    }

    fn component_info() -> Box<[ComponentInfo]> where Self: Sized {
        Box::new([C::component_info()])
    }

    fn archetype_id() -> u32 where Self: Sized {
        let mut hasher = DefaultHasher::default();
        TypeId::of::<C>().hash(&mut hasher);
        hasher.finish() as u32
    }
}
use crate::archetypes::ComponentInfo;
use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::hint::unreachable_unchecked;
use std::ptr::NonNull;
use std::sync::Arc;
use std::sync::atomic::AtomicU8;
use crate::component_ref::{ComponentRef, MutComponentRef};

pub trait Component: Send + Sync + 'static {
    fn component_info() -> ComponentInfo
        where
            Self: Sized,
    {
        ComponentInfo::new::<Self>()
    }
}

//impl<T: Send + Sync + 'static> Component for T {}

/// A Trait that can be converted into a Archetype.
pub trait Bundle {
    fn into_component_ptrs(self) -> Box<[(ComponentInfo, NonNull<u8>)]>
        where
            Self: Sized;
    /// Should be ordered Largest to Smallest
    fn component_info() -> Vec<ComponentInfo>
        where
            Self: Sized;
    /// Should be a unique identifier for this Bundle.
    /// this can be used to tell the World to query a specific Archetype
    fn archetype_id() -> u32
        where
            Self: Sized;
}

pub trait ComponentLookup<'comp> {
    type MutResponse;
    type RefResponse;
    fn component_info() -> Box<[ComponentInfo]>;

    fn length() -> usize;
    /// It is undefined behavior to call this without all the components.
    unsafe fn return_mut(data: Vec<(Arc<AtomicU8>, *mut u8)>) -> Self::MutResponse where Self: Sized;
    /// It is undefined behavior to call this without all the components.
    unsafe fn return_ref(data: Vec<(Arc<AtomicU8>, *mut u8)>) -> Self::RefResponse where Self: Sized;
}

impl<'comp, C: Component> ComponentLookup<'comp> for C {
    type MutResponse = MutComponentRef<'comp, C>;
    type RefResponse = ComponentRef<'comp, C>;

    fn component_info() -> Box<[ComponentInfo]> {
        Box::new([ComponentInfo::new::<C>()])
    }

    fn length() -> usize {
        1
    }

    unsafe fn return_mut(mut data: Vec<(Arc<AtomicU8>, *mut u8)>) -> Self::MutResponse where Self: Sized {
        let (arc, ptr) = data.remove(0);
        let x = &mut *ptr.cast();
        MutComponentRef {
            component: x,
            ref_count: arc,
        }
    }
    unsafe fn return_ref(mut data: Vec<(Arc<AtomicU8>, *mut u8)>) -> Self::RefResponse where Self: Sized {
        let (arc, ptr) = data.remove(0);
        let x = &*ptr.cast();
        ComponentRef {
            component: x,
            ref_count: arc,
        }
    }
}

impl<'comp, C: Component, D: Component> ComponentLookup<'comp> for (C, D) {
    type MutResponse = (MutComponentRef<'comp, C>, MutComponentRef<'comp, D>);
    type RefResponse = (ComponentRef<'comp, C>, ComponentRef<'comp, D>);

    fn component_info() -> Box<[ComponentInfo]> {
        Box::new([ComponentInfo::new::<C>(), ComponentInfo::new::<D>()])
    }

    fn length() -> usize {
        2
    }

    unsafe fn return_mut(mut data: Vec<(Arc<AtomicU8>, *mut u8)>) -> Self::MutResponse where Self: Sized {
        let ((c_arc, c_data), (d_arc, d_data)) = (data.remove(0), data.remove(0));
        let c_data = &mut *c_data.cast();
        let d_data = &mut *d_data.cast();
        (
            MutComponentRef {
                component: c_data,
                ref_count: c_arc,
            },
            MutComponentRef {
                component: d_data,
                ref_count: d_arc,
            },
        )
    }

    unsafe fn return_ref(mut data: Vec<(Arc<AtomicU8>, *mut u8)>) -> Self::RefResponse where Self: Sized {
        let ((c_arc, c_data), (d_arc, d_data)) = (data.remove(0), data.remove(0));
        let c_data = &*c_data.cast();
        let d_data = &*d_data.cast();
        (
            ComponentRef {
                component: c_data,
                ref_count: c_arc,
            },
            ComponentRef {
                component: d_data,
                ref_count: d_arc,
            },
        )
    }
}




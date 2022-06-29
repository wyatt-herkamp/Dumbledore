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

    unsafe fn return_ref<GE, Failed>(get_entity: GE, failed: Failed) -> Option<Self::RefResponse> where Self: Sized, GE: Fn(&TypeId) -> Option<(Arc<AtomicU8>, *mut u8)>, Failed: Fn(Vec<(Arc<AtomicU8>, *mut u8)>);
    unsafe fn return_mut<GE, Failed>(get_entity: GE, failed: Failed) -> Option<Self::MutResponse> where Self: Sized, GE: Fn(&TypeId) -> Option<(Arc<AtomicU8>, *mut u8)>, Failed: Fn(Vec<(Arc<AtomicU8>, *mut u8)>);
}

impl<'comp, C: Component> ComponentLookup<'comp> for C {
    type MutResponse = MutComponentRef<'comp, C>;
    type RefResponse = ComponentRef<'comp, C>;

    unsafe fn return_ref<GE, Failed>(get_entity: GE, failed: Failed) -> Option<Self::RefResponse> where Self: Sized, GE: Fn(&TypeId) -> Option<(Arc<AtomicU8>, *mut u8)>, Failed: Fn(Vec<(Arc<AtomicU8>, *mut u8)>) {
        if let Some((arc, ptr)) = get_entity(&TypeId::of::<C>()) {
            let x = &*ptr.cast();
            Some(ComponentRef {
                component: x,
                ref_count: arc,
            })
        } else {
            failed(vec![]);
            None
        }
    }

    unsafe fn return_mut<GE, Failed>(get_entity: GE, failed: Failed) -> Option<Self::MutResponse> where Self: Sized, GE: Fn(&TypeId) -> Option<(Arc<AtomicU8>, *mut u8)>, Failed: Fn(Vec<(Arc<AtomicU8>, *mut u8)>) {
        if let Some((arc, ptr)) = get_entity(&TypeId::of::<C>()) {
            let x = &mut *ptr.cast();
            Some(MutComponentRef {
                component: x,
                ref_count: arc,
            })
        } else {
            failed(vec![]);
            None
        }
    }
}
impl<'comp,C: Component, D:Component> ComponentLookup<'comp> for (C,D){
    type MutResponse = (MutComponentRef<'comp, C>,MutComponentRef<'comp, D>);
    type RefResponse = (ComponentRef<'comp, C>,ComponentRef<'comp, D>);

    unsafe fn return_ref<GE, Failed>(get_entity: GE, failed: Failed) -> Option<Self::RefResponse> where Self: Sized, GE: Fn(&TypeId) -> Option<(Arc<AtomicU8>, *mut u8)>, Failed: Fn(Vec<(Arc<AtomicU8>, *mut u8)>) {
       let value = get_entity(&TypeId::of::<C>());
        if value.is_none(){
            failed(vec![]);
            return None;
        }
        let (arc_one, data_one) = value.unwrap();
        let value_two = get_entity(&TypeId::of::<D>());
        if value_two.is_none(){
            failed(vec![(arc_one, data_one)]);
            return None;
        }
        let (arc_two, data_two) = value_two.unwrap();
        Some((ComponentRef {
            component: &*data_one.cast(),
            ref_count: arc_one,
        },ComponentRef {
            component: &*data_two.cast(),
            ref_count: arc_two,
        }))
    }

    unsafe fn return_mut<GE, Failed>(get_entity: GE, failed: Failed) -> Option<Self::MutResponse> where Self: Sized, GE: Fn(&TypeId) -> Option<(Arc<AtomicU8>, *mut u8)>, Failed: Fn(Vec<(Arc<AtomicU8>, *mut u8)>) {
        let value = get_entity(&TypeId::of::<C>());
        if value.is_none(){
            failed(vec![]);
            return None;
        }
        let (arc_one, data_one) = value.unwrap();
        let value_two = get_entity(&TypeId::of::<D>());
        if value_two.is_none(){
            failed(vec![]);
            return None;
        }
        let (arc_two, data_two) = value_two.unwrap();
        Some((MutComponentRef {
            component: &mut *data_one.cast(),
            ref_count: arc_one,
        },MutComponentRef{
            component: &mut *data_two.cast(),
            ref_count: arc_two,
        }))
    }
}
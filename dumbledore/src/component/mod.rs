use crate::archetypes::ComponentInfo;
use std::any::TypeId;

use crate::component_ref::{ComponentRef, MutComponentRef};
use std::sync::atomic::AtomicU8;
use std::sync::Arc;

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
    unsafe fn put_self(self, f: impl FnMut(*mut u8, ComponentInfo))
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
    #[allow(clippy::missing_safety_doc)]
    unsafe fn return_ref<GE>(get_entity: GE) -> Option<Self::RefResponse>
    where
        Self: Sized,
        GE: Fn(&TypeId) -> Option<(Arc<AtomicU8>, *mut u8)>;
    #[allow(clippy::missing_safety_doc)]
    unsafe fn return_mut<GE>(get_entity: GE) -> Option<Self::MutResponse>
    where
        Self: Sized,
        GE: Fn(&TypeId) -> Option<(Arc<AtomicU8>, *mut u8)>;
}

impl<'comp, C: Component> ComponentLookup<'comp> for C {
    type MutResponse = MutComponentRef<'comp, C>;
    type RefResponse = ComponentRef<'comp, C>;

    unsafe fn return_ref<GE>(get_entity: GE) -> Option<Self::RefResponse>
    where
        Self: Sized,
        GE: Fn(&TypeId) -> Option<(Arc<AtomicU8>, *mut u8)>,
    {
        if let Some((arc, ptr)) = get_entity(&TypeId::of::<C>()) {
            let x = &*ptr.cast();
            Some(ComponentRef {
                component: x,
                ref_count: arc,
            })
        } else {
            None
        }
    }

    unsafe fn return_mut<GE>(get_entity: GE) -> Option<Self::MutResponse>
    where
        Self: Sized,
        GE: Fn(&TypeId) -> Option<(Arc<AtomicU8>, *mut u8)>,
    {
        if let Some((arc, ptr)) = get_entity(&TypeId::of::<C>()) {
            let x = &mut *ptr.cast();
            Some(MutComponentRef {
                component: x,
                ref_count: arc,
            })
        } else {
            None
        }
    }
}

///
/// Implements ComponentLookup for a tuple
///
///
/// # Example Expanded Code
///
/// ```no_run, rust, ignore
///
/// impl<'comp,C: Component, D:Component> ComponentLookup<'comp> for (C,D){
///     type MutResponse = (MutComponentRef<'comp, C>,MutComponentRef<'comp, D>);
///     type RefResponse = (ComponentRef<'comp, C>,ComponentRef<'comp, D>);
///
///     unsafe fn return_ref<GE>(get_entity: GE) -> Option<Self::RefResponse> where Self: Sized, GE: Fn(&TypeId) -> Option<(Arc<AtomicU8>, *mut u8)> {
///         let (arc_one, data_one) =get_entity(&TypeId::of::<C>())?;
///         let c = ComponentRef {
///             component: &*data_one.cast(),
///             ref_count: arc_one,
///         };
///         let (arc_two, data_two) = get_entity(&TypeId::of::<D>())?;
///         let d = ComponentRef {
///             component: &*data_two.cast(),
///             ref_count: arc_two,
///         };
///         Some((c,d))
///     }
///
///     unsafe fn return_mut<GE>(get_entity: GE) -> Option<Self::MutResponse> where Self: Sized, GE: Fn(&TypeId) -> Option<(Arc<AtomicU8>, *mut u8)>{
///         let (arc_one, data_one) = get_entity(&TypeId::of::<C>())?;
///        let c = MutComponentRef {
///             component: &mut *data_one.cast(),
///             ref_count: arc_one,
///         };
///         let (arc_two, data_two) = get_entity(&TypeId::of::<D>())?;
///         let d =MutComponentRef{
///             component: &mut *data_two.cast(),
///             ref_count: arc_two,
///         };
///         Some((c,d))
///     }
/// }
/// ```
macro_rules! define_lookup {
    ($($name: ident),*) => {
        #[allow(non_snake_case)]
        impl<'comp, $($name: Component),*> ComponentLookup<'comp> for ($($name,)*){
            type MutResponse = ($(MutComponentRef<'comp, $name>,)*);
            type RefResponse = ($(ComponentRef<'comp, $name>,)*);
            unsafe fn return_ref<GE>(get_entity: GE) -> Option<Self::RefResponse> where Self: Sized, GE: Fn(&TypeId) -> Option<(Arc<AtomicU8>, *mut u8)> {
                $(
                    let (arc, data) = get_entity(&TypeId::of::<$name>())?;
                    let $name = ComponentRef {
                        component: &*data.cast(),
                        ref_count: arc,
                    };
                )*
                Some(($($name,)*))
            }
            unsafe fn return_mut<GE>(get_entity: GE) -> Option<Self::MutResponse> where Self: Sized, GE: Fn(&TypeId) -> Option<(Arc<AtomicU8>, *mut u8)>    {
                $(
                    let (arc, data) = get_entity(&TypeId::of::<$name>())?;
                    let $name = MutComponentRef {
                        component: &mut *data.cast(),
                        ref_count: arc,
                    };
                )*
                Some(($($name,)*))
            }
        }
    }
}

define_lookup!(A, B);
define_lookup!(A, B, C);
define_lookup!(A, B, C, D);
define_lookup!(A, B, C, D, E);
define_lookup!(A, B, C, D, E, F);
define_lookup!(A, B, C, D, E, F, G);
define_lookup!(A, B, C, D, E, F, G, H);
define_lookup!(A, B, C, D, E, F, G, H, I);
define_lookup!(A, B, C, D, E, F, G, H, I, J);
define_lookup!(A, B, C, D, E, F, G, H, I, J, K);
define_lookup!(A, B, C, D, E, F, G, H, I, J, K, L);
define_lookup!(A, B, C, D, E, F, G, H, I, J, K, L, M);
define_lookup!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
define_lookup!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
define_lookup!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
define_lookup!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
define_lookup!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
define_lookup!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
define_lookup!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
define_lookup!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
define_lookup!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
define_lookup!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
define_lookup!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
define_lookup!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
define_lookup!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);


use std::any::TypeId;
use crate::archetypes::ComponentInfo;

pub trait Component: Send + Sync + 'static {
    fn component_info() -> ComponentInfo where Self: Sized {
        ComponentInfo::new::<Self>()
    }
}

impl<T: Send + Sync + 'static> Component for T {}

/// A Grouping of Components.
///
/// Built up of several Components.
///
/// Default implementations found for Tuples.
pub trait Bundle {
    /// Should be ordered Largest to Smallest
    fn create_archetype_id<T>(&self, f: impl FnOnce(&[TypeId]) -> T) -> T;
    /// Should be ordered Largest to Smallest
    fn component_info() -> Vec<ComponentInfo>
        where Self: Sized;
}

impl<C: Component> Bundle for C {
    fn create_archetype_id<T>(&self, f: impl FnOnce(&[TypeId]) -> T) -> T {
        f(&[TypeId::of::<C>()])
    }

    fn component_info() -> Vec<ComponentInfo> where Self: Sized {
        vec![C::component_info()]
    }
}
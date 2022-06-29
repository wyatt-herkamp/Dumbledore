pub mod arche;

use std::alloc::Layout;
use std::any::TypeId;
use std::cmp::Ordering;
use crate::component::Component;

/// The Information about a Component.
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    pub(crate) layout: Layout,
    pub(crate) id: TypeId,
    pub(crate) drop: unsafe fn(*mut u8),
}

impl ComponentInfo {
    pub fn new<T: Component>() -> Self {
        unsafe fn drop_ptr<T>(ptr: *mut u8) {
            ptr.drop_in_place()
        }

        ComponentInfo {
            layout: Layout::new::<T>(),
            id: TypeId::of::<T>(),
            drop: drop_ptr::<T>,
        }
    }
}

impl PartialEq<Self> for ComponentInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ComponentInfo {}


impl Ord for ComponentInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.layout
            .align()
            .cmp(&other.layout.align())
            .reverse()
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for ComponentInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


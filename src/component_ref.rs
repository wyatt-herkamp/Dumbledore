use std::fmt::{Debug, Display, Formatter};

use crate::component::Component;
use std::sync::atomic::AtomicU8;
use std::sync::Arc;

/// A Reference to a Component.
///
/// Drops the Ref Count down when the Component is dropped.
pub struct ComponentRef<'comp, T: Component> {
    pub(crate) component: &'comp T,
    pub(crate) ref_count: Arc<AtomicU8>,
}

impl<T: Component> Drop for ComponentRef<'_, T> {
    fn drop(&mut self) {
        self.ref_count
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }
}

impl<T: Component> AsRef<T> for ComponentRef<'_, T> {
    fn as_ref(&self) -> &T {
        self.component
    }
}

impl<T: Component + PartialEq> PartialEq for ComponentRef<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.component == other.component
    }
}

impl<T: Component + Debug> Debug for ComponentRef<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.component.fmt(f)
    }
}

impl<T: Component + Display> Display for ComponentRef<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.component.fmt(f)
    }
}

impl<T: Component> Clone for ComponentRef<'_, T> {
    fn clone(&self) -> Self {
        self.ref_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self {
            component: self.component,
            ref_count: self.ref_count.clone(),
        }
    }
}

/// A Reference to a Component.
///
/// Drops the Ref Count down when the Component is dropped.
pub struct MutComponentRef<'comp, T: Component> {
    pub(crate) component: &'comp mut T,
    pub(crate) ref_count: Arc<AtomicU8>,
}

impl<T: Component + Debug> Debug for MutComponentRef<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.component.fmt(f)
    }
}

impl<T: Component> Drop for MutComponentRef<'_, T> {
    fn drop(&mut self) {
        self.ref_count
            .fetch_sub(255, std::sync::atomic::Ordering::Relaxed);
    }
}

impl<T: Component> AsRef<T> for MutComponentRef<'_, T> {
    fn as_ref(&self) -> &T {
        self.component
    }
}

impl<T: Component> AsMut<T> for MutComponentRef<'_, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.component
    }
}

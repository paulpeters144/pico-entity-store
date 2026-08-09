use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use crate::entity_ref::EntityRef;
use crate::storage::TypedStorage;
use crate::store::StoreInner;

/// A read guard on the store shared between an [`AllIter`](crate::store::AllIter)
/// and every [`Ref`] it yields.
///
/// A plain `Arc` around the lock guard: cloning a `Ref`'s guard is one atomic
/// increment, and the lock is released when the last reference drops.
pub(crate) struct SharedGuard<'a>(Arc<RwLockReadGuard<'a, StoreInner>>);

impl<'a> SharedGuard<'a> {
    pub(crate) fn new(guard: RwLockReadGuard<'a, StoreInner>) -> Self {
        Self(Arc::new(guard))
    }

    pub(crate) fn clone_ref(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

/// Internal guard ownership for [`Ref`].
///
/// Single-entity lookups (`first`, `get_by_id`) own their read lock
/// directly. Bulk iteration via [`EntityStore::all`](crate::store::EntityStore::all)
/// shares one read lock across every yielded reference through an `Arc`.
pub(crate) enum Guard<'a> {
    Owned(RwLockReadGuard<'a, StoreInner>),
    Shared(SharedGuard<'a>),
}

/// Resolves the slot coordinates held by a reference into the store's storage.
#[inline]
fn resolve<T: 'static>(store: &StoreInner, storage_idx: usize, slot: usize) -> &T {
    let storage = store.storages[storage_idx]
        .as_any()
        .downcast_ref::<TypedStorage<T>>()
        .expect("storage type mismatch");
    &storage.slots[slot].data
}

/// A read guard for a component of type `T`.
///
/// Holds a read lock on the underlying store for the lifetime of the reference.
/// Derefs to `&T`.
pub struct Ref<'a, T: 'static> {
    pub(crate) id: usize,
    pub(crate) slot: usize,
    pub(crate) storage_idx: usize,
    pub(crate) guard: Guard<'a>,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T: 'static> Deref for Ref<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        match &self.guard {
            Guard::Owned(g) => resolve(g, self.storage_idx, self.slot),
            Guard::Shared(g) => resolve(&g.0, self.storage_idx, self.slot),
        }
    }
}

impl<T: 'static> Ref<'_, T> {
    /// Returns the numeric entity id.
    pub fn id(&self) -> u64 {
        self.id as u64
    }

    /// Converts this reference into a type-erased [`EntityRef`].
    pub fn entity_ref(&self) -> EntityRef {
        EntityRef { id: self.id, type_id: std::any::TypeId::of::<T>() }
    }
}

/// A write guard for a component of type `T`.
///
/// Holds a write lock on the underlying store for the lifetime of the reference.
/// Derefs to `&mut T`.
pub struct RefMut<'a, T: 'static> {
    pub(crate) id: usize,
    pub(crate) slot: usize,
    pub(crate) storage_idx: usize,
    pub(crate) guard: RwLockWriteGuard<'a, StoreInner>,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T: 'static> Deref for RefMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        resolve(&self.guard, self.storage_idx, self.slot)
    }
}

impl<T: 'static> DerefMut for RefMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        let storage = self.guard.storages[self.storage_idx]
            .as_any_mut()
            .downcast_mut::<TypedStorage<T>>()
            .expect("storage type mismatch");
        &mut storage.slots[self.slot].data
    }
}

impl<T: 'static> RefMut<'_, T> {
    /// Returns the numeric entity id.
    pub fn id(&self) -> u64 {
        self.id as u64
    }

    /// Converts this reference into a type-erased [`EntityRef`].
    pub fn entity_ref(&self) -> EntityRef {
        EntityRef { id: self.id, type_id: std::any::TypeId::of::<T>() }
    }
}

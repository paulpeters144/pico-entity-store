use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::entity_ref::EntityRef;
use crate::storage::TypedStorage;
use crate::store::StoreInner;

/// Resolves the slot coordinates held by a reference into the store's storage.
#[inline]
fn resolve<T: 'static>(store: &StoreInner, storage_idx: usize, slot: usize) -> &T {
    let storage = store.storages[storage_idx]
        .as_any()
        .downcast_ref::<TypedStorage<T>>()
        .expect("storage type mismatch");
    &storage.data[slot]
}

/// A read guard for a component of type `T`.
///
/// Holds a read lock on the underlying store for the lifetime of the reference.
/// Derefs to `&T`.
pub struct Ref<'a, T: 'static> {
    pub(crate) id: usize,
    pub(crate) slot: usize,
    pub(crate) storage_idx: usize,
    pub(crate) guard: RwLockReadGuard<'a, StoreInner>,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T: 'static> Deref for Ref<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        resolve(&self.guard, self.storage_idx, self.slot)
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
        &mut storage.data[self.slot]
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

// ── Bulk read iterator (all) ─────────────────────────────────────────────

/// A contiguous read-only view of every component of type `T` in the store.
///
/// Created by [`EntityStore::all`](crate::store::EntityStore::all).  Holds a
/// shared read lock for its lifetime; yields `&T` directly with zero
/// per-element overhead (a single pointer walk over a flat `Vec<T>`).
pub struct RefVec<'a, T: 'static> {
    #[allow(dead_code)]
    guard: RwLockReadGuard<'a, StoreInner>,
    ptr: *const T,
    remaining: usize,
    _phantom: PhantomData<T>,
}

impl<'a, T: 'static> RefVec<'a, T> {
    pub(crate) fn from_raw(
        guard: RwLockReadGuard<'a, StoreInner>,
        ptr: *const T,
        len: usize,
    ) -> Self {
        Self { guard, ptr, remaining: len, _phantom: PhantomData }
    }
}

impl<'a, T: 'static> Iterator for RefVec<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if self.remaining == 0 {
            return None;
        }
        let r = unsafe { &*self.ptr };
        self.ptr = unsafe { self.ptr.add(1) };
        self.remaining -= 1;
        Some(r)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }

    fn count(self) -> usize {
        self.remaining
    }
}

impl<'a, T: 'static> std::iter::ExactSizeIterator for RefVec<'a, T> {}

// ── Bulk write iterator (all_mut) ────────────────────────────────────────

/// A contiguous mutable view of every component of type `T` in the store.
///
/// Created by [`EntityStore::all_mut`](crate::store::EntityStore::all_mut).
/// Holds an exclusive write lock for its lifetime; yields `&mut T` directly
/// with zero per-element overhead.
pub struct RefMutVec<'a, T: 'static> {
    #[allow(dead_code)]
    guard: RwLockWriteGuard<'a, StoreInner>,
    ptr: *mut T,
    remaining: usize,
    _phantom: PhantomData<T>,
}

impl<'a, T: 'static> RefMutVec<'a, T> {
    pub(crate) fn from_raw(
        guard: RwLockWriteGuard<'a, StoreInner>,
        ptr: *mut T,
        len: usize,
    ) -> Self {
        Self { guard, ptr, remaining: len, _phantom: PhantomData }
    }
}

impl<'a, T: 'static> Iterator for RefMutVec<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        if self.remaining == 0 {
            return None;
        }
        let r = unsafe { &mut *self.ptr };
        self.ptr = unsafe { self.ptr.add(1) };
        self.remaining -= 1;
        Some(r)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }

    fn count(self) -> usize {
        self.remaining
    }
}

impl<'a, T: 'static> std::iter::ExactSizeIterator for RefMutVec<'a, T> {}

use std::cell::Cell;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::entity_ref::EntityRef;
use crate::store::StoreInner;

struct SharedGuardInner {
    guard: RwLockReadGuard<'static, StoreInner>,
    refs: Cell<usize>,
}

pub(crate) struct SharedGuard(NonNull<SharedGuardInner>);

impl SharedGuard {
    pub(crate) fn new(guard: RwLockReadGuard<'_, StoreInner>) -> Self {
        let inner = Box::new(SharedGuardInner {
            guard: unsafe {
                mem::transmute::<
                    RwLockReadGuard<'_, StoreInner>,
                    RwLockReadGuard<'static, StoreInner>,
                >(guard)
            },
            refs: Cell::new(1),
        });
        Self(unsafe { NonNull::new_unchecked(Box::into_raw(inner)) })
    }

    pub(crate) fn clone_ref(&self) -> Self {
        unsafe {
            self.0.as_ref().refs.set(self.0.as_ref().refs.get() + 1);
        }
        Self(self.0)
    }

    pub(crate) fn store(&self) -> &StoreInner {
        unsafe { &self.0.as_ref().guard }
    }
}

impl Drop for SharedGuard {
    fn drop(&mut self) {
        unsafe {
            let refs = self.0.as_ref().refs.get() - 1;
            if refs == 0 {
                drop(Box::from_raw(self.0.as_ptr()));
            } else {
                self.0.as_ref().refs.set(refs);
            }
        }
    }
}

unsafe impl Send for SharedGuard {}
unsafe impl Sync for SharedGuard {}

/// Internal guard ownership for [`Ref`].
///
/// Single-entity lookups (`first`, `get_by_id`, `resolve`) own their read lock
/// directly. Bulk iteration via [`EntityStore::all`](crate::store::EntityStore::all)
/// shares one read lock across every yielded reference through a `Cell`-based
/// manual refcount (avoids the atomic increment of `Arc::clone`).
#[allow(dead_code)] // variant fields are only ever held for RAII, never read
pub(crate) enum Guard<'a> {
    Owned(RwLockReadGuard<'a, StoreInner>),
    Shared(SharedGuard),
}

/// A read guard for a component of type `T`.
///
/// Holds a read lock on the underlying store for the lifetime of the reference.
/// Derefs to `&T`.
pub struct Ref<'a, T> {
    pub(crate) id: usize,
    pub(crate) ptr: *const T,
    pub(crate) _guard: Guard<'a>,
}

impl<'a, T: 'a> Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &'a T {
        unsafe { &*self.ptr }
    }
}

impl<'a, T: 'static> Ref<'a, T> {
    /// Returns the numeric entity id.
    pub fn id(&self) -> u64 {
        self.id as u64
    }

    /// Converts this reference into a type-erased [`EntityRef`].
    pub fn entity_ref(&self) -> EntityRef {
        EntityRef {
            id: self.id,
            type_id: std::any::TypeId::of::<T>(),
        }
    }
}

/// A write guard for a component of type `T`.
///
/// Holds a write lock on the underlying store for the lifetime of the reference.
/// Derefs to `&mut T`.
pub struct RefMut<'a, T> {
    pub(crate) id: usize,
    pub(crate) ptr: *mut T,
    pub(crate) _guard: RwLockWriteGuard<'a, StoreInner>,
}

impl<'a, T> Deref for RefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<'a, T> DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<'a, T: 'static> RefMut<'a, T> {
    /// Returns the numeric entity id.
    pub fn id(&self) -> u64 {
        self.id as u64
    }

    /// Converts this reference into a type-erased [`EntityRef`].
    pub fn entity_ref(&self) -> EntityRef {
        EntityRef {
            id: self.id,
            type_id: std::any::TypeId::of::<T>(),
        }
    }
}

unsafe impl<T> Send for Ref<'_, T> {}
unsafe impl<T> Sync for Ref<'_, T> {}
unsafe impl<T> Send for RefMut<'_, T> {}
unsafe impl<T> Sync for RefMut<'_, T> {}

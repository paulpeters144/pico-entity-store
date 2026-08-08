use std::ops::{Deref, DerefMut};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::entity_ref::EntityRef;
use crate::store::StoreInner;

pub struct Ref<'a, T> {
    pub(crate) id: usize,
    pub(crate) ptr: *const T,
    pub(crate) _guard: RwLockReadGuard<'a, StoreInner>,
}

impl<'a, T: 'a> Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &'a T {
        unsafe { &*self.ptr }
    }
}

impl<'a, T: 'static> Ref<'a, T> {
    pub fn id(&self) -> u64 {
        self.id as u64
    }

    pub fn entity_ref(&self) -> EntityRef {
        EntityRef {
            id: self.id,
            type_id: std::any::TypeId::of::<T>(),
        }
    }
}

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
    pub fn id(&self) -> u64 {
        self.id as u64
    }

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

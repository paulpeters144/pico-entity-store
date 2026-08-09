use std::any::TypeId;

/// A lightweight, type-erased handle to an entity in an [`EntityStore`](crate::store::EntityStore).
///
/// Can be stored cheaply and later resolved back to a typed [`Ref`](crate::refs::Ref)
/// or [`RefMut`](crate::refs::RefMut) via [`resolve`](crate::store::EntityStore::resolve)
/// or [`resolve_mut`](crate::store::EntityStore::resolve_mut).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityRef {
    pub(crate) id: usize,
    pub(crate) type_id: TypeId,
}

impl EntityRef {
    /// Returns the numeric entity id.
    pub fn id(&self) -> u64 {
        self.id as u64
    }
}

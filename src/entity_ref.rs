use std::any::TypeId;

/// A lightweight, type-erased handle to an entity in an [`EntityStore`](crate::store::EntityStore).
///
/// Can be stored cheaply and later looked up via
/// [`get_by_id`](crate::store::EntityStore::get_by_id) or
/// [`get_by_id_mut`](crate::store::EntityStore::get_by_id_mut).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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

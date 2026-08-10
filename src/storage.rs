use std::any::Any;

/// Type-erased storage for the elements of one component type.
///
/// Implemented by [`TypedStorage`], which stores components in a contiguous
/// `Vec<T>` alongside a parallel `Vec<u64>` of entity IDs.  This keeps
/// iteration (`all`) a zero-overhead pointer walk with no per-element
/// indirection.
///
/// The `Send + Sync` supertrait bound lets `StoreInner` be `Send + Sync`
/// automatically: registering a component type requires `T: Send + Sync`, so
/// every `Box<dyn Storage>` is safe to move and share across threads behind
/// the store's `RwLock`.
#[allow(dead_code)]
pub(crate) trait Storage: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn len(&self) -> usize;
    fn entity_id_at(&self, slot: usize) -> u64;

    /// Returns the entity id of the first slot, or `None` if the storage is empty.
    /// Combines `len() == 0` + `entity_id_at(0)` into a single virtual call.
    fn first_entity_id(&self) -> Option<u64>;

    /// Swaps the last element into `slot`, shrinking the storage by one.
    ///
    /// Returns `Some((displaced_entity_id, last_slot))` when a swap occurred,
    /// or `None` when `slot` was the last element.
    fn swap_remove(&mut self, slot: usize) -> Option<(u64, usize)>;
}

/// Contiguous storage for components of type `T`.
///
/// Components are kept in a raw `Vec<T>` (not wrapped in an `EntitySlot`)
/// so that iteration via [`crate::store::EntityStore::all`] can walk a
/// single flat slice with no per-element indirection or allocation.
/// A parallel `entity_ids: Vec<u64>` tracks which entity occupies each slot.
pub(crate) struct TypedStorage<T: 'static> {
    pub(crate) data: Vec<T>,
    pub(crate) entity_ids: Vec<u64>,
}

impl<T: 'static> TypedStorage<T> {
    pub(crate) fn new() -> Self {
        Self { data: Vec::new(), entity_ids: Vec::new() }
    }
}

impl<T: 'static> TypedStorage<T> {
    pub(crate) fn push(&mut self, entity: T, entity_id: u64) -> usize {
        let slot = self.data.len();
        self.data.push(entity);
        self.entity_ids.push(entity_id);
        slot
    }
}

impl<T: Send + Sync + 'static> Storage for TypedStorage<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn entity_id_at(&self, slot: usize) -> u64 {
        self.entity_ids[slot]
    }

    fn swap_remove(&mut self, slot: usize) -> Option<(u64, usize)> {
        let len = self.data.len();
        if slot >= len {
            return None;
        }

        let last_idx = len - 1;
        self.data.swap_remove(slot);
        self.entity_ids.swap_remove(slot);

        if slot == last_idx {
            None
        } else {
            Some((self.entity_ids[slot], last_idx))
        }
    }

    fn first_entity_id(&self) -> Option<u64> {
        self.entity_ids.first().copied()
    }
}

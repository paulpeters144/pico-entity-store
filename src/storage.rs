use std::any::Any;

pub(crate) struct EntitySlot<T> {
    pub(crate) entity_id: u64,
    pub(crate) data: T,
}

/// Type-erased storage for the elements of one component type.
///
/// Implemented by [`TypedStorage`], which keeps elements in a plain `Vec<EntitySlot<T>>`,
/// so alignment, growth, and dropping are all handled by `Vec`.  `EntitySlot.entity_id`
/// tracks which entity occupies each slot.
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
pub(crate) struct TypedStorage<T: 'static> {
    pub(crate) slots: Vec<EntitySlot<T>>,
}

impl<T: 'static> TypedStorage<T> {
    pub(crate) fn new() -> Self {
        Self { slots: Vec::new() }
    }
}

impl<T: 'static> TypedStorage<T> {
    pub(crate) fn push(&mut self, entity: T, entity_id: u64) -> usize {
        let slot = self.slots.len();
        self.slots.push(EntitySlot {
            data: entity,
            entity_id,
        });
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
        self.slots.len()
    }

    fn entity_id_at(&self, slot: usize) -> u64 {
        self.slots[slot].entity_id
    }

    fn swap_remove(&mut self, slot: usize) -> Option<(u64, usize)> {
        let len = self.slots.len();
        if slot >= len {
            return None;
        }

        let last_idx = len - 1;
        self.slots.swap_remove(slot);

        if slot == last_idx {
            None
        } else {
            Some((self.slots[slot].entity_id, last_idx))
        }
    }

    fn first_entity_id(&self) -> Option<u64> {
        self.slots.first().map(|s| s.entity_id)
    }
}

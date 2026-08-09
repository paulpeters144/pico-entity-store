use std::any::TypeId;
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;
use parking_lot::RwLock;

use crate::entity_ref::EntityRef;
use crate::refs::{Guard, Ref, RefMut, SharedGuard};
use crate::storage::{Storage, TypedStorage};

/// Per-entity bookkeeping, kept in a single parallel array so one cache
/// line covers everything the store needs to know about an entity.
#[derive(Clone, Copy)]
pub(crate) struct EntityMeta {
    pub(crate) type_id: TypeId,
    pub(crate) slot: usize,
    pub(crate) storage_idx: usize,
    pub(crate) parent: u64,
    pub(crate) alive: bool,
}

impl Default for EntityMeta {
    fn default() -> Self {
        Self {
            type_id: TypeId::of::<u8>(),
            slot: 0,
            storage_idx: 0,
            parent: u64::MAX,
            alive: false,
        }
    }
}

// One entity = one 64-byte cache line or less. (TypeId is 16 bytes, so the
// struct packs to 48 rather than the 40 a u64-sized TypeId would give.)
const _: () = assert!(std::mem::size_of::<EntityMeta>() == 48);

pub(crate) struct StoreInner {
    pub(crate) storages: Vec<Box<dyn Storage>>,
    pub(crate) storage_map: HashMap<TypeId, usize>,

    pub(crate) meta: Vec<EntityMeta>,
    pub(crate) children: Vec<Vec<u64>>,
    pub(crate) live_count: usize,

    pub(crate) next_id: usize,

    pub(crate) cached_t0: TypeId,
    pub(crate) cached_i0: usize,
    pub(crate) cached_t1: TypeId,
    pub(crate) cached_i1: usize,
}

impl StoreInner {
    pub(crate) fn new() -> Self {
        Self {
            storages: Vec::new(),
            storage_map: HashMap::new(),
            meta: Vec::new(),
            children: Vec::new(),
            live_count: 0,
            next_id: 0,
            cached_t0: TypeId::of::<u8>(),
            cached_i0: usize::MAX,
            cached_t1: TypeId::of::<u8>(),
            cached_i1: usize::MAX,
        }
    }

    pub(crate) fn allocate_id(&mut self) -> usize {
        self.next_id += 1;
        let id = self.next_id - 1;

        if id >= self.meta.len() {
            self.meta.resize(id + 1, EntityMeta::default());
            self.children.resize(id + 1, Vec::new());
        }

        id
    }

    pub(crate) fn ensure_storage_mut<T: 'static + Send + Sync>(
        &mut self,
        type_id: TypeId,
    ) -> &mut TypedStorage<T> {
        if type_id == self.cached_t0 && self.cached_i0 != usize::MAX {
            return self.typed_storage_mut::<T>(self.cached_i0);
        }
        if type_id == self.cached_t1 && self.cached_i1 != usize::MAX {
            let idx = self.cached_i1;
            self.cached_t1 = self.cached_t0;
            self.cached_i1 = self.cached_i0;
            self.cached_t0 = type_id;
            self.cached_i0 = idx;
            return self.typed_storage_mut::<T>(idx);
        }
        let idx = if let Some(&idx) = self.storage_map.get(&type_id) {
            idx
        } else {
            let idx = self.storages.len();
            self.storages.push(Box::new(TypedStorage::<T>::new()));
            self.storage_map.insert(type_id, idx);
            idx
        };
        self.cached_t1 = self.cached_t0;
        self.cached_i1 = self.cached_i0;
        self.cached_t0 = type_id;
        self.cached_i0 = idx;
        self.typed_storage_mut::<T>(idx)
    }

    #[inline]
    fn typed_storage_mut<T: 'static + Send + Sync>(&mut self, idx: usize) -> &mut TypedStorage<T> {
        self.storages[idx]
            .as_any_mut()
            .downcast_mut::<TypedStorage<T>>()
            .expect("storage type mismatch")
    }

    /// Validates that every child is alive and has no parent yet.
    fn validate_attachable(&self, children: &[EntityRef]) -> Result<(), PicoError> {
        for child in children {
            let cid = child.id;
            if cid >= self.meta.len() {
                return Err(PicoError::EntityNotAlive);
            }
            let m = self.meta[cid];
            if !m.alive {
                return Err(PicoError::EntityNotAlive);
            }
            if m.parent != u64::MAX {
                return Err(PicoError::AlreadyHasParent);
            }
        }
        Ok(())
    }

    /// Links already-validated children to `parent`.
    fn link_children(&mut self, parent: usize, children: &[EntityRef]) {
        let parent_u64 = parent as u64;
        for child in children {
            self.meta[child.id].parent = parent_u64;
            self.children[parent].push(child.id as u64);
        }
    }
}

/// The core entity-component store.
///
/// Entities are identified by a `usize` id assigned at insertion time.
/// Components are stored per-type in contiguous `Vec<T>` buffers and accessed
/// through [`Ref`] / [`RefMut`] guards.
///
/// # Example
///
/// ```rust
/// use pico_entity_store::prelude::*;
///
/// #[derive(Clone)]
/// struct Health(i32);
///
/// let store = EntityStore::new();
/// store.add(Health(100), &[]).unwrap();
///
/// if let Some(h) = store.first::<Health>() {
///     assert_eq!(h.0, 100);
/// }
/// ```
pub struct EntityStore {
    pub(crate) inner: RwLock<StoreInner>,
}

impl EntityStore {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(StoreInner::new()),
        }
    }
}

impl Default for EntityStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors returned by [`EntityStore`] operations.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PicoError {
    /// Attempted to query a component type that was never added.
    TypeNotRegistered,
    /// The entity already has a parent in the hierarchy.
    AlreadyHasParent,
    /// The entity has been removed or was never alive.
    EntityNotAlive,
}

impl std::fmt::Display for PicoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PicoError::TypeNotRegistered => write!(f, "type not registered"),
            PicoError::AlreadyHasParent => write!(f, "entity already has a parent"),
            PicoError::EntityNotAlive => write!(f, "entity is not alive"),
        }
    }
}

impl std::error::Error for PicoError {}

/// Conversion into an [`EntityStore::add`] target.
///
/// Implemented for components (`T`), [`Ref`], and [`RefMut`], which lets the
/// single [`add`](EntityStore::add) method both create new entities (from a
/// component value) and attach children to an existing entity (from a guard).
pub trait IntoAdd<T> {
    #[doc(hidden)]
    fn into_add_target(self) -> AddAction<T>;
}

#[doc(hidden)]
pub enum AddAction<T> {
    New(T),
    Existing(u64),
}

impl<T: 'static + Clone + Send + Sync> IntoAdd<T> for T {
    #[inline]
    fn into_add_target(self) -> AddAction<T> {
        AddAction::New(self)
    }
}

impl<T: 'static> IntoAdd<T> for Ref<'_, T> {
    #[inline]
    fn into_add_target(self) -> AddAction<T> {
        AddAction::Existing(self.id as u64)
    }
}

impl<T: 'static> IntoAdd<T> for RefMut<'_, T> {
    #[inline]
    fn into_add_target(self) -> AddAction<T> {
        AddAction::Existing(self.id as u64)
    }
}

impl EntityStore {
    // ── Count ─────────────────────────────────────────────────────────────

    /// Returns the number of currently alive entities.
    pub fn count(&self) -> usize {
        self.inner.read().live_count
    }

    // ── Add ───────────────────────────────────────────────────────────────

    /// Adds an entity to the store, or attaches children to an existing entity.
    ///
    /// - Pass a **component by value** to create a new entity; the new entity
    ///   id is returned.
    /// - Pass a [`Ref`] or [`RefMut`] to attach `children` to that existing
    ///   entity; the parent's id is returned.
    ///
    /// `children` are [`EntityRef`]s, typically built with the
    /// [`children!`](crate::children) macro. Attachment is all-or-nothing: if
    /// any child is dead or already has a parent, nothing is attached and no
    /// new entity is created.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pico_entity_store::prelude::*;
    ///
    /// #[derive(Clone)]
    /// struct Health(i32);
    ///
    /// let store = EntityStore::new();
    ///
    /// // Create a new entity with no children.
    /// let id = store.add(Health(100), &[]).unwrap();
    ///
    /// // Attach a child to an existing entity.
    /// let child_id = store.add(Health(50), &[]).unwrap();
    /// let parent = store.get_by_id::<Health>(id).unwrap();
    /// let child = store.get_by_id::<Health>(child_id).unwrap();
    /// store.add(parent, &children![child]).unwrap();
    /// ```
    #[inline]
    pub fn add<T: 'static + Clone + Send + Sync>(
        &self,
        target: impl IntoAdd<T>,
        children: &[EntityRef],
    ) -> Result<u64, PicoError> {
        match target.into_add_target() {
            AddAction::New(component) => {
                let type_id = TypeId::of::<T>();
                let mut guard = self.inner.write();
                // Empty-children fast path: skip the validate/link calls.
                if !children.is_empty() {
                    guard.validate_attachable(children)?;
                }
                let id = guard.allocate_id();
                let storage = guard.ensure_storage_mut::<T>(type_id);
                let slot = storage.push(component, id as u64);
                let storage_idx = guard.cached_i0;
                guard.meta[id] = EntityMeta {
                    type_id,
                    slot,
                    storage_idx,
                    parent: u64::MAX,
                    alive: true,
                };
                guard.live_count += 1;
                if !children.is_empty() {
                    guard.link_children(id, children);
                }
                Ok(id as u64)
            }
            AddAction::Existing(parent_id) => {
                let parent = parent_id as usize;
                let mut guard = self.inner.write();
                if parent >= guard.meta.len() || !guard.meta[parent].alive {
                    return Err(PicoError::EntityNotAlive);
                }
                if !children.is_empty() {
                    guard.validate_attachable(children)?;
                    guard.link_children(parent, children);
                }
                Ok(parent_id)
            }
        }
    }

    // ── Remove ────────────────────────────────────────────────────────────

    /// Removes each entity in `entities`, along with all its descendants,
    /// under a single write-lock acquisition. Dead or already-removed
    /// entities are skipped.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pico_entity_store::prelude::*;
    ///
    /// #[derive(Clone)]
    /// struct Health(i32);
    ///
    /// let store = EntityStore::new();
    /// store.add(Health(100), &[]).unwrap();
    ///
    /// let eref = store.first::<Health>().unwrap().entity_ref();
    /// store.remove(&[eref]);
    /// assert_eq!(store.count(), 0);
    /// ```
    pub fn remove(&self, entities: &[EntityRef]) {
        let mut guard = self.inner.write();
        for eref in entities {
            self.remove_internal(&mut guard, eref.id);
        }
    }

    fn remove_internal(&self, guard: &mut StoreInner, id: usize) -> bool {
        if id >= guard.meta.len() || !guard.meta[id].alive {
            return false;
        }
        self.remove_recursive(guard, id);
        true
    }

    fn remove_recursive(&self, guard: &mut StoreInner, id: usize) {
        // Take the child list (moving the buffer, no fresh allocation) so we
        // don't iterate a vec that recursive removals mutate: each child
        // unlinks itself from `children[id]` below, which is a no-op on the
        // empty vec left behind by `take`.
        let children_to_remove = std::mem::take(&mut guard.children[id]);

        for child_id in children_to_remove {
            self.remove_recursive(guard, child_id as usize);
        }

        let m = guard.meta[id];

        if let Some((displaced_id, _last_slot)) = guard.storages[m.storage_idx].swap_remove(m.slot)
        {
            guard.meta[displaced_id as usize].slot = m.slot;
        }

        let parent_id = m.parent;
        if parent_id != u64::MAX {
            let pid = parent_id as usize;
            guard.children[pid].retain(|&c| c != id as u64);
        }

        guard.meta[id].alive = false;
        guard.meta[id].parent = u64::MAX;
        guard.children[id].clear();
        guard.live_count -= 1;
    }

    /// Removes all entities and resets the store.
    pub fn clear(&self) {
        let mut guard = self.inner.write();
        guard.storages.clear();
        guard.storage_map.clear();
        guard.meta.clear();
        guard.children.clear();
        guard.live_count = 0;
        guard.next_id = 0;
        guard.cached_i0 = usize::MAX;
        guard.cached_i1 = usize::MAX;
    }

    // ── Query (read) ──────────────────────────────────────────────────────

    /// Returns a read guard to the first live entity of type `T`, or `None`.
    #[inline]
    pub fn first<T: 'static>(&self) -> Option<Ref<'_, T>> {
        let guard = self.inner.read();
        let &storage_idx = guard.storage_map.get(&TypeId::of::<T>())?;
        let entity_id = guard.storages[storage_idx].first_entity_id()?;
        let id = entity_id as usize;
        let slot = guard.meta[id].slot;
        Some(Ref {
            id,
            slot,
            storage_idx,
            guard: Guard::Owned(guard),
            _phantom: PhantomData,
        })
    }

    /// Returns a read guard to the entity with the given numeric id, or `None`.
    #[inline]
    pub fn get_by_id<T: 'static>(&self, entity_id: u64) -> Option<Ref<'_, T>> {
        let guard = self.inner.read();
        let id = entity_id as usize;
        if id >= guard.meta.len() {
            return None;
        }
        let m = guard.meta[id];
        if !m.alive || m.type_id != TypeId::of::<T>() {
            return None;
        }
        Some(Ref {
            id,
            slot: m.slot,
            storage_idx: m.storage_idx,
            guard: Guard::Owned(guard),
            _phantom: PhantomData,
        })
    }

    /// Resolves an [`EntityRef`] back into a typed read guard, or `None` if the
    /// type doesn't match or the entity is dead.
    pub fn resolve<T: 'static>(&self, entity_ref: &EntityRef) -> Option<Ref<'_, T>> {
        if entity_ref.type_id != TypeId::of::<T>() {
            return None;
        }
        self.get_by_id::<T>(entity_ref.id as u64)
    }

    /// Calls `f` with a shared reference to every live entity of type `T`.
    pub fn each<T: 'static, F: FnMut(&T)>(&self, mut f: F) {
        let guard = self.inner.read();
        let Some(&storage_idx) = guard.storage_map.get(&TypeId::of::<T>()) else {
            return;
        };
        let storage = guard.storages[storage_idx]
            .as_any()
            .downcast_ref::<TypedStorage<T>>()
            .expect("storage type mismatch");
        for slot in &storage.slots {
            f(&slot.data);
        }
    }

    /// Returns an iterator over read guards for all live entities of type `T`.
    ///
    /// The read lock is acquired once and shared across every yielded [`Ref`]
    /// via an `Arc`, so iteration costs a single lock acquisition instead of
    /// one per element.
    pub fn all<T: 'static>(&self) -> AllIter<'_, T> {
        let guard = self.inner.read();
        let (storage_idx, entity_ids) = match guard.storage_map.get(&TypeId::of::<T>()) {
            Some(&storage_idx) => {
                let storage = guard.storages[storage_idx]
                    .as_any()
                    .downcast_ref::<TypedStorage<T>>()
                    .expect("storage type mismatch");
                let ids: Vec<u64> = storage.slots.iter().map(|s| s.entity_id).collect();
                (storage_idx, ids)
            }
            None => (0, Vec::new()),
        };
        AllIter {
            guard: SharedGuard::new(guard),
            pos: 0,
            entity_ids,
            storage_idx,
            _phantom_t: PhantomData,
        }
    }

    // ── Query (write) ─────────────────────────────────────────────────────

    /// Returns a write guard to the first live entity of type `T`, or `None`.
    #[inline]
    pub fn first_mut<T: 'static>(&self) -> Option<RefMut<'_, T>> {
        let guard = self.inner.write();
        let &storage_idx = guard.storage_map.get(&TypeId::of::<T>())?;
        let entity_id = guard.storages[storage_idx].first_entity_id()?;
        let id = entity_id as usize;
        let slot = guard.meta[id].slot;
        Some(RefMut {
            id,
            slot,
            storage_idx,
            guard,
            _phantom: PhantomData,
        })
    }

    /// Returns a write guard to the entity with the given numeric id, or `None`.
    #[inline]
    pub fn get_by_id_mut<T: 'static>(&self, entity_id: u64) -> Option<RefMut<'_, T>> {
        let guard = self.inner.write();
        let id = entity_id as usize;
        if id >= guard.meta.len() {
            return None;
        }
        let m = guard.meta[id];
        if !m.alive || m.type_id != TypeId::of::<T>() {
            return None;
        }
        Some(RefMut {
            id,
            slot: m.slot,
            storage_idx: m.storage_idx,
            guard,
            _phantom: PhantomData,
        })
    }

    /// Resolves an [`EntityRef`] back into a typed write guard, or `None`.
    pub fn resolve_mut<T: 'static>(&self, entity_ref: &EntityRef) -> Option<RefMut<'_, T>> {
        if entity_ref.type_id != TypeId::of::<T>() {
            return None;
        }
        self.get_by_id_mut::<T>(entity_ref.id as u64)
    }

    /// Mutates the component of the given entity in-place. Returns `true` if
    /// the entity was alive and the types matched.
    pub fn update<T: 'static, F: FnOnce(&mut T)>(&self, entity_ref: &EntityRef, f: F) -> bool {
        if entity_ref.type_id != TypeId::of::<T>() {
            return false;
        }
        let mut guard = self.inner.write();
        let id = entity_ref.id;
        if id >= guard.meta.len() || !guard.meta[id].alive {
            return false;
        }
        let Some(&storage_idx) = guard.storage_map.get(&TypeId::of::<T>()) else {
            return false;
        };
        let slot = guard.meta[id].slot;
        let storage = guard.storages[storage_idx]
            .as_any_mut()
            .downcast_mut::<TypedStorage<T>>()
            .expect("storage type mismatch");
        f(&mut storage.slots[slot].data);
        true
    }

    /// Calls `f` with an exclusive reference to every live entity of type `T`.
    pub fn each_mut<T: 'static, F: FnMut(&mut T)>(&self, mut f: F) {
        let mut guard = self.inner.write();
        let Some(&storage_idx) = guard.storage_map.get(&TypeId::of::<T>()) else {
            return;
        };
        let storage = guard.storages[storage_idx]
            .as_any_mut()
            .downcast_mut::<TypedStorage<T>>()
            .expect("storage type mismatch");
        for slot in &mut storage.slots {
            f(&mut slot.data);
        }
    }

    // ── Hierarchy ─────────────────────────────────────────────────────────

    /// Returns the parent of the given entity, or `None`.
    pub fn parent<T: 'static>(&self, entity: &Ref<T>) -> Option<EntityRef> {
        let guard = self.inner.read();
        let parent_id = guard.meta[entity.id].parent;
        if parent_id == u64::MAX {
            return None;
        }
        let pid = parent_id as usize;
        if pid >= guard.meta.len() || !guard.meta[pid].alive {
            return None;
        }
        Some(EntityRef {
            id: pid,
            type_id: guard.meta[pid].type_id,
        })
    }

    /// Returns the direct children of the given entity.
    pub fn children<T: 'static>(&self, entity: &Ref<T>) -> Vec<EntityRef> {
        let guard = self.inner.read();
        guard.children[entity.id]
            .iter()
            .map(|&cid| {
                let id = cid as usize;
                EntityRef {
                    id,
                    type_id: guard.meta[id].type_id,
                }
            })
            .collect()
    }

    /// Returns all descendants (breadth-first) of the given entity.
    pub fn descendants<T: 'static>(&self, entity: &Ref<T>) -> Vec<EntityRef> {
        let guard = self.inner.read();
        let mut result = Vec::new();
        let mut queue: VecDeque<usize> = VecDeque::new();

        for &cid in &guard.children[entity.id] {
            queue.push_back(cid as usize);
        }

        while let Some(id) = queue.pop_front() {
            if guard.meta[id].alive {
                result.push(EntityRef {
                    id,
                    type_id: guard.meta[id].type_id,
                });
                for &gcid in &guard.children[id] {
                    queue.push_back(gcid as usize);
                }
            }
        }

        result
    }

    // ── Liveness ──────────────────────────────────────────────────────────

    /// Returns `true` if the entity is still alive in the store.
    pub fn is_alive<T: 'static>(&self, entity: &Ref<T>) -> bool {
        let guard = self.inner.read();
        entity.id < guard.meta.len() && guard.meta[entity.id].alive
    }
}

/// Collects [`EntityRef`](crate::entity_ref::EntityRef)s from
/// [`Ref`](crate::refs::Ref) / [`RefMut`](crate::refs::RefMut) handles and
/// drops the guards, returning an `[EntityRef; N]` array suitable for
/// [`add`](crate::store::EntityStore::add).
///
/// Dropping the guards releases their locks, so the result can be passed
/// straight into `add` (which acquires a write lock) without deadlocking.
///
/// # Example
///
/// ```rust
/// use pico_entity_store::prelude::*;
///
/// #[derive(Clone)] struct Node;
///
/// let store = EntityStore::new();
/// store.add(Node, &[]).unwrap();
/// store.add(Node, &[]).unwrap();
///
/// let parent = store.first::<Node>().unwrap();
/// let child = store.get_by_id::<Node>(1).unwrap();
/// store.add(parent, &children![child]).unwrap();
/// ```
#[macro_export]
macro_rules! children {
    ($($child:expr),+ $(,)?) => {{
        let erefs = [$($child.entity_ref()),+];
        $(drop($child);)+
        erefs
    }};
}

// ── AllIter ──────────────────────────────────────────────────────────────

/// An iterator over read guards for all live entities of a given type.
///
/// Created by [`EntityStore::all`].
///
/// Acquires the store's read lock once and shares it (via an `Arc`) with every
/// yielded reference, so iteration costs a single lock acquisition instead of
/// one per element. The lock is held for as long as the iterator or any
/// yielded [`Ref`] is alive, which also guarantees the storage is stable: no
/// per-element liveness or bounds re-checks are needed.
pub struct AllIter<'a, T: 'static> {
    guard: SharedGuard<'a>,
    pos: usize,
    entity_ids: Vec<u64>,
    storage_idx: usize,
    _phantom_t: PhantomData<T>,
}

impl<'a, T: 'static> Iterator for AllIter<'a, T> {
    type Item = Ref<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.entity_ids.len() {
            return None;
        }
        let slot = self.pos;
        self.pos += 1;

        let entity_id = self.entity_ids[slot];
        Some(Ref {
            id: entity_id as usize,
            slot,
            storage_idx: self.storage_idx,
            guard: Guard::Shared(self.guard.clone_ref()),
            _phantom: PhantomData,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.entity_ids.len() - self.pos;
        (remaining, Some(remaining))
    }

    fn count(self) -> usize {
        self.entity_ids.len() - self.pos
    }
}

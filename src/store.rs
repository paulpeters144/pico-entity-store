use std::any::TypeId;
use std::collections::{HashMap, VecDeque};
use std::sync::RwLock;

use crate::entity_ref::EntityRef;
use crate::refs::{Ref, RefMut};
use crate::storage::TypedStorage;

pub(crate) struct StoreInner {
    pub(crate) storages: Vec<TypedStorage>,
    pub(crate) storage_map: HashMap<TypeId, usize>,

    pub(crate) type_storage_idx: Vec<usize>,

    pub(crate) type_ids: Vec<TypeId>,
    pub(crate) slots: Vec<usize>,
    pub(crate) parents: Vec<u64>,
    pub(crate) children: Vec<Vec<u64>>,
    pub(crate) alive: Vec<bool>,

    pub(crate) next_id: usize,
}

impl StoreInner {
    pub(crate) fn new() -> Self {
        Self {
            storages: Vec::new(),
            storage_map: HashMap::new(),
            type_storage_idx: Vec::new(),
            type_ids: Vec::new(),
            slots: Vec::new(),
            parents: Vec::new(),
            children: Vec::new(),
            alive: Vec::new(),
            next_id: 0,
        }
    }

    pub(crate) fn count(&self) -> usize {
        self.alive.iter().filter(|&&a| a).count()
    }

    pub(crate) fn allocate_id(&mut self) -> usize {
        self.next_id += 1;
        let id = self.next_id - 1;

        if id >= self.type_ids.len() {
            self.type_ids.resize(id + 1, TypeId::of::<u8>());
            self.slots.resize(id + 1, 0);
            self.parents.resize(id + 1, u64::MAX);
            self.children.resize(id + 1, Vec::new());
            self.alive.resize(id + 1, false);
            self.type_storage_idx.resize(id + 1, 0);
        }

        id
    }

    pub(crate) fn ensure_storage<T: 'static>(&mut self) -> usize {
        let type_id = TypeId::of::<T>();
        if let Some(&idx) = self.storage_map.get(&type_id) {
            return idx;
        }
        let storage = TypedStorage::new::<T>();
        let idx = self.storages.len();
        self.storages.push(storage);
        self.storage_map.insert(type_id, idx);
        idx
    }
}

/// The core entity-component store.
///
/// Entities are identified by a `usize` id assigned at insertion time.
/// Components are stored per-type in contiguous byte buffers and accessed
/// through [`Ref`] / [`RefMut`] guards.
///
/// # Example
///
/// ```rust
/// use pico_ecs::prelude::*;
///
/// #[derive(Clone)]
/// struct Health(i32);
///
/// let store = EntityStore::new();
/// store.add(&Health(100));
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

impl EntityStore {
    // ── Count ─────────────────────────────────────────────────────────────

    /// Returns the number of currently alive entities.
    pub fn count(&self) -> usize {
        self.inner.read().unwrap().count()
    }

    // ── Add ───────────────────────────────────────────────────────────────

    /// Adds a component to the store. The entity is assigned a new id.
    pub fn add<T: 'static + Clone>(&self, entity: &T) {
        let mut guard = self.inner.write().unwrap();
        let storage_idx = guard.ensure_storage::<T>();
        let storage = &mut guard.storages[storage_idx];
        let slot = storage.push_raw(entity);
        let id = guard.allocate_id();
        guard.type_ids[id] = TypeId::of::<T>();
        guard.slots[id] = slot;
        guard.alive[id] = true;
        guard.type_storage_idx[id] = storage_idx;
    }

    // ── Query (read) ──────────────────────────────────────────────────────

    /// Returns a read guard to the first live entity of type `T`, or `None`.
    pub fn first<T: 'static>(&self) -> Option<Ref<'_, T>> {
        let guard = self.inner.read().unwrap();
        let storage_idx = guard.storage_map.get(&TypeId::of::<T>())?;
        let storage = &guard.storages[*storage_idx];

        for id in 0..guard.alive.len() {
            if guard.alive[id]
                && guard.type_ids[id] == TypeId::of::<T>()
                && guard.slots[id] < storage.len()
            {
                let ptr = storage.raw_ptr::<T>(guard.slots[id]);
                return Some(Ref {
                    id,
                    ptr,
                    _guard: guard,
                });
            }
        }
        None
    }

    /// Returns a read guard to the entity with the given numeric id, or `None`.
    pub fn get_by_id<T: 'static>(&self, entity_id: u64) -> Option<Ref<'_, T>> {
        let guard = self.inner.read().unwrap();
        let id = entity_id as usize;
        if id >= guard.alive.len()
            || !guard.alive[id]
            || guard.type_ids[id] != TypeId::of::<T>()
        {
            return None;
        }
        let &storage_idx = guard.storage_map.get(&TypeId::of::<T>())?;
        let storage = &guard.storages[storage_idx];
        let ptr = storage.raw_ptr::<T>(guard.slots[id]);
        Some(Ref {
            id,
            ptr,
            _guard: guard,
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
        let guard = self.inner.read().unwrap();
        let Some(&storage_idx) = guard.storage_map.get(&TypeId::of::<T>()) else {
            return;
        };
        let storage = &guard.storages[storage_idx];
        for id in 0..guard.alive.len() {
            if guard.alive[id] && guard.type_ids[id] == TypeId::of::<T>() {
                f(storage.get::<T>(guard.slots[id]));
            }
        }
    }

    /// Returns an iterator over read guards for all live entities of type `T`.
    pub fn all<T: 'static>(&self) -> AllIter<'_, T> {
        let ids: Vec<usize>;
        let storage_idx: usize;

        {
            let guard = self.inner.read().unwrap();
            let Some(&si) = guard.storage_map.get(&TypeId::of::<T>()) else {
                return AllIter {
                    store: &self.inner,
                    ids: Vec::new(),
                    pos: 0,
                    storage_idx: 0,
                    _phantom: std::marker::PhantomData,
                };
            };
            storage_idx = si;
            ids = guard
                .alive
                .iter()
                .enumerate()
                .filter(|(idx, a)| **a && guard.type_ids[*idx] == TypeId::of::<T>())
                .map(|(id, _)| id)
                .collect();
        }

        AllIter {
            store: &self.inner,
            ids,
            pos: 0,
            storage_idx,
            _phantom: std::marker::PhantomData,
        }
    }

    // ── Query (write) ─────────────────────────────────────────────────────

    /// Returns a write guard to the first live entity of type `T`, or `None`.
    pub fn first_mut<T: 'static>(&self) -> Option<RefMut<'_, T>> {
        let mut guard = self.inner.write().unwrap();
        let &storage_idx = guard.storage_map.get(&TypeId::of::<T>())?;

        for id in 0..guard.alive.len() {
            let slot = guard.slots[id];
            if guard.alive[id]
                && guard.type_ids[id] == TypeId::of::<T>()
                && slot < guard.storages[storage_idx].len()
            {
                let ptr = guard.storages[storage_idx].mut_raw_ptr::<T>(slot);
                return Some(RefMut {
                    id,
                    ptr,
                    _guard: guard,
                });
            }
        }
        None
    }

    /// Returns a write guard to the entity with the given numeric id, or `None`.
    pub fn get_by_id_mut<T: 'static>(&self, entity_id: u64) -> Option<RefMut<'_, T>> {
        let mut guard = self.inner.write().unwrap();
        let id = entity_id as usize;
        if id >= guard.alive.len()
            || !guard.alive[id]
            || guard.type_ids[id] != TypeId::of::<T>()
        {
            return None;
        }
        let &storage_idx = guard.storage_map.get(&TypeId::of::<T>())?;
        let slot = guard.slots[id];
        let ptr = guard.storages[storage_idx].mut_raw_ptr::<T>(slot);
        Some(RefMut {
            id,
            ptr,
            _guard: guard,
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
        let mut guard = self.inner.write().unwrap();
        let id = entity_ref.id;
        if id >= guard.alive.len() || !guard.alive[id] {
            return false;
        }
        let Some(&storage_idx) = guard.storage_map.get(&TypeId::of::<T>()) else {
            return false;
        };
        let slot = guard.slots[id];
        f(guard.storages[storage_idx].get_mut::<T>(slot));
        true
    }

    /// Calls `f` with an exclusive reference to every live entity of type `T`.
    pub fn each_mut<T: 'static, F: FnMut(&mut T)>(&self, mut f: F) {
        let mut guard = self.inner.write().unwrap();
        let Some(&storage_idx) = guard.storage_map.get(&TypeId::of::<T>()) else {
            return;
        };

        let stable_ids: Vec<(usize, usize)> = (0..guard.alive.len())
            .filter(|&id| guard.alive[id] && guard.type_ids[id] == TypeId::of::<T>())
            .map(|id| (id, guard.slots[id]))
            .collect();

        for (_, slot) in stable_ids {
            f(guard.storages[storage_idx].get_mut::<T>(slot));
        }
    }

    // ── Hierarchy ─────────────────────────────────────────────────────────

    /// Makes `child` a child of `parent`. Fails if the child already has a
    /// parent or either entity is dead.
    pub fn add_child<P: 'static, C: 'static>(
        &self,
        parent: Ref<P>,
        child: Ref<C>,
    ) -> Result<(), PicoError> {
        let parent_id = parent.id;
        let child_id = child.id;
        drop(parent);
        drop(child);

        let mut guard = self.inner.write().unwrap();

        if !guard.alive[parent_id] || !guard.alive[child_id] {
            return Err(PicoError::EntityNotAlive);
        }

        if guard.parents[child_id] != u64::MAX {
            return Err(PicoError::AlreadyHasParent);
        }

        let parent_id_u64 = parent_id as u64;
        let child_id_u64 = child_id as u64;

        guard.parents[child_id] = parent_id_u64;
        guard.children[parent_id].push(child_id_u64);

        Ok(())
    }

    /// Attaches multiple children (by id) to a parent (by id) in one call.
    pub fn add_children_ids(
        &self,
        parent: u64,
        children: &[u64],
    ) -> Result<(), PicoError> {
        let parent_usize = parent as usize;
        let mut guard = self.inner.write().unwrap();

        for &cid_u64 in children {
            let cid = cid_u64 as usize;
            if !guard.alive[parent_usize] || !guard.alive[cid] {
                return Err(PicoError::EntityNotAlive);
            }
            if guard.parents[cid] != u64::MAX {
                return Err(PicoError::AlreadyHasParent);
            }
        }

        for &cid_u64 in children {
            let cid = cid_u64 as usize;
            guard.parents[cid] = parent;
            guard.children[parent_usize].push(cid_u64);
        }

        Ok(())
    }
}

/// Collects child entity ids from [`Ref`](crate::refs::Ref) handles and drops
/// the guards, returning a `Vec<u64>` suitable for
/// [`add_children`](EntityStore::add_children).
///
/// # Example
///
/// ```rust
/// use pico_ecs::prelude::*;
///
/// #[derive(Clone)] struct Node;
///
/// let store = EntityStore::new();
/// store.add(&Node);
/// store.add(&Node);
///
/// let a = store.first::<Node>().unwrap();
/// let b = store.first::<Node>().unwrap();
/// let ids = children![a, b];
/// assert_eq!(ids.len(), 2);
/// ```
#[macro_export]
macro_rules! children {
    ($($child:expr),+ $(,)?) => {{
        let ids = [$($child.id()),+];
        $(drop($child);)+
        ids
    }};
}

impl EntityStore {
    /// Attaches multiple children (by id) to a parent reference.
    pub fn add_children<P: 'static>(
        &self,
        parent: Ref<P>,
        children: &[u64],
    ) -> Result<(), PicoError> {
        let parent_id = parent.id();
        drop(parent);
        self.add_children_ids(parent_id, children)
    }
    /// Returns the parent of the given entity, or `None`.
    pub fn parent<T: 'static>(&self, entity: &Ref<T>) -> Option<EntityRef> {
        let guard = self.inner.read().unwrap();
        let parent_id = guard.parents[entity.id];
        if parent_id == u64::MAX {
            return None;
        }
        let pid = parent_id as usize;
        if pid >= guard.alive.len() || !guard.alive[pid] {
            return None;
        }
        Some(EntityRef {
            id: pid,
            type_id: guard.type_ids[pid],
        })
    }

    /// Returns the direct children of the given entity.
    pub fn children<T: 'static>(&self, entity: &Ref<T>) -> Vec<EntityRef> {
        let guard = self.inner.read().unwrap();
        guard.children[entity.id]
            .iter()
            .map(|&cid| {
                let id = cid as usize;
                EntityRef {
                    id,
                    type_id: guard.type_ids[id],
                }
            })
            .collect()
    }

    /// Returns all descendants (breadth-first) of the given entity.
    pub fn descendants<T: 'static>(&self, entity: &Ref<T>) -> Vec<EntityRef> {
        let guard = self.inner.read().unwrap();
        let mut result = Vec::new();
        let mut queue: VecDeque<usize> = VecDeque::new();

        for &cid in &guard.children[entity.id] {
            queue.push_back(cid as usize);
        }

        while let Some(id) = queue.pop_front() {
            if guard.alive[id] {
                result.push(EntityRef {
                    id,
                    type_id: guard.type_ids[id],
                });
                for &gcid in &guard.children[id] {
                    queue.push_back(gcid as usize);
                }
            }
        }

        result
    }

    // ── Removal ───────────────────────────────────────────────────────────

    /// Removes an entity and all its descendants. Returns `true` if the entity
    /// was alive.
    pub fn remove<T: 'static>(&self, entity: Ref<T>) -> bool {
        let id = entity.id;
        drop(entity);

        let mut guard = self.inner.write().unwrap();
        self.remove_internal(&mut guard, id)
    }

    fn remove_internal(&self, guard: &mut StoreInner, id: usize) -> bool {
        if id >= guard.alive.len() || !guard.alive[id] {
            return false;
        }

        let children_to_remove: Vec<usize> = guard.children[id]
            .iter()
            .map(|&c| c as usize)
            .collect();
        drop(children_to_remove);

        for child_id in &guard.children[id].clone() {
            self.remove_recursive(guard, *child_id as usize);
        }

        let storage_idx = guard.type_storage_idx[id];
        let slot = guard.slots[id];
        let _old_type_id = guard.type_ids[id];

        if guard.storages[storage_idx].swap_remove::<u8>(slot).is_some() {
            for search_id in 0..guard.type_ids.len() {
                if guard.alive[search_id]
                    && guard.type_storage_idx[search_id] == storage_idx
                    && guard.slots[search_id] == guard.storages[storage_idx].len()
                    && search_id != id
                {
                    guard.slots[search_id] = slot;
                    break;
                }
            }
        }

        let parent_id = guard.parents[id];
        if parent_id != u64::MAX {
            let pid = parent_id as usize;
            guard.children[pid].retain(|&c| c != id as u64);
        }

        guard.alive[id] = false;
        guard.children[id].clear();
        guard.parents[id] = u64::MAX;

        true
    }

    fn remove_recursive(&self, guard: &mut StoreInner, id: usize) {
        if id >= guard.alive.len() || !guard.alive[id] {
            return;
        }

        let children_to_remove: Vec<usize> =
            guard.children[id].iter().map(|&c| c as usize).collect();

        for child_id in children_to_remove {
            self.remove_recursive(guard, child_id);
        }

        let storage_idx = guard.type_storage_idx[id];
        let slot = guard.slots[id];

        if guard.storages[storage_idx].swap_remove::<u8>(slot).is_some() {
            for search_id in 0..guard.type_ids.len() {
                if guard.alive[search_id]
                    && guard.type_storage_idx[search_id] == storage_idx
                    && guard.slots[search_id] == guard.storages[storage_idx].len()
                    && search_id != id
                {
                    guard.slots[search_id] = slot;
                    break;
                }
            }
        }

        let parent_id = guard.parents[id];
        if parent_id != u64::MAX {
            let pid = parent_id as usize;
            guard.children[pid].retain(|&c| c != id as u64);
        }

        guard.alive[id] = false;
        guard.children[id].clear();
        guard.parents[id] = u64::MAX;
    }

    /// Removes an entity by numeric id (including descendants). Returns `true`
    /// if the entity was alive.
    pub fn remove_by_id(&self, entity_id: u64) -> bool {
        let mut guard = self.inner.write().unwrap();
        let id = entity_id as usize;
        self.remove_internal(&mut guard, id)
    }

    /// Removes all entities and resets the store.
    pub fn clear(&self) {
        let mut guard = self.inner.write().unwrap();
        guard.storages.clear();
        guard.storage_map.clear();
        guard.type_storage_idx.clear();
        guard.type_ids.clear();
        guard.slots.clear();
        guard.parents.clear();
        guard.children.clear();
        guard.alive.clear();
        guard.next_id = 0;
    }

    /// Returns `true` if the entity is still alive in the store.
    pub fn is_alive<T: 'static>(&self, entity: &Ref<T>) -> bool {
        let guard = self.inner.read().unwrap();
        entity.id < guard.alive.len() && guard.alive[entity.id]
    }
}

// ── AllIter ──────────────────────────────────────────────────────────────

/// An iterator over read guards for all live entities of a given type.
///
/// Created by [`EntityStore::all`].
pub struct AllIter<'a, T> {
    store: &'a RwLock<StoreInner>,
    ids: Vec<usize>,
    pos: usize,
    storage_idx: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T: 'static> Iterator for AllIter<'a, T> {
    type Item = Ref<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.ids.len() {
            let id = self.ids[self.pos];
            self.pos += 1;

            let guard = self.store.read().unwrap();

            if id < guard.alive.len()
                && guard.alive[id]
                && guard.type_ids[id] == TypeId::of::<T>()
            {
                let storage = &guard.storages[self.storage_idx];
                let slot = guard.slots[id];
                if slot < storage.len() {
                    let ptr = storage.raw_ptr::<T>(slot);
                    return Some(Ref {
                        id,
                        ptr,
                        _guard: guard,
                    });
                }
            }
        }
        None
    }
}

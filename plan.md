# PicoEntityStore — Rust Implementation Plan

A fast, thread-safe, hierarchical entity store. Users work with their own structs directly — `Ref<T>` derefs to `&T` so it feels like using the plain struct, plus it carries the store-assigned ID. No handles, no `Entity<T>` wrapper in the primary API. One allocation per type (type-erased `Vec<u8>`), zero per-entity heap allocations, zero external dependencies.

---

## Usage

```rust
use pico_ecs::prelude::*;

struct Dwarf { name: String, health: i32 }
struct Axe   { damage: u32, durability: u32 }

let store = EntityStore::new();

// 1. Add entities — takes &T, clones into store
store.add(&Dwarf { name: "Gimli".into(), health: 100 });
store.add(&Axe { damage: 45, durability: 80 });

// 2. Get entities back — Ref<Dwarf> derefs to &Dwarf
//    So the user works with their Dwarf struct directly.
{
    let d1 = store.first::<Dwarf>().unwrap();
    let a1 = store.first::<Axe>().unwrap();

    println!("{} has {} hp", d1.name, d1.health);  // Deref → feels like Dwarf
    println!("id: {}", d1.id());                     // Ref<T>::id() method

    // 3. Hierarchy — pass the entities directly
    store.add_child(d1, a1).unwrap();
} // read locks released when Refs go out of scope

// 4. Query children — EntityRef can be resolved to typed entity
let d2 = store.first::<Dwarf>().unwrap();
for child_ref in store.children(&d2) {
    if let Some(a) = store.resolve::<Axe>(&child_ref) {
        // a is Ref<Axe> — Deref works, .id() works
        println!("Axe (id={}) damage: {}", a.id(), a.damage);
    }
}

// 5. Bulk iteration — &T passed to callback, no Ref needed
store.each::<Axe>(|a| {
    println!("Axe damage: {}", a.damage);
});

// 6. All entities of a type — iterator of Ref<T>
//    Each Ref<T> holds its own read lock guard.
for d in store.all::<Dwarf>() {
    println!("{} (id={})", d.name, d.id());
}

// 7. Remove by entity — consumes the Ref
let d3 = store.first::<Dwarf>().unwrap();
store.remove(d3);

// 8. Get by ID when you've saved one elsewhere
if let Some(d) = store.get_by_id::<Dwarf>(42) {
    println!("{}", d.name);
}

// 9. Parent of an entity
let d4 = store.first::<Dwarf>().unwrap();
if let Some(parent_ref) = store.parent(&d4) {
    println!("Parent id: {}", parent_ref.id());
}
```

---

## Mutation

```rust
// Single entity mutation — RefMut<T> derefs to &mut T
{
    let mut d = store.first_mut::<Dwarf>().unwrap();
    d.health -= 30;                          // direct field mutation
    println!("{} now has {} hp (id={})", d.name, d.health, d.id());
} // write lock released

// Bulk mutation — each_mut gives &mut T to the callback
store.each_mut::<Axe>(|a| {
    a.durability -= 1;
    a.damage += 5;
});

// Get by ID and mutate
if let Some(mut a) = store.get_by_id_mut::<Axe>(1) {
    a.damage += 10;
}

// Update a single entity by EntityRef — closure gets &mut T
store.update::<Dwarf>(&entity_ref, |d| {
    d.health -= 10;
});
```

---

## Core Public Types

### `Ref<T>` — THE primary entity reference

```rust
pub struct Ref<'a, T> {
    id: usize,
    ptr: *const T,
    _guard: RwLockReadGuard<'a, StoreInner>,
}

impl<'a, T> Deref for Ref<'a, T> {
    type Target = T;
    fn deref(&self) -> &'a T {
        unsafe { &*self.ptr }
    }
}

impl<'a, T> Ref<'a, T> {
    pub fn id(&self) -> u64 {
        self.id as u64
    }
}
```

This IS the entity from the user's perspective:
- `ref.name` — works via Deref (no wrapper visible)
- `ref.id()` — store-assigned numeric ID
- Holds the store's read lock — released on drop

### `RefMut<T>` — Mutable entity reference

```rust
pub struct RefMut<'a, T> {
    id: usize,
    ptr: *mut T,
    _guard: RwLockWriteGuard<'a, StoreInner>,
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

impl<'a, T> RefMut<'a, T> {
    pub fn id(&self) -> u64 { self.id as u64 }
}
```

Holds an exclusive write lock — no reads or other writes can happen while it's alive. Implements both `Deref` and `DerefMut`.

### `EntityRef` — Type-erased reference (for `children()`, `parent()`)

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityRef {
    id: usize,
    type_id: TypeId,
}

impl EntityRef {
    pub fn id(&self) -> u64 { self.id as u64 }
}
```

Used when a method returns entities of unknown/varying types. Resolvable to `Ref<T>` via `store.resolve::<T>(&eref)`.

---

## Public API

| Method | Returns | Lock |
|---|---|---|
| `store.new()` | `EntityStore` | — |
| `store.count()` | `usize` | read |
| `store.add::<T>(&entity)` | — | write |
| `store.first::<T>()` | `Option<Ref<T>>` | read* |
| `store.first_mut::<T>()` | `Option<RefMut<T>>` | write* |
| `store.all::<T>()` | `impl Iterator<Item = Ref<T>>` | — |
| `store.each::<T>(f)` | — | read |
| `store.each_mut::<T>(f)` | — | write |
| `store.get_by_id::<T>(id)` | `Option<Ref<T>>` | read* |
| `store.get_by_id_mut::<T>(id)` | `Option<RefMut<T>>` | write* |
| `store.resolve::<T>(&eref)` | `Option<Ref<T>>` | read* |
| `store.resolve_mut::<T>(&eref)` | `Option<RefMut<T>>` | write* |
| `store.update::<T>(&eref, f)` | `bool` | write |
| `store.add_child(parent, child)` | `Result<(), Error>` | write |
| `store.parent(&ref)` | `Option<EntityRef>` | read |
| `store.children(&ref)` | `Vec<EntityRef>` | read |
| `store.descendants(&ref)` | `Vec<EntityRef>` | read |
| `store.remove(ref)` | `bool` | write |
| `store.remove_by_id(id)` | `bool` | write |
| `store.clear()` | — | write |
| `store.is_alive(&ref)` | `bool` | read |

\* Returns `Ref<T>` which holds the read lock until dropped.

### Hierarchy method signatures (ownership model)

`add_child` and `remove` take `Ref<T>` **by value** — they consume the Ref, extract the internal ID, drop the Ref (releasing the read lock), then acquire a write lock to do the mutation. This avoids deadlocks (no read guard held while acquiring write).

```rust
pub fn add_child<P: 'static, C: 'static>(
    &self,
    parent: Ref<P>,
    child: Ref<C>,
) -> Result<(), Error> { ... }

pub fn remove<T: 'static>(&self, entity: Ref<T>) -> bool { ... }
```

Methods that query (`parent`, `children`, `descendants`, `is_alive`) take `&Ref<T>` — they read the id within the existing read lock.

### Trait Bounds

- `'static` on all type parameters (for `TypeId::of::<T>()`)
- **No derives required** on entity structs. Plain `struct Dwarf { ... }` is sufficient.
- `add()` internally clones via `memcpy`-equivalent byte copy — no `Clone` trait needed since we store raw bytes with `ptr::write`.

---

## How `add()` Works Without `Clone` Bound

```rust
pub fn add<T: 'static>(&self, entity: &T) {
    let mut guard = self.inner.write().unwrap();
    let storage = guard.ensure_storage::<T>();
    let slot = storage.push_raw(entity);  // ptr::read + ptr::write bytes
    let id = guard.allocate_id();
    guard.type_ids[id] = TypeId::of::<T>();
    guard.slots[id] = slot;
}
```

`push_raw` does a byte copy via `ptr::read`/`ptr::write` — works for any `T`. No `Clone` bound. Caveat: `T` must not be `Pin`, must be `Unpin` (trivially moveable), which all normal game structs are.

---

## Internal Types

### `StoreInner` (`src/store.rs`)

```rust
pub(crate) struct StoreInner {
    // Type-erased storage — one TypedStorage per registered type
    storages: Vec<TypedStorage>,
    storage_map: HashMap<TypeId, usize>,

    // Per-type storage index (allows heterogeneous children resolution)
    type_storage_idx: Vec<usize>,

    // Dense parallel arrays indexed by entity id (usize)
    type_ids: Vec<TypeId>,
    slots: Vec<usize>,
    parents: Vec<u64>,
    children: Vec<Vec<u64>>,
    alive: Vec<bool>,

    free_ids: Vec<usize>,
}
```

### `TypedStorage` (`src/storage.rs`)

```rust
pub(crate) struct TypedStorage {
    type_id: TypeId,
    data: Vec<u8>,       // raw bytes with correct alignment
    len: usize,
    elem_size: usize,
    alignment: usize,
    padding: usize,
    drop_fn: unsafe fn(*mut u8, usize),
}
```

Methods: `new::<T>()`, `push_raw::<T>(&T) → slot`, `get::<T>(slot) → &T`, `get_mut::<T>(slot) → &mut T`, `raw_ptr::<T>(slot) → *const T`, `mut_raw_ptr::<T>(slot) → *mut T`, `swap_remove::<T>(slot, last) → Option<slot>`.

---

## Files

```
pico-ecs/
├── Cargo.toml
├── plan.md
└── src/
    ├── lib.rs
    ├── entity_ref.rs     (EntityRef)
    ├── storage.rs         (TypedStorage)
    ├── store.rs           (EntityStore + StoreInner)
    └── refs.rs            (Ref<T>)
```

---

## Implementation Order

| Step | File | What |
|---|---|---|
| 1 | `entity_ref.rs` | `EntityRef` — id + type_id |
| 2 | `storage.rs` | `TypedStorage` — new, push_raw, get, raw_ptr, swap_remove |
| 3 | `store.rs` | `StoreInner`, `EntityStore::new()`, `count()`, `allocate_id()` |
| 4 | `store.rs` | `ensure_storage()`, `add::<T>(&T)` |
| 5 | `refs.rs` | `Ref<T>` with Deref + .id(), `RefMut<T>` with DerefMut + .id() |
| 6 | `store.rs` | `first::<T>()`, `first_mut::<T>()`, `get_by_id::<T>()`, `get_by_id_mut::<T>()`, `resolve::<T>()`, `resolve_mut::<T>()`, `update::<T>()` |
| 7 | `store.rs` | `each::<T>()`, `each_mut::<T>()`, `all::<T>()` lazy iterator |
| 8 | `store.rs` | `add_child()`, `parent()`, `children()`, `descendants()` |
| 9 | `store.rs` | `remove()`, `remove_by_id()`, `clear()`, `is_alive()` |
| 10 | `lib.rs` | Re-exports, prelude module |
| 11 | Tests | Full suite |

---

## Test Coverage

- `add` increases count
- `first::<T>()` returns Ref that derefs correctly
- `Ref::id()` returns consistent value
- `RefMut` deref_mut mutates entity in store
- `first_mut::<T>()` returns mutable access
- `get_by_id_mut` returns mutable access
- `each_mut::<T>()` callback mutates all entities
- `update::<T>()` updates single entity by EntityRef
- `each::<T>()` callback receives all entities of type
- `all::<T>()` iterator yields all entities
- `get_by_id` returns correct entity / None for wrong id
- `add_child` links parent-child, `parent()` returns correct EntityRef
- `add_child` with already-parented child → Error
- `children()` returns correct list
- `descendants()` returns all recursive children (BFS)
- `remove()` by Ref removes entity + descendants
- `remove_by_id()` works with numeric ID
- `clear()` resets everything
- Re-add after clear works
- `is_alive()` correct before/after remove
- `resolve::<T>()` returns Some for correct type, None for wrong
- `EntityRef::id()` returns correct value
- Concurrent reads (multiple threads using `each`/`all`/`first`)
- Deep hierarchy (100 levels)

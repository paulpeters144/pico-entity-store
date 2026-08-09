# pico-entity-store

[![Crates.io](https://img.shields.io/crates/v/pico-entity-store.svg)](https://crates.io/crates/pico-entity-store)
[![docs.rs](https://docs.rs/pico-entity-store/badge.svg)](https://docs.rs/pico-entity-store)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)

A tiny, fast entity-component store for Rust. Store typed components per entity, query them by type, and build parent-child hierarchies — all with a simple, no-macro API.

## Features

- **Type-erased storage** — components are stored in contiguous byte buffers, one per type
- **Read & write queries** — `first`, `get_by_id`, `each`, `all` (read) and `first_mut`, `get_by_id_mut`, `each_mut`, `update` (write)
- **Parent-child hierarchies** — `add` with `children!` macro, `children`, `descendants`, with recursive removal
- **Thread-safe** — the store is protected by an `RwLock`; readers don't block each other
- **Zero dependencies** — only `criterion` for benchmarks (dev-only)

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
pico-entity-store = "0"
```

```rust
use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Clone)]
struct Velocity {
    dx: f32,
    dy: f32,
}

fn main() {
    let store = EntityStore::new();

    // spawn entities with components
    store.add(Position { x: 0.0, y: 0.0 }, &[]).unwrap();
    store.add(Velocity { dx: 1.0, dy: 2.0 }, &[]).unwrap();

    // read the first Position
    if let Some(pos) = store.first::<Position>() {
        println!("({}, {})", pos.x, pos.y);
    }

    // iterate all Velocities mutably
    store.each_mut::<Velocity, _>(|v| {
        v.dx *= 2.0;
    });
}
```

## API Reference

### `EntityStore::new`

Creates a new, empty entity-component store.

```rust
let store = EntityStore::new();
```

### `EntityStore::count`

Returns the number of currently alive entities in the store.

```rust
let store = EntityStore::new();
assert_eq!(store.count(), 0);
store.add(Position { x: 0.0, y: 0.0 }, &[]).unwrap();
assert_eq!(store.count(), 1);
```

### `EntityStore::add`

Creates a new entity from a component value, or attaches children to an existing entity.
Returns the entity's numeric id on success. All validation in a single call is all-or-nothing.

**Create a new entity:**

```rust
let id: u64 = store.add(Position { x: 1.0, y: 2.0 }, &[]).unwrap();
```

**Create a new entity with children:**

```rust
let parent_id = store
    .add(Parent { name: "root".into() }, &[child_a.entity_ref(), child_b.entity_ref()])
    .unwrap();
```

**Attach children to an existing entity:**

```rust
let parent = store.first::<Parent>().unwrap();
store.add(parent, &children![child_a, child_b]).unwrap();
```

Errors returned:
- `PicoError::AlreadyHasParent` — a child entity already belongs to a parent
- `PicoError::EntityNotAlive` — a child entity has been removed

### `EntityStore::remove`

Removes each entity plus all its descendants recursively. Dead or already-removed entities are silently skipped.

```rust
let entity = store.first::<Enemy>().unwrap();
store.remove(&[entity.entity_ref()]);

// batch remove multiple
store.remove(&[eref_a, eref_b, eref_c]);
```

### `EntityStore::clear`

Removes every entity and resets all internal state (storages, maps, caches, id counter).
The store can be reused immediately after.

```rust
store.clear();
assert_eq!(store.count(), 0);
```

### `EntityStore::first`

Returns a read guard to the first live entity of type `T`, or `None` if none exist.

```rust
if let Some(pos) = store.first::<Position>() {
    println!("({}, {})", pos.x, pos.y);
}
```

The returned `Ref<T>` dereferences to `&T`. The read lock is held for the lifetime of the guard.

### `EntityStore::get_by_id`

Returns a read guard to the entity with the given numeric id, or `None` if the id is out of range, the entity is dead, or the type doesn't match.

```rust
let entity_id = store.add(Health { hp: 100 }, &[]).unwrap();
let health = store.get_by_id::<Health>(entity_id).unwrap();
assert_eq!(health.hp, 100);
```

### `EntityStore::resolve`

Resolves a type-erased `EntityRef` back to a typed read guard. Returns `None` if the entity is dead or the type doesn't match.

```rust
let eref = store.first::<Position>().unwrap().entity_ref();
let pos = store.resolve::<Position>(&eref).unwrap();
```

### `EntityStore::each`

Calls a closure with a shared reference to every live entity of type `T`.
A single read lock is held for the duration of the iteration.

```rust
store.each::<Position, _>(|pos| {
    println!("({}, {})", pos.x, pos.y);
});
```

### `EntityStore::all`

Returns an iterator that yields `Ref<T>` read guards for all live entities of type `T`.
The read lock is shared via `Arc` across all yielded guards, so iteration costs a single lock acquisition.

```rust
for pos in store.all::<Position>() {
    println!("entity {} at ({}, {})", pos.id(), pos.x, pos.y);
}

// collect into a Vec
let positions: Vec<_> = store.all::<Position>().collect();
```

### `EntityStore::first_mut`

Returns a write guard to the first live entity of type `T`, or `None` if none exist.

```rust
if let Some(mut vel) = store.first_mut::<Velocity>() {
    vel.dx += 0.1;
    vel.dy -= 0.1;
}
```

The returned `RefMut<T>` dereferences to `&mut T`. The write lock is held for the lifetime of the guard.

### `EntityStore::get_by_id_mut`

Returns a write guard to the entity with the given numeric id, or `None` if the id is out of range, the entity is dead, or the type doesn't match.

```rust
let id = store.add(Health { hp: 100 }, &[]).unwrap();
if let Some(mut health) = store.get_by_id_mut::<Health>(id) {
    health.hp -= 10;
}
```

### `EntityStore::resolve_mut`

Resolves a type-erased `EntityRef` back to a typed write guard. Returns `None` if the entity is dead or the type doesn't match.

```rust
let eref = store.first::<Position>().unwrap().entity_ref();
let pos = store.resolve_mut::<Position>(&eref).unwrap();
pos.x = 10.0;
```

### `EntityStore::update`

Mutates a single entity's component in-place via a closure. Accepts an `EntityRef` (no lock guard needed).
Returns `true` if the entity was alive and the types matched.

```rust
let eref = store.first::<Health>().unwrap().entity_ref();
let ok = store.update::<Health, _>(&eref, |h| {
    h.hp = h.hp.saturating_sub(10);
});
assert!(ok);
```

### `EntityStore::each_mut`

Calls a closure with a mutable reference to every live entity of type `T`.
A single write lock is held for the duration of the iteration.

```rust
store.each_mut::<Velocity, _>(|v| {
    v.dx *= 0.9;
    v.dy += 0.1;
});
```

### `EntityStore::parent`

Returns the type-erased `EntityRef` of the parent of the given entity, or `None` if it has no parent or the parent is dead.

```rust
let child = store.first::<Child>().unwrap();
if let Some(parent_eref) = store.parent(&child) {
    let parent = store.resolve::<Parent>(&parent_eref).unwrap();
    println!("parent: {}", parent.name);
}
```

### `EntityStore::children`

Returns a `Vec<EntityRef>` of the direct children of the entity.

```rust
let parent = store.first::<Parent>().unwrap();
for child_eref in store.children(&parent) {
    if let Some(child) = store.resolve::<Child>(&child_eref) {
        println!("child id: {}", child.id());
    }
}
```

### `EntityStore::descendants`

Returns all descendants (breadth-first traversal) of the entity as `Vec<EntityRef>`.

```rust
let parent = store.first::<Parent>().unwrap();
let all_descendants = store.descendants(&parent);
println!("total descendants: {}", all_descendants.len());
```

### `EntityStore::is_alive`

Returns `true` if the entity's id is still alive in the store.

```rust
let guard = store.first::<Enemy>().unwrap();
assert!(store.is_alive(&guard));
store.remove(&[guard.entity_ref()]);
assert!(!store.is_alive(&guard));
```

### `children!` macro

Collects `EntityRef`s from `Ref`/`RefMut` handles and returns a `[EntityRef; N]` array.
Guards are dropped before the array is returned to avoid deadlocks when `add` acquires a write lock.

```rust
let parent = store.first::<Parent>().unwrap();
let child_a = store.first::<Child>().unwrap();
let child_b = store.first::<Child>().unwrap();
store.add(parent, &children![child_a, child_b]).unwrap();
```

Multiple types and trailing commas are supported:

```rust
let warrior = store.first::<Warrior>().unwrap();
let axe = store.first::<Axe>().unwrap();
let shield = store.first::<Shield>().unwrap();
store.add(warrior, &children![axe, shield]).unwrap();
```

### `EntityRef`

A type-erased, lightweight entity handle that is `Clone`, `Copy`, `PartialEq`, `Eq`, and `Hash`.
Used to reference entities without holding a lock.

```rust
let eref: EntityRef = store.first::<Position>().unwrap().entity_ref();
let id: u64 = eref.id();
```

**`EntityRef::id`** — returns the numeric entity id.

```rust
assert_eq!(eref.id(), entity_id);
```

### `Ref<'a, T>`

A read guard that dereferences to `&T` and holds a shared read lock.

```rust
let pos: Ref<Position> = store.first::<Position>().unwrap();
// Deref to &Position
println!("x: {}, y: {}", pos.x, pos.y);
```

**`Ref::id`** — returns the numeric entity id.

```rust
let entity_id: u64 = pos.id();
```

**`Ref::entity_ref`** — converts into a type-erased `EntityRef`, dropping the lock guard.

```rust
let eref: EntityRef = pos.entity_ref();
```

### `RefMut<'a, T>`

A write guard that dereferences to `&T` and `&mut T`, holding an exclusive write lock.

```rust
let mut vel: RefMut<Velocity> = store.first_mut::<Velocity>().unwrap();
vel.dx = 5.0;           // DerefMut to &mut Velocity
println!("dx: {}", vel.dx); // Deref to &Velocity
```

**`RefMut::id`** — returns the numeric entity id.

```rust
let entity_id: u64 = vel.id();
```

**`RefMut::entity_ref`** — converts into a type-erased `EntityRef`, dropping the write lock guard.

```rust
let eref: EntityRef = vel.entity_ref();
```

### `PicoError`

The error type returned by `EntityStore::add`.

```rust
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PicoError {
    TypeNotRegistered,
    AlreadyHasParent,
    EntityNotAlive,
}
```

| Variant | Description |
|---|---|
| `TypeNotRegistered` | The queried component type was never added to the store |
| `AlreadyHasParent` | An entity already belongs to a parent in the hierarchy |
| `EntityNotAlive` | The entity was removed or was never alive |

### `IntoAdd<T>`

A trait for specifying the target of an `add` operation. Implemented for:

- `T` — a component value (creates a new entity)
- `Ref<'_, T>` — a read guard (attaches children to an existing entity)
- `RefMut<'_, T>` — a write guard (attaches children to an existing entity)

Normally you don't need to reference this trait directly; it's used implicitly when calling `add`.

## Edition & MSRV

- Rust edition: **2024**
- Minimum supported Rust version: **1.85** (edition 2024 requires it)

## License

Licensed under [MIT](LICENSE-MIT).

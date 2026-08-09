# pico-entity-store

[![Crates.io](https://img.shields.io/crates/v/pico-entity-store.svg)](https://crates.io/crates/pico-entity-store)
[![docs.rs](https://docs.rs/pico-entity-store/badge.svg)](https://docs.rs/pico-entity-store)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)

A tiny, fast entity-component store for Rust. Store typed components per entity, query them by type, and build parent-child hierarchies — all with a simple, no-macro API.

## Features

- **Type-erased storage** — components are stored in contiguous byte buffers, one per type
- **Read & write queries** — `first`, `get_by_id`, `each`, `all` (read) and `first_mut`, `get_by_id_mut`, `each_mut`, `update` (write)
- **Parent-child hierarchies** — `add_child`, `children`, `descendants`, with recursive removal
- **Thread-safe** — the store is protected by an `RwLock`; readers don't block each other
- **Zero dependencies** — only `criterion` for benchmarks (dev-only)

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
pico-entity-store = "0.3"
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
    store.add(&Position { x: 0.0, y: 0.0 });
    store.add(&Velocity { dx: 1.0, dy: 2.0 });

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

## API Overview

### Spawning

```rust
store.add(&Position { x: 0.0, y: 0.0 }); // returns ()
```

### Querying (read)

| Method | Returns | Description |
|---|---|---|
| `first::<T>()` | `Option<Ref<T>>` | First live entity of type `T` |
| `get_by_id::<T>(id)` | `Option<Ref<T>>` | Entity by numeric id |
| `resolve::<T>(entity_ref)` | `Option<Ref<T>>` | Entity by `EntityRef` |
| `each::<T>(\|&T\|)` | `()` | Iterate all of type `T` (read) |
| `all::<T>()` | `AllIter<T>` | Iterator of `Ref<T>` |

### Querying (write)

| Method | Returns | Description |
|---|---|---|
| `first_mut::<T>()` | `Option<RefMut<T>>` | First live entity of type `T` |
| `get_by_id_mut::<T>(id)` | `Option<RefMut<T>>` | Entity by numeric id |
| `resolve_mut::<T>(entity_ref)` | `Option<RefMut<T>>` | Entity by `EntityRef` |
| `each_mut::<T>(\|&mut T\|)` | `()` | Iterate all of type `T` (write) |
| `update::<T>(entity_ref, \|&mut T\|)` | `bool` | Mutate a specific entity |

### Hierarchies

```rust
let parent = store.first::<Parent>().unwrap();
let child = store.first::<Child>().unwrap();
store.add_child(parent, child)?;

// query descendants
let kids = store.children(&parent_ref);
let all = store.descendants(&parent_ref);

// removing a parent recursively removes children
store.remove(parent);
```

Use the `children!` macro to collect ids and drop guards in one expression:

```rust
let ids = children![child_a, child_b];
store.add_children(parent_ref, &ids)?;
```

### Removal

| Method | Description |
|---|---|
| `remove(Ref<T>)` | Remove entity and its descendants |
| `remove_by_id(u64)` | Remove by numeric id |
| `clear()` | Remove everything |
| `is_alive(&Ref<T>)` | Check if entity is still alive |

## Edition & MSRV

- Rust edition: **2024**
- Minimum supported Rust version: **1.85** (edition 2024 requires it)

## License

Licensed under [MIT](LICENSE-MIT).

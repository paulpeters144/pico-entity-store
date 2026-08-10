//! A tiny, fast entity-component store for Rust.
//!
//! Store typed components per entity, query them by type, and build
//! parent-child hierarchies — all with a simple, no-macro API.
//!
//! # Quick Start
//!
//! ```rust
//! use pico_entity_store::prelude::*;
//!
//! #[derive(Clone)]
//! struct Position { x: f32, y: f32 }
//!
//! #[derive(Clone)]
//! struct Velocity { dx: f32, dy: f32 }
//!
//! let store = EntityStore::new();
//!
//! store.add(Position { x: 0.0, y: 0.0 }, &[]).unwrap();
//! store.add(Velocity { dx: 1.0, dy: 2.0 }, &[]).unwrap();
//!
//! if let Some(pos) = store.first::<Position>() {
//!     println!("({}, {})", pos.x, pos.y);
//! }
//!
//! store.all_mut::<Velocity>().for_each(|v| {
//!     v.dx *= 2.0;
//! });
//! ```

pub mod entity_ref;
pub mod refs;
pub mod storage;
pub mod store;

/// Convenient re-exports for working with the ECS.
pub mod prelude {
    pub use crate::children;
    pub use crate::entity_ref::EntityRef;
    pub use crate::refs::{Ref, RefMut, RefMutVec, RefVec};
    pub use crate::store::{ChildSource, EntityStore, IntoChild, PicoError};
}

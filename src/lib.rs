pub mod entity_ref;
pub mod refs;
pub mod storage;
pub mod store;

#[cfg(test)]
mod test;

pub mod prelude {
    pub use crate::entity_ref::EntityRef;
    pub use crate::refs::{Ref, RefMut};
    pub use crate::store::{EntityStore, PicoError};
}

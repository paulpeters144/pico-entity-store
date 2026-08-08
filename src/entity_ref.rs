use std::any::TypeId;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityRef {
    pub(crate) id: usize,
    pub(crate) type_id: TypeId,
}

impl EntityRef {
    pub fn id(&self) -> u64 {
        self.id as u64
    }
}

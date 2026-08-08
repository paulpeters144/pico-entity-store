use std::alloc::Layout;
use std::any::TypeId;

pub(crate) struct TypedStorage {
    pub(crate) type_id: TypeId,
    data: Vec<u8>,
    len: usize,
    #[allow(dead_code)]
    elem_size: usize,
    #[allow(dead_code)]
    alignment: usize,
    drop_fn: unsafe fn(*mut u8, usize),
}

impl TypedStorage {
    pub(crate) fn new<T: 'static>() -> Self {
        let layout = Layout::new::<T>();
        Self {
            type_id: TypeId::of::<T>(),
            data: Vec::new(),
            len: 0,
            elem_size: layout.size(),
            alignment: layout.align(),
            drop_fn: drop_slice::<T>,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn push_raw<T: Clone + 'static>(&mut self, entity: &T) -> usize {
        assert_eq!(
            TypeId::of::<T>(),
            self.type_id,
            "push_raw called with wrong type"
        );

        let layout = Layout::new::<T>();
        let offset = self.data.len();

        self.data.reserve(layout.size());

        let needed = offset + layout.size();
        if self.data.len() < needed {
            self.data.resize(needed, 0);
        }

        let dst = unsafe { self.data.as_mut_ptr().add(offset) } as *mut T;
        let cloned = entity.clone();
        unsafe {
            std::ptr::write(dst, cloned);
        }

        let slot = self.len;
        self.len += 1;
        slot
    }

    pub(crate) fn get<T>(&self, slot: usize) -> &T {
        assert!(slot < self.len);
        let layout = Layout::new::<T>();
        let offset = slot * layout.size();
        unsafe {
            let ptr = self.data.as_ptr().add(offset) as *const T;
            &*ptr
        }
    }

    pub(crate) fn get_mut<T>(&mut self, slot: usize) -> &mut T {
        assert!(slot < self.len);
        let layout = Layout::new::<T>();
        let offset = slot * layout.size();
        unsafe {
            let ptr = self.data.as_mut_ptr().add(offset) as *mut T;
            &mut *ptr
        }
    }

    pub(crate) fn raw_ptr<T>(&self, slot: usize) -> *const T {
        assert!(slot < self.len);
        let layout = Layout::new::<T>();
        let offset = slot * layout.size();
        unsafe { self.data.as_ptr().add(offset) as *const T }
    }

    pub(crate) fn mut_raw_ptr<T>(&mut self, slot: usize) -> *mut T {
        assert!(slot < self.len);
        let layout = Layout::new::<T>();
        let offset = slot * layout.size();
        unsafe { self.data.as_mut_ptr().add(offset) as *mut T }
    }

    pub(crate) fn swap_remove<T>(&mut self, slot: usize) -> Option<usize> {
        if slot >= self.len {
            return None;
        }

        let last_idx = self.len - 1;
        if slot == last_idx {
            self.len -= 1;
            return None;
        }

        let layout = Layout::new::<T>();
        unsafe {
            let src = self.data.as_ptr().add(last_idx * layout.size());
            let dst = self.data.as_mut_ptr().add(slot * layout.size());
            std::ptr::copy_nonoverlapping(src, dst, layout.size());
        }
        self.len -= 1;
        Some(last_idx)
    }
}

impl Drop for TypedStorage {
    fn drop(&mut self) {
        unsafe {
            (self.drop_fn)(self.data.as_mut_ptr(), self.len);
        }
    }
}

unsafe fn drop_slice<T>(ptr: *mut u8, len: usize) {
    unsafe {
        let slice = std::slice::from_raw_parts_mut(ptr as *mut T, len);
        for item in slice.iter_mut() {
            std::ptr::drop_in_place(item);
        }
    }
}

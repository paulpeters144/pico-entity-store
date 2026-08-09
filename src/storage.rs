use std::alloc::{Layout, alloc, dealloc, handle_alloc_error, realloc};
use std::any::TypeId;
use std::ptr::NonNull;

/// Type-erased, correctly aligned storage for elements of one component type.
///
/// Elements are stored in a contiguous buffer allocated with the layout of `T`
/// (unlike a `Vec<u8>`, which would only guarantee 1-byte alignment). The
/// invariant is that `entity_ids` and the element buffer are kept parallel:
/// `entity_ids[slot]` holds the entity id of the element occupying `slot`, for
/// `slot in 0..len`.
///
/// # Safety invariants
///
/// - `ptr` points to a valid allocation for `cap` elements of `elem_layout`
///   (or to an aligned dangling pointer when `cap == 0` / element size is 0).
/// - Elements in slots `0..len` are initialized; the remaining capacity is
///   uninitialized and never read.
/// - `entity_ids.len() == len` always.
/// - `drop_fn` matches the element type of this storage.
pub(crate) struct TypedStorage {
    pub(crate) type_id: TypeId,
    ptr: NonNull<u8>,
    cap: usize,
    len: usize,
    elem_layout: Layout,
    /// Entity ids of alive elements, kept in slot order.
    /// `entity_ids[slot]` = the entity id occupying that slot.
    pub(crate) entity_ids: Vec<u64>,
    drop_fn: unsafe fn(*mut u8, usize),
}

impl TypedStorage {
    pub(crate) fn new<T: 'static>() -> Self {
        let elem_layout = Layout::new::<T>();
        Self {
            type_id: TypeId::of::<T>(),
            ptr: aligned_dangling(elem_layout),
            cap: 0,
            len: 0,
            elem_layout,
            entity_ids: Vec::new(),
            drop_fn: drop_slice::<T>,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    fn array_layout(&self, n: usize) -> Layout {
        self.elem_layout.repeat(n).expect("layout overflow").0
    }

    fn grow(&mut self) {
        if self.elem_layout.size() == 0 {
            // Zero-sized element: no allocation needed, just a valid aligned
            // dangling pointer.
            self.cap = (self.cap.max(1)) * 2;
            return;
        }
        let new_cap = (self.cap.max(1)) * 2;
        let new_layout = self.array_layout(new_cap);
        let new_ptr = if self.cap == 0 {
            unsafe { alloc(new_layout) }
        } else {
            let old_layout = self.array_layout(self.cap);
            unsafe { realloc(self.ptr.as_ptr(), old_layout, new_layout.size()) }
        };
        if new_ptr.is_null() {
            handle_alloc_error(new_layout);
        }
        self.ptr = unsafe { NonNull::new_unchecked(new_ptr) };
        self.cap = new_cap;
    }

    pub(crate) fn push_raw<T: Clone + 'static>(&mut self, entity: &T, entity_id: u64) -> usize {
        assert_eq!(
            TypeId::of::<T>(),
            self.type_id,
            "push_raw called with wrong type"
        );

        if self.len == self.cap {
            self.grow();
        }

        let slot = self.len;
        let dst = unsafe { (self.ptr.as_ptr() as *mut T).add(slot) };
        let cloned = entity.clone();
        unsafe {
            std::ptr::write(dst, cloned);
        }

        self.len += 1;
        self.entity_ids.push(entity_id);
        slot
    }

    pub(crate) fn get<T>(&self, slot: usize) -> &T {
        assert!(slot < self.len);
        unsafe { &*((self.ptr.as_ptr() as *const T).add(slot)) }
    }

    pub(crate) fn get_mut<T>(&mut self, slot: usize) -> &mut T {
        assert!(slot < self.len);
        unsafe { &mut *((self.ptr.as_ptr() as *mut T).add(slot)) }
    }

    pub(crate) fn raw_ptr<T>(&self, slot: usize) -> *const T {
        assert!(slot < self.len);
        unsafe { (self.ptr.as_ptr() as *const T).add(slot) }
    }

    pub(crate) fn mut_raw_ptr<T>(&mut self, slot: usize) -> *mut T {
        assert!(slot < self.len);
        unsafe { (self.ptr.as_ptr() as *mut T).add(slot) }
    }

    /// Swaps the last element into `slot`, shrinking the buffer by one.
    /// Also swaps `entity_ids` so that `entity_ids[slot]` still maps to
    /// whichever entity now occupies `slot`.
    ///
    /// The element is moved byte-for-byte using this storage's own element
    /// layout, so callers do not need to know the concrete type `T`.
    ///
    /// Returns `Some((displaced_entity_id, last_slot))` when a swap occurred,
    /// or `None` when `slot` was the last element.
    pub(crate) fn swap_remove(&mut self, slot: usize) -> Option<(u64, usize)> {
        if slot >= self.len {
            return None;
        }

        let last_idx = self.len - 1;
        if slot == last_idx {
            self.len -= 1;
            self.entity_ids.pop();
            return None;
        }

        if self.elem_layout.size() > 0 {
            let size = self.elem_layout.size();
            unsafe {
                let src = self.ptr.as_ptr().add(last_idx * size);
                let dst = self.ptr.as_ptr().add(slot * size);
                std::ptr::copy_nonoverlapping(src, dst, size);
            }
        }
        self.len -= 1;

        // The entity that was at `last_idx` now occupies `slot`.
        let displaced_entity_id = self.entity_ids[last_idx];
        self.entity_ids[slot] = displaced_entity_id;
        self.entity_ids.pop();

        Some((displaced_entity_id, last_idx))
    }
}

/// `NonNull` is `!Send`/`!Sync`, but `TypedStorage` is only ever accessed under
/// the store's `RwLock` (the previous `Vec<u8>` backing buffer was `Send +
/// Sync`), so the raw buffer is safe to move and share alongside the lock.
unsafe impl Send for TypedStorage {}
unsafe impl Sync for TypedStorage {}

/// Returns an aligned dangling pointer suitable for zero-capacity or
/// zero-sized-element buffers. Never dereferenced while `cap == 0`.
fn aligned_dangling(layout: Layout) -> NonNull<u8> {
    if layout.size() == 0 {
        // `alloc` with a zero-size layout returns an aligned, non-null dangling
        // pointer per the allocator contract; we never dealloc it.
        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            handle_alloc_error(layout);
        }
        unsafe { NonNull::new_unchecked(ptr) }
    } else {
        // Replaced by a real allocation on the first grow(); never dereferenced
        // while `cap == 0`.
        NonNull::dangling()
    }
}

impl Drop for TypedStorage {
    fn drop(&mut self) {
        unsafe {
            (self.drop_fn)(self.ptr.as_ptr(), self.len);
        }
        if self.cap > 0 && self.elem_layout.size() > 0 {
            let old_layout = self.array_layout(self.cap);
            unsafe {
                dealloc(self.ptr.as_ptr(), old_layout);
            }
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

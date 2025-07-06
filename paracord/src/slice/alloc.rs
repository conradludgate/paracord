use std::mem::MaybeUninit;

use sync_wrapper::SyncWrapper;
use typed_arena::Arena;

use crate::Key;

pub(super) struct Alloc<T>(SyncWrapper<Arena<T>>);

impl<T> Default for Alloc<T> {
    fn default() -> Self {
        Self(SyncWrapper::new(Arena::new()))
    }
}

impl<T> Alloc<T> {
    #[cfg(test)]
    pub(super) fn size(&mut self) -> usize {
        self.0.get_mut().len() * std::mem::size_of::<T>()
    }
}

/// Represents a `&'_ [T]`, with a length limited to u32 and with an
/// undescribed lifetime because it's technically self-ref.
#[derive(Clone, Copy)]
#[repr(align(8))]
pub(super) struct InternedPtr<T> {
    ptr: *const T,
    len: u32,
    pub(super) key: Key,
}

// Safety: `VecEntry` has the same safety requirements as `&[T]`
unsafe impl<T: Sync> Sync for InternedPtr<T> {}
// Safety: `VecEntry` has the same safety requirements as `&[T]`
unsafe impl<T: Sync> Send for InternedPtr<T> {}

impl<T> InternedPtr<T> {
    pub(super) fn new(s: &[T], key: Key) -> Self {
        let len = u32::try_from(s.len()).expect("slice lengths must be less than u32::MAX");
        Self {
            len,
            ptr: s.as_ptr(),
            key,
        }
    }

    pub(super) fn slice(&self) -> &[T] {
        // Safety: the ptr and len came from a &[T] to begin with.
        unsafe { &*core::ptr::slice_from_raw_parts(self.ptr, self.len as usize) }
    }
}

impl<T: Copy> Alloc<T> {
    #[inline]
    pub(super) fn alloc(&mut self, s: &[T]) -> &mut [T] {
        /// Polyfill for [`MaybeUninit::copy_from_slice`]
        fn copy_from_slice<'a, T: Copy>(this: &'a mut [MaybeUninit<T>], src: &[T]) -> &'a mut [T] {
            let uninit_src: &[MaybeUninit<T>] =
                // SAFETY: &[T] and &[MaybeUninit<T>] have the same layout
                unsafe { &*(src as *const [T] as *const [std::mem::MaybeUninit<T>]) };

            this.copy_from_slice(uninit_src);

            // SAFETY: Valid elements have just been copied into `this` so it is initialized
            unsafe { slice_assume_init_mut(this) }
        }

        /// Polyfill for [`MaybeUninit::slice_assume_init_mut`]
        const unsafe fn slice_assume_init_mut<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
            // SAFETY: similar to safety notes for `slice_get_ref`, but we have a
            // mutable reference which is also guaranteed to be valid for writes.
            unsafe { &mut *(slice as *mut [MaybeUninit<T>] as *mut [T]) }
        }

        let arena = self.0.get_mut();

        // Safety: we are making sure to init all the elements without panicking.
        let uninit = unsafe { arena.alloc_uninitialized(s.len()) };
        copy_from_slice(uninit, s)
    }
}

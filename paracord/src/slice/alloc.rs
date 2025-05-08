use std::{
    hash::{BuildHasher, Hash},
    mem::MaybeUninit,
};

use hashbrown::hash_table::Entry;
use papaya::Equivalent;
use sync_wrapper::SyncWrapper;
use typed_arena::Arena;

use crate::{slice::ParaCord, Key};

use super::TableEntry;

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
#[repr(packed)]
pub(super) struct InternedPtr<T> {
    ptr: *const T,
    len: u32,
}

// Safety: `VecEntry` has the same safety requirements as `&[T]`
unsafe impl<T: Sync> Sync for InternedPtr<T> {}
// Safety: `VecEntry` has the same safety requirements as `&[T]`
unsafe impl<T: Sync> Send for InternedPtr<T> {}

impl<T> InternedPtr<T> {
    fn new(s: &[T]) -> Self {
        let len = u32::try_from(s.len()).expect("slice lengths must be less than u32::MAX");
        Self {
            len,
            ptr: s.as_ptr(),
        }
    }

    pub(super) fn slice(&self) -> &[T] {
        // Safety: the ptr and len came from a &[T] to begin with.
        unsafe { &*core::ptr::slice_from_raw_parts(self.ptr, self.len as usize) }
    }
}

impl<T: Hash> Hash for InternedPtr<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.slice().hash(state);
    }
}

impl<T: Eq> Equivalent<InternedPtr<T>> for [T] {
    fn equivalent(&self, key: &InternedPtr<T>) -> bool {
        self == key.slice()
    }
}

impl<T: PartialEq> PartialEq for InternedPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.slice() == other.slice()
    }
}

impl<T: Eq> Eq for InternedPtr<T> {}

impl<T: Copy> Alloc<T> {
    #[inline]
    fn alloc(&mut self, s: &[T]) -> &mut [T] {
        /// Polyfill for [`MaybeUninit::copy_from_slice`]
        fn copy_from_slice<'a, T: Copy>(this: &'a mut [MaybeUninit<T>], src: &[T]) -> &'a mut [T] {
            // SAFETY: &[T] and &[MaybeUninit<T>] have the same layout
            let uninit_src: &[MaybeUninit<T>] = unsafe { core::mem::transmute(src) };

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

impl<T: Hash + Eq + Copy, S: BuildHasher> ParaCord<T, S> {
    #[cold]
    pub(super) fn intern_slow(&self, s: &[T]) -> Key {
        let _len = u32::try_from(s.len()).expect("slice lengths must be less than u32::MAX");

        let mut alloc = self.alloc.lock().unwrap();

        let pin = self.slice_to_keys.pin();
        if let Some(key) = pin.get(s) {
            return *key;
        }

        let key = self.keys_to_slice.push_with(|key| {
            let key = Key::from_index(key);
            let s = alloc.alloc(s);
            let s = InternedPtr::new(s);
            pin.insert(s, key);
            s
        });

        // SAFETY: as asserted the key is correct
        unsafe { Key::new_unchecked(key as u32) }
    }

    #[cold]
    pub(super) fn intern_slow_mut(&mut self, s: &[T]) -> Key {
        let _len = u32::try_from(s.len()).expect("slice lengths must be less than u32::MAX");

        let alloc = self.alloc.get_mut().unwrap();

        let pin = self.slice_to_keys.pin();
        if let Some(key) = pin.get(s) {
            return *key;
        }

        let key = self.keys_to_slice.push_with(|key| {
            let key = Key::from_index(key);
            let s = alloc.alloc(s);
            let s = InternedPtr::new(s);
            pin.insert(s, key);
            s
        });

        // SAFETY: as asserted the key is correct
        unsafe { Key::new_unchecked(key as u32) }
    }
}

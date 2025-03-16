use std::{
    hash::{BuildHasher, Hash},
    mem::MaybeUninit,
};

use clashmap::tableref::{entry::Entry, entrymut::EntryMut};
use typed_arena::Arena;

use crate::{send_if_sync::SendIfSync, slice::ParaCord, Key};

pub(super) struct Alloc<T>(Arena<SendIfSync<T>>);

impl<T> Default for Alloc<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Copy> Alloc<T> {
    pub(super) fn size(&self) -> usize {
        self.0.len() * std::mem::size_of::<T>()
    }

    #[inline]
    unsafe fn alloc(&self, s: &[T]) -> &'static [T] {
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

        let s = SendIfSync::cast_from_slice(s);
        // Safety: we are making sure to init all the elements without panicking.
        let s = copy_from_slice(unsafe { self.0.alloc_uninitialized(s.len()) }, s);
        let s = SendIfSync::cast_to_slice(s);

        // SAFETY: caller will not drop alloc until it drops the containers storing this
        unsafe { &*(s as *const [T]) }
    }
}

impl<T: 'static + Sync + Hash + Eq + Copy, S: BuildHasher> ParaCord<T, S> {
    #[cold]
    pub(super) fn intern_slow(&self, s: &[T], hash: u64) -> Key {
        match self.slice_to_keys.entry(hash, |k| s == &*k.0, |k| k.2) {
            Entry::Occupied(entry) => entry.get().1,
            Entry::Vacant(entry) => {
                // SAFETY: we will not drop bump until we drop the containers storing these `&'static [T]`.
                let s = unsafe { self.alloc.get_or_default().alloc(s) };

                let key = self.keys_to_slice.push(s);
                let key = Key::from_index(key);
                entry.insert((typesize::Ref(s), key, hash));

                // SAFETY: as asserted the key is correct
                key
            }
        }
    }

    #[cold]
    pub(super) fn intern_slow_mut(&mut self, s: &[T], hash: u64) -> Key {
        match self.slice_to_keys.entry_mut(hash, |k| s == &*k.0, |k| k.2) {
            EntryMut::Occupied(entry) => entry.get().1,
            EntryMut::Vacant(entry) => {
                let alloca = match self.alloc.iter_mut().next() {
                    Some(alloc) => alloc,
                    None => self.alloc.get_or_default(),
                };

                // SAFETY: we will not drop bump until we drop the containers storing these `&'static [T]`.
                let s = unsafe { alloca.alloc(s) };

                let key = self.keys_to_slice.push(s);
                let key = Key::from_index(key);
                entry.insert((typesize::Ref(s), key, hash));

                // SAFETY: as asserted the key is correct
                key
            }
        }
    }
}

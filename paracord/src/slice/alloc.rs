use std::{
    hash::{BuildHasher, Hash},
    mem::{ManuallyDrop, MaybeUninit},
};

use hashbrown::hash_table::Entry;
use sync_wrapper::SyncWrapper;
use typed_arena::Arena;

use crate::{slice::ParaCord, Key};

use super::TableEntry;

pub(super) struct Alloc<T>(SyncWrapper<Arena<ManuallyDrop<T>>>);

// Safety: We never give out `&mut T` access from the arena, so T only needs to be `Sync`.
unsafe impl<T: Sync> Send for Alloc<T> {}

impl<T> Default for Alloc<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> Alloc<T> {
    pub(super) fn size(&mut self) -> usize {
        self.0.get_mut().len() * std::mem::size_of::<T>()
    }
}

const fn manually_drop_cast_slice_uninit_mut<T>(
    slice: &mut [MaybeUninit<ManuallyDrop<T>>],
) -> &mut [MaybeUninit<T>] {
    // Safety: MaybeUninit/ManuallyDrop are transparent
    unsafe { &mut *(slice as *mut [MaybeUninit<ManuallyDrop<T>>] as *mut [MaybeUninit<T>]) }
}

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
        copy_from_slice(manually_drop_cast_slice_uninit_mut(uninit), s)
    }
}

impl<T: 'static + Sync + Hash + Eq + Copy, S: BuildHasher> ParaCord<T, S> {
    #[cold]
    pub(super) fn intern_slow(&self, s: &[T], hash: u64) -> Key {
        let _len = u32::try_from(s.len()).expect("slice lengths must be less than u32::MAX");

        let shard = &mut *self.slice_to_keys.get_write_shard(hash);
        match shard.table.entry(hash, |k| s == k.slice(), |k| k.hash) {
            Entry::Occupied(entry) => entry.get().key,
            Entry::Vacant(entry) => {
                let s = shard.alloc.alloc(s);

                // SAFETY: we will not drop bump until we drop the containers storing these `&'static [T]`.
                let s = unsafe { &*(s as *const [T]) };

                let key = self.keys_to_slice.push(s);
                let key = Key::from_index(key);
                entry.insert(TableEntry::new(s, key, hash));

                // SAFETY: as asserted the key is correct
                key
            }
        }
    }

    #[cold]
    pub(super) fn intern_slow_mut(&mut self, s: &[T], hash: u64) -> Key {
        let _len = u32::try_from(s.len()).expect("slice lengths must be less than u32::MAX");

        let shard = &mut *self.slice_to_keys.get_mut(hash);
        match shard.table.entry(hash, |k| s == k.slice(), |k| k.hash) {
            Entry::Occupied(entry) => entry.get().key,
            Entry::Vacant(entry) => {
                let s = shard.alloc.alloc(s);

                // SAFETY: we will not drop bump until we drop the containers storing these `&'static [T]`.
                let s = unsafe { &*(s as *const [T]) };

                let key = self.keys_to_slice.push(s);
                let key = Key::from_index(key);
                entry.insert(TableEntry::new(s, key, hash));

                // SAFETY: as asserted the key is correct
                key
            }
        }
    }
}

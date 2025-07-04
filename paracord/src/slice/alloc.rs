use std::hash::{BuildHasher, Hash};
use std::mem::MaybeUninit;

use papaya::table::{HashTable, InsertResult, VerifiedGuard};
use sync_wrapper::SyncWrapper;
use typed_arena::Arena;

use super::TableEntry;
use crate::slice::{find, NoDealloc, ParaCord};
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

impl<T: Copy> Alloc<T> {
    #[inline]
    fn alloc(&mut self, s: &[T]) -> &mut [T] {
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

impl<T: Hash + Eq + Copy, S: BuildHasher> ParaCord<T, S> {
    #[cold]
    pub(super) fn intern_slow(&self, s: &[T], hash: u64, guard: &impl VerifiedGuard) -> Key {
        let _len = u32::try_from(s.len()).expect("slice lengths must be less than u32::MAX");

        let alloc = &mut *self.alloc.get_write_shard(hash);

        intern_inner_locked(
            &self.slice_to_keys,
            &self.keys_to_slice,
            alloc,
            &self.hasher,
            s,
            hash,
            guard,
        )
    }

    #[cold]
    pub(super) fn intern_slow_mut(&mut self, s: &[T], hash: u64) -> Key {
        let _len = u32::try_from(s.len()).expect("slice lengths must be less than u32::MAX");

        let alloc = &mut *self.alloc.get_mut(hash);
        let guard = self.slice_to_keys.guard();

        intern_inner_locked(
            &self.slice_to_keys,
            &self.keys_to_slice,
            alloc,
            &self.hasher,
            s,
            hash,
            &guard,
        )
    }
}

fn intern_inner_locked<T: Hash + Eq + Copy, S: BuildHasher>(
    slice_to_keys: &HashTable<TableEntry<T>, NoDealloc>,
    keys_to_slice: &boxcar::Vec<TableEntry<T>>,
    alloc: &mut Alloc<T>,
    h: &S,
    s: &[T],
    hash: u64,
    guard: &impl VerifiedGuard,
) -> Key {
    if let Some(key) = find(slice_to_keys, s, hash, guard) {
        return key;
    }

    let key = keys_to_slice.push_with(|key| {
        let key = Key::from_index(key);
        let s = alloc.alloc(s);
        let s = InternedPtr::new(s);
        TableEntry::new(s, key)
    });
    // safety: we have just inserted this entry
    let entry = unsafe { keys_to_slice.get_unchecked(key) };

    // safety: k is allocated correct
    let eq = |k: *mut TableEntry<T>| unsafe { s == (*k).slice() };
    // safety: k is allocated correct
    let hasher = |k: *mut TableEntry<T>| unsafe { h.hash_one((*k).slice()) };

    let k = entry as *const TableEntry<T> as *mut TableEntry<T>;
    let res = slice_to_keys.insert(hash, k, eq, hasher, false, guard);

    match res {
        InsertResult::Inserted => Key::from_index(key),
        InsertResult::Replaced(_) => unreachable!("we do not replace"),
        InsertResult::Error(_) => unreachable!(
            "while holding the lock, we checked for this entry already and it was not in there"
        ),
    }
}

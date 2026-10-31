use std::hash::{BuildHasher, Hash};
use std::mem::MaybeUninit;

use hashbrown::hash_table::Entry;
use sync_wrapper::SyncWrapper;
use typed_arena::Arena;

use crate::slice::{Collection, ParaCord};
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

/// Non-owning slice metadata. Borrowing its contents requires the owning
/// interner's arena to remain live; the erased lifetime is checked by callers
/// of `slice`, not by the raw pointer itself.
#[derive(Clone, Copy)]
#[repr(align(8))]
pub(super) struct InternedPtr<T> {
    // Safety invariant: non-null, aligned pointer obtained from a valid &[T].
    // Its allocation is not owned by this metadata and may only be accessed
    // under the lifetime and aliasing contract of `slice`.
    ptr: *const T,
    // Safety invariant: this is the original slice length, so its byte extent
    // fits in isize and was contained in one allocation (or is zero-sized).
    len: u32,
    pub(super) key: Key,
}

// SAFETY: InternedPtr owns no T and never dereferences its raw pointer in Drop.
// Access through slice is unsafe and must preserve the arena borrow. With
// T: Sync, the resulting shared slice may be read from another thread.
unsafe impl<T: Sync> Sync for InternedPtr<T> {}
// SAFETY: Moving this non-owning metadata neither moves nor drops its T values.
// Any later slice access must uphold slice's owner/lifetime contract, and
// T: Sync permits the resulting shared reads on the destination thread.
unsafe impl<T: Sync> Send for InternedPtr<T> {}

impl<T> InternedPtr<T> {
    fn new(s: &[T], key: Key) -> Self {
        let len = u32::try_from(s.len()).expect("slice lengths must be less than u32::MAX");
        Self {
            ptr: s.as_ptr(),
            len,
            key,
        }
    }

    /// # Safety
    ///
    /// The caller must retain the allocation containing the original initialized
    /// slice for the entire returned borrow, and must prevent mutation of its
    /// elements during that borrow except through nested UnsafeCell values.
    ///
    /// In ParaCord this is supplied by the owning interner's storage invariant
    /// and a shared borrow of that same interner.
    pub(super) unsafe fn slice(&self) -> &[T] {
        // SAFETY: The field invariants supply non-nullness, alignment, the
        // original length and a single-allocation byte range fitting isize.
        // The caller retains that initialized allocation and shared-read
        // permissions for the full returned lifetime.
        unsafe { &*core::ptr::slice_from_raw_parts(self.ptr, self.len as usize) }
    }
}

impl<T: Copy> Alloc<T> {
    /// # Safety-usable invariant
    ///
    /// The returned elements are fully initialized copies of s. Their arena
    /// allocation remains at the same address and is not reused by later
    /// allocations until this Alloc is dropped.
    #[inline]
    fn alloc(&mut self, s: &[T]) -> &mut [T] {
        /// Polyfill for [`MaybeUninit::copy_from_slice`]
        fn copy_from_slice<'a, T: Copy>(this: &'a mut [MaybeUninit<T>], src: &[T]) -> &'a mut [T] {
            let uninit_src: &[MaybeUninit<T>] =
                // SAFETY: MaybeUninit<T> has T's size and alignment; casting
                // the slice pointer preserves its element count and original
                // live allocation. Every valid T is valid as MaybeUninit<T>,
                // and this shared view neither mutates nor outlives src.
                unsafe { &*(src as *const [T] as *const [std::mem::MaybeUninit<T>]) };

            this.copy_from_slice(uninit_src);

            // SAFETY: Valid elements have just been copied into `this` so it is initialized
            unsafe { slice_assume_init_mut(this) }
        }

        /// Polyfill for [`MaybeUninit::slice_assume_init_mut`].
        ///
        /// # Safety
        /// All elements must be initialized valid T values. The returned borrow
        /// retains the input slice's exclusive access and allocation lifetime.
        const unsafe fn slice_assume_init_mut<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
            // SAFETY: MaybeUninit<T> and T have equal size/alignment, and the
            // slice pointer cast preserves length and allocation. The caller
            // supplies initialized valid values; the existing mutable borrow
            // supplies exclusive access for the returned lifetime.
            unsafe { &mut *(slice as *mut [MaybeUninit<T>] as *mut [T]) }
        }

        let arena = self.0.get_mut();

        // SAFETY: copy_from_slice writes all s.len() returned slots before any
        // other arena use. The lengths match and T: Copy, so this copy invokes
        // no caller-controlled code or destructor and cannot panic midway.
        let uninit = unsafe { arena.alloc_uninitialized(s.len()) };
        copy_from_slice(uninit, s)
    }
}

impl<T: Hash + Eq + Copy, S: BuildHasher> ParaCord<T, S> {
    #[cold]
    pub(super) fn intern_slow(&self, s: &[T], hash: u64) -> Key {
        let _len = u32::try_from(s.len()).expect("slice lengths must be less than u32::MAX");

        let Collection { table, alloc } = &mut *self.slice_to_keys.get_write_shard(hash);

        // SAFETY: The owning interner's storage invariant keeps every table
        // pointer in keys_to_slice and its slice allocation live. This shard's
        // write guard prevents table changes, and &self prevents clear/drop.
        let eq = |k: &*const InternedPtr<T>| unsafe { s == (**k).slice() };
        // SAFETY: The same storage invariant and guard keep the metadata and
        // immutable arena slice valid while this callback hashes it.
        let hasher = |k: &*const InternedPtr<T>| unsafe { self.hasher.hash_one((**k).slice()) };

        match table.entry(hash, eq, hasher) {
            // safety: entry is allocated correct
            Entry::Occupied(entry) => unsafe { (**entry.get()).key },
            Entry::Vacant(entry) => {
                let key = self.keys_to_slice.push_with(|key| {
                    let key = Key::from_index(key);
                    let s = alloc.alloc(s);
                    InternedPtr::new(s, key)
                });

                // safety: we have just inserted this entry
                let interned_ptr = unsafe { self.keys_to_slice.get_unchecked(key) };
                entry.insert(interned_ptr as *const InternedPtr<T>);

                interned_ptr.key
            }
        }
    }

    #[cold]
    pub(super) fn intern_slow_mut(&mut self, s: &[T], hash: u64) -> Key {
        let _len = u32::try_from(s.len()).expect("slice lengths must be less than u32::MAX");

        let Collection { table, alloc } = &mut *self.slice_to_keys.get_mut(hash);

        // SAFETY: Exclusive access to the owning interner preserves its storage
        // invariant while table callbacks dereference metadata and borrow slices.
        let eq = |k: &*const InternedPtr<T>| unsafe { s == (**k).slice() };
        // SAFETY: The same exclusive owner borrow retains the metadata and arena
        // allocation while this callback hashes the initialized slice.
        let hasher = |k: &*const InternedPtr<T>| unsafe { self.hasher.hash_one((**k).slice()) };

        match table.entry(hash, eq, hasher) {
            // safety: entry is allocated correct
            Entry::Occupied(entry) => unsafe { (**entry.get()).key },
            Entry::Vacant(entry) => {
                let key = self.keys_to_slice.push_with(|key| {
                    let key = Key::from_index(key);
                    let s = alloc.alloc(s);
                    InternedPtr::new(s, key)
                });

                // safety: we have just inserted this entry
                let interned_ptr = unsafe { self.keys_to_slice.get_unchecked(key) };

                entry.insert(interned_ptr as *const InternedPtr<T>);

                interned_ptr.key
            }
        }
    }
}

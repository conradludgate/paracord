//! Lock-free portion of paracord.

use std::hash::{BuildHasher, Hash};

use papaya::table::{HashTable, InsertResult, VerifiedGuard};
use seize::Collector;

use crate::slice::InternedPtr;
use crate::Key;

fn table<T>(cap: usize) -> HashTable<InternedPtr<T>, NoDealloc> {
    HashTable::new(cap, Collector::default(), papaya::ResizeMode::default())
}

pub(super) struct NoDealloc;
impl<T> papaya::table::Dealloc<T> for NoDealloc {
    #[inline(always)]
    unsafe fn dealloc(_: *mut T) {}
}

pub(super) struct LockFreeParacord<T, S = foldhash::fast::RandomState> {
    // stores pointers to keys_to_slice, so must be dropped first
    pub(super) slice_to_keys: HashTable<InternedPtr<T>, NoDealloc>,
    pub(super) keys_to_slice: boxcar::Vec<InternedPtr<T>>,
    pub(super) hasher: S,
}

impl<T, S> LockFreeParacord<T, S> {
    pub(super) fn with_capacity_and_hasher(cap: usize, hasher: S) -> Self {
        Self {
            slice_to_keys: table(cap),
            keys_to_slice: boxcar::Vec::with_capacity(cap),
            hasher,
        }
    }

    pub(super) fn clear(&mut self) {
        // must clear slice_to_keys first
        self.slice_to_keys = table(self.slice_to_keys.len());
        self.keys_to_slice.clear();
    }
}

impl<T: Hash + Eq, S: BuildHasher> LockFreeParacord<T, S> {
    pub(super) fn find(&self, s: &[T], hash: u64, guard: &impl VerifiedGuard) -> Option<Key> {
        // safety: k is allocated correct
        let eq = |k: *mut InternedPtr<T>| unsafe { s == (*k).slice() };
        // safety: k is allocated correct
        let map = |k: *mut InternedPtr<T>| unsafe { (*k).key };

        self.slice_to_keys.find(hash, eq, guard).map(map)
    }

    pub(super) fn insert(&self, s: &mut [T], hash: u64, guard: &impl VerifiedGuard) -> Key {
        let key = self.keys_to_slice.push_with(|key| {
            let key = Key::from_index(key);
            InternedPtr::new(s, key)
        });

        // safety: we have just inserted this entry
        let entry = unsafe { self.keys_to_slice.get_unchecked(key) };

        // safety: k is allocated correct
        let eq = |k: *mut InternedPtr<T>| unsafe { s == (*k).slice() };
        // safety: k is allocated correct
        let hasher = |k: *mut InternedPtr<T>| unsafe { self.hasher.hash_one((*k).slice()) };

        let k = (entry as *const InternedPtr<T>).cast_mut();
        let res = self.slice_to_keys.insert(hash, k, eq, hasher, false, guard);

        match res {
            InsertResult::Inserted => Key::from_index(key),
            InsertResult::Replaced(_) => unreachable!("we do not replace"),
            InsertResult::Error(_) => unreachable!(
                "while holding the lock, we checked for this entry already and it was not in there"
            ),
        }
    }
}

//! Support for interning more than just string slices
//!
//! ```
//! use paracord::slice::ParaCord;
//!
//! let paracord = ParaCord::default();
//!
//! let foo = paracord.get_or_intern(&[1,2,3,4]);
//! let bar = paracord.get_or_intern(&[5,6,7,8]);
//!
//! assert_ne!(foo, bar);
//!
//! // returns the same key, no insert
//! let foo2 = paracord.get_or_intern(&[1,2,3,4]);
//! assert_eq!(foo, foo2);
//!
//! // returns the same key, guaranteed no insert
//! let foo3 = paracord.get(&[1,2,3,4]).unwrap();
//! assert_eq!(foo, foo3);
//!
//! // can be exchanged for the string
//! assert_eq!(paracord.resolve(foo), &[1,2,3,4]);
//! assert_eq!(paracord.resolve(bar), &[5,6,7,8]);
//! ```

use std::{
    hash::{BuildHasher, Hash},
    mem::size_of,
    ops::Index,
};

use alloc::Alloc;
use clashmap::ClashCollection;
use hashbrown::HashTable;

use crate::Key;

mod alloc;

/// [`ParaCord`] is a lightweight, thread-safe, memory efficient [string interer](https://en.wikipedia.org/wiki/String_interning).
///
/// When calling [`ParaCord::get_or_intern`], a [`Key`] is returned. This [`Key`] is guaranteed to be unique if the input slice is unique,
/// and is guaranteed to be the same if the input slice is the same. [`Key`] is 32bits, and has a niche value which allows `Option<Key>` to
/// also be 32bits.
///
/// If you don't want to intern the slice, but check for it's existence, you can use [`ParaCord::get`], which returns `None` if not
/// present.
///
/// [`Key`]s can be exchanged back into slices using [`ParaCord::resolve`]. It's important to keep in mind that this might panic
/// or return nonsense results if given a key returned by some other [`ParaCord`] instance.
///
/// This slice interner is not garbage collected, so slices that are allocated in the interner are not released
/// until the [`ParaCord`] instance is dropped.
///
/// # Examples
///
/// ```
/// use paracord::slice::ParaCord;
///
/// let paracord = ParaCord::default();
///
/// let foo = paracord.get_or_intern(&[1,2,3,4]);
/// let bar = paracord.get_or_intern(&[5,6,7,8]);
///
/// assert_ne!(foo, bar);
///
/// // returns the same key, no insert
/// let foo2 = paracord.get_or_intern(&[1,2,3,4]);
/// assert_eq!(foo, foo2);
///
/// // returns the same key, guaranteed no insert
/// let foo3 = paracord.get(&[1,2,3,4]).unwrap();
/// assert_eq!(foo, foo3);
///
/// // can be exchanged for the string
/// assert_eq!(paracord.resolve(foo), &[1,2,3,4]);
/// assert_eq!(paracord.resolve(bar), &[5,6,7,8]);
/// ```
pub struct ParaCord<T: 'static, S = foldhash::fast::RandomState> {
    keys_to_slice: boxcar::Vec<&'static [T]>,
    slice_to_keys: ClashCollection<Collection<T>>,
    hasher: S,
}

struct Collection<T: 'static> {
    table: HashTable<TableEntry<T>>,
    alloc: Alloc<T>,
}

impl<T: 'static + Sync> Default for Collection<T> {
    fn default() -> Self {
        Self {
            table: Default::default(),
            alloc: Default::default(),
        }
    }
}

struct TableEntry<T> {
    hash: u64,
    key: Key,
    len: u32,
    ptr: *const T,
}

// Safety: `TableEntry` has the same safety requirements as `&[T]`
unsafe impl<T: Sync> Sync for TableEntry<T> {}
// Safety: `TableEntry` has the same safety requirements as `&[T]`
unsafe impl<T: Sync> Send for TableEntry<T> {}

impl<T: 'static + Eq> TableEntry<T> {
    fn new(s: &'static [T], key: Key, hash: u64) -> Self {
        let len = u32::try_from(s.len()).expect("slice lengths must be less than u32::MAX");
        TableEntry {
            hash,
            key,
            len,
            ptr: s.as_ptr(),
        }
    }

    fn slice(&self) -> &'static [T] {
        // Safety: the ptr and len came from a &'static [T] to begin with.
        unsafe { &*core::ptr::slice_from_raw_parts(self.ptr, self.len as usize) }
    }
}

impl<T: 'static + Sync + Copy> Default for ParaCord<T> {
    fn default() -> Self {
        Self::with_hasher(Default::default())
    }
}

impl<T: 'static + Sync, S: BuildHasher> ParaCord<T, S> {
    /// Create a new `ParaCord` instance with the given hasher state.
    ///
    /// # Examples
    ///
    /// ```
    /// use paracord::slice::ParaCord;
    /// use std::hash::RandomState;
    ///
    /// let paracord = ParaCord::with_hasher(RandomState::default());
    ///
    /// let foo = paracord.get_or_intern(&[1,2,3,4]);
    /// assert_eq!(paracord.resolve(foo), &[1,2,3,4]);
    /// ```
    pub fn with_hasher(hasher: S) -> Self {
        Self {
            keys_to_slice: boxcar::Vec::default(),
            slice_to_keys: ClashCollection::default(),
            hasher,
        }
    }
}

impl<T: 'static + Sync + Hash + Eq + Copy, S: BuildHasher> ParaCord<T, S> {
    /// Try and get the [`Key`] associated with the given slice.
    /// Returns [`None`] if not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use paracord::slice::ParaCord;
    /// let paracord = ParaCord::default();
    /// let foo = paracord.get_or_intern(&[1,2,3,4]);
    ///
    /// assert_eq!(paracord.get(&[1,2,3,4]), Some(foo));
    /// assert_eq!(paracord.get(&[5,6,7,8]), None);
    /// ```
    pub fn get(&self, s: &[T]) -> Option<Key> {
        let hash = self.hasher.hash_one(s);
        let shard = self.slice_to_keys.get_read_shard(hash);
        shard.table.find(hash, |k| s == k.slice()).map(|k| k.key)
    }

    /// Try and get the [`Key`] associated with the given slice.
    /// Allocates a new key if not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use paracord::slice::ParaCord;
    /// let paracord = ParaCord::default();
    ///
    /// let foo = paracord.get_or_intern(&[1,2,3,4]);
    /// let bar = paracord.get_or_intern(&[5,6,7,8]);
    /// let foo2 = paracord.get_or_intern(&[1,2,3,4]);
    ///
    /// assert_ne!(foo, bar);
    /// assert_eq!(foo, foo2);
    /// ```
    pub fn get_or_intern(&self, s: &[T]) -> Key {
        let hash = self.hasher.hash_one(s);

        let key = {
            let shard = self.slice_to_keys.get_read_shard(hash);
            shard.table.find(hash, |k| s == k.slice()).map(|k| k.key)
        };

        let Some(key) = key else {
            return self.intern_slow(s, hash);
        };
        key
    }

    /// Try and resolve the slice associated with this [`Key`].
    ///
    /// This can only return `None` if given a key that was allocated from
    /// a different [`ParaCord`] instance, but it might return an arbitrary slice
    /// as well.
    pub fn try_resolve(&self, key: Key) -> Option<&[T]> {
        let s = self.keys_to_slice.get(key.into_repr() as usize)?;
        Some(*s)
    }

    /// Resolve the slice associated with this [`Key`].
    ///
    /// # Panics
    /// This can panic if given a key that was allocated from
    /// a different [`ParaCord`] instance, but it might return an arbitrary slice
    /// as well.
    pub fn resolve(&self, key: Key) -> &[T] {
        self.keys_to_slice[key.into_repr() as usize]
    }

    /// Resolve the slice associated with this [`Key`].
    ///
    /// # Safety
    /// This key must have been allocated in this paracord instance,
    /// and [`ParaCord::reset`] must not have been called.
    pub unsafe fn resolve_unchecked(&self, key: Key) -> &[T] {
        // Safety: If the key was allocated in self, then key is inbounds.
        unsafe { self.keys_to_slice.get_unchecked(key.into_repr() as usize) }
    }

    /// Determine how many slices have been allocated
    pub fn len(&self) -> usize {
        self.keys_to_slice.count()
    }

    /// Determine if no slices have been allocated
    pub fn is_empty(&self) -> bool {
        self.keys_to_slice.is_empty()
    }

    /// Get an iterator over every ([`Key`], `&[T]`) pair
    /// that has been allocated in this [`ParaCord`] instance.
    pub fn iter(&self) -> impl Iterator<Item = (Key, &[T])> {
        self.keys_to_slice
            .iter()
            // SAFETY: we assume the key is correct given its existence in the set
            .map(|(key, s)| unsafe { (Key::new_unchecked(key as u32), &**s) })
    }

    /// Deallocate all interned slices, but can retain some allocated memory
    pub fn reset(&mut self) {
        self.keys_to_slice.clear();
        self.slice_to_keys.shards_mut().iter_mut().for_each(|s| {
            s.get_mut().table.clear();
            drop(core::mem::take(&mut s.get_mut().alloc))
        });
    }

    /// Determine how much space has been used to allocate all the slices.
    pub fn current_memory_usage(&mut self) -> usize {
        let keys_size = self.keys_to_slice.count() * size_of::<*const str>();

        let shards_size = {
            let acc = core::mem::size_of_val(self.slice_to_keys.shards());
            self.slice_to_keys
                .shards_mut()
                .iter_mut()
                .fold(acc, |acc, shard| {
                    let shard = shard.get_mut();
                    acc + shard.table.allocation_size() + shard.alloc.size()
                })
        };

        size_of::<Self>() + keys_size + shards_size
    }
}

impl<T: 'static + Sync + Hash + Eq + Copy, I: AsRef<[T]>, S: BuildHasher + Default> FromIterator<I>
    for ParaCord<T, S>
{
    fn from_iter<A: IntoIterator<Item = I>>(iter: A) -> Self {
        let iter = iter.into_iter();
        let len = iter.size_hint().0;

        let mut this = Self {
            keys_to_slice: boxcar::Vec::with_capacity(len),
            slice_to_keys: ClashCollection::default(),
            hasher: S::default(),
        };
        this.extend(iter);
        this
    }
}

impl<T: 'static + Sync + Hash + Eq + Copy, I: AsRef<[T]>, S: BuildHasher> Extend<I>
    for ParaCord<T, S>
{
    fn extend<A: IntoIterator<Item = I>>(&mut self, iter: A) {
        // assumption, the iterator has mostly unique entries, thus this should always use the slow insert mode.
        for s in iter {
            let s = s.as_ref();
            let hash = self.hasher.hash_one(s);
            self.intern_slow_mut(s, hash);
        }
    }
}

impl<I: AsRef<str>, S: BuildHasher + Default> FromIterator<I> for crate::ParaCord<S> {
    fn from_iter<A: IntoIterator<Item = I>>(iter: A) -> Self {
        let iter = iter.into_iter();
        let len = iter.size_hint().0;

        let mut this = Self {
            inner: ParaCord {
                keys_to_slice: boxcar::Vec::with_capacity(len),
                slice_to_keys: ClashCollection::default(),
                hasher: S::default(),
            },
        };
        this.extend(iter);
        this
    }
}

impl<I: AsRef<str>, S: BuildHasher> Extend<I> for crate::ParaCord<S> {
    fn extend<A: IntoIterator<Item = I>>(&mut self, iter: A) {
        // assumption, the iterator has mostly unique entries, thus this should always use the slow insert mode.
        for s in iter {
            let s = s.as_ref().as_bytes();
            let hash = self.inner.hasher.hash_one(s);
            self.inner.intern_slow_mut(s, hash);
        }
    }
}

impl<T: 'static + Sync + Hash + Eq + Copy, S: BuildHasher> Index<Key> for ParaCord<T, S> {
    type Output = [T];

    fn index(&self, index: Key) -> &Self::Output {
        self.resolve(index)
    }
}

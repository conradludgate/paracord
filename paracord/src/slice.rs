use std::{
    hash::{BuildHasher, Hash},
    mem::size_of,
    ops::Index,
    thread::available_parallelism,
};


use clashmap::{
    tableref::{entry::Entry, entrymut::EntryMut},
    ClashTable,
};
use thread_local::ThreadLocal;
use typed_arena::Arena;

use crate::Key;

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
pub struct ParaCord<T: 'static + Send, S = foldhash::fast::RandomState> {
    keys_to_slice: boxcar::Vec<&'static [T]>,
    slice_to_keys: ClashTable<(typesize::Ref<'static, [T]>, u32)>,
    alloc: ThreadLocal<Arena<T>>,
    hasher: S,
}

impl<T: 'static + Send> Default for ParaCord<T> {
    fn default() -> Self {
        Self::with_hasher(Default::default())
    }
}

fn assert_key(key: usize) -> u32 {
    if usize::BITS >= 32 {
        assert!(key < u32::MAX as usize);
    }
    key as u32
}

unsafe fn alloc<T: Clone>(alloc: &Arena<T>, s: &[T]) -> &'static [T] {
    alloc.reserve_extend(s.len());
    let s = alloc.alloc_extend(s.iter().cloned());

    // SAFETY: caller will not drop alloc until it drops the containers storing this
    unsafe { &*(s as &[T] as *const [T]) }
}

impl<T: 'static + Send, S: BuildHasher> ParaCord<T, S> {
    /// Create a new `ParaCord` instance with the given hasher state.
    pub fn with_hasher(hasher: S) -> Self {
        Self {
            keys_to_slice: boxcar::Vec::default(),
            slice_to_keys: ClashTable::new(),
            alloc: ThreadLocal::with_capacity(available_parallelism().map_or(0, |x| x.get())),
            hasher,
        }
    }
}

impl<T: 'static + Send + Hash + Eq + Clone, S: BuildHasher> ParaCord<T, S> {
    /// Try and get the [`Key`] associated with the given slice.
    /// Returns [`None`] if not found.
    pub fn get(&self, s: &[T]) -> Option<Key> {
        let hash = self.hasher.hash_one(s);
        let key = self.slice_to_keys.find(hash, |k| s == &*k.0)?;
        // SAFETY: we assume the key is correct given its existence in the set
        Some(unsafe { Key::new_unchecked(key.1) })
    }

    /// Try and get the [`Key`] associated with the given slice.
    /// Allocates a new key if not found.
    pub fn get_or_intern(&self, s: &[T]) -> Key {
        let hash = self.hasher.hash_one(s);
        let Some(key) = self.slice_to_keys.find(hash, |k| s == &*k.0) else {
            return self.intern_slow(s, hash);
        };
        // SAFETY: we assume the key is correct given its existence in the set
        unsafe { Key::new_unchecked(key.1) }
    }

    #[cold]
    fn intern_slow(&self, s: &[T], hash: u64) -> Key {
        match self
            .slice_to_keys
            .entry(hash, |k| s == &*k.0, |k| self.hasher.hash_one(&*k.0))
        {
            // SAFETY: we assume the key is correct given its existence in the set
            Entry::Occupied(entry) => unsafe { Key::new_unchecked(entry.get().1) },
            Entry::Vacant(entry) => {
                // SAFETY: we will not drop bump until we drop the containers storing these `&'static [T]`.
                let s = unsafe { alloc(self.alloc.get_or_default(), s) };

                let key = self.keys_to_slice.push(s);
                let key = assert_key(key);
                entry.insert((typesize::Ref(s), key));

                // SAFETY: as asserted the key is correct
                unsafe { Key::new_unchecked(key) }
            }
        }
    }

    #[cold]
    fn intern_slow_mut(&mut self, s: &[T], hash: u64) -> Key {
        match self
            .slice_to_keys
            .entry_mut(hash, |k| s == &*k.0, |k| self.hasher.hash_one(&*k.0))
        {
            // SAFETY: we assume the key is correct given its existence in the set
            EntryMut::Occupied(entry) => unsafe { Key::new_unchecked(entry.get().1) },
            EntryMut::Vacant(entry) => {
                // SAFETY: we will not drop bump until we drop the containers storing these `&'static [T]`.
                let s = unsafe { alloc(self.alloc.get_or_default(), s) };

                let key = self.keys_to_slice.push(s);
                let key = assert_key(key);
                entry.insert((typesize::Ref(s), key));

                // SAFETY: as asserted the key is correct
                unsafe { Key::new_unchecked(key) }
            }
        }
    }

    /// Try and get the [`Key`] associated with the given slice.
    /// Allocates a new key if not found.
    ///
    /// Unlike [`ParaCord::get_or_intern`], this function does not need to also allocate the slice.
    pub fn get_or_intern_static(&self, s: &'static [T]) -> Key {
        let hash = self.hasher.hash_one(s);
        let Some(key) = self.slice_to_keys.find(hash, |k| s == &*k.0) else {
            return self.intern_static_slow(s, hash);
        };
        // SAFETY: we assume the key is correct given its existence in the set
        unsafe { Key::new_unchecked(key.1) }
    }

    #[cold]
    fn intern_static_slow(&self, s: &'static [T], hash: u64) -> Key {
        match self
            .slice_to_keys
            .entry(hash, |k| s == &*k.0, |k| self.hasher.hash_one(&*k.0))
        {
            // SAFETY: we assume the key is correct given its existence in the set
            Entry::Occupied(entry) => unsafe { Key::new_unchecked(entry.get().1) },
            Entry::Vacant(entry) => {
                let key = self.keys_to_slice.push(s);
                let key = assert_key(key);
                entry.insert((typesize::Ref(s), key));

                // SAFETY: as asserted the key is correct
                unsafe { Key::new_unchecked(key) }
            }
        }
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
        self.slice_to_keys.clear();
        self.alloc.iter_mut().for_each(|b| drop(core::mem::take(b)));
    }

    /// Determine how much space has been used to allocate all the slices.
    pub fn current_memory_usage(&mut self) -> usize {
        use typesize::TypeSize;
        size_of::<Self>()
            + self.keys_to_slice.count() * size_of::<*const str>()
            + self.slice_to_keys.extra_size()
            + self
                .alloc
                .iter_mut()
                .map(|b| b.len() * size_of::<T>())
                .sum::<usize>()
    }
}

impl<T: 'static + Send + Hash + Eq + Clone, I: AsRef<[T]>, S: BuildHasher + Default> FromIterator<I>
    for ParaCord<T, S>
{
    fn from_iter<A: IntoIterator<Item = I>>(iter: A) -> Self {
        let iter = iter.into_iter();
        let len = iter.size_hint().0;

        let mut this = Self {
            keys_to_slice: boxcar::Vec::with_capacity(len),
            slice_to_keys: ClashTable::with_capacity(len),
            alloc: ThreadLocal::with_capacity(available_parallelism().map_or(0, |x| x.get())),
            hasher: S::default(),
        };
        this.extend(iter);
        this
    }
}

impl<T: 'static + Send + Hash + Eq + Clone, I: AsRef<[T]>, S: BuildHasher> Extend<I>
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
                slice_to_keys: ClashTable::with_capacity(len),
                alloc: ThreadLocal::with_capacity(available_parallelism().map_or(0, |x| x.get())),
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

impl<T: 'static + Send + Hash + Eq + Clone, S: BuildHasher> Index<Key> for ParaCord<T, S> {
    type Output = [T];

    fn index(&self, index: Key) -> &Self::Output {
        self.resolve(index)
    }
}

//! [`ParaCord`] is a lightweight, thread-safe, memory efficient [string interner](https://en.wikipedia.org/wiki/String_interning).
//!
//! When calling [`ParaCord::get_or_intern`], a [`Key`] is returned. This [`Key`] is guaranteed to be unique if the input string is unique,
//! and is guaranteed to be the same if the input string is the same. [`Key`] is 32bits, and has a niche value which allows `Option<Key>` to
//! also be 32bits.
//!
//! The 32bit key imposes a limitation that allocating 2^32 strings will panic. There's an additional self-imposed limitation that
//! no string can be longer than 2^32 bytes long.
//!
//! If you don't want to intern the string, but check for it's existence, you can use [`ParaCord::get`], which returns `None` if not
//! present.
//!
//! [`Key`]s can be exchanged back into strings using [`ParaCord::resolve`]. It's important to keep in mind that this might panic
//! or return nonsense results if given a key returned by some other [`ParaCord`] instance.
//!
//! This string interner is not garbage collected, so strings that are allocated in the interner are not released
//! until the [`ParaCord`] instance is dropped.
//!
//! # Examples
//!
//! With a self-managed `ParaCord` instance.
//!
//! ```
//! use paracord::ParaCord;
//!
//! let paracord = ParaCord::default();
//!
//! let foo = paracord.get_or_intern("foo");
//! let bar = paracord.get_or_intern("bar");
//!
//! assert_ne!(foo, bar);
//!
//! // returns the same key, no insert
//! let foo2 = paracord.get_or_intern("foo");
//! assert_eq!(foo, foo2);
//!
//! // returns the same key, guaranteed no insert
//! let foo3 = paracord.get("foo").unwrap();
//! assert_eq!(foo, foo3);
//!
//! // can be exchanged for the string
//! assert_eq!(paracord.resolve(foo), "foo");
//! assert_eq!(paracord.resolve(bar), "bar");
//! ```
//!
//! With a globally managed instance, with typed keys
//!
//! ```
//! paracord::custom_key!(pub struct NameKey);
//!
//! let foo = NameKey::new("foo");
//! let bar = NameKey::new("bar");
//!
//! assert_ne!(foo, bar);
//!
//! // returns the same key, no insert
//! let foo2 = NameKey::new("foo");
//! assert_eq!(foo, foo2);
//!
//! // returns the same key, guaranteed no insert
//! let foo3 = NameKey::try_new_existing("foo").unwrap();
//! assert_eq!(foo, foo3);
//!
//! // can be exchanged for the string
//! assert_eq!(foo.as_str(), "foo");
//! assert_eq!(bar.as_str(), "bar");
//! ```
#![warn(
    unsafe_op_in_unsafe_fn,
    clippy::missing_safety_doc,
    clippy::multiple_unsafe_ops_per_block,
    clippy::undocumented_unsafe_blocks
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use core::fmt;
use std::hash::{BuildHasher, Hash};
use std::num::NonZeroU32;
use std::ops::Index;

pub mod slice;

mod macros;

#[cfg(feature = "serde")]
mod serde;
#[cfg(not(feature = "serde"))]
mod serde {
    #[doc(hidden)]
    #[macro_export]
    macro_rules! custom_key_serde {
        ($key:ident) => {};
    }

    pub use custom_key_serde;
}

#[doc(hidden)]
pub mod __private {
    pub use foldhash::fast::RandomState;
    pub mod serde {
        pub use crate::serde::*;
    }
}

custom_key!(
    /// A key that allocates in a global [`ParaCord`] instance.
    ///
    /// Custom global keys can be defined using [`custom_key`]
    ///
    /// ```
    /// use paracord::DefaultKey;
    ///
    /// let key = DefaultKey::new("foo");
    /// assert_eq!(key.as_str(), "foo");
    ///
    /// let key2 = DefaultKey::try_new_existing("foo").unwrap();
    /// assert_eq!(key, key2);
    /// ```
    pub struct DefaultKey;
);

/// Key type returned by [`ParaCord`].
///
/// [`Key`] implements [`core::cmp::Ord`] for use within collections like [`BTreeMap`](std::collections::BTreeMap),
/// but the order is not defined to be meaningful or relied upon. Treat [`Key`]s as opaque blobs, with an unstable representation.
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
#[repr(transparent)]
pub struct Key(NonZeroU32);

impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Key").field(&self.into_repr()).finish()
    }
}

impl Key {
    /// Turn the key into a u32.
    ///
    /// The only guarantee is that [`Key::try_from_repr`] is the inverse of this function,
    /// and will always return the same key.
    ///
    /// ```
    /// use paracord::Key;
    /// # let paracord = paracord::ParaCord::default();
    /// # let key = paracord.get_or_intern("");
    /// let key2 = Key::try_from_repr(key.into_repr()).unwrap();
    /// assert_eq!(key, key2);
    /// ``````
    #[inline]
    pub fn into_repr(self) -> u32 {
        self.0.get() ^ u32::MAX
    }

    /// Recreate the key from a u32.
    ///
    /// The only guarantee is that [`Key::into_repr`] is the inverse of this function,
    /// and will always return the same u32.
    #[inline]
    pub fn try_from_repr(x: u32) -> Option<Self> {
        NonZeroU32::new(x ^ u32::MAX).map(Self)
    }

    /// Safety: i must be less than `u32::MAX`
    #[inline]
    unsafe fn new_unchecked(i: u32) -> Self {
        // SAFETY: from caller
        Key(unsafe { NonZeroU32::new_unchecked(i ^ u32::MAX) })
    }

    #[inline]
    fn from_index(i: usize) -> Self {
        if usize::BITS >= 32 {
            assert!(i < u32::MAX as usize);
        }

        // SAFETY: checked it is less than u32::MAX.
        unsafe { Self::new_unchecked(i as u32) }
    }
}

/// [`ParaCord`] is a lightweight, thread-safe, memory efficient [string interer](https://en.wikipedia.org/wiki/String_interning).
///
/// When calling [`ParaCord::get_or_intern`], a [`Key`] is returned. This [`Key`] is guaranteed to be unique if the input string is unique,
/// and is guaranteed to be the same if the input string is the same. [`Key`] is 32bits, and has a niche value which allows `Option<Key>` to
/// also be 32bits.
///
/// If you don't want to intern the string, but check for it's existence, you can use [`ParaCord::get`], which returns `None` if not
/// present.
///
/// [`Key`]s can be exchanged back into strings using [`ParaCord::resolve`]. It's important to keep in mind that this might panic
/// or return nonsense results if given a key returned by some other [`ParaCord`] instance.
///
/// This string interner is not garbage collected, so strings that are allocated in the interner are not released
/// until the [`ParaCord`] instance is dropped.
///
/// # Examples
///
/// ```
/// use paracord::ParaCord;
///
/// let paracord = ParaCord::default();
///
/// let foo = paracord.get_or_intern("foo");
/// let bar = paracord.get_or_intern("bar");
///
/// assert_ne!(foo, bar);
///
/// // returns the same key, no insert
/// let foo2 = paracord.get_or_intern("foo");
/// assert_eq!(foo, foo2);
///
/// // returns the same key, guaranteed no insert
/// let foo3 = paracord.get("foo").unwrap();
/// assert_eq!(foo, foo3);
///
/// // can be exchanged for the string
/// assert_eq!(paracord.resolve(foo), "foo");
/// assert_eq!(paracord.resolve(bar), "bar");
/// ```
pub struct ParaCord<S = foldhash::fast::RandomState> {
    inner: slice::ParaCord<u8, S>,
}

impl<S> fmt::Debug for ParaCord<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl Default for ParaCord {
    #[inline]
    fn default() -> Self {
        Self::with_hasher(foldhash::fast::RandomState::default())
    }
}

impl<S: BuildHasher> ParaCord<S> {
    /// Create a new `ParaCord` instance with the given hasher state.
    ///
    /// # Examples
    ///
    /// ```
    /// use paracord::ParaCord;
    /// use std::hash::RandomState;
    ///
    /// let paracord = ParaCord::with_hasher(RandomState::default());
    ///
    /// let foo = paracord.get_or_intern("foo");
    /// assert_eq!(paracord.resolve(foo), "foo");
    /// ```
    #[inline]
    pub fn with_hasher(hasher: S) -> Self {
        Self {
            inner: slice::ParaCord::with_hasher(hasher),
        }
    }

    /// Try and get the [`Key`] associated with the given string.
    /// Returns [`None`] if not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use paracord::ParaCord;
    /// let paracord = ParaCord::default();
    /// let foo = paracord.get_or_intern("foo");
    ///
    /// assert_eq!(paracord.get("foo"), Some(foo));
    /// assert_eq!(paracord.get("bar"), None);
    /// ```
    #[inline]
    pub fn get(&self, s: &str) -> Option<Key> {
        self.inner.get(s.as_bytes())
    }

    /// Try and get the [`Key`] associated with the given string.
    /// Allocates a new key if not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use paracord::ParaCord;
    /// let paracord = ParaCord::default();
    ///
    /// let foo = paracord.get_or_intern("foo");
    /// let bar = paracord.get_or_intern("bar");
    /// let foo2 = paracord.get_or_intern("foo");
    ///
    /// assert_ne!(foo, bar);
    /// assert_eq!(foo, foo2);
    /// ```
    #[inline]
    pub fn get_or_intern(&self, s: &str) -> Key {
        self.inner.get_or_intern(s.as_bytes())
    }
}

impl<S> ParaCord<S> {
    /// Try and resolve the string associated with this [`Key`].
    ///
    /// This can only return `None` if given a key that was allocated from
    /// a different [`ParaCord`] instance, but it might return an arbitrary string
    /// as well.
    ///
    /// # Examples
    ///
    /// ```
    /// use paracord::ParaCord;
    /// let paracord = ParaCord::default();
    ///
    /// let foo = paracord.get_or_intern("foo");
    /// assert_eq!(paracord.try_resolve(foo), Some("foo"));
    ///
    /// let paracord = ParaCord::default();
    /// assert_eq!(paracord.try_resolve(foo), None);
    /// ```
    #[inline]
    pub fn try_resolve(&self, key: Key) -> Option<&str> {
        self.inner
            .try_resolve(key)
            // Safety: we insert only strings, so it's valid utf8
            .map(|s| unsafe { core::str::from_utf8_unchecked(s) })
    }

    /// Resolve the string associated with this [`Key`].
    ///
    /// # Panics
    /// This can panic if given a key that was allocated from
    /// a different [`ParaCord`] instance, but it might return an arbitrary string
    /// as well.
    ///
    /// # Examples
    ///
    /// ```
    /// use paracord::ParaCord;
    /// let paracord = ParaCord::default();
    ///
    /// let foo = paracord.get_or_intern("foo");
    /// assert_eq!(paracord.resolve(foo), "foo");
    /// ```
    #[inline]
    pub fn resolve(&self, key: Key) -> &str {
        let b = self.inner.resolve(key);

        // Safety: we insert only strings, so it's valid utf8
        unsafe { core::str::from_utf8_unchecked(b) }
    }

    /// Resolve the string associated with this [`Key`].
    ///
    /// # Safety
    /// This key must have been allocated in this paracord instance,
    /// and [`ParaCord::clear`] must not have been called.
    ///
    /// # Examples
    ///
    /// ```
    /// use paracord::ParaCord;
    /// let paracord = ParaCord::default();
    ///
    /// let foo = paracord.get_or_intern("foo");
    /// // Safety: `foo` was allocated within paracord just above,
    /// // and we never clear the paracord instance.
    /// assert_eq!(unsafe { paracord.resolve_unchecked(foo) }, "foo");
    /// ```
    #[inline]
    pub unsafe fn resolve_unchecked(&self, key: Key) -> &str {
        // Safety: from caller.
        let b = unsafe { self.inner.resolve_unchecked(key) };

        // Safety: we insert only strings, so it's valid utf8
        unsafe { core::str::from_utf8_unchecked(b) }
    }

    /// Determine how many strings have been allocated
    ///
    /// # Examples
    ///
    /// ```
    /// use paracord::ParaCord;
    /// let paracord = ParaCord::default();
    ///
    /// let _ = paracord.get_or_intern("foo");
    /// assert_eq!(paracord.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Determine if no strings have been allocated
    ///
    /// # Examples
    ///
    /// ```
    /// use paracord::ParaCord;
    /// let paracord = ParaCord::default();
    ///
    /// assert!(paracord.is_empty());
    ///
    /// let _ = paracord.get_or_intern("foo");
    /// assert!(!paracord.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get an iterator over every ([`Key`], [`&str`]) pair
    /// that has been allocated in this [`ParaCord`] instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use paracord::ParaCord;
    /// let paracord = ParaCord::default();
    ///
    /// let foo = paracord.get_or_intern("foo");
    /// let bar = paracord.get_or_intern("bar");
    ///
    /// let entries: Vec<_> = paracord.iter().collect();
    /// assert_eq!(entries, vec![(foo, "foo"), (bar, "bar")]);
    /// ```
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (Key, &str)> {
        self.into_iter()
    }

    /// Deallocate all interned strings, but can retain some allocated memory
    ///
    /// # Examples
    ///
    /// ```
    /// use paracord::ParaCord;
    /// let mut paracord = ParaCord::default();
    ///
    /// let foo = paracord.get_or_intern("foo");
    /// assert_eq!(paracord.try_resolve(foo), Some("foo"));
    ///
    /// paracord.clear();
    /// assert!(paracord.is_empty());
    ///
    /// assert_eq!(paracord.try_resolve(foo), None);
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[cfg(test)]
    /// Determine how much space has been used to allocate all the strings.
    ///
    /// # Examples
    ///
    /// ```
    /// use paracord::ParaCord;
    /// let mut paracord = ParaCord::default();
    ///
    /// let _mem = paracord.current_memory_usage();
    /// ```
    pub(crate) fn current_memory_usage(&mut self) -> usize {
        self.inner.current_memory_usage()
    }
}

impl<S> Index<Key> for ParaCord<S> {
    type Output = str;

    fn index(&self, index: Key) -> &Self::Output {
        self.resolve(index)
    }
}

#[repr(transparent)]
struct AsBytes<S: AsRef<str>>(S);
impl<S: AsRef<str>> AsRef<[u8]> for AsBytes<S> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref().as_bytes()
    }
}

impl<I: AsRef<str>, S: BuildHasher + Default> FromIterator<I> for crate::ParaCord<S> {
    fn from_iter<A: IntoIterator<Item = I>>(iter: A) -> Self {
        Self {
            inner: iter.into_iter().map(AsBytes).collect(),
        }
    }
}

impl<I: AsRef<str>, S: BuildHasher> Extend<I> for crate::ParaCord<S> {
    fn extend<A: IntoIterator<Item = I>>(&mut self, iter: A) {
        self.inner.extend(iter.into_iter().map(AsBytes));
    }
}

mod iter_private {
    use crate::Key;

    pub struct Iter<'a> {
        pub(crate) inner: crate::slice::iter_private::Iter<'a, u8>,
    }

    impl<'a> Iterator for Iter<'a> {
        type Item = (Key, &'a str);

        fn next(&mut self) -> Option<Self::Item> {
            let (key, s) = self.inner.next()?;
            // Safety: we insert only strings, so it's valid utf8
            Some(unsafe { (key, core::str::from_utf8_unchecked(s)) })
        }
    }
}

impl<'a, S> IntoIterator for &'a ParaCord<S> {
    type Item = (Key, &'a str);
    type IntoIter = iter_private::Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        iter_private::Iter {
            inner: self.inner.into_iter(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::RandomState;
    use std::sync::{Arc, Barrier};
    use std::thread;

    use crate::{DefaultKey, Key, ParaCord};

    #[test]
    fn works() {
        let paracord = ParaCord::default();

        let foo = paracord.get_or_intern("foo");
        let bar = paracord.get_or_intern("bar");
        let foo2 = paracord.get_or_intern("foo");

        assert_eq!(foo, foo2);
        assert_ne!(foo, bar);
        assert_eq!(paracord.resolve(foo), "foo");
        assert_eq!(paracord.resolve(bar), "bar");
    }

    #[test]
    fn with_hasher() {
        let paracord: ParaCord<RandomState> = ParaCord::with_hasher(RandomState::new());

        let key = paracord.get_or_intern("Test");
        assert_eq!("Test", paracord.resolve(key));
    }

    #[test]
    fn get_or_intern() {
        let paracord = ParaCord::default();

        let a = paracord.get_or_intern("A");
        assert_eq!(a, paracord.get_or_intern("A"));

        let b = paracord.get_or_intern("B");
        assert_eq!(b, paracord.get_or_intern("B"));

        let c = paracord.get_or_intern("C");
        assert_eq!(c, paracord.get_or_intern("C"));
    }

    #[test]
    fn get_or_intern_threaded() {
        const THREADS: usize = 10;

        let barrier = Barrier::new(THREADS);
        let paracord = ParaCord::default();

        std::thread::scope(|s| {
            for _ in 0..THREADS {
                s.spawn(|| {
                    barrier.wait();

                    let a = paracord.get_or_intern("A");
                    assert_eq!(a, paracord.get_or_intern("A"));

                    let b = paracord.get_or_intern("B");
                    assert_eq!(b, paracord.get_or_intern("B"));

                    let c = paracord.get_or_intern("C");
                    assert_eq!(c, paracord.get_or_intern("C"));
                });
            }
        });
    }

    #[test]
    fn get() {
        let paracord = ParaCord::default();
        let key = paracord.get_or_intern("A");

        assert_eq!(Some(key), paracord.get("A"));
    }

    #[test]

    fn get_threaded() {
        let paracord = Arc::new(ParaCord::default());
        let key = paracord.get_or_intern("A");

        let moved = Arc::clone(&paracord);
        thread::spawn(move || {
            assert_eq!(Some(key), moved.get("A"));
        });

        assert_eq!(Some(key), paracord.get("A"));
    }

    #[test]
    fn resolve() {
        let paracord = ParaCord::default();
        let key = paracord.get_or_intern("A");

        assert_eq!("A", paracord.resolve(key));
    }

    #[test]
    #[should_panic]
    fn resolve_panics() {
        let paracord = ParaCord::default();
        paracord.resolve(Key::try_from_repr(100).unwrap());
    }

    #[test]
    fn resolve_threaded() {
        let paracord = Arc::new(ParaCord::default());
        let key = paracord.get_or_intern("A");

        let moved = Arc::clone(&paracord);
        thread::spawn(move || {
            assert_eq!("A", moved.resolve(key));
        });

        assert_eq!("A", paracord.resolve(key));
    }

    #[test]
    fn resolve_panics_threaded() {
        let paracord = Arc::new(ParaCord::default());
        let key = paracord.get_or_intern("A");

        let moved = Arc::clone(&paracord);
        let handle = thread::spawn(move || {
            assert_eq!("A", moved.resolve(key));
            moved.resolve(Key::try_from_repr(100).unwrap());
        });

        assert_eq!("A", paracord.resolve(key));
        assert!(handle.join().is_err());
    }

    #[test]
    fn try_resolve() {
        let paracord = ParaCord::default();
        let key = paracord.get_or_intern("A");

        assert_eq!(Some("A"), paracord.try_resolve(key));
        assert_eq!(None, paracord.try_resolve(Key::try_from_repr(100).unwrap()));
    }

    #[test]
    fn try_resolve_threaded() {
        let paracord = Arc::new(ParaCord::default());
        let key = paracord.get_or_intern("A");

        let moved = Arc::clone(&paracord);
        thread::spawn(move || {
            assert_eq!(Some("A"), moved.try_resolve(key));
            assert_eq!(None, moved.try_resolve(Key::try_from_repr(100).unwrap()));
        });

        assert_eq!(Some("A"), paracord.try_resolve(key));
        assert_eq!(None, paracord.try_resolve(Key::try_from_repr(100).unwrap()));
    }

    #[test]
    fn len() {
        let paracord = ParaCord::default();
        paracord.get_or_intern("A");
        paracord.get_or_intern("B");
        paracord.get_or_intern("C");

        assert_eq!(paracord.len(), 3);
    }

    #[test]
    fn empty() {
        let paracord = ParaCord::default();

        assert!(paracord.is_empty());
    }

    #[test]
    fn drops() {
        let _ = ParaCord::default();
    }

    #[test]
    fn drop_threaded() {
        let paracord = Arc::new(ParaCord::default());

        let moved = Arc::clone(&paracord);
        thread::spawn(move || {
            let _ = moved;
        });
    }

    #[test]
    fn memory() {
        let mut paracord = ParaCord::default();
        paracord.get_or_intern("A");
        paracord.get_or_intern("B");
        paracord.get_or_intern("C");

        assert!(paracord.current_memory_usage() > 0);
    }

    #[test]
    fn clear() {
        let mut paracord = ParaCord::default();
        let k1 = paracord.get_or_intern("A");
        paracord.clear();

        assert!(paracord.try_resolve(k1).is_none());
        assert!(paracord.is_empty());
    }

    #[test]
    fn iter() {
        let paracord = ParaCord::default();
        paracord.get_or_intern("A");
        paracord.get_or_intern("B");
        paracord.get_or_intern("C");
        let values: Vec<_> = paracord.iter().map(|(k, v)| (k.into_repr(), v)).collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&(0, "A")));
        assert!(values.contains(&(1, "B")));
        assert!(values.contains(&(2, "C")));
    }

    #[test]
    fn from_iter() {
        let paracord: ParaCord = ["a", "b", "c", "d", "e"].iter().collect();

        assert!(paracord.get("a").is_some());
        assert!(paracord.get("b").is_some());
        assert!(paracord.get("c").is_some());
        assert!(paracord.get("d").is_some());
        assert!(paracord.get("e").is_some());
    }

    #[test]
    fn index() {
        let paracord = ParaCord::default();
        let key = paracord.get_or_intern("A");

        assert_eq!("A", &paracord[key]);
    }

    #[test]
    fn extend() {
        let mut paracord = ParaCord::default();
        assert!(paracord.is_empty());

        paracord.extend(["a", "b", "c", "d", "e"].iter());
        assert!(paracord.get("a").is_some());
        assert!(paracord.get("b").is_some());
        assert!(paracord.get("c").is_some());
        assert!(paracord.get("d").is_some());
        assert!(paracord.get("e").is_some());
    }

    // Test for race conditions on key insertion
    // https://github.com/Kixiron/lasso/issues/18
    #[test]
    fn get_or_intern_threaded_racy() {
        const THREADS: usize = 10;

        let barrier = Barrier::new(THREADS);
        let paracord = ParaCord::default();
        let expected = Key::try_from_repr(0).unwrap();

        std::thread::scope(|s| {
            for _ in 0..THREADS {
                s.spawn(|| {
                    barrier.wait();
                    assert_eq!(expected, paracord.get_or_intern("A"));
                    assert_eq!(expected, paracord.get_or_intern("A"));
                    assert_eq!(expected, paracord.get_or_intern("A"));
                    assert_eq!(expected, paracord.get_or_intern("A"));
                });
            }
        });
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde() {
        let key = DefaultKey::new("hello");

        serde_test::assert_de_tokens(&key, &[serde_test::Token::Str("hello")]);
        serde_test::assert_ser_tokens(&key, &[serde_test::Token::Str("hello")]);
    }

    #[test]
    #[cfg(not(miri))]
    fn memory_usage() {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};
        use rand_distr::Zipf;

        let endpoint_dist = Zipf::new(500000.0, 0.8).unwrap();
        let endpoints = StdRng::seed_from_u64(272488357).sample_iter(endpoint_dist);

        let mut interner = ParaCord::default();

        const N: usize = 1_000_000;
        let mut verify = Vec::with_capacity(N);
        for endpoint in endpoints.take(N) {
            let endpoint = format!("ep-string-interning-{endpoint}");
            let key = interner.get_or_intern(&endpoint);
            verify.push((endpoint, key));
        }

        for (s, key) in verify {
            assert_eq!(interner[key], s);
        }

        let mem = interner.current_memory_usage();
        let len = interner.len();

        // average 70 bytes per string.
        // average string length is 24, so 46 bytes overhead.
        assert_eq!(mem / len, 70);
    }
}

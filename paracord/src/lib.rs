//! [`ParaCord`] is a lightweight, thread-safe, memory efficient [string interer](https://en.wikipedia.org/wiki/String_interning).
//!
//! When calling [`ParaCord::get_or_intern`], a [`Key`] is returned. This [`Key`] is guaranteed to be unique if the input string is unique,
//! and is guaranteed to be the same if the input string is the same. [`Key`] is 32bits, and has a niche value which allows `Option<Key>` to
//! also be 32bits.
//!
//! If you don't want to intern the string, but check for it's existence, you can use [`ParaCord::get`], which returns `None` if not
//! present.
//!
//! [`Key`]s can be exchanged back into strings using [`ParaCord::resolve`]. It's important to keep in mind that this might panic
//! or return nonsense results if given a key returned by some other [`ParaCord`] instance.
//!
//! This string interner is not garbage collected, so strings that are allocated in the interner are not released
//! until the [`ParaCord`] instance is dropped.

#![warn(
    unsafe_op_in_unsafe_fn,
    clippy::missing_safety_doc,
    clippy::multiple_unsafe_ops_per_block,
    clippy::undocumented_unsafe_blocks
)]

use std::{
    hash::{BuildHasher, Hash},
    num::NonZeroU32,
    ops::Index,
};

/// Support for interning more than just string slices
pub mod slice;

#[cfg(feature = "serde")]
mod serde;
#[cfg(not(feature = "serde"))]
mod serde {
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

/// Create a new custom key, with a static-backed allocator.
///
/// See [`DefaultKey`] for docs on what this macro generates.
///
/// ## Create a custom key
///
/// ```
/// paracord::custom_key!(
///     /// My custom key
///     pub struct MyKey;
/// );
///
/// let key = MyKey::get_or_intern("foo");
/// assert_eq!(key.resolve(), "foo");
///
/// let key2 = MyKey::get("foo").unwrap();
/// assert_eq!(key, key2);
/// ```
///
/// ## Create a custom key with a different default hasher
///
/// ```
/// use foldhash::quality::RandomState;
///
/// paracord::custom_key!(
///     /// My custom key
///     pub struct MyKey;
///
///     let hasher: RandomState;
/// );
/// ```
///
/// ## Create a custom key with a different hasher and init function
///
/// ```
/// use foldhash::quality::FixedState;
///
/// paracord::custom_key!(
///     /// My custom key
///     pub struct MyKey;
///
///     let hasher: FixedState = FixedState::with_seed(1);
/// );
/// ```
#[macro_export]
macro_rules! custom_key {
    ($(#[$($meta:meta)*])* $vis:vis struct $key:ident $(;)?) => {
        $crate::custom_key!(
            $(#[$($meta)*])*
            $vis struct $key;

            let hasher: $crate::__private::RandomState;
        );
    };
    ($(#[$($meta:meta)*])* $vis:vis struct $key:ident; let hasher: $s:ty $(;)?) => {
        $crate::custom_key!(
            $(#[$($meta)*])*
            $vis struct $key;

            let hasher: $s = <$s as ::core::default::Default>::default();
        );
    };
    ($(#[$($meta:meta)*])* $vis:vis struct $key:ident; let hasher: $s:ty = $init:expr $(;)?) => {
        $(#[$($meta)*])*
        #[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Clone, Copy)]
        #[repr(transparent)]
        $vis struct $key($crate::Key);

        impl $key {
            fn paracord() -> &'static $crate::ParaCord<$s> {
                static S: ::std::sync::OnceLock<$crate::ParaCord<$s>> = ::std::sync::OnceLock::new();
                S.get_or_init(|| $crate::ParaCord::with_hasher($init))
            }

            /// Try and get the key associated with the given string.
            /// Returns [`None`] if not found.
            pub fn get(s: &str) -> Option<Self> {
                Self::paracord().get(s).map(Self)
            }

            /// Try and get the key associated with the given string.
            /// Allocates a new key if not found.
            pub fn get_or_intern(s: &str) -> Self {
                Self(Self::paracord().get_or_intern(s))
            }

            /// Resolve the string associated with this key.
            pub fn resolve(self) -> &'static str {
                // Safety: The key can only be constructed from the static paracord,
                // and the paracord will never be reset.
                unsafe { Self::paracord().resolve_unchecked(self.0) }
            }

            /// Determine how many strings have been allocated
            pub fn len() -> usize {
                Self::paracord().len()
            }

            /// Determine if no strings have been allocated
            pub fn is_empty() -> bool {
                Self::paracord().is_empty()
            }

            /// Get an iterator over every
            #[doc = concat!("(`",stringify!($key),"`, `&str`)")]
            /// pair that has been allocated in this [`ParaCord`] instance.
            pub fn iter() -> impl Iterator<Item = (Self, &'static str)> {
                Self::paracord().iter().map(|(k, s)| (Self(k), s))
            }
        }

        $crate::__private::serde::custom_key_serde!($key);
    };
}

custom_key!(
    /// A key that allocates in a global [`ParaCord`] instance.
    ///
    /// Custom global keys can be defined using [`custom_key`]
    ///
    /// ```
    /// use paracord::DefaultKey;
    ///
    /// let key = DefaultKey::get_or_intern("foo");
    /// assert_eq!(key.resolve(), "foo");
    ///
    /// let key2 = DefaultKey::get("foo").unwrap();
    /// assert_eq!(key, key2);
    /// ```
    pub struct DefaultKey;
);

/// Key type returned by [`ParaCord`].
///
/// [`Key`] implements [`core::cmp::Ord`] for use within collections like [`BTreeMap`](std::collections::BTreeMap),
/// but the order is not defined to be meaningful or relied upon. Treat [`Key`]s as opaque blobs, with an unstable representation.
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Key(NonZeroU32);

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
    pub fn into_repr(self) -> u32 {
        self.0.get() ^ u32::MAX
    }

    /// Recreate the key from a u32.
    ///
    /// The only guarantee is that [`Key::into_repr`] is the inverse of this function,
    /// and will always return the same u32.
    pub fn try_from_repr(x: u32) -> Option<Self> {
        NonZeroU32::new(x ^ u32::MAX).map(Self)
    }

    /// Safety: i must be less than u32::MAX
    unsafe fn new_unchecked(i: u32) -> Self {
        // SAFETY: from caller
        Key(unsafe { NonZeroU32::new_unchecked(i ^ u32::MAX) })
    }

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
pub struct ParaCord<S = foldhash::fast::RandomState> {
    inner: slice::ParaCord<u8, S>,
}

impl Default for ParaCord {
    fn default() -> Self {
        Self::with_hasher(Default::default())
    }
}

impl<S: BuildHasher> ParaCord<S> {
    /// Create a new `ParaCord` instance with the given hasher state.
    pub fn with_hasher(hasher: S) -> Self {
        Self {
            inner: slice::ParaCord::with_hasher(hasher),
        }
    }

    /// Try and get the [`Key`] associated with the given string.
    /// Returns [`None`] if not found.
    pub fn get(&self, s: &str) -> Option<Key> {
        self.inner.get(s.as_bytes())
    }

    /// Try and get the [`Key`] associated with the given string.
    /// Allocates a new key if not found.
    pub fn get_or_intern(&self, s: &str) -> Key {
        self.inner.get_or_intern(s.as_bytes())
    }

    /// Try and resolve the string associated with this [`Key`].
    ///
    /// This can only return `None` if given a key that was allocated from
    /// a different [`ParaCord`] instance, but it might return an arbitrary string
    /// as well.
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
    pub fn resolve(&self, key: Key) -> &str {
        // Safety: we insert only strings, so it's valid utf8
        unsafe { core::str::from_utf8_unchecked(self.inner.resolve(key)) }
    }

    /// Resolve the string associated with this [`Key`].
    ///
    /// # Safety
    /// This key must have been allocated in this paracord instance,
    /// and [`ParaCord::reset`] must not have been called.
    pub unsafe fn resolve_unchecked(&self, key: Key) -> &str {
        // Safety: from caller.
        let b = unsafe { self.inner.resolve_unchecked(key) };

        // Safety: we insert only strings, so it's valid utf8
        unsafe { core::str::from_utf8_unchecked(b) }
    }

    /// Determine how many strings have been allocated
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Determine if no strings have been allocated
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get an iterator over every ([`Key`], [`&str`]) pair
    /// that has been allocated in this [`ParaCord`] instance.
    pub fn iter(&self) -> impl Iterator<Item = (Key, &str)> {
        self.inner
            .iter()
            // Safety: we insert only strings, so it's valid utf8
            .map(|(key, s)| unsafe { (key, core::str::from_utf8_unchecked(s)) })
    }

    /// Deallocate all interned strings, but can retain some allocated memory
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Determine how much space has been used to allocate all the strings.
    pub fn current_memory_usage(&mut self) -> usize {
        self.inner.current_memory_usage()
    }
}

impl<S: BuildHasher> Index<Key> for ParaCord<S> {
    type Output = str;

    fn index(&self, index: Key) -> &Self::Output {
        self.resolve(index)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::hash_map::RandomState,
        sync::{Arc, Barrier},
        thread,
    };

    use crate::ParaCord;
    use crate::{DefaultKey, Key};

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
        let rodeo: ParaCord<RandomState> = ParaCord::with_hasher(RandomState::new());

        let key = rodeo.get_or_intern("Test");
        assert_eq!("Test", rodeo.resolve(key));
    }

    #[test]
    fn get_or_intern() {
        let rodeo = ParaCord::default();

        let a = rodeo.get_or_intern("A");
        assert_eq!(a, rodeo.get_or_intern("A"));

        let b = rodeo.get_or_intern("B");
        assert_eq!(b, rodeo.get_or_intern("B"));

        let c = rodeo.get_or_intern("C");
        assert_eq!(c, rodeo.get_or_intern("C"));
    }

    #[test]
    #[cfg(not(miri))]
    fn get_or_intern_threaded() {
        let rodeo = Arc::new(ParaCord::default());

        let moved = Arc::clone(&rodeo);
        thread::spawn(move || {
            let a = moved.get_or_intern("A");
            assert_eq!(a, moved.get_or_intern("A"));

            let b = moved.get_or_intern("B");
            assert_eq!(b, moved.get_or_intern("B"));

            let c = moved.get_or_intern("C");
            assert_eq!(c, moved.get_or_intern("C"));
        });

        let a = rodeo.get_or_intern("A");
        assert_eq!(a, rodeo.get_or_intern("A"));

        let b = rodeo.get_or_intern("B");
        assert_eq!(b, rodeo.get_or_intern("B"));

        let c = rodeo.get_or_intern("C");
        assert_eq!(c, rodeo.get_or_intern("C"));
    }

    #[test]
    fn get() {
        let rodeo = ParaCord::default();
        let key = rodeo.get_or_intern("A");

        assert_eq!(Some(key), rodeo.get("A"));
    }

    #[test]
    #[cfg(not(miri))]
    fn get_threaded() {
        let rodeo = Arc::new(ParaCord::default());
        let key = rodeo.get_or_intern("A");

        let moved = Arc::clone(&rodeo);
        thread::spawn(move || {
            assert_eq!(Some(key), moved.get("A"));
        });

        assert_eq!(Some(key), rodeo.get("A"));
    }

    #[test]
    fn resolve() {
        let rodeo = ParaCord::default();
        let key = rodeo.get_or_intern("A");

        assert_eq!("A", rodeo.resolve(key));
    }

    #[test]
    #[should_panic]
    #[cfg(not(miri))]
    fn resolve_panics() {
        let rodeo = ParaCord::default();
        rodeo.resolve(Key::try_from_repr(100).unwrap());
    }

    #[test]
    #[cfg(not(miri))]
    fn resolve_threaded() {
        let rodeo = Arc::new(ParaCord::default());
        let key = rodeo.get_or_intern("A");

        let moved = Arc::clone(&rodeo);
        thread::spawn(move || {
            assert_eq!("A", moved.resolve(key));
        });

        assert_eq!("A", rodeo.resolve(key));
    }

    #[test]
    #[cfg(not(any(miri)))]
    fn resolve_panics_threaded() {
        let rodeo = Arc::new(ParaCord::default());
        let key = rodeo.get_or_intern("A");

        let moved = Arc::clone(&rodeo);
        let handle = thread::spawn(move || {
            assert_eq!("A", moved.resolve(key));
            moved.resolve(Key::try_from_repr(100).unwrap());
        });

        assert_eq!("A", rodeo.resolve(key));
        assert!(handle.join().is_err());
    }

    #[test]
    fn try_resolve() {
        let rodeo = ParaCord::default();
        let key = rodeo.get_or_intern("A");

        assert_eq!(Some("A"), rodeo.try_resolve(key));
        assert_eq!(None, rodeo.try_resolve(Key::try_from_repr(100).unwrap()));
    }

    #[test]
    #[cfg(not(miri))]
    fn try_resolve_threaded() {
        let rodeo = Arc::new(ParaCord::default());
        let key = rodeo.get_or_intern("A");

        let moved = Arc::clone(&rodeo);
        thread::spawn(move || {
            assert_eq!(Some("A"), moved.try_resolve(key));
            assert_eq!(None, moved.try_resolve(Key::try_from_repr(100).unwrap()));
        });

        assert_eq!(Some("A"), rodeo.try_resolve(key));
        assert_eq!(None, rodeo.try_resolve(Key::try_from_repr(100).unwrap()));
    }

    #[test]
    fn len() {
        let rodeo = ParaCord::default();
        rodeo.get_or_intern("A");
        rodeo.get_or_intern("B");
        rodeo.get_or_intern("C");

        assert_eq!(rodeo.len(), 3);
    }

    #[test]
    fn empty() {
        let rodeo = ParaCord::default();

        assert!(rodeo.is_empty());
    }

    #[test]
    fn drops() {
        let _ = ParaCord::default();
    }

    #[test]
    #[cfg(not(miri))]
    fn drop_threaded() {
        let rodeo = Arc::new(ParaCord::default());

        let moved = Arc::clone(&rodeo);
        thread::spawn(move || {
            let _ = moved;
        });
    }

    #[test]
    fn memory() {
        let mut rodeo = ParaCord::default();
        rodeo.get_or_intern("A");
        rodeo.get_or_intern("B");
        rodeo.get_or_intern("C");

        assert!(rodeo.current_memory_usage() > 0);
    }

    #[test]
    fn reset() {
        let mut rodeo = ParaCord::default();
        let k1 = rodeo.get_or_intern("A");
        rodeo.reset();

        assert!(rodeo.try_resolve(k1).is_none());
        assert!(rodeo.is_empty());
    }

    // #[test]
    // #[cfg(not(miri))]
    // fn debug() {
    //     let rodeo = ParaCord::default();
    //     println!("{:?}", rodeo);
    // }

    #[test]
    fn iter() {
        let rodeo = ParaCord::default();
        rodeo.get_or_intern("A");
        rodeo.get_or_intern("B");
        rodeo.get_or_intern("C");
        let values: Vec<_> = rodeo.iter().map(|(k, v)| (k.into_repr(), v)).collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&(0, "A")));
        assert!(values.contains(&(1, "B")));
        assert!(values.contains(&(2, "C")));
    }

    #[test]
    fn from_iter() {
        let rodeo: ParaCord = ["a", "b", "c", "d", "e"].iter().collect();

        assert!(rodeo.get("a").is_some());
        assert!(rodeo.get("b").is_some());
        assert!(rodeo.get("c").is_some());
        assert!(rodeo.get("d").is_some());
        assert!(rodeo.get("e").is_some());
    }

    #[test]
    fn index() {
        let rodeo = ParaCord::default();
        let key = rodeo.get_or_intern("A");

        assert_eq!("A", &rodeo[key]);
    }

    #[test]
    fn extend() {
        let mut rodeo = ParaCord::default();
        assert!(rodeo.is_empty());

        rodeo.extend(["a", "b", "c", "d", "e"].iter());
        assert!(rodeo.get("a").is_some());
        assert!(rodeo.get("b").is_some());
        assert!(rodeo.get("c").is_some());
        assert!(rodeo.get("d").is_some());
        assert!(rodeo.get("e").is_some());
    }

    // Test for race conditions on key insertion
    // https://github.com/Kixiron/lasso/issues/18
    #[test]
    #[cfg(not(miri))]
    fn get_or_intern_threaded_racy() {
        const THREADS: usize = 10;

        let mut handles = Vec::with_capacity(THREADS);
        let barrier = Arc::new(Barrier::new(THREADS));
        let rodeo = Arc::new(ParaCord::default());
        let expected = Key::try_from_repr(0).unwrap();

        for _ in 0..THREADS {
            let moved_rodeo = Arc::clone(&rodeo);
            let moved_barrier = Arc::clone(&barrier);

            handles.push(thread::spawn(move || {
                moved_barrier.wait();
                assert_eq!(expected, moved_rodeo.get_or_intern("A"));
                assert_eq!(expected, moved_rodeo.get_or_intern("A"));
                assert_eq!(expected, moved_rodeo.get_or_intern("A"));
                assert_eq!(expected, moved_rodeo.get_or_intern("A"));
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde() {
        let key = DefaultKey::get_or_intern("hello");

        serde_test::assert_de_tokens(&key, &[serde_test::Token::Str("hello")]);
        serde_test::assert_ser_tokens(&key, &[serde_test::Token::Str("hello")]);
    }
}

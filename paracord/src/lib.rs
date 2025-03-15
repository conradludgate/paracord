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

#[doc(hidden)]
pub mod __private {
    pub use foldhash::fast::RandomState;
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

            /// Try and get the key associated with the given string.
            /// Allocates a new key if not found.
            ///
            /// Unlike
            #[doc = concat!("[`",stringify!($key),"::get_or_intern`],")]
            /// this function does not need to also allocate the string.
            pub fn get_or_intern_static(s: &'static str) -> Self {
                Self(Self::paracord().get_or_intern_static(s))
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
    };
}

custom_key!(
    /// A key that allocates in a global [`ParaCord`] instance.
    ///
    /// Custom global keys can be defined using [`custom_key`]
    pub struct DefaultKey;
);

/// Key type returned by [`ParaCord`].
///
/// [`Key`] implements [`core::cmp::Ord`] for use within collections like [`BTreeMap`](std::collections::BTreeMap),
/// but the order is not defined to be meaningful or relied upon. Treat [`Key`]s as opaque blobs, with an unstable representation.
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Clone, Copy)]
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

    /// Try and get the [`Key`] associated with the given string.
    /// Allocates a new key if not found.
    ///
    /// Unlike [`ParaCord::get_or_intern`], this function does not need to also allocate the string.
    pub fn get_or_intern_static(&self, s: &'static str) -> Key {
        self.inner.get_or_intern_static(s.as_bytes())
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

    use crate::Key;
    use crate::ParaCord;

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
    fn get_or_intern_static() {
        let rodeo = ParaCord::default();

        let a = rodeo.get_or_intern_static("A");
        assert_eq!(a, rodeo.get_or_intern_static("A"));

        let b = rodeo.get_or_intern_static("B");
        assert_eq!(b, rodeo.get_or_intern_static("B"));

        let c = rodeo.get_or_intern_static("C");
        assert_eq!(c, rodeo.get_or_intern_static("C"));
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

    // #[test]
    // #[cfg(not(miri))]
    // fn debug() {
    //     let rodeo = ParaCord::default();
    //     println!("{:?}", rodeo);
    // }

    #[test]
    fn iter() {
        let rodeo = ParaCord::default();
        rodeo.get_or_intern_static("A");
        rodeo.get_or_intern_static("B");
        rodeo.get_or_intern_static("C");
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

    // #[test]
    // #[cfg(feature = "serialize")]
    // fn empty_serialize() {
    //     let rodeo = ParaCord::default();

    //     let ser = serde_json::to_string(&rodeo).unwrap();
    //     let ser2 = serde_json::to_string(&rodeo).unwrap();
    //     assert_eq!(ser, ser2);

    //     let deser: ParaCord = serde_json::from_str(&ser).unwrap();
    //     assert!(deser.is_empty());
    //     let deser2: ParaCord = serde_json::from_str(&ser2).unwrap();
    //     assert!(deser2.is_empty());
    // }

    // #[test]
    // #[cfg(feature = "serialize")]
    // fn filled_serialize() {
    //     let rodeo = ParaCord::default();
    //     let a = rodeo.get_or_intern("a");
    //     let b = rodeo.get_or_intern("b");
    //     let c = rodeo.get_or_intern("c");
    //     let d = rodeo.get_or_intern("d");

    //     let ser = serde_json::to_string(&rodeo).unwrap();
    //     let ser2 = serde_json::to_string(&rodeo).unwrap();

    //     let deser: ParaCord = serde_json::from_str(&ser).unwrap();
    //     let deser2: ParaCord = serde_json::from_str(&ser2).unwrap();

    //     for (correct_key, correct_str) in [(a, "a"), (b, "b"), (c, "c"), (d, "d")].iter().copied() {
    //         assert_eq!(correct_key, deser.get(correct_str).unwrap());
    //         assert_eq!(correct_key, deser2.get(correct_str).unwrap());

    //         assert_eq!(correct_str, deser.resolve(&correct_key));
    //         assert_eq!(correct_str, deser2.resolve(&correct_key));
    //     }
    // }

    // #[test]
    // fn threaded_rodeo_eq() {
    //     let a = ParaCord::default();
    //     let b = ParaCord::default();
    //     assert_eq!(a, b);

    //     let a = ParaCord::default();
    //     a.get_or_intern("a");
    //     a.get_or_intern("b");
    //     a.get_or_intern("c");
    //     let b = ParaCord::default();
    //     b.get_or_intern("a");
    //     b.get_or_intern("b");
    //     b.get_or_intern("c");
    //     assert_eq!(a, b);
    // }

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

    // Test for race conditions on key insertion
    // https://github.com/Kixiron/lasso/issues/18
    #[test]
    #[cfg(not(miri))]
    fn get_or_intern_static_threaded_racy() {
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
                assert_eq!(expected, moved_rodeo.get_or_intern_static("A"));
                assert_eq!(expected, moved_rodeo.get_or_intern_static("A"));
                assert_eq!(expected, moved_rodeo.get_or_intern_static("A"));
                assert_eq!(expected, moved_rodeo.get_or_intern_static("A"));
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}

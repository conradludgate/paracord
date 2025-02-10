use std::{
    hash::{BuildHasher, Hash},
    num::NonZeroU32,
    ops::Index,
    thread::available_parallelism,
};

use bumpalo::Bump;
use clashmap::{
    tableref::{entry::Entry, entrymut::EntryMut},
    ClashTable,
};
use thread_local::ThreadLocal;
use typesize::TypeSize;

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct Key(NonZeroU32);

impl Key {
    pub fn into_repr(self) -> u32 {
        self.0.get() - 1
    }

    pub fn try_from_repr(x: u32) -> Option<Self> {
        NonZeroU32::new(x.checked_add(1)?).map(Self)
    }
}

pub struct Inner(*const str, u32);
impl TypeSize for Inner {}

pub struct ParaCord<S = foldhash::fast::RandomState> {
    keys_to_strings: boxcar::Vec<*const str>,
    strings_to_keys: ClashTable<Inner>,
    alloc: ThreadLocal<Bump>,
    hasher: S,
}

unsafe impl<S: Sync> Sync for ParaCord<S> {}
unsafe impl<S: Send> Send for ParaCord<S> {}

impl Default for ParaCord {
    fn default() -> Self {
        Self::with_hasher(Default::default())
    }
}

impl<S: BuildHasher> ParaCord<S> {
    pub fn with_hasher(hasher: S) -> Self {
        Self {
            keys_to_strings: boxcar::Vec::default(),
            strings_to_keys: ClashTable::new(),
            alloc: ThreadLocal::with_capacity(available_parallelism().map_or(0, |x| x.get())),
            hasher,
        }
    }

    pub fn get(&self, s: &str) -> Option<Key> {
        let hash = self.hasher.hash_one(s);
        let key = self.strings_to_keys.find(hash, |k| unsafe { s == &*k.0 })?;
        Some(Key(unsafe { NonZeroU32::new_unchecked(key.1 + 1) }))
    }

    pub fn get_or_intern(&self, s: &str) -> Key {
        let hash = self.hasher.hash_one(s);
        let Some(key) = self.strings_to_keys.find(hash, |k| unsafe { s == &*k.0 }) else {
            return self.intern_slow(s, hash);
        };
        Key(unsafe { NonZeroU32::new_unchecked(key.1 + 1) })
    }

    #[cold]
    fn intern_slow(&self, s: &str, hash: u64) -> Key {
        match self.strings_to_keys.entry(
            hash,
            |k| unsafe { s == &*k.0 },
            |k| unsafe { self.hasher.hash_one(&*k.0) },
        ) {
            Entry::Occupied(entry) => Key(unsafe { NonZeroU32::new_unchecked(entry.get().1 + 1) }),
            Entry::Vacant(entry) => {
                let bump = self.alloc.get_or_default();
                let s = bump.alloc_str(s) as &str as *const str;
                let key = self.keys_to_strings.push(s);
                Key(unsafe {
                    NonZeroU32::new_unchecked(entry.insert(Inner(s, key as u32)).value().1 + 1)
                })
            }
        }
    }

    pub fn get_or_intern_static(&self, s: &'static str) -> Key {
        let hash = self.hasher.hash_one(s);
        let Some(key) = self.strings_to_keys.find(hash, |k| unsafe { s == &*k.0 }) else {
            return self.intern_static_slow(s, hash);
        };
        Key(unsafe { NonZeroU32::new_unchecked(key.1 + 1) })
    }

    #[cold]
    fn intern_static_slow(&self, s: &'static str, hash: u64) -> Key {
        match self.strings_to_keys.entry(
            hash,
            |k| unsafe { s == &*k.0 },
            |k| unsafe { self.hasher.hash_one(&*k.0) },
        ) {
            Entry::Occupied(entry) => Key(unsafe { NonZeroU32::new_unchecked(entry.get().1 + 1) }),
            Entry::Vacant(entry) => {
                let s = s as *const str;
                let key = self.keys_to_strings.push(s);
                Key(unsafe {
                    NonZeroU32::new_unchecked(entry.insert(Inner(s, key as u32)).value().1 + 1)
                })
            }
        }
    }

    pub fn try_resolve(&self, key: Key) -> Option<&str> {
        let key = key.0.get() - 1;
        let s = self.keys_to_strings.get(key as usize)?;
        unsafe { Some(&**s) }
    }

    pub fn resolve(&self, key: Key) -> &str {
        let key = key.0.get() - 1;
        unsafe { &*self.keys_to_strings[key as usize] }
    }

    pub fn len(&self) -> usize {
        self.keys_to_strings.count()
    }

    pub fn is_empty(&self) -> bool {
        self.keys_to_strings.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Key, &str)> {
        self.keys_to_strings
            .iter()
            .map(|(key, s)| unsafe { (Key(NonZeroU32::new_unchecked(key as u32 + 1)), &**s) })
    }

    pub fn reset(&mut self) {
        self.keys_to_strings.clear();
        self.strings_to_keys.clear();
        self.alloc.iter_mut().for_each(|b| b.reset());
    }

    pub fn current_memory_usage(&mut self) -> usize {
        use typesize::TypeSize;
        self.keys_to_strings.count() * size_of::<*const str>()
            + self.strings_to_keys.get_size()
            + self
                .alloc
                .iter_mut()
                .map(|b| b.iter_allocated_chunks().map(|c| c.len()).sum::<usize>())
                .sum::<usize>()
    }
}

impl<I: AsRef<str>, S: BuildHasher + Default> FromIterator<I> for ParaCord<S> {
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let len = iter.size_hint().0;

        let mut this = Self {
            keys_to_strings: boxcar::Vec::with_capacity(len),
            strings_to_keys: ClashTable::with_capacity(len),
            alloc: ThreadLocal::with_capacity(available_parallelism().map_or(0, |x| x.get())),
            hasher: S::default(),
        };
        this.extend(iter);
        this
    }
}

impl<I: AsRef<str>, S: BuildHasher> Extend<I> for ParaCord<S> {
    fn extend<T: IntoIterator<Item = I>>(&mut self, iter: T) {
        let bump = self.alloc.get_or_default();

        // assumption, the iterator has mostly unique entries, thus this should always use the slow insert mode.
        for s in iter {
            let s = s.as_ref();
            let hash = self.hasher.hash_one(s);
            match self.strings_to_keys.entry_mut(
                hash,
                |k| unsafe { s == &*k.0 },
                |k| unsafe { self.hasher.hash_one(&*k.0) },
            ) {
                EntryMut::Occupied(_) => continue,
                EntryMut::Vacant(entry) => {
                    let s = bump.alloc_str(s) as &str as *const str;
                    let key = self.keys_to_strings.push(s);
                    entry.insert(Inner(s, key as u32));
                }
            }
        }
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
    use std::hash::RandomState;
    use std::{
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

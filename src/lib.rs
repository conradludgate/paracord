use std::{
    hash::{BuildHasher, Hash},
    num::NonZeroU32,
    thread::available_parallelism,
};

use bumpalo::Bump;
use clashmap::{tableref::entry::Entry, ClashTable};
use thread_local::ThreadLocal;
use typesize::TypeSize;

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
        Self {
            keys_to_strings: boxcar::Vec::default(),
            strings_to_keys: ClashTable::new(),
            alloc: ThreadLocal::with_capacity(available_parallelism().map_or(0, |x| x.get())),
            hasher: Default::default(),
        }
    }
}

pub struct Inner(*const str, u32);
impl TypeSize for Inner {}

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

impl<S: BuildHasher> ParaCord<S> {
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

#[cfg(test)]
mod tests {
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
}

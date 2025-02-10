use std::{
    alloc::Layout,
    hash::{BuildHasher, Hash},
    num::NonZeroU32,
    thread::available_parallelism,
};

use bumpalo::Bump;
use clashmap::{tableref::entry::Entry, ClashTable};
use hashbrown::Equivalent;
use short_string::ShortString;
use thread_local::ThreadLocal;

mod short_string;

pub struct ParaCord<S = foldhash::fast::RandomState> {
    keys_to_strings: boxcar::Vec<ShortString>,
    strings_to_keys: ClashTable<(ShortString, u32)>,
    hasher: S,
    alloc: ThreadLocal<Bump>,
}

impl<S: Default + BuildHasher> Default for ParaCord<S> {
    fn default() -> Self {
        Self {
            keys_to_strings: boxcar::Vec::default(),
            strings_to_keys: ClashTable::new(),
            hasher: S::default(),
            alloc: ThreadLocal::with_capacity(available_parallelism().map_or(0, |x| x.get())),
        }
    }
}

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

impl ParaCord {
    pub fn intern(&self, s: &str) -> Key {
        let hash = self.hasher.hash_one(s);
        if let Some(key) = self
            .strings_to_keys
            .find(hash, |k| unsafe { s.equivalent(k.0.as_str()) })
        {
            return Key(unsafe { NonZeroU32::new_unchecked(key.1 + 1) });
        }

        self.intern_slow(s, hash)
    }

    #[cold]
    fn intern_slow(&self, s: &str, hash: u64) -> Key {
        match self.strings_to_keys.entry(
            hash,
            |k| unsafe { s.equivalent(k.0.as_str()) },
            |k| unsafe { self.hasher.hash_one(k.0.as_str()) },
        ) {
            Entry::Occupied(entry) => Key(unsafe { NonZeroU32::new_unchecked(entry.get().1 + 1) }),
            Entry::Vacant(entry) => {
                let len = ShortString::len_of(s);
                let bump = self.alloc.get_or_default();
                let s = unsafe {
                    let alloc = bump.alloc_layout(Layout::from_size_align_unchecked(len, 1));
                    ShortString::encode_into(s, alloc.as_ptr())
                };

                let key = self.keys_to_strings.push(s.clone());
                Key(unsafe {
                    NonZeroU32::new_unchecked(entry.insert((s, key as u32)).value().1 + 1)
                })
            }
        }
    }

    pub fn try_get(&self, key: Key) -> Option<&str> {
        let key = key.0.get() - 1;
        let s = self.keys_to_strings.get(key as usize)?;
        unsafe { Some(s.as_str()) }
    }

    pub fn get(&self, key: Key) -> &str {
        let key = key.0.get() - 1;
        unsafe { self.keys_to_strings[key as usize].as_str() }
    }

    pub fn reset(&mut self) {
        self.keys_to_strings.clear();
        self.strings_to_keys.clear();
        self.alloc.iter_mut().for_each(|b| b.reset());
    }
}

#[cfg(test)]
mod tests {
    use crate::ParaCord;

    #[test]
    fn works() {
        let paracord = ParaCord::default();

        let foo = paracord.intern("foo");
        let bar = paracord.intern("bar");
        let foo2 = paracord.intern("foo");

        assert_eq!(foo, foo2);
        assert_ne!(foo, bar);
        assert_eq!(paracord.get(foo), "foo");
        assert_eq!(paracord.get(bar), "bar");
    }
}

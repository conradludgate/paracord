use std::{alloc::Layout, hash::Hash, num::NonZeroU32, thread::available_parallelism};

use bumpalo::Bump;
use clashmap::{ClashMap, EntryRef};
use foldhash::fast::RandomState;
use short_string::ShortString;
use thread_local::ThreadLocal;

mod short_string;

pub struct ParaCord {
    keys_to_strings: boxcar::Vec<ShortString>,
    strings_to_keys: ClashMap<ShortString, u32, RandomState>,
    alloc: ThreadLocal<Bump>,
}

impl Default for ParaCord {
    fn default() -> Self {
        Self {
            keys_to_strings: boxcar::Vec::default(),
            strings_to_keys: ClashMap::with_hasher(RandomState::default()),
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
        if let Some(key) = self.strings_to_keys.get(s) {
            return Key(unsafe { NonZeroU32::new_unchecked(*key + 1) });
        }

        self.intern_slow(s)
    }

    #[cold]
    fn intern_slow(&self, s: &str) -> Key {
        match self.strings_to_keys.entry_ref(s) {
            EntryRef::Occupied(entry) => {
                Key(unsafe { NonZeroU32::new_unchecked(*entry.get() + 1) })
            }
            EntryRef::Vacant(entry) => {
                let len = ShortString::len_of(s);
                let bump = self.alloc.get_or_default();
                let s = unsafe {
                    let alloc = bump.alloc_layout(Layout::from_size_align_unchecked(len, 1));
                    ShortString::encode_into(s, alloc.as_ptr())
                };

                let key = self.keys_to_strings.push(s.clone());
                Key(unsafe { NonZeroU32::new_unchecked(*entry.insert(s, key as u32).value() + 1) })
            }
        }
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

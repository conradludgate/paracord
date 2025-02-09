use std::{alloc::Layout, hash::Hash, num::NonZeroU32};

use bumpalo::Bump;
use foldhash::fast::RandomState;
use papaya::Guard;
use short_string::ShortString;
use thread_local::ThreadLocal;

mod short_string;

#[derive(Default)]
pub struct ParaCord {
    keys_to_strings: boxcar::Vec<ShortString>,
    strings_to_keys: papaya::HashMap<ShortString, u32, RandomState>,
    alloc: ThreadLocal<Bump>,
}

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct Key(NonZeroU32);

impl ParaCord {
    pub fn intern(&self, s: &str) -> Key {
        let guard = self.strings_to_keys.guard();
        if let Some(key) = self.strings_to_keys.get(s, &guard) {
            return Key(unsafe { NonZeroU32::new_unchecked(key + 1) });
        }

        self.intern_slow(s, guard)
    }

    #[cold]
    fn intern_slow(&self, s: &str, guard: impl Guard) -> Key {
        let len = ShortString::len_of(s);
        let bump = self.alloc.get_or_default();
        let s = unsafe {
            let alloc = bump.alloc_layout(Layout::from_size_align_unchecked(len, 1));
            ShortString::encode_into(s, alloc.as_ptr())
        };
        let key = self.keys_to_strings.push(s);
        match self.strings_to_keys.try_insert(s, key as u32, &guard) {
            Ok(key) => Key(unsafe { NonZeroU32::new_unchecked(key + 1) }),
            Err(entry) => Key(unsafe { NonZeroU32::new_unchecked(entry.current + 1) }),
        }
    }

    pub fn get(&self, key: Key) -> &str {
        let key = key.0.get() - 1;
        unsafe { self.keys_to_strings[key as usize].as_str() }
    }

    pub fn reset(&mut self) {
        self.keys_to_strings.clear();
        core::mem::take(&mut self.strings_to_keys);
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

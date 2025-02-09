//! Small String hacks
//!
//! Instead of storing `(*u8, usize)` twice for each key in `ParaCord`,
//! we can get away with just `*u8` (similar to a cstr).
//! Dissimilar to a cstr, this is length-prefixed rather than null terminated.
//!
//! Since I only need to store small strings, I have a simple optimisation
//! to use a single byte length prefix, with a nine byte prefix if long (255+ bytes)

use std::hash::Hash;

use hashbrown::Equivalent;

#[derive(Clone)]
pub(crate) struct ShortString(*const u8);

unsafe impl Sync for ShortString {}
unsafe impl Send for ShortString {}

impl ShortString {
    pub(crate) unsafe fn as_str<'a>(&self) -> &'a str {
        let len = *self.0;
        let p = self.0.add(1);
        if len == 255 {
            return long(p);
        }
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(p, len as usize))
    }

    pub(crate) fn len_of(s: &str) -> usize {
        let l = s.len();
        if s.len() < 255 {
            l + 1
        } else {
            l + 1 + size_of::<usize>()
        }
    }

    pub(crate) unsafe fn encode_into(s: &str, p: *mut u8) -> Self {
        if s.len() < 255 {
            p.write(s.len() as u8);
            p.add(1).copy_from_nonoverlapping(s.as_ptr(), s.len());
        } else {
            encode_long(s, p);
        }
        Self(p.cast_const())
    }
}

#[cold]
#[inline(never)]
unsafe fn long<'a>(p: *const u8) -> &'a str {
    let len = usize::from_ne_bytes(*p.cast());
    core::str::from_utf8_unchecked(core::slice::from_raw_parts(p.add(size_of::<usize>()), len))
}

#[cold]
#[inline(never)]
unsafe fn encode_long(s: &str, p: *mut u8) {
    p.write(255);
    p.add(1)
        .copy_from_nonoverlapping(s.len().to_ne_bytes().as_ptr(), 8);
    p.add(1 + size_of::<usize>())
        .copy_from_nonoverlapping(s.as_ptr(), s.len());
}

impl Hash for ShortString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { self.as_str().hash(state) };
    }
}

impl PartialEq for ShortString {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for ShortString {}

impl Equivalent<ShortString> for str {
    fn equivalent(&self, key: &ShortString) -> bool {
        unsafe { self == key.as_str() }
    }
}

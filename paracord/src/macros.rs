/// Create a new custom key, with a static-backed allocator.
///
/// See [`DefaultKey`](crate::DefaultKey) for docs on what this macro generates.
///
/// ## Create a custom key
///
/// ```
/// paracord::custom_key!(
///     /// My custom key
///     pub struct MyKey;
/// );
///
/// let key = MyKey::new("foo");
/// assert_eq!(key.as_str(), "foo");
///
/// let key2 = MyKey::try_new_existing("foo").unwrap();
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
            #[inline]
            fn paracord() -> &'static $crate::ParaCord<$s> {
                static S: ::std::sync::OnceLock<$crate::ParaCord<$s>> = ::std::sync::OnceLock::new();
                S.get_or_init(|| $crate::ParaCord::with_hasher($init))
            }

            /// Try and get the key associated with the given string.
            /// Returns [`None`] if not found.
            #[inline]
            pub fn try_new_existing(s: &str) -> Option<Self> {
                Self::paracord().get(s).map(Self)
            }

            /// Create a new key associated with the given string.
            /// Returns the same key if called repeatedly.
            #[inline]
            pub fn new(s: &str) -> Self {
                Self(Self::paracord().get_or_intern(s))
            }

            /// Resolve the string associated with this key.
            #[inline]
            pub fn as_str(&self) -> &'static str {
                // Safety: The key can only be constructed from the static paracord,
                // and the paracord will never be reset.
                unsafe { Self::paracord().resolve_unchecked(self.0) }
            }

            /// Determine how many keys have been allocated
            #[inline]
            pub fn count() -> usize {
                Self::paracord().len()
            }

            /// Get an iterator over every
            #[doc = concat!("(`",stringify!($key),"`, `&str`)")]
            /// pair that has been allocated.
            #[inline]
            pub fn iter() -> impl Iterator<Item = (Self, &'static str)> {
                Self::paracord().iter().map(|(k, s)| (Self(k), s))
            }
        }

        /// Displays the string that this key represents.
        impl ::core::fmt::Display for $key {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::core::convert::AsRef<str> for $key {
            #[inline]
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl ::core::ops::Deref for $key {
            type Target = str;
            #[inline]
            fn deref(&self) -> &str {
                self.as_str()
            }
        }

        $crate::__private::serde::custom_key_serde!($key);
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn misc() {
        custom_key!(pub struct Foo);

        let _foo = Foo::new("foo");
        let foo = Foo::try_new_existing("foo").unwrap();

        assert_eq!(foo.to_string(), "foo");
        assert_eq!(foo.as_ref(), "foo");
        assert_eq!(Foo::count(), 1);
        let keys: Vec<_> = Foo::iter().collect();
        assert_eq!(keys, [(foo, "foo")]);
    }
}

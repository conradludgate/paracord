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
/// let key = MyKey::from_str_or_intern("foo");
/// assert_eq!(key.as_str(), "foo");
///
/// let key2 = MyKey::try_from_str("foo").unwrap();
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
            pub fn try_from_str(s: &str) -> Option<Self> {
                Self::paracord().get(s).map(Self)
            }

            /// Try and get the key associated with the given string.
            /// Allocates a new key if not found.
            pub fn from_str_or_intern(s: &str) -> Self {
                Self(Self::paracord().get_or_intern(s))
            }

            /// Resolve the string associated with this key.
            pub fn as_str(&self) -> &'static str {
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

        impl ::core::fmt::Display for $key {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::core::convert::AsRef<str> for $key {
            fn as_ref(&self) -> &str {
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

        assert!(Foo::is_empty());

        let _foo = Foo::from_str_or_intern("foo");
        let foo = Foo::try_from_str("foo").unwrap();

        assert_eq!(foo.to_string(), "foo");
        assert_eq!(foo.as_ref(), "foo");
        assert_eq!(Foo::len(), 1);
        let keys: Vec<_> = Foo::iter().collect();
        assert_eq!(keys, [(foo, "foo")]);
    }
}

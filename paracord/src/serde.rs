use std::hash::BuildHasher;

use crate::{Key, ParaCord};
use serde::de::Visitor;

pub struct SerdeVisitor<'a, S>(pub &'a ParaCord<S>);

impl<S: BuildHasher> Visitor<'_> for SerdeVisitor<'_, S> {
    type Value = Key;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string value")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.0.get_or_intern(v))
    }
}

// pub fn size_hint_cautious<Element>(hint: Option<usize>) -> usize {
//     const MAX_PREALLOC_BYTES: usize = 1024 * 1024;

//     if std::mem::size_of::<Element>() == 0 {
//         0
//     } else {
//         std::cmp::min(
//             hint.unwrap_or(0),
//             MAX_PREALLOC_BYTES / std::mem::size_of::<Element>(),
//         )
//     }
// }

// pub struct SerdeSliceVisitor<'a, T: 'static, S>(pub &'a slice::ParaCord<T, S>);

// impl<'de, T: 'static + Deserialize<'de> + Sync + Hash + Eq + Copy, S: BuildHasher> Visitor<'de>
//     for SerdeSliceVisitor<'_, T, S>
// {
//     type Value = Key;

//     fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//         formatter.write_str("a sequence")
//     }

//     fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
//     where
//         A: serde::de::SeqAccess<'de>,
//     {
//         let capacity = size_hint_cautious::<T>(seq.size_hint());
//         let mut values = Vec::<T>::with_capacity(capacity);

//         while let Some(value) = seq.next_element()? {
//             values.push(value);
//         }

//         Ok(self.0.get_or_intern(&values))
//     }
// }

#[doc(hidden)]
#[macro_export]
macro_rules! custom_key_serde {
    ($key:ident) => {
        const _: () = {
            use $crate::__private::serde::{
                Deserialize, Deserializer, SerdeVisitor, Serialize, Serializer,
            };

            impl Serialize for $key {
                fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    serializer.serialize_str(self.as_str())
                }
            }

            impl<'de> Deserialize<'de> for $key {
                fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    deserializer
                        .deserialize_str(SerdeVisitor(Self::paracord()))
                        .map(Self)
                }
            }
        };
    };
}
pub use {
    custom_key_serde,
    serde::{Deserialize, Deserializer, Serialize, Serializer},
};

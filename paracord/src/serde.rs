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
                    serializer.serialize_str(self.resolve())
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

extern crate alloc;

use alloc::borrow::Cow;
use alloc::collections::BTreeMap;
use core::fmt;
use serde::de::{Deserialize, MapAccess, SeqAccess, Visitor};

macro_rules! tri {
    ($e:expr $(,)?) => {
        match $e {
            core::result::Result::Ok(val) => val,
            core::result::Result::Err(err) => return core::result::Result::Err(err),
        }
    };
}

/// Typedef for the inside of an object.
pub type Object<'a> = BTreeMap<String, Value<'a>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConvertedNumber {
    Float(f64),
    Int(i64),
    UInt(u64),
}

/// Reference to JSON data.
#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    /// A `null`
    Null,
    /// A string (i.e. something quoted; quotes are not part of this. Data has not been UTF-8 validated)
    String(Cow<'a, str>),
    /// A number that's actually been converted (used for test code for benching with typed serde json)
    Number(ConvertedNumber),
    /// A bool (i.e. `false` or `true`)
    Bool(bool),
    /// An object (i.e. items inside curly brackets `{}` separated by colon `:` and comma `,`)
    Object(Object<'a>),
    /// An array (i.e. items inside squared brackets `[]` separated by comma `,`)
    Array(Vec<Value<'a>>),
}

impl<'de> Deserialize<'de> for Value<'de> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Value<'de>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value<'de>;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid JSON value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Value<'de>, E> {
                Ok(Value::Bool(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Value<'de>, E> {
                Ok(Value::Number(ConvertedNumber::Int(value)))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Value<'de>, E> {
                Ok(Value::Number(ConvertedNumber::UInt(value)))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Value<'de>, E> {
                Ok(Value::Number(ConvertedNumber::Float(value)))
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E> {
                Ok(Value::String(Cow::Borrowed(v)))
            }

            #[inline]
            fn visit_string<E>(self, value: alloc::string::String) -> Result<Value<'de>, E> {
                Ok(Value::String(Cow::Owned(value)))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Value<'de>, E> {
                Ok(Value::String(Cow::Owned(value.to_owned())))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Value<'de>, E> {
                Ok(Value::Null)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Value<'de>, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                serde::Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Value<'de>, E> {
                Ok(Value::Null)
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Value<'de>, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut vec = vec![];

                while let Some(elem) = tri!(visitor.next_element()) {
                    vec.push(elem);
                }

                Ok(Value::Array(vec))
            }

            #[inline]
            fn visit_map<V>(self, mut visitor: V) -> Result<Value<'de>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut values = BTreeMap::new();

                while let Some((key, value)) = tri!(visitor.next_entry()) {
                    values.insert(key, value);
                }

                Ok(Value::Object(values))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

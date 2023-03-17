use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

/// An ASCII character that can be serialized as a string.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct AsciiGlyph(u8);

impl From<u8> for AsciiGlyph {
    fn from(value: u8) -> Self {
        AsciiGlyph(value)
    }
}

impl From<AsciiGlyph> for u8 {
    fn from(glyph: AsciiGlyph) -> Self {
        glyph.0
    }
}

impl From<AsciiGlyph> for char {
    fn from(glyph: AsciiGlyph) -> Self {
        char::from_u32(glyph.0 as u32).unwrap()
    }
}

impl Display for AsciiGlyph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", <Self as Into<char>>::into(*self))
    }
}

#[cfg(feature = "serde")]
impl Serialize for AsciiGlyph {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(std::str::from_utf8(&[self.0]).unwrap())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for AsciiGlyph {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StrVisitor;

        impl<'de> Visitor<'de> for StrVisitor {
            type Value = AsciiGlyph;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("single character ASCII string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v.as_bytes() {
                    [b] => Ok(AsciiGlyph(*b)),
                    _ => Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(v),
                        &self,
                    )),
                }
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(v)
            }
        }

        deserializer.deserialize_str(StrVisitor {})
    }
}

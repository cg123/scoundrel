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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_u8() {
        let glyph = AsciiGlyph::from(65); // ASCII 'A'
        assert_eq!(glyph.0, 65);
    }

    #[test]
    fn test_to_u8() {
        let glyph = AsciiGlyph(97); // ASCII 'a'
        let value: u8 = glyph.into();
        assert_eq!(value, 97);
    }

    #[test]
    fn test_to_char() {
        let glyph = AsciiGlyph(66); // ASCII 'B'
        let c: char = glyph.into();
        assert_eq!(c, 'B');
    }

    #[test]
    fn test_display() {
        let glyph = AsciiGlyph(67); // ASCII 'C'
        assert_eq!(format!("{}", glyph), "C");
    }

    #[test]
    fn test_equality() {
        let glyph1 = AsciiGlyph(68); // ASCII 'D'
        let glyph2 = AsciiGlyph(68);
        let glyph3 = AsciiGlyph(69); // ASCII 'E'

        assert_eq!(glyph1, glyph2);
        assert_ne!(glyph1, glyph3);
    }

    #[test]
    fn test_debug_output() {
        let glyph = AsciiGlyph(70); // ASCII 'F'
        assert_eq!(format!("{:?}", glyph), "AsciiGlyph(70)");
    }

    #[test]
    fn test_special_characters() {
        // Test with non-printable ASCII characters
        let tab = AsciiGlyph(9); // Tab
        assert_eq!(char::from(tab), '\t');

        let newline = AsciiGlyph(10); // LF
        assert_eq!(char::from(newline), '\n');

        let space = AsciiGlyph(32); // Space
        assert_eq!(char::from(space), ' ');
    }

    #[test]
    fn test_clone() {
        let glyph1 = AsciiGlyph(71); // ASCII 'G'
        let glyph2 = glyph1;
        assert_eq!(glyph1, glyph2);
    }

    // Test serde if the feature is enabled
    #[cfg(feature = "serde")]
    mod serde_tests {
        use super::*;
        use serde_json;

        #[test]
        fn test_serialize() {
            let glyph = AsciiGlyph(72); // ASCII 'H'
            let serialized = serde_json::to_string(&glyph).unwrap();
            assert_eq!(serialized, "\"H\"");
        }

        #[test]
        fn test_deserialize() {
            let json = "\"I\"";
            let glyph: AsciiGlyph = serde_json::from_str(json).unwrap();
            assert_eq!(glyph.0, 73); // ASCII 'I'
        }

        #[test]
        fn test_deserialize_error() {
            // Test with a multi-character string (should fail)
            let json = "\"ABC\"";
            let result = serde_json::from_str::<AsciiGlyph>(json);
            assert!(result.is_err());

            // Test with an empty string (should fail)
            let json = "\"\"";
            let result = serde_json::from_str::<AsciiGlyph>(json);
            assert!(result.is_err());
        }
    }
}

use std::fmt;

use serde::de::{self, Deserializer, Visitor};

/// Deserializes either a number or a string representing a human-readable file size into a `u64`.
///
/// The string format supported is roughly (expressed as a regular expression):
/// `^\s*(?P<number>\d+)\s*(?P<unit>B|kB|MB|GB|TB|kiB|MiB|GiB|TiB)\s*$`
pub fn deserialize_file_size<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct FileSizeVisitor;

    impl<'de> Visitor<'de> for FileSizeVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a positive integer number (as bytes), or a string containing a positive integer number followed by a unit")
        }

        fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_u64(u64::from(value))
        }

        fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_u64(u64::from(value))
        }

        fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_u64(u64::from(value))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(value.as_str())
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            parse_file_size(self, value)
        }

        fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_i64(i64::from(value))
        }

        fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_i64(i64::from(value))
        }

        fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_i64(i64::from(value))
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            u64::try_from(value).map_err(|_| {
                de::Error::invalid_value(de::Unexpected::Signed(i64::from(value)), &self)
            })
        }
    }

    deserializer.deserialize_any(FileSizeVisitor)
}

/// Same as `deserialize_file_size`, but parses into an `Option` instead, allowing the field to be missing.
pub fn deserialize_file_size_opt<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct FileSizeOptVisitor;

    impl<'de> Visitor<'de> for FileSizeOptVisitor {
        type Value = Option<u64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a positive integer number (as bytes), or a string containing a positive integer number followed by a unit")
        }

        fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_u64(u64::from(value))
        }

        fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_u64(u64::from(value))
        }

        fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_u64(u64::from(value))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value))
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(value.as_str())
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            parse_file_size(self, value).map(Some)
        }

        fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_i64(i64::from(value))
        }

        fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_i64(i64::from(value))
        }

        fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_i64(i64::from(value))
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            u64::try_from(value).map(Some).map_err(|_| {
                de::Error::invalid_value(de::Unexpected::Signed(i64::from(value)), &self)
            })
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(self)
        }
    }

    deserializer.deserialize_any(FileSizeOptVisitor)
}

fn parse_file_size<'de, V, E>(visitor: V, value: &str) -> Result<u64, E>
where
    V: Visitor<'de>,
    E: de::Error,
{
    let value = value.trim();
    let position = value.chars().take_while(|it| it.is_ascii_digit()).count();
    if position == 0 {
        return Err(de::Error::invalid_value(
            de::Unexpected::Str(value),
            &visitor,
        ));
    };

    let (number, unit) = value.split_at(position);
    let Ok(number) = number.trim().parse::<u64>() else {
        return Err(de::Error::invalid_value(
            de::Unexpected::Str(number.trim()),
            &"a positive integer number parsable into a `u64`",
        ));
    };

    let factor = match unit.trim() {
        "B" => 1,
        "kB" => 1_000,
        "MB" => 1_000_000,
        "GB" => 1_000_000_000,
        "TB" => 1_000_000_000_000,
        "kiB" => 1_024,
        "MiB" => 1_048_576,         // 1_024 * 1_024
        "GiB" => 1_073_741_824,     // 1_024 * 1_024 * 1_024
        "TiB" => 1_099_511_627_776, // 1_024 * 1_024 * 1_024 * 1_024
        unit => {
            return Err(de::Error::invalid_value(
                de::Unexpected::Str(unit),
                &"a valid file size unit (`B`, `kB`, `MB`, `GB`, `TB`, `kiB`, `MiB`, `GiB` or `TiB`)",
            ));
        }
    };

    let Some(file_size) = number.checked_mul(factor) else {
        return Err(de::Error::invalid_value(
            de::Unexpected::Str(value),
            &"the computed file size is bigger than `u64::MAX`",
        ));
    };

    Ok(file_size)
}

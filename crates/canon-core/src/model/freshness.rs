use std::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Score(f64);

impl Score {
    pub fn one() -> Self {
        Self(1.0)
    }

    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn get(self) -> f64 {
        self.0
    }

    pub fn is_integer(self) -> bool {
        self.0.fract() == 0.0 && self.0.is_finite() && self.0.abs() <= i64::MAX as f64
    }

    pub fn yaml_display(self) -> String {
        if self.is_integer() {
            format!("{}", self.0 as i64)
        } else {
            format!("{}", self.0)
        }
    }
}

impl Eq for Score {}

impl Serialize for Score {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.is_integer() {
            serializer.serialize_i64(self.0 as i64)
        } else {
            serializer.serialize_f64(self.0)
        }
    }
}

impl<'de> Deserialize<'de> for Score {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ScoreVisitor;

        impl Visitor<'_> for ScoreVisitor {
            type Value = Score;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a number")
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Score, E> {
                Ok(Score(v as f64))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Score, E> {
                Ok(Score(v as f64))
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Score, E> {
                Ok(Score(v))
            }
        }

        deserializer.deserialize_any(ScoreVisitor)
    }
}

/// Live implementation paths this atom's body is checked against.
///
/// One path serializes as a string; several as a JSON/YAML list. A lone string
/// still deserializes. Empty means omitted / skip scoring.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImplPaths(Vec<String>);

impl ImplPaths {
    pub fn new(paths: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let paths: Vec<String> = paths
            .into_iter()
            .map(Into::into)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Self(paths)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_slice(&self) -> &[String] {
        &self.0
    }
}

impl From<&str> for ImplPaths {
    fn from(value: &str) -> Self {
        Self::new([value])
    }
}

impl From<String> for ImplPaths {
    fn from(value: String) -> Self {
        Self::new([value])
    }
}

impl Serialize for ImplPaths {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0.as_slice() {
            [one] => serializer.serialize_str(one),
            many => many.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for ImplPaths {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PathsVisitor;

        impl<'de> Visitor<'de> for PathsVisitor {
            type Value = ImplPaths;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a path string or a list of path strings")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<ImplPaths, E> {
                Ok(ImplPaths::from(v))
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<ImplPaths, E> {
                Ok(ImplPaths::from(v))
            }

            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<ImplPaths, A::Error> {
                let mut paths = Vec::new();
                while let Some(item) = seq.next_element::<String>()? {
                    paths.push(item);
                }
                Ok(ImplPaths::new(paths))
            }

            fn visit_none<E: de::Error>(self) -> Result<ImplPaths, E> {
                Ok(ImplPaths::default())
            }

            fn visit_unit<E: de::Error>(self) -> Result<ImplPaths, E> {
                Ok(ImplPaths::default())
            }
        }

        deserializer.deserialize_any(PathsVisitor)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Freshness {
    #[serde(
        rename = "last-verified",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub last_verified: Option<String>,
    #[serde(
        rename = "impl-path",
        default,
        skip_serializing_if = "ImplPaths::is_empty"
    )]
    pub impl_path: ImplPaths,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<Score>,
}

impl Freshness {
    pub fn is_empty(&self) -> bool {
        self.last_verified.is_none() && self.impl_path.is_empty() && self.score.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::{Freshness, ImplPaths, Score};

    #[test]
    fn empty_freshness_serializes_as_empty_object() {
        let json = serde_json::to_string(&Freshness::default()).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn integer_score_serializes_without_decimal() {
        let json = serde_json::to_string(&Score::one()).unwrap();
        assert_eq!(json, "1");
    }

    #[test]
    fn kebab_case_keys_roundtrip() {
        let freshness = Freshness {
            last_verified: Some("2026-09-01 13:05:00".into()),
            impl_path: "gamesvr/DurabilityManager.java".into(),
            score: Some(Score::one()),
        };
        let json = serde_json::to_value(&freshness).unwrap();
        assert_eq!(json["last-verified"], "2026-09-01 13:05:00");
        assert_eq!(json["impl-path"], "gamesvr/DurabilityManager.java");
        assert_eq!(json["score"], 1);
        let back: Freshness = serde_json::from_value(json).unwrap();
        assert_eq!(back, freshness);
    }

    #[test]
    fn impl_path_list_roundtrips_and_string_still_reads() {
        let many = Freshness {
            impl_path: ImplPaths::new(["a.rs", "b.rs"]),
            ..Freshness::default()
        };
        let json = serde_json::to_value(&many).unwrap();
        assert_eq!(json["impl-path"], serde_json::json!(["a.rs", "b.rs"]));
        let back: Freshness = serde_json::from_value(json).unwrap();
        assert_eq!(back.impl_path.as_slice(), ["a.rs", "b.rs"]);

        let from_string: Freshness =
            serde_json::from_value(serde_json::json!({"impl-path": "solo.rs"})).unwrap();
        assert_eq!(from_string.impl_path.as_slice(), ["solo.rs"]);
    }

    #[test]
    fn blank_impl_path_is_empty() {
        assert!(ImplPaths::from("  ").is_empty());
    }
}


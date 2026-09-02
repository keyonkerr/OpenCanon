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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Freshness {
    #[serde(
        rename = "last-verified",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub last_verified: Option<String>,
    #[serde(rename = "impl-path", default, skip_serializing_if = "Option::is_none")]
    pub impl_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<Score>,
}

impl Freshness {
    pub fn is_empty(&self) -> bool {
        self.last_verified.is_none() && self.impl_path.is_none() && self.score.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::{Freshness, Score};

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
            impl_path: Some("gamesvr/DurabilityManager.java".into()),
            score: Some(Score::one()),
        };
        let json = serde_json::to_value(&freshness).unwrap();
        assert_eq!(json["last-verified"], "2026-09-01 13:05:00");
        assert_eq!(json["impl-path"], "gamesvr/DurabilityManager.java");
        assert_eq!(json["score"], 1);
        let back: Freshness = serde_json::from_value(json).unwrap();
        assert_eq!(back, freshness);
    }
}

use serde::{Deserialize, Serialize};

use super::{Freshness, Status};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Atom {
    pub id: String,
    pub status: Status,
    pub title: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub freshness: Freshness,
    pub body: String,
}

pub fn validate_title_body(index: usize, title: &str, body: &str) -> Result<(), crate::Error> {
    if title.is_empty() {
        return Err(crate::Error::validation(
            index,
            Some("title".into()),
            "title must be non-empty",
        ));
    }
    if body.is_empty() {
        return Err(crate::Error::validation(
            index,
            Some("body".into()),
            "body must be non-empty",
        ));
    }
    Ok(())
}

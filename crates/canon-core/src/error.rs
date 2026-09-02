use crate::model::Status;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlugConflict {
    pub index: usize,
    pub slug: String,
    pub status: Option<Status>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("{message}")]
    Validation {
        index: usize,
        field: Option<String>,
        message: String,
    },
    #[error("field `{field}` is immutable")]
    ImmutableField { index: usize, field: String },
    #[error("invalid transition for `{id}` from {from} to {to}")]
    InvalidTransition {
        id: String,
        from: Status,
        to: Status,
    },
    #[error("slug already exists")]
    SlugConflict { conflicts: Vec<SlugConflict> },
}

impl Error {
    pub fn validation(
        index: usize,
        field: impl Into<Option<String>>,
        message: impl Into<String>,
    ) -> Self {
        Self::Validation {
            index,
            field: field.into(),
            message: message.into(),
        }
    }
}

use canon_core::Error as CoreError;
use canon_store::Error as StoreError;
use serde_json::{json, Value};

use crate::error::CliError;

pub struct MappedError {
    pub code: &'static str,
    pub message: String,
    pub details: Option<Value>,
}

pub fn mapped(err: &CliError) -> MappedError {
    match err {
        CliError::InvalidJson { message } => MappedError {
            code: "INVALID_JSON",
            message: message.clone(),
            details: None,
        },
        CliError::Io { message } => MappedError {
            code: "IO_ERROR",
            message: message.clone(),
            details: None,
        },
        CliError::Validation {
            index,
            field,
            message,
        } => validation(*index, field.as_deref(), message),
        CliError::Core(core) => map_core(core),
        CliError::Store(store) => map_store(store),
        CliError::AtomNotFound { id, index } => {
            let mut details = json!({ "id": id });
            if let Some(index) = index {
                details["index"] = json!(index);
            }
            MappedError {
                code: "ATOM_NOT_FOUND",
                message: format!("atom `{id}` not found"),
                details: Some(details),
            }
        }
    }
}

fn map_core(err: &CoreError) -> MappedError {
    match err {
        CoreError::Validation {
            index,
            field,
            message,
        } => validation(*index, field.as_deref(), message),
        CoreError::ImmutableField { index, field } => MappedError {
            code: "IMMUTABLE_FIELD",
            message: format!("field `{field}` is immutable"),
            details: Some(json!({ "index": index, "field": field })),
        },
        CoreError::InvalidTransition { id, from, to } => MappedError {
            code: "INVALID_TRANSITION",
            message: format!("invalid transition for `{id}` from {from} to {to}"),
            details: Some(json!({
                "id": id,
                "from": from.as_str(),
                "to": to.as_str(),
            })),
        },
        CoreError::SlugConflict { conflicts } => {
            let mut slugs = Vec::new();
            for item in conflicts {
                if !slugs.iter().any(|s| s == &item.slug) {
                    slugs.push(item.slug.clone());
                }
            }
            let listed: Vec<Value> = conflicts
                .iter()
                .map(|item| {
                    let mut obj = json!({
                        "index": item.index,
                        "slug": item.slug,
                    });
                    if let Some(status) = item.status {
                        obj["status"] = json!(status.as_str());
                    }
                    obj
                })
                .collect();
            MappedError {
                code: "SLUG_CONFLICT",
                message: "slug already exists".into(),
                details: Some(json!({
                    "slugs": slugs,
                    "conflicts": listed,
                })),
            }
        }
    }
}

fn map_store(err: &StoreError) -> MappedError {
    match err {
        StoreError::NotFound { id } => MappedError {
            code: "ATOM_NOT_FOUND",
            message: format!("atom `{id}` not found"),
            details: Some(json!({ "id": id })),
        },
        StoreError::Parse { id, message } => MappedError {
            code: "INVALID_ATOM_FILE",
            message: format!("failed to parse atom `{id}`: {message}"),
            details: Some(json!({ "id": id })),
        },
        StoreError::IdMismatch { path_id, atom_id } => MappedError {
            code: "INVALID_ATOM_FILE",
            message: format!("atom id mismatch: path `{path_id}` vs frontmatter `{atom_id}`"),
            details: Some(json!({ "path_id": path_id, "atom_id": atom_id })),
        },
        StoreError::Io(e) => MappedError {
            code: "IO_ERROR",
            message: e.to_string(),
            details: None,
        },
    }
}

fn validation(index: usize, field: Option<&str>, message: &str) -> MappedError {
    let details = match field {
        Some(field) => json!({ "index": index, "field": field }),
        None => json!({ "index": index }),
    };
    MappedError {
        code: "VALIDATION_FAILED",
        message: message.to_string(),
        details: Some(details),
    }
}

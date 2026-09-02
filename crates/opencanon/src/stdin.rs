use std::io::{self, Read};

use canon_core::ops::{AddDraft, EditPatch, FreshnessPatch};
use canon_core::{Freshness, Status};
use serde_json::{Map, Value};

use crate::error::CliError;

pub fn read_stdin() -> Result<String, CliError> {
    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| CliError::Io {
            message: e.to_string(),
        })?;
    Ok(buf)
}

pub fn parse_add_drafts(raw: &str) -> Result<Vec<AddDraft>, CliError> {
    let items = parse_object_array(raw)?;
    let mut drafts = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let obj = object_at(item, index)?;
        let slug = required_string(obj, index, "slug")?;
        let title = required_string(obj, index, "title")?;
        let body = required_string(obj, index, "body")?;
        let tags = optional_tags(obj, index)?;
        let freshness = optional_freshness(obj, index)?;
        drafts.push(AddDraft {
            slug,
            title,
            body,
            tags,
            freshness,
        });
    }
    Ok(drafts)
}

pub fn parse_edit_patches(raw: &str) -> Result<Vec<EditPatch>, CliError> {
    let items = parse_object_array(raw)?;
    let mut patches = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let obj = object_at(item, index)?;
        let id = required_string(obj, index, "id")?;
        let title = optional_present_string(obj, index, "title")?;
        let tags = if obj.contains_key("tags") {
            Some(required_tags(obj, index)?)
        } else {
            None
        };
        let body = optional_present_string(obj, index, "body")?;
        let freshness = if obj.contains_key("freshness") {
            Some(required_freshness_patch(obj, index)?)
        } else {
            None
        };
        let status = optional_status(obj, index)?;
        patches.push(EditPatch {
            id,
            title,
            tags,
            body,
            freshness,
            status,
        });
    }
    Ok(patches)
}

fn parse_object_array(raw: &str) -> Result<Vec<Value>, CliError> {
    let trimmed = raw.trim_start_matches('\u{feff}').trim();
    if trimmed.is_empty() {
        return Err(CliError::invalid_json("stdin is empty"));
    }
    let value: Value =
        serde_json::from_str(trimmed).map_err(|e| CliError::invalid_json(e.to_string()))?;
    match value {
        Value::Array(items) => Ok(items),
        _ => Err(CliError::invalid_json("stdin must be a JSON array")),
    }
}

fn object_at(item: &Value, index: usize) -> Result<&Map<String, Value>, CliError> {
    item.as_object()
        .ok_or_else(|| CliError::validation(index, None::<String>, "element must be a JSON object"))
}

fn required_string(
    obj: &Map<String, Value>,
    index: usize,
    field: &str,
) -> Result<String, CliError> {
    match obj.get(field) {
        None => Err(CliError::validation(
            index,
            Some(field.to_string()),
            format!("missing {field}"),
        )),
        Some(Value::String(s)) => Ok(s.clone()),
        Some(_) => Err(CliError::validation(
            index,
            Some(field.to_string()),
            format!("{field} must be a string"),
        )),
    }
}

fn optional_present_string(
    obj: &Map<String, Value>,
    index: usize,
    field: &str,
) -> Result<Option<String>, CliError> {
    if !obj.contains_key(field) {
        return Ok(None);
    }
    Ok(Some(required_string(obj, index, field)?))
}

fn optional_tags(obj: &Map<String, Value>, index: usize) -> Result<Vec<String>, CliError> {
    if !obj.contains_key("tags") {
        return Ok(Vec::new());
    }
    required_tags(obj, index)
}

fn required_tags(obj: &Map<String, Value>, index: usize) -> Result<Vec<String>, CliError> {
    match obj.get("tags") {
        Some(Value::Array(items)) => {
            let mut tags = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    Value::String(s) => tags.push(s.clone()),
                    _ => {
                        return Err(CliError::validation(
                            index,
                            Some("tags".into()),
                            "tags must be an array of strings",
                        ));
                    }
                }
            }
            Ok(tags)
        }
        Some(_) => Err(CliError::validation(
            index,
            Some("tags".into()),
            "tags must be an array",
        )),
        None => Ok(Vec::new()),
    }
}

fn optional_freshness(obj: &Map<String, Value>, index: usize) -> Result<Freshness, CliError> {
    if !obj.contains_key("freshness") {
        return Ok(Freshness::default());
    }
    parse_freshness(obj.get("freshness").unwrap(), index)
}

fn parse_freshness(value: &Value, index: usize) -> Result<Freshness, CliError> {
    serde_json::from_value(value.clone()).map_err(|e| {
        CliError::validation(
            index,
            Some("freshness".into()),
            format!("invalid freshness: {e}"),
        )
    })
}

fn required_freshness_patch(
    obj: &Map<String, Value>,
    index: usize,
) -> Result<FreshnessPatch, CliError> {
    match obj.get("freshness") {
        Some(Value::Object(_)) => serde_json::from_value(obj.get("freshness").unwrap().clone())
            .map_err(|e| {
                CliError::validation(
                    index,
                    Some("freshness".into()),
                    format!("invalid freshness: {e}"),
                )
            }),
        Some(_) => Err(CliError::validation(
            index,
            Some("freshness".into()),
            "freshness must be an object",
        )),
        None => Ok(FreshnessPatch::default()),
    }
}

fn optional_status(obj: &Map<String, Value>, index: usize) -> Result<Option<Status>, CliError> {
    if !obj.contains_key("status") {
        return Ok(None);
    }
    match obj.get("status") {
        Some(Value::String(s)) => s
            .parse::<Status>()
            .map(Some)
            .map_err(|message| CliError::validation(index, Some("status".into()), message)),
        Some(_) => Err(CliError::validation(
            index,
            Some("status".into()),
            "status must be a string",
        )),
        None => Ok(None),
    }
}

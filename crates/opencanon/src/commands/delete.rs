use canon_store::Store;
use serde_json::{json, Value};

use crate::error::CliError;

pub fn run(store: &Store, id: &str) -> Result<Value, CliError> {
    store
        .delete(id)
        .map_err(|e| CliError::from_store(e, None))?;
    Ok(json!({ "deleted": id }))
}

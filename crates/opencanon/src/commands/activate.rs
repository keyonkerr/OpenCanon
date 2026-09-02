use canon_core::ops;
use canon_core::Timestamp;
use canon_store::Store;
use serde_json::{json, Value};

use crate::error::CliError;

pub fn run(store: &Store, now: Timestamp, id: &str) -> Result<Value, CliError> {
    let atom = store.read(id).map_err(|e| CliError::from_store(e, None))?;
    let atom = ops::activate(atom, now)?;
    store
        .write(&atom)
        .map_err(|e| CliError::from_store(e, None))?;
    Ok(json!({ "atom": atom }))
}

use canon_core::compute;
use canon_core::ops::ListFilter;
use canon_store::Store;
use serde_json::{json, Value};

use crate::error::CliError;

pub fn run(store: &Store, keywords: &[String], filter: ListFilter) -> Result<Value, CliError> {
    let atoms = store.list().map_err(|e| CliError::from_store(e, None))?;
    let atoms = compute::query(atoms, keywords, filter)?;
    let count = atoms.len();
    Ok(json!({
        "atoms": atoms,
        "count": count,
    }))
}

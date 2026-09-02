use canon_core::ops::{self, ListFilter};
use canon_store::Store;
use serde_json::{json, Value};

use crate::error::CliError;

pub fn run(store: &Store, filter: ListFilter) -> Result<Value, CliError> {
    let atoms = store.list().map_err(|e| CliError::from_store(e, None))?;
    let atoms = ops::filter_atoms(atoms, filter);
    let count = atoms.len();
    Ok(json!({
        "atoms": atoms,
        "count": count,
    }))
}

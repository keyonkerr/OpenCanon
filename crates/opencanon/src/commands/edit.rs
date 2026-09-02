use std::collections::HashSet;

use canon_core::ops;
use canon_store::Store;
use serde_json::{json, Value};

use crate::error::CliError;
use crate::stdin;

pub fn run(store: &Store) -> Result<Value, CliError> {
    let raw = stdin::read_stdin()?;
    let patches = stdin::parse_edit_patches(&raw)?;

    let mut loaded = Vec::new();
    let mut seen = HashSet::new();
    for (index, patch) in patches.iter().enumerate() {
        if !seen.insert(patch.id.clone()) {
            continue;
        }
        let atom = store
            .read(&patch.id)
            .map_err(|e| CliError::from_store(e, Some(index)))?;
        loaded.push(atom);
    }

    let atoms = ops::apply_edits(&loaded, &patches)?;
    for atom in &atoms {
        store
            .write(atom)
            .map_err(|e| CliError::from_store(e, None))?;
    }
    Ok(json!({
        "atoms": atoms,
        "count": atoms.len(),
    }))
}

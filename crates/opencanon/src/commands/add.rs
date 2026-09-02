use std::collections::HashMap;

use canon_core::ops;
use canon_store::Store;
use serde_json::{json, Value};

use crate::error::CliError;
use crate::stdin;

pub fn run(store: &Store) -> Result<Value, CliError> {
    let raw = stdin::read_stdin()?;
    let drafts = stdin::parse_add_drafts(&raw)?;
    let occupied: HashMap<_, _> = store
        .list()
        .map_err(|e| CliError::from_store(e, None))?
        .into_iter()
        .map(|atom| (atom.id, atom.status))
        .collect();
    let atoms = ops::add_drafts(&drafts, &occupied)?;
    for atom in &atoms {
        store
            .write(atom)
            .map_err(|e| CliError::from_store(e, None))?;
    }
    let summaries: Vec<Value> = atoms
        .iter()
        .map(|atom| json!({ "id": atom.id, "title": atom.title }))
        .collect();
    Ok(json!({
        "atoms": summaries,
        "count": atoms.len(),
    }))
}

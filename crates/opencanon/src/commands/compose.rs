use std::collections::HashMap;

use canon_core::ops;
use canon_store::Store;
use serde_json::{json, Value};

use crate::error::CliError;
use crate::stdin;

pub fn run(store: &Store) -> Result<Value, CliError> {
    let raw = stdin::read_stdin()?;
    let draft = stdin::parse_compose_draft(&raw)?;
    let mut known = HashMap::new();
    for (index, id) in draft.atoms.iter().enumerate() {
        let atom = store
            .read(id)
            .map_err(|e| CliError::from_store(e, Some(index)))?;
        known.insert(atom.id.clone(), atom.status);
    }
    let doc = ops::compose(&draft, &known)?;
    store
        .write_doc(&doc)
        .map_err(|e| CliError::from_store(e, None))?;
    Ok(json!({
        "id": doc.id,
        "title": doc.title,
        "path": format!("opencanon/docs/{}.md", doc.id),
    }))
}

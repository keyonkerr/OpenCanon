use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path};
use std::process::{Command, Stdio};

use canon_core::compute::freshness::{self, Factor, FactorKind, ImplSnapshot};
use canon_core::ops::{self, ListFilter};
use canon_core::{Atom, Status, Timestamp};
use canon_store::Store;
use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};
use serde_json::{json, Value};

use crate::error::CliError;

pub fn run(store: &Store, ids: &[String]) -> Result<Value, CliError> {
    let atoms = load_atoms(store, ids)?;
    let mut rows = Vec::with_capacity(atoms.len());
    let mut updated = 0usize;

    for atom in atoms {
        if !freshness::has_impl_path(&atom) {
            rows.push(json!({
                "id": atom.id,
                "skipped": true,
            }));
            continue;
        }
        let path = atom.freshness.impl_path.as_deref().unwrap();
        let snapshot = probe(store.root(), path);
        let evaluation = freshness::evaluate(&atom, &snapshot).expect("impl-path present");
        let next = ops::apply_score(atom.clone(), evaluation.score);
        if !ops::score_unchanged(&atom, &next) {
            store
                .write(&next)
                .map_err(|e| CliError::from_store(e, None))?;
            updated += 1;
        }
        rows.push(json!({
            "id": atom.id,
            "skipped": false,
            "score": evaluation.score,
            "factors": factors_json(&evaluation.factors),
        }));
    }

    Ok(json!({
        "atoms": rows,
        "count": rows.len(),
        "updated-count": updated,
    }))
}

fn load_atoms(store: &Store, ids: &[String]) -> Result<Vec<Atom>, CliError> {
    if ids.is_empty() {
        let atoms = store.list().map_err(|e| CliError::from_store(e, None))?;
        return Ok(ops::filter_atoms(atoms, ListFilter::Active));
    }
    let mut seen = HashSet::new();
    let mut ordered = Vec::new();
    for id in ids {
        if seen.insert(id.clone()) {
            ordered.push(id.clone());
        }
    }
    let mut atoms = Vec::with_capacity(ordered.len());
    for id in ordered {
        let atom = store.read(&id).map_err(|e| CliError::from_store(e, None))?;
        if atom.status != Status::Active {
            return Err(CliError::validation(
                0,
                Some("status".to_string()),
                format!("atom `{id}` is not active"),
            ));
        }
        atoms.push(atom);
    }
    Ok(atoms)
}

fn factors_json(factors: &[Factor]) -> Value {
    let items: Vec<Value> = factors
        .iter()
        .map(|f| {
            let mut obj = json!({
                "id": f.id,
                "kind": kind_str(f.kind),
                "value": json_num(f.value),
            });
            if let Some(weight) = f.weight {
                obj["weight"] = json_num(weight);
            }
            obj
        })
        .collect();
    Value::Array(items)
}

fn kind_str(kind: FactorKind) -> &'static str {
    match kind {
        FactorKind::Gate => "gate",
        FactorKind::Weighted => "weighted",
        FactorKind::Multiplier => "multiplier",
        FactorKind::Observe => "observe",
    }
}

fn json_num(value: f64) -> Value {
    if value == 0.0 {
        json!(0)
    } else if value == 1.0 {
        json!(1)
    } else {
        json!(value)
    }
}

fn probe(root: &Path, impl_path: &str) -> ImplSnapshot {
    let rel = impl_path.trim();
    if !is_safe_relative(rel) {
        return ImplSnapshot::default();
    }
    let dest = root.join(rel);
    if !dest.is_file() {
        return ImplSnapshot::default();
    }
    let text = fs::read_to_string(&dest).ok();
    let changed_at = git_changed_at(root, rel).or_else(|| file_mtime(&dest));
    ImplSnapshot {
        exists: true,
        changed_at,
        text,
    }
}

fn is_safe_relative(raw: &str) -> bool {
    if raw.is_empty() {
        return false;
    }
    let lower = raw.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("file:") {
        return false;
    }
    let path = Path::new(raw);
    if path.is_absolute() {
        return false;
    }
    let mut has_normal = false;
    for component in path.components() {
        match component {
            Component::Normal(_) => has_normal = true,
            Component::CurDir => {}
            _ => return false,
        }
    }
    has_normal
}

fn git_changed_at(root: &Path, rel: &str) -> Option<Timestamp> {
    if !root.join(".git").exists() {
        return None;
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "-1", "--format=%ct", "--"])
        .arg(rel)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let secs: i64 = text.trim().parse().ok()?;
    unix_to_timestamp(secs)
}

fn file_mtime(path: &Path) -> Option<Timestamp> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    let dt: DateTime<Local> = modified.into();
    Some(Timestamp::from_ymd_hms(
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
    ))
}

fn unix_to_timestamp(secs: i64) -> Option<Timestamp> {
    let dt = Local.timestamp_opt(secs, 0).single()?;
    Some(Timestamp::from_ymd_hms(
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
    ))
}

#[cfg(test)]
mod tests {
    use super::is_safe_relative;

    #[test]
    fn safe_relative_accepts_project_paths() {
        assert!(is_safe_relative("gamesvr/DurabilityManager.java"));
        assert!(is_safe_relative("./gamesvr/Foo.java"));
    }

    #[test]
    fn safe_relative_rejects_escape_and_urls() {
        assert!(!is_safe_relative(""));
        assert!(!is_safe_relative("../secret.java"));
        assert!(!is_safe_relative("gamesvr/../../etc/passwd"));
        assert!(!is_safe_relative("/etc/passwd"));
        assert!(!is_safe_relative("https://example.com/Foo.java"));
        assert!(!is_safe_relative("http://example.com/Foo.java"));
        assert!(!is_safe_relative("file:Foo.java"));
        #[cfg(windows)]
        assert!(!is_safe_relative(r"C:\Windows\Foo.java"));
    }
}

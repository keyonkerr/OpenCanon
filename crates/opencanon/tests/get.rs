mod common;

use common::{add_sample, assert_err, assert_ok, run, SAMPLE_BODY, SAMPLE_ID, SAMPLE_TITLE};
use tempfile::tempdir;

#[test]
fn get_returns_full_atom_from_spec() {
    let dir = tempdir().unwrap();
    add_sample(dir.path());
    let output = run(dir.path(), &["get", SAMPLE_ID]);
    let data = assert_ok(&output, "get");
    let atom = &data["atom"];
    assert_eq!(atom["id"], SAMPLE_ID);
    assert_eq!(atom["status"], "draft");
    assert_eq!(atom["title"], SAMPLE_TITLE);
    assert_eq!(atom["tags"], serde_json::json!(["armybreak", "durability"]));
    assert_eq!(
        atom["freshness"],
        serde_json::json!({ "impl-path": "gamesvr/DurabilityManager.java" })
    );
    assert_eq!(atom["body"], SAMPLE_BODY);
}

#[test]
fn get_missing_is_atom_not_found() {
    let dir = tempdir().unwrap();
    let output = run(dir.path(), &["get", SAMPLE_ID]);
    let err = assert_err(&output, "get", "ATOM_NOT_FOUND");
    assert_eq!(err["details"]["id"], SAMPLE_ID);
    assert!(!dir.path().join("opencanon").exists());
}

#[test]
fn get_after_active_shows_active_status() {
    let dir = tempdir().unwrap();
    add_sample(dir.path());
    let output = run(dir.path(), &["active", SAMPLE_ID]);
    assert_ok(&output, "active");
    let output = run(dir.path(), &["get", SAMPLE_ID]);
    let data = assert_ok(&output, "get");
    assert_eq!(data["atom"]["status"], "active");
    assert_eq!(data["atom"]["id"], SAMPLE_ID);
}

#[test]
fn get_without_id_is_usage_error() {
    let dir = tempdir().unwrap();
    let output = run(dir.path(), &["get"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn get_corrupt_file_is_invalid_atom_file() {
    let dir = tempdir().unwrap();
    let atoms = common::atoms_dir(dir.path());
    std::fs::create_dir_all(&atoms).unwrap();
    std::fs::write(atoms.join("broken_atom.md"), "not an atom").unwrap();
    let output = run(dir.path(), &["get", "broken_atom"]);
    assert_err(&output, "get", "INVALID_ATOM_FILE");
}

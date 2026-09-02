mod common;

use canon_core::Status;
use canon_store::Store;
use common::{add_sample, assert_ok, atoms_dir, cmd, run, run_stdin, run_stdin_now, SAMPLE_ID};
use tempfile::tempdir;

const SECOND_ID: &str = "durability_cap_from_table";

fn seed_active_and_draft(dir: &std::path::Path) {
    let earlier = run_stdin_now(
        dir,
        "2026-09-01 12:30:00",
        &["add"],
        r#"[{"slug":"durability_cap_from_table","title":"装备耐久从实现表读取上限","tags":["armybreak"],"body":"耐久上限以配表为准。"}]"#,
    );
    assert_ok(&earlier, "add");
    let activated = cmd(dir)
        .env("OPENCANON_NOW", "2026-09-01 12:30:00")
        .args(["active", SECOND_ID])
        .output()
        .unwrap();
    assert_ok(&activated, "active");
    add_sample(dir);
}

#[test]
fn list_default_is_active_only() {
    let dir = tempdir().unwrap();
    seed_active_and_draft(dir.path());
    let output = run(dir.path(), &["list"]);
    let data = assert_ok(&output, "list");
    assert_eq!(data["count"], 1);
    assert_eq!(data["atoms"][0]["id"], SECOND_ID);
    assert_eq!(data["atoms"][0]["status"], "active");
    assert_eq!(data["atoms"][0]["title"], "装备耐久从实现表读取上限");
    assert_eq!(data["atoms"][0]["tags"], serde_json::json!(["armybreak"]));
    assert_eq!(data["atoms"][0]["body"], "耐久上限以配表为准。");
}

#[test]
fn list_status_draft_shows_pending() {
    let dir = tempdir().unwrap();
    seed_active_and_draft(dir.path());
    let output = run(dir.path(), &["list", "--status", "draft"]);
    let data = assert_ok(&output, "list");
    assert_eq!(data["count"], 1);
    assert_eq!(data["atoms"][0]["id"], SAMPLE_ID);
    assert_eq!(data["atoms"][0]["status"], "draft");
}

#[test]
fn list_all_includes_both() {
    let dir = tempdir().unwrap();
    seed_active_and_draft(dir.path());
    let output = run(dir.path(), &["list", "--all"]);
    let data = assert_ok(&output, "list");
    assert_eq!(data["count"], 2);
    let ids: Vec<&str> = data["atoms"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&SAMPLE_ID));
    assert!(ids.contains(&SECOND_ID));
}

#[test]
fn list_empty_dir_does_not_create_namespace() {
    let dir = tempdir().unwrap();
    let output = run(dir.path(), &["list"]);
    let data = assert_ok(&output, "list");
    assert_eq!(data["count"], 0);
    assert_eq!(data["atoms"], serde_json::json!([]));
    assert!(!dir.path().join("opencanon").exists());
}

#[test]
fn list_status_and_all_conflict_is_usage_error() {
    let dir = tempdir().unwrap();
    let output = run(dir.path(), &["list", "--status", "draft", "--all"]);
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("\"ok\""), "{stdout}");
}

#[test]
fn list_status_deprecated() {
    let dir = tempdir().unwrap();
    add_sample(dir.path());
    let store = Store::open(dir.path());
    let mut atom = store.read(SAMPLE_ID).unwrap();
    atom.status = Status::Deprecated;
    store.write(&atom).unwrap();

    let output = run(dir.path(), &["list", "--status", "deprecated"]);
    let data = assert_ok(&output, "list");
    assert_eq!(data["count"], 1);
    assert_eq!(data["atoms"][0]["status"], "deprecated");

    let output = run(dir.path(), &["list"]);
    let data = assert_ok(&output, "list");
    assert_eq!(data["count"], 0);
}

#[test]
fn list_invalid_status_is_usage_error() {
    let dir = tempdir().unwrap();
    let output = run(dir.path(), &["list", "--status", "pending"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn list_does_not_create_dir_when_atoms_missing() {
    let dir = tempdir().unwrap();
    let _ = run(dir.path(), &["list", "--all"]);
    assert!(!atoms_dir(dir.path()).exists());
}

#[test]
fn list_after_empty_add_json_still_empty() {
    let dir = tempdir().unwrap();
    let output = run_stdin(dir.path(), &["add"], "[]");
    assert_ok(&output, "add");
    let output = run(dir.path(), &["list", "--all"]);
    let data = assert_ok(&output, "list");
    assert_eq!(data["count"], 0);
}

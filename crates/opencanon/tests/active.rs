mod common;

use common::{add_sample, assert_err, assert_ok, atoms_dir, cmd, run, SAMPLE_BODY, SAMPLE_ID};
use tempfile::tempdir;

#[test]
fn active_promotes_draft_and_stamps_freshness() {
    let dir = tempdir().unwrap();
    add_sample(dir.path());
    let output = run(dir.path(), &["active", SAMPLE_ID]);
    let data = assert_ok(&output, "active");
    let atom = &data["atom"];
    assert_eq!(atom["id"], SAMPLE_ID);
    assert_eq!(atom["status"], "active");
    assert_eq!(atom["title"], "禁军突围装备耐久恢复机制");
    assert_eq!(atom["tags"], serde_json::json!(["armybreak", "durability"]));
    assert_eq!(atom["body"], SAMPLE_BODY);
    assert_eq!(atom["freshness"]["last-verified"], "2026-09-01 13:05:00");
    assert_eq!(
        atom["freshness"]["impl-path"],
        "gamesvr/DurabilityManager.java"
    );
    assert_eq!(atom["freshness"]["score"], 1);

    let md =
        std::fs::read_to_string(atoms_dir(dir.path()).join(format!("{SAMPLE_ID}.md"))).unwrap();
    assert!(md.contains("status: active"));
    assert!(md.contains("last-verified: 2026-09-01 13:05:00"));
    assert!(md.contains("score: 1\n"));
    assert!(md.contains("impl-path: gamesvr/DurabilityManager.java"));
    assert!(md.contains("正文：只描述一个事实。"));

    let listed = run(dir.path(), &["list"]);
    let data = assert_ok(&listed, "list");
    assert_eq!(data["count"], 1);
    assert_eq!(data["atoms"][0]["id"], SAMPLE_ID);
}

#[test]
fn active_twice_is_invalid_transition() {
    let dir = tempdir().unwrap();
    add_sample(dir.path());
    assert_ok(&run(dir.path(), &["active", SAMPLE_ID]), "active");
    let output = run(dir.path(), &["active", SAMPLE_ID]);
    let err = assert_err(&output, "active", "INVALID_TRANSITION");
    assert_eq!(err["details"]["id"], SAMPLE_ID);
    assert_eq!(err["details"]["from"], "active");
    assert_eq!(err["details"]["to"], "active");
}

#[test]
fn active_missing_is_atom_not_found() {
    let dir = tempdir().unwrap();
    let output = run(dir.path(), &["active", SAMPLE_ID]);
    assert_err(&output, "active", "ATOM_NOT_FOUND");
}

#[test]
fn active_without_id_is_usage_error() {
    let dir = tempdir().unwrap();
    let output = run(dir.path(), &["active"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn active_invalid_clock_env_is_io_error() {
    let dir = tempdir().unwrap();
    add_sample(dir.path());
    let output = cmd(dir.path())
        .env("OPENCANON_NOW", "bogus")
        .args(["active", SAMPLE_ID])
        .output()
        .unwrap();
    assert_err(&output, "active", "IO_ERROR");
}

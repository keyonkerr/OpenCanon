mod common;

use canon_core::Status;
use canon_store::Store;
use common::{
    add_sample, assert_err, assert_ok, atoms_dir, run_stdin, SAMPLE_ADD_JSON, SAMPLE_ID, SAMPLE_MD,
    SAMPLE_TITLE,
};
use tempfile::tempdir;

#[test]
fn add_spec_example_creates_draft_file_and_summary() {
    let dir = tempdir().unwrap();
    let data = add_sample(dir.path());
    assert_eq!(data["count"], 1);
    assert_eq!(data["atoms"][0]["id"], SAMPLE_ID);
    assert_eq!(data["atoms"][0]["title"], SAMPLE_TITLE);
    assert!(data["atoms"][0].get("slug").is_none());
    assert!(data["atoms"][0].get("status").is_none());
    assert!(data["atoms"][0].get("body").is_none());

    let path = atoms_dir(dir.path()).join(format!("{SAMPLE_ID}.md"));
    let md = std::fs::read_to_string(path).unwrap();
    assert_eq!(md, SAMPLE_MD);
}

#[test]
fn add_batch_ids_are_slugs() {
    let dir = tempdir().unwrap();
    let stdin = r#"[
      {"slug":"durability_daily_restore","title":"one","body":"a"},
      {"slug":"durability_cap_from_table","title":"two","body":"b"},
      {"slug":"third_fact","title":"three","body":"c"}
    ]"#;
    let output = run_stdin(dir.path(), &["add"], stdin);
    let data = assert_ok(&output, "add");
    assert_eq!(data["count"], 3);
    assert_eq!(data["atoms"][0]["id"], "durability_daily_restore");
    assert_eq!(data["atoms"][1]["id"], "durability_cap_from_table");
    assert_eq!(data["atoms"][2]["id"], "third_fact");
    assert_eq!(data["atoms"][0]["title"], "one");
}

#[test]
fn add_occupied_slug_is_slug_conflict() {
    let dir = tempdir().unwrap();
    add_sample(dir.path());
    let stdin = r#"[{"slug":"durability_daily_restore","title":"again","body":"x"}]"#;
    let output = run_stdin(dir.path(), &["add"], stdin);
    let err = assert_err(&output, "add", "SLUG_CONFLICT");
    assert_eq!(err["details"]["slugs"], serde_json::json!([SAMPLE_ID]));
    assert_eq!(err["details"]["conflicts"][0]["index"], 0);
    assert_eq!(err["details"]["conflicts"][0]["slug"], SAMPLE_ID);
    assert_eq!(err["details"]["conflicts"][0]["status"], "draft");
    assert_eq!(std::fs::read_dir(atoms_dir(dir.path())).unwrap().count(), 1);
}

#[test]
fn add_in_batch_duplicate_slug_is_slug_conflict() {
    let dir = tempdir().unwrap();
    let stdin = r#"[
      {"slug":"same_slug","title":"one","body":"a"},
      {"slug":"same_slug","title":"two","body":"b"}
    ]"#;
    let output = run_stdin(dir.path(), &["add"], stdin);
    let err = assert_err(&output, "add", "SLUG_CONFLICT");
    assert_eq!(err["details"]["slugs"], serde_json::json!(["same_slug"]));
    assert_eq!(err["details"]["conflicts"][0]["index"], 1);
    assert!(err["details"]["conflicts"][0].get("status").is_none());
    assert!(!dir.path().join("opencanon").exists());
}

#[test]
fn add_deprecated_slug_is_slug_conflict() {
    let dir = tempdir().unwrap();
    add_sample(dir.path());
    let store = Store::open(dir.path());
    let mut atom = store.read(SAMPLE_ID).unwrap();
    atom.status = Status::Deprecated;
    store.write(&atom).unwrap();

    let stdin = r#"[{"slug":"durability_daily_restore","title":"again","body":"x"}]"#;
    let output = run_stdin(dir.path(), &["add"], stdin);
    let err = assert_err(&output, "add", "SLUG_CONFLICT");
    assert_eq!(err["details"]["conflicts"][0]["status"], "deprecated");
}

#[test]
fn add_ignores_input_id_and_status() {
    let dir = tempdir().unwrap();
    let stdin = r#"[{
      "id": "should-ignore",
      "status": "active",
      "slug": "durability_daily_restore",
      "title": "禁军突围装备耐久恢复机制",
      "body": "正文：只描述一个事实。"
    }]"#;
    let output = run_stdin(dir.path(), &["add"], stdin);
    let data = assert_ok(&output, "add");
    assert_eq!(data["atoms"][0]["id"], SAMPLE_ID);
    let md =
        std::fs::read_to_string(atoms_dir(dir.path()).join(format!("{SAMPLE_ID}.md"))).unwrap();
    assert!(md.contains("status: draft"));
    assert!(!md.contains("status: active"));
}

#[test]
fn add_empty_array_does_not_create_namespace() {
    let dir = tempdir().unwrap();
    let output = run_stdin(dir.path(), &["add"], "[]");
    let data = assert_ok(&output, "add");
    assert_eq!(data["count"], 0);
    assert_eq!(data["atoms"], serde_json::json!([]));
    assert!(!dir.path().join("opencanon").exists());
}

#[test]
fn add_invalid_slug_writes_nothing() {
    let dir = tempdir().unwrap();
    let stdin = r#"[{"slug":"bad/slug","title":"t","body":"b"}]"#;
    let output = run_stdin(dir.path(), &["add"], stdin);
    let err = assert_err(&output, "add", "VALIDATION_FAILED");
    assert_eq!(err["details"]["index"], 0);
    assert_eq!(err["details"]["field"], "slug");
    assert!(!dir.path().join("opencanon").exists());
}

#[test]
fn add_partial_invalid_is_atomic() {
    let dir = tempdir().unwrap();
    let stdin = r#"[
      {"slug":"ok_slug","title":"t","body":"b"},
      {"slug":"also_ok","title":"","body":"b"}
    ]"#;
    let output = run_stdin(dir.path(), &["add"], stdin);
    let err = assert_err(&output, "add", "VALIDATION_FAILED");
    assert_eq!(err["details"]["index"], 1);
    assert_eq!(err["details"]["field"], "title");
    assert!(!dir.path().join("opencanon").exists());
}

#[test]
fn add_rejects_non_array_and_malformed_json() {
    let dir = tempdir().unwrap();
    let output = run_stdin(
        dir.path(),
        &["add"],
        r#"{"slug":"x","title":"t","body":"b"}"#,
    );
    assert_err(&output, "add", "INVALID_JSON");
    assert!(!dir.path().join("opencanon").exists());

    let output = run_stdin(dir.path(), &["add"], "[");
    assert_err(&output, "add", "INVALID_JSON");
}

#[test]
fn add_uses_spec_json_shape() {
    let dir = tempdir().unwrap();
    let output = run_stdin(dir.path(), &["add"], SAMPLE_ADD_JSON);
    let json = common::stdout_json(&output);
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], "add");
    assert!(json["data"]["atoms"].is_array());
    assert!(json["data"]["count"].is_number());
}

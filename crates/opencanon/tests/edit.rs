mod common;

use common::{
    add_sample, assert_err, assert_ok, atoms_dir, run, run_stdin, SAMPLE_BODY, SAMPLE_ID,
};
use tempfile::tempdir;

#[test]
fn edit_title_keeps_other_fields_and_file_id() {
    let dir = tempdir().unwrap();
    add_sample(dir.path());
    let stdin = format!(r#"[{{"id":"{SAMPLE_ID}","title":"禁军突围：耐久按日恢复"}}]"#);
    let output = run_stdin(dir.path(), &["edit"], &stdin);
    let data = assert_ok(&output, "edit");
    assert_eq!(data["count"], 1);
    let atom = &data["atoms"][0];
    assert_eq!(atom["id"], SAMPLE_ID);
    assert_eq!(atom["status"], "draft");
    assert_eq!(atom["title"], "禁军突围：耐久按日恢复");
    assert_eq!(atom["tags"], serde_json::json!(["armybreak", "durability"]));
    assert_eq!(
        atom["freshness"],
        serde_json::json!({ "impl-path": "gamesvr/DurabilityManager.java" })
    );
    assert_eq!(atom["body"], SAMPLE_BODY);

    let md =
        std::fs::read_to_string(atoms_dir(dir.path()).join(format!("{SAMPLE_ID}.md"))).unwrap();
    assert!(md.contains("title: 禁军突围：耐久按日恢复"));
    assert!(md.contains("status: draft"));
    assert!(md.contains("正文：只描述一个事实。"));
    assert!(!md.contains("禁军突围装备耐久恢复机制"));
}

#[test]
fn edit_body_only_replaces_markdown_body() {
    let dir = tempdir().unwrap();
    add_sample(dir.path());
    let stdin = format!(r#"[{{"id":"{SAMPLE_ID}","body":"新的正文。"}}]"#);
    let output = run_stdin(dir.path(), &["edit"], &stdin);
    let data = assert_ok(&output, "edit");
    assert_eq!(data["atoms"][0]["body"], "新的正文。");
    assert_eq!(data["atoms"][0]["title"], "禁军突围装备耐久恢复机制");
    let md =
        std::fs::read_to_string(atoms_dir(dir.path()).join(format!("{SAMPLE_ID}.md"))).unwrap();
    let body = md.split("---\n").nth(2).unwrap();
    assert_eq!(body, "新的正文。\n");
}

#[test]
fn edit_tags_replace_including_empty() {
    let dir = tempdir().unwrap();
    add_sample(dir.path());
    let stdin = format!(r#"[{{"id":"{SAMPLE_ID}","tags":[]}}]"#);
    let output = run_stdin(dir.path(), &["edit"], &stdin);
    let data = assert_ok(&output, "edit");
    assert_eq!(data["atoms"][0]["tags"], serde_json::json!([]));
}

#[test]
fn edit_freshness_merges_and_empty_object_keeps_subkeys() {
    let dir = tempdir().unwrap();
    add_sample(dir.path());
    let stdin = format!(
        r#"[{{"id":"{SAMPLE_ID}","freshness":{{"last-verified":"2026-09-01 12:00:00"}}}}]"#
    );
    let output = run_stdin(dir.path(), &["edit"], &stdin);
    let data = assert_ok(&output, "edit");
    assert_eq!(
        data["atoms"][0]["freshness"]["last-verified"],
        "2026-09-01 12:00:00"
    );
    assert_eq!(
        data["atoms"][0]["freshness"]["impl-path"],
        "gamesvr/DurabilityManager.java"
    );

    let stdin = format!(r#"[{{"id":"{SAMPLE_ID}","freshness":{{}}}}]"#);
    let output = run_stdin(dir.path(), &["edit"], &stdin);
    let data = assert_ok(&output, "edit");
    assert_eq!(
        data["atoms"][0]["freshness"]["last-verified"],
        "2026-09-01 12:00:00"
    );
    assert_eq!(
        data["atoms"][0]["freshness"]["impl-path"],
        "gamesvr/DurabilityManager.java"
    );
}

#[test]
fn edit_same_status_ignored_different_status_fails_without_write() {
    let dir = tempdir().unwrap();
    add_sample(dir.path());
    let stdin = format!(r#"[{{"id":"{SAMPLE_ID}","status":"draft","title":"kept-if-ok"}}]"#);
    let output = run_stdin(dir.path(), &["edit"], &stdin);
    assert_ok(&output, "edit");

    let stdin = format!(r#"[{{"id":"{SAMPLE_ID}","status":"active","title":"should-not-apply"}}]"#);
    let output = run_stdin(dir.path(), &["edit"], &stdin);
    let err = assert_err(&output, "edit", "IMMUTABLE_FIELD");
    assert_eq!(err["details"]["field"], "status");
    let output = run(dir.path(), &["get", SAMPLE_ID]);
    let data = assert_ok(&output, "get");
    assert_eq!(data["atom"]["status"], "draft");
    assert_eq!(data["atom"]["title"], "kept-if-ok");
}

#[test]
fn edit_batch_is_atomic_when_second_missing() {
    let dir = tempdir().unwrap();
    add_sample(dir.path());
    let stdin = format!(
        r#"[{{"id":"{SAMPLE_ID}","title":"changed"}},{{"id":"missing_slug","title":"nope"}}]"#
    );
    let output = run_stdin(dir.path(), &["edit"], &stdin);
    let err = assert_err(&output, "edit", "ATOM_NOT_FOUND");
    assert_eq!(err["details"]["index"], 1);
    let output = run(dir.path(), &["get", SAMPLE_ID]);
    let data = assert_ok(&output, "get");
    assert_eq!(data["atom"]["title"], "禁军突围装备耐久恢复机制");
}

#[test]
fn edit_requires_a_mutable_field() {
    let dir = tempdir().unwrap();
    add_sample(dir.path());
    let stdin = format!(r#"[{{"id":"{SAMPLE_ID}"}}]"#);
    let output = run_stdin(dir.path(), &["edit"], &stdin);
    assert_err(&output, "edit", "VALIDATION_FAILED");
}

#[test]
fn edit_invalid_json() {
    let dir = tempdir().unwrap();
    let output = run_stdin(dir.path(), &["edit"], "not-json");
    assert_err(&output, "edit", "INVALID_JSON");
}

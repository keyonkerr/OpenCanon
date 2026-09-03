mod common;

use canon_core::Status;
use canon_store::Store;
use common::{
    add_sample, assert_err, assert_ok, atoms_dir, cmd, run, run_stdin_now, SAMPLE_BODY, SAMPLE_ID,
    SAMPLE_TITLE,
};
use tempfile::tempdir;

const SECOND_ID: &str = "durability_cap_from_table";
const SECOND_BODY: &str = "耐久上限以配表为准。";

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
fn default_is_active_only_and_returns_body() {
    let dir = tempdir().unwrap();
    seed_active_and_draft(dir.path());
    let output = run(dir.path(), &["query", "上限"]);
    let data = assert_ok(&output, "query");
    assert_eq!(data["count"], 1);
    assert_eq!(data["atoms"][0]["id"], SECOND_ID);
    assert_eq!(data["atoms"][0]["status"], "active");
    assert_eq!(data["atoms"][0]["title"], "装备耐久从实现表读取上限");
    assert_eq!(data["atoms"][0]["tags"], serde_json::json!(["armybreak"]));
    assert_eq!(data["atoms"][0]["body"], SECOND_BODY);
    assert!(data["atoms"][0].get("freshness").is_some());
}

#[test]
fn all_returns_draft_hits() {
    let dir = tempdir().unwrap();
    seed_active_and_draft(dir.path());
    let output = run(dir.path(), &["query", "--all", "事实"]);
    let data = assert_ok(&output, "query");
    assert_eq!(data["count"], 1);
    assert_eq!(data["atoms"][0]["id"], SAMPLE_ID);
    assert_eq!(data["atoms"][0]["status"], "draft");
    assert_eq!(data["atoms"][0]["title"], SAMPLE_TITLE);
    assert_eq!(data["atoms"][0]["body"], SAMPLE_BODY);
}

#[test]
fn status_draft_returns_only_draft() {
    let dir = tempdir().unwrap();
    seed_active_and_draft(dir.path());
    let output = run(dir.path(), &["query", "--status", "draft", "事实"]);
    let data = assert_ok(&output, "query");
    assert_eq!(data["count"], 1);
    assert_eq!(data["atoms"][0]["id"], SAMPLE_ID);
}

#[test]
fn default_excludes_draft_even_when_body_matches() {
    let dir = tempdir().unwrap();
    seed_active_and_draft(dir.path());
    let output = run(dir.path(), &["query", "事实"]);
    let data = assert_ok(&output, "query");
    assert_eq!(data["count"], 0);
    assert_eq!(data["atoms"], serde_json::json!([]));
}

#[test]
fn title_substring_matches() {
    let dir = tempdir().unwrap();
    seed_active_and_draft(dir.path());
    let output = run(dir.path(), &["query", "实现表"]);
    let data = assert_ok(&output, "query");
    assert_eq!(data["count"], 1);
    assert_eq!(data["atoms"][0]["id"], SECOND_ID);
}

#[test]
fn tag_substring_matches() {
    let dir = tempdir().unwrap();
    seed_active_and_draft(dir.path());
    let output = run(dir.path(), &["query", "armybreak"]);
    let data = assert_ok(&output, "query");
    assert_eq!(data["count"], 1);
    assert_eq!(data["atoms"][0]["id"], SECOND_ID);
}

#[test]
fn id_substring_matches_when_body_omits_keyword() {
    let dir = tempdir().unwrap();
    seed_active_and_draft(dir.path());
    let output = run(dir.path(), &["query", "--all", "daily"]);
    let data = assert_ok(&output, "query");
    assert_eq!(data["count"], 1);
    assert_eq!(data["atoms"][0]["id"], SAMPLE_ID);
}

#[test]
fn or_keywords_and_all_includes_deprecated() {
    let dir = tempdir().unwrap();
    seed_active_and_draft(dir.path());
    let store = Store::open(dir.path());
    let mut atom = store.read(SECOND_ID).unwrap();
    atom.status = Status::Deprecated;
    store.write(&atom).unwrap();

    let output = run(dir.path(), &["query", "上限", "事实", "--all"]);
    let data = assert_ok(&output, "query");
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
fn exact_id_matches_when_body_omits_keyword() {
    let dir = tempdir().unwrap();
    seed_active_and_draft(dir.path());
    let output = run(dir.path(), &["query", "--all", SAMPLE_ID]);
    let data = assert_ok(&output, "query");
    assert_eq!(data["count"], 1);
    assert_eq!(data["atoms"][0]["id"], SAMPLE_ID);
}

#[test]
fn status_and_all_conflict_is_usage_error() {
    let dir = tempdir().unwrap();
    let output = run(dir.path(), &["query", "--status", "draft", "--all", "词"]);
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("\"ok\""), "{stdout}");
}

#[test]
fn no_match_is_empty_success() {
    let dir = tempdir().unwrap();
    seed_active_and_draft(dir.path());
    let output = run(dir.path(), &["query", "不存在的词"]);
    let data = assert_ok(&output, "query");
    assert_eq!(data["count"], 0);
    assert_eq!(data["atoms"], serde_json::json!([]));
}

#[test]
fn missing_keyword_is_usage_error() {
    let dir = tempdir().unwrap();
    let output = run(dir.path(), &["query"]);
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("\"ok\""), "{stdout}");
}

#[test]
fn empty_keyword_is_validation_failed() {
    let dir = tempdir().unwrap();
    let output = run(dir.path(), &["query", ""]);
    let err = assert_err(&output, "query", "VALIDATION_FAILED");
    assert_eq!(err["details"]["field"], "keyword");
}

#[test]
fn empty_dir_does_not_create_namespace() {
    let dir = tempdir().unwrap();
    let output = run(dir.path(), &["query", "任何"]);
    let data = assert_ok(&output, "query");
    assert_eq!(data["count"], 0);
    assert_eq!(data["atoms"], serde_json::json!([]));
    assert!(!dir.path().join("opencanon").exists());
    assert!(!atoms_dir(dir.path()).exists());
}

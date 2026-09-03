mod common;

use common::{
    add_sample, assert_err, assert_ok, atoms_dir, docs_dir, run, run_stdin, SAMPLE_BODY, SAMPLE_ID,
    SAMPLE_TITLE,
};
use tempfile::tempdir;

fn compose_body(atom_id: &str) -> String {
    format!("# {SAMPLE_TITLE}\n\n{SAMPLE_BODY} [{atom_id}](../atoms/{atom_id}.md)")
}

fn compose_json(slug: &str, title: &str, atom_id: &str, body: &str) -> String {
    serde_json::json!({
        "slug": slug,
        "title": title,
        "atoms": [atom_id],
        "body": body,
    })
    .to_string()
}

fn add_and_active(dir: &std::path::Path) {
    add_sample(dir);
    assert_ok(&run(dir, &["active", SAMPLE_ID]), "active");
}

#[test]
fn compose_writes_docs_not_atoms_and_returns_path() {
    let dir = tempdir().unwrap();
    add_and_active(dir.path());
    let body = compose_body(SAMPLE_ID);
    let stdin = compose_json("how_ssot_works", SAMPLE_TITLE, SAMPLE_ID, &body);
    let output = run_stdin(dir.path(), &["compose"], &stdin);
    let data = assert_ok(&output, "compose");
    assert_eq!(data["id"], "how_ssot_works");
    assert_eq!(data["title"], SAMPLE_TITLE);
    assert_eq!(data["path"], "opencanon/docs/how_ssot_works.md");
    assert!(data.get("body").is_none());
    assert!(data.get("atoms").is_none());

    let path = docs_dir(dir.path()).join("how_ssot_works.md");
    let md = std::fs::read_to_string(&path).unwrap();
    assert!(md.starts_with("---\nid: how_ssot_works\n"));
    assert!(md.contains("title:"));
    assert!(md.contains("atoms:\n  - durability_daily_restore\n"));
    assert!(md.contains(&format!("[{SAMPLE_ID}](../atoms/{SAMPLE_ID}.md)")));
    assert!(atoms_dir(dir.path())
        .join(format!("{SAMPLE_ID}.md"))
        .is_file());
}

#[test]
fn compose_overwrites_same_slug() {
    let dir = tempdir().unwrap();
    add_and_active(dir.path());
    let body = compose_body(SAMPLE_ID);
    let first = compose_json("how_ssot_works", SAMPLE_TITLE, SAMPLE_ID, &body);
    assert_ok(&run_stdin(dir.path(), &["compose"], &first), "compose");
    let second = compose_json("how_ssot_works", "new title", SAMPLE_ID, &body);
    let data = assert_ok(&run_stdin(dir.path(), &["compose"], &second), "compose");
    assert_eq!(data["title"], "new title");
    let md = std::fs::read_to_string(docs_dir(dir.path()).join("how_ssot_works.md")).unwrap();
    assert!(md.contains("title: new title\n"));
    assert_eq!(std::fs::read_dir(docs_dir(dir.path())).unwrap().count(), 1);
}

#[test]
fn compose_missing_atom_is_atom_not_found() {
    let dir = tempdir().unwrap();
    let body = compose_body("missing_id");
    let stdin = compose_json("how_ssot_works", SAMPLE_TITLE, "missing_id", &body);
    let output = run_stdin(dir.path(), &["compose"], &stdin);
    let err = assert_err(&output, "compose", "ATOM_NOT_FOUND");
    assert_eq!(err["details"]["id"], "missing_id");
    assert_eq!(err["details"]["index"], 0);
    assert!(!dir.path().join("opencanon").join("docs").exists());
}

#[test]
fn compose_draft_atom_is_validation_failed() {
    let dir = tempdir().unwrap();
    add_sample(dir.path());
    let body = compose_body(SAMPLE_ID);
    let stdin = compose_json("how_ssot_works", SAMPLE_TITLE, SAMPLE_ID, &body);
    let output = run_stdin(dir.path(), &["compose"], &stdin);
    let err = assert_err(&output, "compose", "VALIDATION_FAILED");
    assert_eq!(err["details"]["field"], "atoms");
    assert!(!docs_dir(dir.path()).exists());
}

#[test]
fn compose_paragraph_without_citation_writes_nothing() {
    let dir = tempdir().unwrap();
    add_and_active(dir.path());
    let stdin = compose_json(
        "how_ssot_works",
        SAMPLE_TITLE,
        SAMPLE_ID,
        "# t\n\nno citation here\n",
    );
    let output = run_stdin(dir.path(), &["compose"], &stdin);
    let err = assert_err(&output, "compose", "VALIDATION_FAILED");
    assert_eq!(err["details"]["field"], "body");
    assert!(!docs_dir(dir.path()).exists());
}

#[test]
fn compose_rejects_array_and_empty_stdin() {
    let dir = tempdir().unwrap();
    let output = run_stdin(dir.path(), &["compose"], "[]");
    assert_err(&output, "compose", "INVALID_JSON");
    let output = run_stdin(dir.path(), &["compose"], "");
    assert_err(&output, "compose", "INVALID_JSON");
    assert!(!dir.path().join("opencanon").exists());
}

#[test]
fn compose_missing_atoms_field_is_validation_failed() {
    let dir = tempdir().unwrap();
    let stdin = r#"{"slug":"how_ssot_works","title":"t","body":"b"}"#;
    let output = run_stdin(dir.path(), &["compose"], stdin);
    let err = assert_err(&output, "compose", "VALIDATION_FAILED");
    assert_eq!(err["details"]["field"], "atoms");
    assert!(!dir.path().join("opencanon").exists());
}

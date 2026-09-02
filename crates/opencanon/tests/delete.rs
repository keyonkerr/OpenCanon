mod common;

use common::{add_sample, assert_err, assert_ok, atoms_dir, run, run_stdin_now, SAMPLE_ID};
use tempfile::tempdir;

const OTHER_ID: &str = "durability_cap_from_table";

#[test]
fn delete_removes_only_the_named_atom() {
    let dir = tempdir().unwrap();
    add_sample(dir.path());
    let other = run_stdin_now(
        dir.path(),
        "2026-09-01 12:30:00",
        &["add"],
        r#"[{"slug":"durability_cap_from_table","title":"装备耐久从实现表读取上限","body":"耐久上限以配表为准。"}]"#,
    );
    assert_ok(&other, "add");

    let output = run(dir.path(), &["delete", SAMPLE_ID]);
    let data = assert_ok(&output, "delete");
    assert_eq!(data["deleted"], SAMPLE_ID);
    assert!(!atoms_dir(dir.path())
        .join(format!("{SAMPLE_ID}.md"))
        .exists());
    assert!(atoms_dir(dir.path())
        .join(format!("{OTHER_ID}.md"))
        .exists());
}

#[test]
fn delete_missing_is_atom_not_found() {
    let dir = tempdir().unwrap();
    let output = run(dir.path(), &["delete", SAMPLE_ID]);
    let err = assert_err(&output, "delete", "ATOM_NOT_FOUND");
    assert_eq!(err["details"]["id"], SAMPLE_ID);
    assert!(!dir.path().join("opencanon").exists());
}

#[test]
fn delete_without_id_is_usage_error() {
    let dir = tempdir().unwrap();
    let output = run(dir.path(), &["delete"]);
    assert_eq!(output.status.code(), Some(2));
}

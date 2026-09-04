mod common;

use canon_store::Store;
use common::{add_sample, assert_err, assert_ok, atoms_dir, cmd, run, run_stdin, SAMPLE_ID};
use serde_json::{json, Value};
use tempfile::tempdir;

const SKIP_ID: &str = "durability_cap_from_table";

fn write_impl(dir: &std::path::Path, contents: &str) {
    let folder = dir.join("gamesvr");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("DurabilityManager.java"), contents).unwrap();
}

fn add_and_active(dir: &std::path::Path) {
    add_sample(dir);
    assert_ok(&run(dir, &["active", SAMPLE_ID]), "active");
}

fn add_skip_atom(dir: &std::path::Path) {
    let added = run_stdin(
        dir,
        &["add"],
        r#"[{"slug":"durability_cap_from_table","title":"装备耐久从实现表读取上限","tags":["armybreak"],"body":"耐久上限以配表为准。"}]"#,
    );
    assert_ok(&added, "add");
    assert_ok(&run(dir, &["active", SKIP_ID]), "active");
}

fn factor<'a>(row: &'a Value, id: &str) -> &'a Value {
    row["factors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["id"] == id)
        .unwrap_or_else(|| panic!("missing factor {id} in {row}"))
}

fn disk_score(dir: &std::path::Path, id: &str) -> Option<f64> {
    Store::open(dir)
        .read(id)
        .unwrap()
        .freshness
        .score
        .map(|s| s.get())
}

#[test]
fn omit_ids_is_all_active_in_list_order() {
    let dir = tempdir().unwrap();
    add_and_active(dir.path());
    add_skip_atom(dir.path());
    write_impl(dir.path(), "class DurabilityManager {}");

    let output = run(dir.path(), &["freshness"]);
    let data = assert_ok(&output, "freshness");
    assert_eq!(data["count"], 2);
    assert_eq!(data["updated-count"], 1);
    assert_eq!(data["atoms"][0]["id"], SKIP_ID);
    assert_eq!(data["atoms"][0]["skipped"], true);
    assert!(data["atoms"][0].get("score").is_none());
    assert!(data["atoms"][0].get("factors").is_none());
    assert_eq!(data["atoms"][1]["id"], SAMPLE_ID);
    assert_eq!(data["atoms"][1]["skipped"], false);
    assert_eq!(data["atoms"][1]["score"], 0.6);
    assert_eq!(factor(&data["atoms"][1], "impl-exists")["kind"], "gate");
    assert_eq!(factor(&data["atoms"][1], "impl-exists")["value"], 1);
    assert_eq!(factor(&data["atoms"][1], "body-in-impl")["value"], 1);
    assert_eq!(
        factor(&data["atoms"][1], "impl-current")["kind"],
        "weighted"
    );
    assert_eq!(factor(&data["atoms"][1], "impl-current")["value"], 0);
    assert_eq!(factor(&data["atoms"][1], "impl-current")["weight"], 0.4);
    assert_eq!(disk_score(dir.path(), SAMPLE_ID), Some(0.6));
    assert_eq!(disk_score(dir.path(), SKIP_ID), Some(1.0));
}

#[test]
fn chinese_body_and_future_verified_is_one() {
    let dir = tempdir().unwrap();
    add_and_active(dir.path());
    write_impl(dir.path(), "class DurabilityManager {}");
    let first = assert_ok(&run(dir.path(), &["freshness", SAMPLE_ID]), "freshness");
    assert_eq!(first["atoms"][0]["score"], 0.6);

    let edited = run_stdin(
        dir.path(),
        &["edit"],
        &format!(
            r#"[{{"id":"{SAMPLE_ID}","freshness":{{"last-verified":"2099-01-01 00:00:00"}}}}]"#
        ),
    );
    assert_ok(&edited, "edit");

    let output = run(dir.path(), &["freshness", SAMPLE_ID]);
    let data = assert_ok(&output, "freshness");
    assert_eq!(data["count"], 1);
    assert_eq!(data["updated-count"], 1);
    assert_eq!(data["atoms"][0]["score"], 1);
    assert_eq!(factor(&data["atoms"][0], "impl-current")["value"], 1);
    assert_eq!(disk_score(dir.path(), SAMPLE_ID), Some(1.0));
}

#[test]
fn missing_impl_file_is_zero() {
    let dir = tempdir().unwrap();
    add_and_active(dir.path());
    let output = run(dir.path(), &["freshness", SAMPLE_ID]);
    let data = assert_ok(&output, "freshness");
    assert_eq!(data["atoms"][0]["skipped"], false);
    assert_eq!(data["atoms"][0]["score"], 0);
    assert_eq!(factor(&data["atoms"][0], "impl-exists")["value"], 0);
    assert_eq!(factor(&data["atoms"][0], "body-in-impl")["value"], 0);
    assert_eq!(disk_score(dir.path(), SAMPLE_ID), Some(0.0));
}

#[test]
fn missing_code_name_is_zero() {
    let dir = tempdir().unwrap();
    add_and_active(dir.path());
    write_impl(dir.path(), "class DurabilityManager {}");
    let edited = run_stdin(
        dir.path(),
        &["edit"],
        &format!(r#"[{{"id":"{SAMPLE_ID}","body":"see restoreDurability in the manager."}}]"#),
    );
    assert_ok(&edited, "edit");

    let output = run(dir.path(), &["freshness", SAMPLE_ID]);
    let data = assert_ok(&output, "freshness");
    assert_eq!(data["atoms"][0]["score"], 0);
    assert_eq!(factor(&data["atoms"][0], "impl-exists")["value"], 1);
    assert_eq!(factor(&data["atoms"][0], "body-in-impl")["value"], 0);
}

#[test]
fn second_run_does_not_rewrite() {
    let dir = tempdir().unwrap();
    add_and_active(dir.path());
    write_impl(dir.path(), "class DurabilityManager {}");
    let first = assert_ok(&run(dir.path(), &["freshness", SAMPLE_ID]), "freshness");
    assert_eq!(first["updated-count"], 1);
    let second = assert_ok(&run(dir.path(), &["freshness", SAMPLE_ID]), "freshness");
    assert_eq!(second["updated-count"], 0);
    assert_eq!(second["atoms"][0]["score"], 0.6);
}

#[test]
fn argv_dedupes_preserving_order() {
    let dir = tempdir().unwrap();
    add_and_active(dir.path());
    add_skip_atom(dir.path());
    write_impl(dir.path(), "class DurabilityManager {}");
    let output = run(dir.path(), &["freshness", SAMPLE_ID, SKIP_ID, SAMPLE_ID]);
    let data = assert_ok(&output, "freshness");
    assert_eq!(data["count"], 2);
    assert_eq!(data["atoms"][0]["id"], SAMPLE_ID);
    assert_eq!(data["atoms"][0]["skipped"], false);
    assert_eq!(data["atoms"][1]["id"], SKIP_ID);
    assert_eq!(data["atoms"][1]["skipped"], true);
}

#[test]
fn missing_id_is_atom_not_found_and_writes_nothing() {
    let dir = tempdir().unwrap();
    add_and_active(dir.path());
    write_impl(dir.path(), "class DurabilityManager {}");
    let output = run(dir.path(), &["freshness", "no_such_atom", SAMPLE_ID]);
    let err = assert_err(&output, "freshness", "ATOM_NOT_FOUND");
    assert_eq!(err["details"]["id"], "no_such_atom");
    assert_eq!(disk_score(dir.path(), SAMPLE_ID), Some(1.0));
}

#[test]
fn draft_id_is_validation_failed_and_writes_nothing() {
    let dir = tempdir().unwrap();
    add_sample(dir.path());
    add_skip_atom(dir.path());
    write_impl(dir.path(), "class DurabilityManager {}");
    let output = run(dir.path(), &["freshness", SAMPLE_ID, SKIP_ID]);
    let err = assert_err(&output, "freshness", "VALIDATION_FAILED");
    assert_eq!(err["details"]["field"], "status");
    assert_eq!(disk_score(dir.path(), SKIP_ID), Some(1.0));
}

#[test]
fn all_skip_is_ok_with_zero_updates() {
    let dir = tempdir().unwrap();
    add_skip_atom(dir.path());
    let output = run(dir.path(), &["freshness"]);
    let data = assert_ok(&output, "freshness");
    assert_eq!(data["count"], 1);
    assert_eq!(data["updated-count"], 0);
    assert_eq!(data["atoms"][0], json!({"id": SKIP_ID, "skipped": true}));
}

#[test]
fn empty_dir_does_not_create_namespace() {
    let dir = tempdir().unwrap();
    let output = run(dir.path(), &["freshness"]);
    let data = assert_ok(&output, "freshness");
    assert_eq!(data["count"], 0);
    assert_eq!(data["updated-count"], 0);
    assert_eq!(data["atoms"], json!([]));
    assert!(!dir.path().join("opencanon").exists());
    assert!(!atoms_dir(dir.path()).exists());
}

#[test]
fn unknown_flag_is_usage_error() {
    let dir = tempdir().unwrap();
    let output = run(dir.path(), &["freshness", "--bogus"]);
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("\"ok\""), "{stdout}");
}

#[test]
fn does_not_read_opencanon_now() {
    let dir = tempdir().unwrap();
    add_and_active(dir.path());
    write_impl(dir.path(), "class DurabilityManager {}");
    let output = cmd(dir.path())
        .env("OPENCANON_NOW", "2099-01-01 00:00:00")
        .args(["freshness", SAMPLE_ID])
        .output()
        .unwrap();
    let data = assert_ok(&output, "freshness");
    assert_eq!(data["atoms"][0]["score"], 0.6);
    assert_eq!(factor(&data["atoms"][0], "impl-current")["value"], 0);
}

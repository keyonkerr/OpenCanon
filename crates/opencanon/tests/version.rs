mod common;

use common::{cmd, stdout_str};
use tempfile::tempdir;

#[test]
fn version_prints_cargo_version_without_envelope() {
    let dir = tempdir().unwrap();
    let output = cmd(dir.path()).arg("--version").output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        stdout_str(&output),
        format!("{}\n", env!("CARGO_PKG_VERSION"))
    );
    assert!(!dir.path().join("opencanon").exists());
    let text = stdout_str(&output);
    assert!(!text.starts_with('{'));
    assert_eq!(env!("CARGO_PKG_VERSION"), "0.1.0");
}

#[test]
fn version_does_not_write_files() {
    let dir = tempdir().unwrap();
    let _ = cmd(dir.path()).arg("--version").output().unwrap();
    assert!(std::fs::read_dir(dir.path()).unwrap().next().is_none());
}

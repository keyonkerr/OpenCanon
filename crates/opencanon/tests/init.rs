mod common;

use common::run;
use tempfile::tempdir;

#[test]
fn init_without_tty_is_usage_error() {
    let dir = tempdir().unwrap();
    let output = run(dir.path(), &["init"]);
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("\"ok\""), "{stdout}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("interactive terminal"), "stderr={stderr}");
    assert!(!dir.path().join("opencanon").exists());
}

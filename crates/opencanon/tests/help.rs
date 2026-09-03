mod common;

use common::{cmd, stdout_str};
use tempfile::tempdir;

const HELP: &str = "\
opencanon --version
opencanon add                                          # stdin: JSON array of {slug, title, body, tags?, freshness?}
opencanon get <id>
opencanon list [--status draft|active|deprecated] [--all]
opencanon edit                                         # stdin: JSON array of {id, title?, tags?, body?, freshness?}
opencanon delete <id>
opencanon active <id>
opencanon query <keyword>... [--status draft|active|deprecated] [--all]
opencanon compose                                      # stdin: JSON object {slug, title, atoms, body}
opencanon help
";

#[test]
fn help_prints_phase1_usage_lines_in_order() {
    let dir = tempdir().unwrap();
    let output = cmd(dir.path()).arg("help").output().unwrap();
    assert!(output.status.success());
    assert_eq!(stdout_str(&output), HELP);
    assert!(!dir.path().join("opencanon").exists());
}

#[test]
fn help_does_not_accept_a_subcommand() {
    let dir = tempdir().unwrap();
    let output = cmd(dir.path()).args(["help", "add"]).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stdout = stdout_str(&output);
    assert!(
        stdout.is_empty() || !stdout.contains("\"ok\""),
        "usage errors must not emit an envelope: {stdout}"
    );
}

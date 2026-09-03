#![allow(dead_code)]

use std::path::Path;
use std::process::Output;

use assert_cmd::Command;
use serde_json::Value;

pub const NOW: &str = "2026-09-01 13:05:00";
pub const SAMPLE_ID: &str = "durability_daily_restore";
pub const SAMPLE_TITLE: &str = "禁军突围装备耐久恢复机制";
pub const SAMPLE_BODY: &str = "正文：只描述一个事实。";

pub const SAMPLE_ADD_JSON: &str = r#"[
  {
    "slug": "durability_daily_restore",
    "title": "禁军突围装备耐久恢复机制",
    "tags": ["armybreak", "durability"],
    "body": "正文：只描述一个事实。",
    "freshness": { "impl-path": "gamesvr/DurabilityManager.java" }
  }
]"#;

pub const SAMPLE_MD: &str = "\
---
id: durability_daily_restore
status: draft
title: 禁军突围装备耐久恢复机制
tags:
  - armybreak
  - durability
freshness:
  impl-path: gamesvr/DurabilityManager.java
---
正文：只描述一个事实。
";

pub fn cmd(dir: &Path) -> Command {
    let mut command = Command::cargo_bin("opencanon").unwrap();
    command.current_dir(dir);
    command.env("OPENCANON_NOW", NOW);
    command.env("CLICOLOR", "0");
    command
}

pub fn run(dir: &Path, args: &[&str]) -> Output {
    cmd(dir).args(args).output().unwrap()
}

pub fn run_stdin(dir: &Path, args: &[&str], stdin: &str) -> Output {
    cmd(dir).args(args).write_stdin(stdin).output().unwrap()
}

pub fn run_stdin_now(dir: &Path, now: &str, args: &[&str], stdin: &str) -> Output {
    cmd(dir)
        .env("OPENCANON_NOW", now)
        .args(args)
        .write_stdin(stdin)
        .output()
        .unwrap()
}

pub fn stdout_str(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

pub fn stdout_json(output: &Output) -> Value {
    let text = stdout_str(output);
    serde_json::from_str(&text).unwrap_or_else(|e| {
        panic!("expected JSON stdout, got {text:?}: {e}");
    })
}

pub fn assert_ok(output: &Output, command: &str) -> Value {
    assert!(
        output.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&output.stderr),
        stdout_str(output)
    );
    let json = stdout_json(output);
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], command);
    assert!(json.get("error").is_none());
    json["data"].clone()
}

pub fn assert_err(output: &Output, command: &str, code: &str) -> Value {
    assert_eq!(
        output.status.code(),
        Some(1),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json = stdout_json(output);
    assert_eq!(json["ok"], false);
    assert_eq!(json["command"], command);
    assert_eq!(json["error"]["code"], code);
    assert!(json.get("data").is_none());
    json["error"].clone()
}

pub fn add_sample(dir: &Path) -> Value {
    let output = run_stdin(dir, &["add"], SAMPLE_ADD_JSON);
    assert_ok(&output, "add")
}

pub fn atoms_dir(dir: &Path) -> std::path::PathBuf {
    dir.join("opencanon").join("atoms")
}

pub fn docs_dir(dir: &Path) -> std::path::PathBuf {
    dir.join("opencanon").join("docs")
}

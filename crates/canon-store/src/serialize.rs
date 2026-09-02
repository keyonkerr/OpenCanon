use canon_core::{Atom, Freshness, Status};
use serde::Deserialize;

use crate::Error;

#[derive(Deserialize)]
struct FrontMatter {
    id: String,
    status: Status,
    title: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    freshness: Freshness,
}

pub fn to_markdown(atom: &Atom) -> String {
    let mut out = String::from("---\n");
    out.push_str(&format!("id: {}\n", yaml_scalar(&atom.id)));
    out.push_str(&format!("status: {}\n", atom.status));
    out.push_str(&format!("title: {}\n", yaml_scalar(&atom.title)));
    if atom.tags.is_empty() {
        out.push_str("tags: []\n");
    } else {
        out.push_str("tags:\n");
        for tag in &atom.tags {
            out.push_str(&format!("  - {}\n", yaml_scalar(tag)));
        }
    }
    if atom.freshness.is_empty() {
        out.push_str("freshness: {}\n");
    } else {
        out.push_str("freshness:\n");
        if let Some(v) = &atom.freshness.last_verified {
            out.push_str(&format!("  last-verified: {}\n", yaml_scalar(v)));
        }
        if let Some(v) = &atom.freshness.impl_path {
            out.push_str(&format!("  impl-path: {}\n", yaml_scalar(v)));
        }
        if let Some(v) = atom.freshness.score {
            out.push_str(&format!("  score: {}\n", v.yaml_display()));
        }
    }
    out.push_str("---\n");
    out.push_str(&atom.body);
    if !atom.body.ends_with('\n') {
        out.push('\n');
    }
    out
}

pub fn from_markdown(id: &str, text: &str) -> Result<Atom, Error> {
    let (yaml, body) = split_frontmatter(text).ok_or_else(|| Error::Parse {
        id: id.to_string(),
        message: "missing YAML frontmatter delimited by ---".into(),
    })?;
    let fm: FrontMatter = serde_yaml::from_str(yaml).map_err(|e| Error::Parse {
        id: id.to_string(),
        message: e.to_string(),
    })?;
    Ok(Atom {
        id: fm.id,
        status: fm.status,
        title: fm.title,
        tags: fm.tags,
        freshness: fm.freshness,
        body: body.to_string(),
    })
}

fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let rest = text
        .strip_prefix("---\r\n")
        .or_else(|| text.strip_prefix("---\n"))?;
    if let Some(i) = rest.find("\n---\r\n") {
        let yaml = &rest[..i];
        let body = strip_one_trailing_newline(&rest[i + "\n---\r\n".len()..]);
        return Some((yaml, body));
    }
    if let Some(i) = rest.find("\n---\n") {
        let yaml = &rest[..i];
        let body = strip_one_trailing_newline(&rest[i + "\n---\n".len()..]);
        return Some((yaml, body));
    }
    None
}

fn strip_one_trailing_newline(s: &str) -> &str {
    s.strip_suffix("\r\n")
        .or_else(|| s.strip_suffix('\n'))
        .unwrap_or(s)
}

fn yaml_scalar(s: &str) -> String {
    if needs_quotes(s) {
        let escaped = s
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

fn needs_quotes(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    let first = s.chars().next().unwrap();
    if first.is_whitespace()
        || matches!(
            first,
            '.' | '-'
                | '?'
                | ':'
                | '&'
                | '*'
                | '!'
                | '|'
                | '>'
                | '\''
                | '"'
                | '%'
                | '@'
                | '`'
                | '{'
                | '['
                | ','
                | '#'
        )
    {
        return true;
    }
    if s.ends_with([' ', '\t']) {
        return true;
    }
    if s.contains('\n') || s.contains('\r') || s.contains(": ") || s.contains('#') {
        return true;
    }
    let lower = s.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "true" | "false" | "null" | "yes" | "no" | "on" | "off" | "~"
    ) || looks_like_number(s)
}

fn looks_like_number(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '.' | '+' | '-' | 'e' | 'E'))
        && s.chars().any(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use canon_core::{Atom, Freshness, Score, Status};

    use super::{from_markdown, to_markdown};

    fn spec_atom() -> Atom {
        Atom {
            id: "durability_daily_restore".into(),
            status: Status::Draft,
            title: "禁军突围装备耐久恢复机制".into(),
            tags: vec!["armybreak".into(), "durability".into()],
            freshness: Freshness {
                impl_path: Some("gamesvr/DurabilityManager.java".into()),
                ..Freshness::default()
            },
            body: "正文：只描述一个事实。".into(),
        }
    }

    #[test]
    fn spec_example_key_order_and_kebab_case() {
        let md = to_markdown(&spec_atom());
        assert_eq!(
            md,
            "\
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
"
        );
        let back = from_markdown(&spec_atom().id, &md).unwrap();
        assert_eq!(back, spec_atom());
    }

    #[test]
    fn empty_tags_and_freshness_use_flow_empty() {
        let atom = Atom {
            id: "durability_cap_from_table".into(),
            status: Status::Active,
            title: "装备耐久从实现表读取上限".into(),
            tags: vec![],
            freshness: Freshness::default(),
            body: "耐久上限以配表为准。".into(),
        };
        let md = to_markdown(&atom);
        assert!(md.contains("tags: []\n"));
        assert!(md.contains("freshness: {}\n"));
        assert_eq!(from_markdown(&atom.id, &md).unwrap(), atom);
    }

    #[test]
    fn freshness_subkeys_ordered_last_verified_impl_path_score() {
        let atom = Atom {
            id: "x_sample".into(),
            status: Status::Active,
            title: "t".into(),
            tags: vec![],
            freshness: Freshness {
                last_verified: Some("2026-09-01 12:15:00".into()),
                impl_path: Some("gamesvr/DurabilityManager.java".into()),
                score: Some(Score::one()),
            },
            body: "b".into(),
        };
        let md = to_markdown(&atom);
        let freshness = md.split("freshness:\n").nth(1).unwrap();
        let lv = freshness.find("  last-verified:").unwrap();
        let ip = freshness.find("  impl-path:").unwrap();
        let sc = freshness.find("  score: 1\n").unwrap();
        assert!(lv < ip && ip < sc);
        assert!(!md.contains("score: 1.0"));
    }
}

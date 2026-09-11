use std::collections::{HashMap, HashSet};

use crate::model::{validate_slug, validate_title_body, ComposedDoc, Status};
use crate::Error;

const COMPOSE_INDEX: usize = 0;
const ATOM_LINK_MARKER: &str = "](../atoms/";
const INDEX_HEADING: &str = "依据";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComposeDraft {
    pub slug: String,
    pub title: String,
    pub atoms: Vec<String>,
    pub body: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComposeAtom {
    pub status: Status,
    pub title: String,
}

/// Validate a composed document, assign `id` from `slug`, and append a 依据
/// index from atom titles. Does not change atoms.
pub fn compose(
    draft: &ComposeDraft,
    known: &HashMap<String, ComposeAtom>,
) -> Result<ComposedDoc, Error> {
    validate_slug(&draft.slug)
        .map_err(|message| Error::validation(COMPOSE_INDEX, Some("slug".into()), message))?;
    validate_title_body(COMPOSE_INDEX, &draft.title, &draft.body)?;

    let atoms = unique_atom_ids(&draft.atoms)?;
    let mut index_entries = Vec::with_capacity(atoms.len());
    for id in &atoms {
        match known.get(id) {
            None => {
                return Err(Error::validation(
                    COMPOSE_INDEX,
                    Some("atoms".into()),
                    format!("atom `{id}` not found"),
                ));
            }
            Some(atom) if atom.status != Status::Active => {
                return Err(Error::validation(
                    COMPOSE_INDEX,
                    Some("atoms".into()),
                    format!("atom `{id}` is not active"),
                ));
            }
            Some(atom) if atom.title.is_empty() => {
                return Err(Error::validation(
                    COMPOSE_INDEX,
                    Some("atoms".into()),
                    format!("atom `{id}` title must be non-empty"),
                ));
            }
            Some(atom) => index_entries.push((id.clone(), atom.title.clone())),
        }
    }

    reject_atom_links(&draft.body)?;
    reject_index_heading(&draft.body)?;
    if content_paragraphs(&draft.body).is_empty() {
        return Err(Error::validation(
            COMPOSE_INDEX,
            Some("body".into()),
            "body must have at least one paragraph",
        ));
    }

    Ok(ComposedDoc {
        id: draft.slug.clone(),
        title: draft.title.clone(),
        atoms,
        body: append_index(&draft.body, &index_entries),
    })
}

fn unique_atom_ids(ids: &[String]) -> Result<Vec<String>, Error> {
    if ids.is_empty() {
        return Err(Error::validation(
            COMPOSE_INDEX,
            Some("atoms".into()),
            "atoms must be non-empty",
        ));
    }
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        if id.is_empty() {
            return Err(Error::validation(
                COMPOSE_INDEX,
                Some("atoms".into()),
                "atoms must not contain an empty id",
            ));
        }
        if !seen.insert(id.clone()) {
            return Err(Error::validation(
                COMPOSE_INDEX,
                Some("atoms".into()),
                format!("duplicate atom `{id}`"),
            ));
        }
        out.push(id.clone());
    }
    Ok(out)
}

fn reject_atom_links(body: &str) -> Result<(), Error> {
    if body.contains(ATOM_LINK_MARKER) {
        return Err(Error::validation(
            COMPOSE_INDEX,
            Some("body".into()),
            "body must not contain atom links",
        ));
    }
    Ok(())
}

fn reject_index_heading(body: &str) -> Result<(), Error> {
    for line in body.lines() {
        if heading_text(line) == Some(INDEX_HEADING) {
            return Err(Error::validation(
                COMPOSE_INDEX,
                Some("body".into()),
                "body must not contain a 依据 heading",
            ));
        }
    }
    Ok(())
}

fn append_index(body: &str, entries: &[(String, String)]) -> String {
    let mut out = body.trim_end().to_string();
    out.push_str("\n\n## ");
    out.push_str(INDEX_HEADING);
    out.push_str("\n\n");
    for (id, title) in entries {
        out.push_str("- [");
        out.push_str(&escape_link_text(title));
        out.push_str("](../atoms/");
        out.push_str(id);
        out.push_str(".md)\n");
    }
    out
}

fn escape_link_text(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    for c in title.chars() {
        match c {
            '\\' | '[' | ']' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

fn content_paragraphs(body: &str) -> Vec<String> {
    let body = body.replace("\r\n", "\n");
    let mut paras = Vec::new();
    for block in body.split("\n\n") {
        let mut lines: Vec<&str> = block.lines().collect();
        while let Some(first) = lines.first() {
            let trimmed = first.trim();
            if trimmed.is_empty() || is_heading(trimmed) {
                lines.remove(0);
                continue;
            }
            break;
        }
        let rest = lines.join("\n");
        let rest = rest.trim();
        if !rest.is_empty() {
            paras.push(rest.to_string());
        }
    }
    paras
}

fn heading_text(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if !is_heading(trimmed) {
        return None;
    }
    Some(trimmed.trim_start_matches('#').trim())
}

fn is_heading(line: &str) -> bool {
    let mut hashes = 0usize;
    for c in line.chars() {
        if c == '#' && hashes < 6 {
            hashes += 1;
        } else {
            break;
        }
    }
    if hashes == 0 {
        return false;
    }
    matches!(line[hashes..].chars().next(), None | Some(' ') | Some('\t'))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{compose, ComposeAtom, ComposeDraft};
    use crate::model::Status;
    use crate::Error;

    fn known(pairs: &[(&str, Status, &str)]) -> HashMap<String, ComposeAtom> {
        pairs
            .iter()
            .map(|(id, status, title)| {
                (
                    (*id).to_string(),
                    ComposeAtom {
                        status: *status,
                        title: (*title).to_string(),
                    },
                )
            })
            .collect()
    }

    fn draft(atoms: &[&str], body: &str) -> ComposeDraft {
        ComposeDraft {
            slug: "how_ssot_works".into(),
            title: "OpenCanon 如何保证一处事实只记一次".into(),
            atoms: atoms.iter().map(|s| (*s).to_string()).collect(),
            body: body.into(),
        }
    }

    fn ok_body() -> &'static str {
        "\
# OpenCanon 如何保证一处事实只记一次

摘要：一处事实只记一次。

按问题组合原子成文。
"
    }

    fn ok_known() -> HashMap<String, ComposeAtom> {
        known(&[
            (
                "ssot_one_place",
                Status::Active,
                "一处事实只记录一次，其它文档只引用不复制",
            ),
            (
                "compose_by_topic",
                Status::Active,
                "拼接：要派生可读文档时按问题召回原子并由 LLM 组合成文，派生文不是真源",
            ),
        ])
    }

    #[test]
    fn assigns_id_from_slug_keeps_atom_order_and_appends_index() {
        let doc = compose(
            &draft(&["ssot_one_place", "compose_by_topic"], ok_body()),
            &ok_known(),
        )
        .unwrap();
        assert_eq!(doc.id, "how_ssot_works");
        assert_eq!(
            doc.atoms,
            vec!["ssot_one_place".to_string(), "compose_by_topic".to_string()]
        );
        assert_eq!(doc.title, "OpenCanon 如何保证一处事实只记一次");
        assert_eq!(
            doc.body,
            "\
# OpenCanon 如何保证一处事实只记一次

摘要：一处事实只记一次。

按问题组合原子成文。

## 依据

- [一处事实只记录一次，其它文档只引用不复制](../atoms/ssot_one_place.md)
- [拼接：要派生可读文档时按问题召回原子并由 LLM 组合成文，派生文不是真源](../atoms/compose_by_topic.md)
"
        );
    }

    #[test]
    fn escapes_brackets_in_index_titles() {
        let body = "# t\n\nx\n";
        let doc = compose(
            &draft(&["ssot_one_place"], body),
            &known(&[("ssot_one_place", Status::Active, "see [ssot] in title")]),
        )
        .unwrap();
        assert!(doc
            .body
            .contains("- [see \\[ssot\\] in title](../atoms/ssot_one_place.md)\n"));
    }

    #[test]
    fn invalid_slug_fails() {
        let mut input = draft(&["ssot_one_place"], ok_body());
        input.slug = "/bad".into();
        input.atoms = vec!["ssot_one_place".into()];
        input.body = "# t\n\nx\n".into();
        let err = compose(&input, &known(&[("ssot_one_place", Status::Active, "t")])).unwrap_err();
        match err {
            Error::Validation { index, field, .. } => {
                assert_eq!(index, 0);
                assert_eq!(field.as_deref(), Some("slug"));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn empty_atoms_fails() {
        let err = compose(&draft(&[], ok_body()), &ok_known()).unwrap_err();
        assert_eq!(
            err,
            Error::validation(0, Some("atoms".into()), "atoms must be non-empty")
        );
    }

    #[test]
    fn duplicate_atoms_fail() {
        let err = compose(
            &draft(&["ssot_one_place", "ssot_one_place"], "# t\n\nx\n"),
            &known(&[("ssot_one_place", Status::Active, "t")]),
        )
        .unwrap_err();
        assert_eq!(
            err,
            Error::validation(0, Some("atoms".into()), "duplicate atom `ssot_one_place`")
        );
    }

    #[test]
    fn missing_atom_fails() {
        let err = compose(&draft(&["missing_id"], "# t\n\nx\n"), &HashMap::new()).unwrap_err();
        assert_eq!(
            err,
            Error::validation(0, Some("atoms".into()), "atom `missing_id` not found")
        );
    }

    #[test]
    fn draft_atom_fails() {
        let err = compose(
            &draft(&["ssot_one_place"], "# t\n\nx\n"),
            &known(&[("ssot_one_place", Status::Draft, "t")]),
        )
        .unwrap_err();
        assert_eq!(
            err,
            Error::validation(
                0,
                Some("atoms".into()),
                "atom `ssot_one_place` is not active"
            )
        );
    }

    #[test]
    fn atom_link_in_body_fails() {
        let body = "\
# t

x [ssot_one_place](../atoms/ssot_one_place.md)
";
        let err = compose(
            &draft(&["ssot_one_place"], body),
            &known(&[("ssot_one_place", Status::Active, "t")]),
        )
        .unwrap_err();
        assert_eq!(
            err,
            Error::validation(0, Some("body".into()), "body must not contain atom links")
        );
    }

    #[test]
    fn index_heading_in_body_fails() {
        let body = "\
# t

x

## 依据
";
        let err = compose(
            &draft(&["ssot_one_place"], body),
            &known(&[("ssot_one_place", Status::Active, "t")]),
        )
        .unwrap_err();
        assert_eq!(
            err,
            Error::validation(
                0,
                Some("body".into()),
                "body must not contain a 依据 heading"
            )
        );
    }

    #[test]
    fn heading_only_body_fails() {
        let err = compose(
            &draft(&["ssot_one_place"], "# Title\n"),
            &known(&[("ssot_one_place", Status::Active, "t")]),
        )
        .unwrap_err();
        assert_eq!(
            err,
            Error::validation(
                0,
                Some("body".into()),
                "body must have at least one paragraph"
            )
        );
    }
}

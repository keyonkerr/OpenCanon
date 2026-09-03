use std::collections::{HashMap, HashSet};

use crate::model::{validate_slug, validate_title_body, ComposedDoc, Status};
use crate::Error;

const COMPOSE_INDEX: usize = 0;
const CITE_OPEN: &str = "(../atoms/";
const CITE_MARKER: &str = "](../atoms/";
const CITE_CLOSE: &str = ".md)";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComposeDraft {
    pub slug: String,
    pub title: String,
    pub atoms: Vec<String>,
    pub body: String,
}

/// Validate a composed document and assign `id` from `slug`. Does not change atoms.
pub fn compose(
    draft: &ComposeDraft,
    known: &HashMap<String, Status>,
) -> Result<ComposedDoc, Error> {
    validate_slug(&draft.slug)
        .map_err(|message| Error::validation(COMPOSE_INDEX, Some("slug".into()), message))?;
    validate_title_body(COMPOSE_INDEX, &draft.title, &draft.body)?;

    let atoms = unique_atom_ids(&draft.atoms)?;
    for id in &atoms {
        match known.get(id) {
            None => {
                return Err(Error::validation(
                    COMPOSE_INDEX,
                    Some("atoms".into()),
                    format!("atom `{id}` not found"),
                ));
            }
            Some(status) if *status != Status::Active => {
                return Err(Error::validation(
                    COMPOSE_INDEX,
                    Some("atoms".into()),
                    format!("atom `{id}` is not active"),
                ));
            }
            Some(_) => {}
        }
    }

    let cited = all_citations(&draft.body)?;
    let paragraphs = content_paragraphs(&draft.body);
    if paragraphs.is_empty() {
        return Err(Error::validation(
            COMPOSE_INDEX,
            Some("body".into()),
            "body must have at least one cited paragraph",
        ));
    }
    for para in &paragraphs {
        trailing_citations(para)?;
    }

    if cited != atoms.iter().cloned().collect::<HashSet<_>>() {
        return Err(Error::validation(
            COMPOSE_INDEX,
            Some("atoms".into()),
            "atoms must equal the set of citations in body",
        ));
    }

    Ok(ComposedDoc {
        id: draft.slug.clone(),
        title: draft.title.clone(),
        atoms,
        body: draft.body.clone(),
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

fn trailing_citations(para: &str) -> Result<Vec<String>, Error> {
    let mut rest = para.trim_end();
    let mut ids = Vec::new();
    loop {
        match strip_one_citation_suffix(rest) {
            Some((before, id)) => {
                ids.push(id);
                rest = before.trim_end();
            }
            None => break,
        }
    }
    if ids.is_empty() {
        return Err(Error::validation(
            COMPOSE_INDEX,
            Some("body".into()),
            "each paragraph must end with at least one citation",
        ));
    }
    if rest.is_empty() {
        return Err(Error::validation(
            COMPOSE_INDEX,
            Some("body".into()),
            "each paragraph must have text before citations",
        ));
    }
    ids.reverse();
    Ok(ids)
}

fn strip_one_citation_suffix(s: &str) -> Option<(&str, String)> {
    let s = s.trim_end();
    if !s.ends_with(CITE_CLOSE) {
        return None;
    }
    let without_close = &s[..s.len() - CITE_CLOSE.len()];
    let idx = without_close.rfind(CITE_MARKER)?;
    let id = &without_close[idx + CITE_MARKER.len()..];
    if !valid_cite_id(id) {
        return None;
    }
    let before_brack = &without_close[..idx];
    let bracket = before_brack.rfind('[')?;
    let label = &before_brack[bracket + 1..];
    if label != id || label.contains('\n') {
        return None;
    }
    Some((&before_brack[..bracket], id.to_string()))
}

fn all_citations(body: &str) -> Result<HashSet<String>, Error> {
    let mut ids = HashSet::new();
    let mut from = 0;
    while let Some(rel) = body[from..].find(CITE_OPEN) {
        let abs = from + rel;
        if abs == 0 || !body[..abs].ends_with(']') {
            return Err(Error::validation(
                COMPOSE_INDEX,
                Some("body".into()),
                "citation must be [id](../atoms/id.md)",
            ));
        }
        let after = abs + CITE_OPEN.len();
        let rest = &body[after..];
        let Some(end) = rest.find(CITE_CLOSE) else {
            return Err(Error::validation(
                COMPOSE_INDEX,
                Some("body".into()),
                "citation must be [id](../atoms/id.md)",
            ));
        };
        let id = &rest[..end];
        if !valid_cite_id(id) {
            return Err(Error::validation(
                COMPOSE_INDEX,
                Some("body".into()),
                "citation must be [id](../atoms/id.md)",
            ));
        }
        let before = &body[..abs - 1];
        let Some(bracket) = before.rfind('[') else {
            return Err(Error::validation(
                COMPOSE_INDEX,
                Some("body".into()),
                "citation must be [id](../atoms/id.md)",
            ));
        };
        let label = &before[bracket + 1..];
        if label != id || label.contains('\n') {
            return Err(Error::validation(
                COMPOSE_INDEX,
                Some("body".into()),
                "citation link text must equal atom id",
            ));
        }
        ids.insert(id.to_string());
        from = after + end + CITE_CLOSE.len();
    }
    Ok(ids)
}

fn valid_cite_id(id: &str) -> bool {
    !id.is_empty() && !id.contains('/') && !id.contains(')') && !id.contains('\n')
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{compose, ComposeDraft};
    use crate::model::Status;
    use crate::Error;

    fn known(pairs: &[(&str, Status)]) -> HashMap<String, Status> {
        pairs
            .iter()
            .map(|(id, status)| ((*id).to_string(), *status))
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

摘要：一处事实只记一次。 [ssot_one_place](../atoms/ssot_one_place.md)

按问题组合原子成文。 [compose_by_topic](../atoms/compose_by_topic.md) [ssot_one_place](../atoms/ssot_one_place.md)
"
    }

    #[test]
    fn assigns_id_from_slug_and_keeps_atom_order() {
        let doc = compose(
            &draft(&["ssot_one_place", "compose_by_topic"], ok_body()),
            &known(&[
                ("ssot_one_place", Status::Active),
                ("compose_by_topic", Status::Active),
            ]),
        )
        .unwrap();
        assert_eq!(doc.id, "how_ssot_works");
        assert_eq!(
            doc.atoms,
            vec!["ssot_one_place".to_string(), "compose_by_topic".to_string()]
        );
        assert_eq!(doc.title, "OpenCanon 如何保证一处事实只记一次");
        assert_eq!(doc.body, ok_body());
    }

    #[test]
    fn invalid_slug_fails() {
        let mut input = draft(&["ssot_one_place"], ok_body());
        input.slug = "/bad".into();
        input.atoms = vec!["ssot_one_place".into()];
        input.body = "\
# t

x [ssot_one_place](../atoms/ssot_one_place.md)
"
        .into();
        let err = compose(&input, &known(&[("ssot_one_place", Status::Active)])).unwrap_err();
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
        let err = compose(
            &draft(&[], ok_body()),
            &known(&[("ssot_one_place", Status::Active)]),
        )
        .unwrap_err();
        assert_eq!(
            err,
            Error::validation(0, Some("atoms".into()), "atoms must be non-empty")
        );
    }

    #[test]
    fn duplicate_atoms_fail() {
        let body = "\
# t

x [ssot_one_place](../atoms/ssot_one_place.md)
";
        let err = compose(
            &draft(&["ssot_one_place", "ssot_one_place"], body),
            &known(&[("ssot_one_place", Status::Active)]),
        )
        .unwrap_err();
        assert_eq!(
            err,
            Error::validation(0, Some("atoms".into()), "duplicate atom `ssot_one_place`")
        );
    }

    #[test]
    fn missing_atom_fails() {
        let body = "\
# t

x [missing_id](../atoms/missing_id.md)
";
        let err = compose(&draft(&["missing_id"], body), &HashMap::new()).unwrap_err();
        assert_eq!(
            err,
            Error::validation(0, Some("atoms".into()), "atom `missing_id` not found")
        );
    }

    #[test]
    fn draft_atom_fails() {
        let body = "\
# t

x [ssot_one_place](../atoms/ssot_one_place.md)
";
        let err = compose(
            &draft(&["ssot_one_place"], body),
            &known(&[("ssot_one_place", Status::Draft)]),
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
    fn paragraph_without_citation_fails() {
        let body = "\
# t

摘要没有引用。

正文。 [ssot_one_place](../atoms/ssot_one_place.md)
";
        let err = compose(
            &draft(&["ssot_one_place"], body),
            &known(&[("ssot_one_place", Status::Active)]),
        )
        .unwrap_err();
        assert_eq!(
            err,
            Error::validation(
                0,
                Some("body".into()),
                "each paragraph must end with at least one citation"
            )
        );
    }

    #[test]
    fn citation_label_must_equal_id() {
        let body = "\
# t

x [see](../atoms/ssot_one_place.md)
";
        let err = compose(
            &draft(&["ssot_one_place"], body),
            &known(&[("ssot_one_place", Status::Active)]),
        )
        .unwrap_err();
        assert_eq!(
            err,
            Error::validation(
                0,
                Some("body".into()),
                "citation link text must equal atom id"
            )
        );
    }

    #[test]
    fn unused_atom_in_field_fails() {
        let body = "\
# t

x [ssot_one_place](../atoms/ssot_one_place.md)
";
        let err = compose(
            &draft(&["ssot_one_place", "compose_by_topic"], body),
            &known(&[
                ("ssot_one_place", Status::Active),
                ("compose_by_topic", Status::Active),
            ]),
        )
        .unwrap_err();
        assert_eq!(
            err,
            Error::validation(
                0,
                Some("atoms".into()),
                "atoms must equal the set of citations in body"
            )
        );
    }

    #[test]
    fn cited_id_not_in_atoms_field_fails() {
        let body = "\
# t

x [ssot_one_place](../atoms/ssot_one_place.md) [compose_by_topic](../atoms/compose_by_topic.md)
";
        let err = compose(
            &draft(&["ssot_one_place"], body),
            &known(&[
                ("ssot_one_place", Status::Active),
                ("compose_by_topic", Status::Active),
            ]),
        )
        .unwrap_err();
        assert_eq!(
            err,
            Error::validation(
                0,
                Some("atoms".into()),
                "atoms must equal the set of citations in body"
            )
        );
    }

    #[test]
    fn heading_only_body_fails() {
        let err = compose(
            &draft(&["ssot_one_place"], "# Title\n"),
            &known(&[("ssot_one_place", Status::Active)]),
        )
        .unwrap_err();
        assert_eq!(
            err,
            Error::validation(
                0,
                Some("body".into()),
                "body must have at least one cited paragraph"
            )
        );
    }

    #[test]
    fn wrong_citation_path_fails() {
        let body = "\
# t

x [ssot_one_place](opencanon/atoms/ssot_one_place.md)
";
        let err = compose(
            &draft(&["ssot_one_place"], body),
            &known(&[("ssot_one_place", Status::Active)]),
        )
        .unwrap_err();
        assert_eq!(
            err,
            Error::validation(
                0,
                Some("body".into()),
                "each paragraph must end with at least one citation"
            )
        );
    }
}

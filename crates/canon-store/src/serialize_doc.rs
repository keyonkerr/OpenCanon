use canon_core::ComposedDoc;
use serde::Deserialize;

use crate::serialize::{split_frontmatter, yaml_scalar};
use crate::Error;

#[derive(Deserialize)]
struct FrontMatter {
    id: String,
    title: String,
    #[serde(default)]
    atoms: Vec<String>,
}

pub fn to_markdown(doc: &ComposedDoc) -> String {
    let mut out = String::from("---\n");
    out.push_str(&format!("id: {}\n", yaml_scalar(&doc.id)));
    out.push_str(&format!("title: {}\n", yaml_scalar(&doc.title)));
    out.push_str("atoms:\n");
    for id in &doc.atoms {
        out.push_str(&format!("  - {}\n", yaml_scalar(id)));
    }
    out.push_str("---\n");
    out.push_str(&doc.body);
    if !doc.body.ends_with('\n') {
        out.push('\n');
    }
    out
}

pub fn from_markdown(id: &str, text: &str) -> Result<ComposedDoc, Error> {
    let (yaml, body) = split_frontmatter(text).ok_or_else(|| Error::Parse {
        id: id.to_string(),
        message: "missing YAML frontmatter delimited by ---".into(),
    })?;
    let fm: FrontMatter = serde_yaml::from_str(yaml).map_err(|e| Error::Parse {
        id: id.to_string(),
        message: e.to_string(),
    })?;
    Ok(ComposedDoc {
        id: fm.id,
        title: fm.title,
        atoms: fm.atoms,
        body: body.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use canon_core::ComposedDoc;

    use super::{from_markdown, to_markdown};

    fn spec_doc() -> ComposedDoc {
        ComposedDoc {
            id: "how_ssot_works".into(),
            title: "OpenCanon 如何保证一处事实只记一次".into(),
            atoms: vec!["ssot_one_place".into(), "compose_by_topic".into()],
            body: "# OpenCanon 如何保证一处事实只记一次\n\n摘要。 [ssot_one_place](../atoms/ssot_one_place.md)"
                .into(),
        }
    }

    #[test]
    fn key_order_is_id_title_atoms() {
        let md = to_markdown(&spec_doc());
        assert_eq!(
            md,
            "\
---
id: how_ssot_works
title: OpenCanon 如何保证一处事实只记一次
atoms:
  - ssot_one_place
  - compose_by_topic
---
# OpenCanon 如何保证一处事实只记一次

摘要。 [ssot_one_place](../atoms/ssot_one_place.md)
"
        );
        assert_eq!(from_markdown(&spec_doc().id, &md).unwrap(), spec_doc());
    }
}

use crate::model::{Atom, Freshness, ImplPaths, Score, Status};
use crate::Error;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize)]
pub struct FreshnessPatch {
    #[serde(rename = "last-verified")]
    pub last_verified: Option<String>,
    #[serde(rename = "impl-path")]
    pub impl_path: Option<ImplPaths>,
    pub score: Option<Score>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditPatch {
    pub id: String,
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub body: Option<String>,
    /// `None` = key omitted; `Some` even if empty = key appeared.
    pub freshness: Option<FreshnessPatch>,
    pub status: Option<Status>,
}

impl EditPatch {
    pub fn has_mutable_field(&self) -> bool {
        self.title.is_some()
            || self.tags.is_some()
            || self.body.is_some()
            || self.freshness.is_some()
    }
}

pub fn apply_edits(current: &[Atom], patches: &[EditPatch]) -> Result<Vec<Atom>, Error> {
    let mut working: Vec<Atom> = current.to_vec();
    let mut out = Vec::with_capacity(patches.len());

    for (index, patch) in patches.iter().enumerate() {
        let pos = working
            .iter()
            .position(|a| a.id == patch.id)
            .ok_or_else(|| {
                Error::validation(
                    index,
                    Some("id".into()),
                    format!("atom `{}` not in batch", patch.id),
                )
            })?;

        let applied = apply_one(index, working[pos].clone(), patch)?;
        working[pos] = applied.clone();
        out.push(applied);
    }

    Ok(out)
}

fn apply_one(index: usize, mut atom: Atom, patch: &EditPatch) -> Result<Atom, Error> {
    if let Some(status) = patch.status {
        if status != atom.status {
            return Err(Error::ImmutableField {
                index,
                field: "status".into(),
            });
        }
    }

    if !patch.has_mutable_field() {
        return Err(Error::validation(
            index,
            None,
            "edit requires at least one of title, tags, body, freshness",
        ));
    }

    if let Some(title) = &patch.title {
        if title.is_empty() {
            return Err(Error::validation(
                index,
                Some("title".into()),
                "title must be non-empty",
            ));
        }
        atom.title = title.clone();
    }
    if let Some(body) = &patch.body {
        if body.is_empty() {
            return Err(Error::validation(
                index,
                Some("body".into()),
                "body must be non-empty",
            ));
        }
        atom.body = body.clone();
    }
    if let Some(tags) = &patch.tags {
        atom.tags = tags.clone();
    }
    if let Some(freshness) = &patch.freshness {
        atom.freshness = merge_freshness(atom.freshness, freshness);
    }

    Ok(atom)
}

fn merge_freshness(old: Freshness, patch: &FreshnessPatch) -> Freshness {
    Freshness {
        last_verified: patch.last_verified.clone().or(old.last_verified),
        impl_path: patch.impl_path.clone().unwrap_or(old.impl_path),
        score: patch.score.or(old.score),
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_edits, EditPatch, FreshnessPatch};
    use crate::model::{Atom, Freshness, Score, Status};
    use crate::Error;

    fn atom() -> Atom {
        Atom {
            id: "durability_daily_restore".into(),
            status: Status::Draft,
            title: "禁军突围装备耐久恢复机制".into(),
            tags: vec!["armybreak".into(), "durability".into()],
            freshness: Freshness {
                impl_path: "gamesvr/DurabilityManager.java".into(),
                ..Freshness::default()
            },
            body: "正文：只描述一个事实。".into(),
        }
    }

    #[test]
    fn title_change_keeps_id_status_tags_freshness_body() {
        let patch = EditPatch {
            id: atom().id,
            title: Some("禁军突围：耐久按日恢复".into()),
            tags: None,
            body: None,
            freshness: None,
            status: None,
        };
        let out = apply_edits(&[atom()], &[patch]).unwrap();
        assert_eq!(out[0].id, atom().id);
        assert_eq!(out[0].status, Status::Draft);
        assert_eq!(out[0].title, "禁军突围：耐久按日恢复");
        assert_eq!(out[0].tags, atom().tags);
        assert_eq!(out[0].freshness, atom().freshness);
        assert_eq!(out[0].body, atom().body);
    }

    #[test]
    fn tags_replace_and_empty_clears() {
        let mut patch = EditPatch {
            id: atom().id,
            title: None,
            tags: Some(vec!["only".into()]),
            body: None,
            freshness: None,
            status: None,
        };
        let out = apply_edits(&[atom()], &[patch.clone()]).unwrap();
        assert_eq!(out[0].tags, vec!["only".to_string()]);

        patch.tags = Some(vec![]);
        let out = apply_edits(&[atom()], &[patch]).unwrap();
        assert!(out[0].tags.is_empty());
    }

    #[test]
    fn freshness_merges_subkeys_and_empty_object_keeps_existing() {
        let patch = EditPatch {
            id: atom().id,
            title: None,
            tags: None,
            body: None,
            freshness: Some(FreshnessPatch {
                last_verified: Some("2026-09-01 12:15:00".into()),
                impl_path: None,
                score: None,
            }),
            status: None,
        };
        let out = apply_edits(&[atom()], &[patch]).unwrap();
        assert_eq!(
            out[0].freshness.last_verified.as_deref(),
            Some("2026-09-01 12:15:00")
        );
        assert_eq!(
            out[0].freshness.impl_path.as_slice(),
            ["gamesvr/DurabilityManager.java"]
        );

        let keep = EditPatch {
            id: atom().id,
            title: None,
            tags: None,
            body: None,
            freshness: Some(FreshnessPatch::default()),
            status: None,
        };
        let out = apply_edits(&[atom()], &[keep]).unwrap();
        assert_eq!(out[0].freshness, atom().freshness);
    }

    #[test]
    fn same_status_is_ignored_different_status_fails_and_id_never_changes() {
        let same = EditPatch {
            id: atom().id,
            title: Some("n".into()),
            tags: None,
            body: None,
            freshness: None,
            status: Some(Status::Draft),
        };
        assert_eq!(
            apply_edits(&[atom()], &[same]).unwrap()[0].status,
            Status::Draft
        );

        let different = EditPatch {
            id: atom().id,
            title: Some("n".into()),
            tags: None,
            body: None,
            freshness: None,
            status: Some(Status::Active),
        };
        assert_eq!(
            apply_edits(&[atom()], &[different]).unwrap_err(),
            Error::ImmutableField {
                index: 0,
                field: "status".into(),
            }
        );
    }

    #[test]
    fn requires_a_mutable_field() {
        let patch = EditPatch {
            id: atom().id,
            title: None,
            tags: None,
            body: None,
            freshness: None,
            status: Some(Status::Draft),
        };
        match apply_edits(&[atom()], &[patch]).unwrap_err() {
            Error::Validation { index, .. } => assert_eq!(index, 0),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn activate_score_patch_merges() {
        let patch = EditPatch {
            id: atom().id,
            title: None,
            tags: None,
            body: None,
            freshness: Some(FreshnessPatch {
                last_verified: None,
                impl_path: None,
                score: Some(Score::one()),
            }),
            status: None,
        };
        let out = apply_edits(&[atom()], &[patch]).unwrap();
        assert_eq!(out[0].freshness.score, Some(Score::one()));
        assert!(!out[0].freshness.impl_path.is_empty());
    }
}

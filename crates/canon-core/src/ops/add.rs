use std::collections::HashMap;

use crate::model::{validate_slug, validate_title_body, Atom, Freshness, Status};
use crate::Error;

use super::id::{assign_ids, Occupied};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddDraft {
    pub slug: String,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub freshness: Freshness,
}

/// Validate every draft, use each slug as id, force `draft`.
/// Ignores any caller-supplied id/status — those never appear on [`AddDraft`].
pub fn add_drafts(
    drafts: &[AddDraft],
    occupied: &HashMap<String, Status>,
) -> Result<Vec<Atom>, Error> {
    for (index, draft) in drafts.iter().enumerate() {
        validate_slug(&draft.slug)
            .map_err(|message| Error::validation(index, Some("slug".into()), message))?;
        validate_title_body(index, &draft.title, &draft.body)?;
    }

    let occupied: HashMap<String, Occupied> = occupied
        .iter()
        .map(|(id, status)| (id.clone(), Occupied::Disk(*status)))
        .collect();
    let ids = assign_ids(drafts.iter().map(|d| d.slug.as_str()), &occupied)?;
    Ok(drafts
        .iter()
        .zip(ids)
        .map(|(draft, id)| Atom {
            id,
            status: Status::Draft,
            title: draft.title.clone(),
            tags: draft.tags.clone(),
            freshness: draft.freshness.clone(),
            body: draft.body.clone(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{add_drafts, AddDraft};
    use crate::model::{Freshness, Status};
    use crate::Error;

    fn draft(slug: &str, title: &str, body: &str) -> AddDraft {
        AddDraft {
            slug: slug.into(),
            title: title.into(),
            body: body.into(),
            tags: vec!["armybreak".into()],
            freshness: Freshness {
                impl_path: Some("gamesvr/DurabilityManager.java".into()),
                ..Freshness::default()
            },
        }
    }

    #[test]
    fn forces_draft_and_uses_slug_as_id() {
        let atoms = add_drafts(
            &[draft(
                "durability_daily_restore",
                "禁军突围装备耐久恢复机制",
                "正文：只描述一个事实。",
            )],
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(atoms.len(), 1);
        assert_eq!(atoms[0].id, "durability_daily_restore");
        assert_eq!(atoms[0].status, Status::Draft);
        assert_eq!(atoms[0].title, "禁军突围装备耐久恢复机制");
        assert!(!atoms[0].id.contains("ATOM-"));
    }

    #[test]
    fn changing_title_does_not_change_id() {
        let atoms = add_drafts(
            &[draft("durability_daily_restore", "another title", "body")],
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(atoms[0].id, "durability_daily_restore");
        assert_eq!(atoms[0].title, "another title");
    }

    #[test]
    fn empty_title_fails_before_any_id_is_issued() {
        let err = add_drafts(
            &[draft("ok_slug", "ok", "body"), draft("also_ok", "", "body")],
            &HashMap::new(),
        )
        .unwrap_err();
        assert_eq!(
            err,
            Error::validation(1, Some("title".into()), "title must be non-empty")
        );
    }

    #[test]
    fn invalid_slug_is_validation_failed() {
        let err = add_drafts(&[draft("/bad", "t", "b")], &HashMap::new()).unwrap_err();
        match err {
            Error::Validation { index, field, .. } => {
                assert_eq!(index, 0);
                assert_eq!(field.as_deref(), Some("slug"));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn occupied_slug_reports_all_conflicts() {
        let mut occupied = HashMap::new();
        occupied.insert("durability_daily_restore".into(), Status::Active);
        occupied.insert("durability_cap_from_table".into(), Status::Deprecated);
        let err = add_drafts(
            &[
                draft("durability_daily_restore", "t", "b"),
                draft("fresh_slug", "t", "b"),
                draft("durability_cap_from_table", "t", "b"),
            ],
            &occupied,
        )
        .unwrap_err();
        match err {
            Error::SlugConflict { conflicts } => {
                assert_eq!(conflicts.len(), 2);
                assert_eq!(conflicts[0].index, 0);
                assert_eq!(conflicts[0].slug, "durability_daily_restore");
                assert_eq!(conflicts[0].status, Some(Status::Active));
                assert_eq!(conflicts[1].index, 2);
                assert_eq!(conflicts[1].slug, "durability_cap_from_table");
                assert_eq!(conflicts[1].status, Some(Status::Deprecated));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn in_batch_duplicate_slug_conflicts_without_disk_status() {
        let err = add_drafts(
            &[
                draft("same_slug", "one", "a"),
                draft("same_slug", "two", "b"),
            ],
            &HashMap::new(),
        )
        .unwrap_err();
        match err {
            Error::SlugConflict { conflicts } => {
                assert_eq!(conflicts.len(), 1);
                assert_eq!(conflicts[0].index, 1);
                assert_eq!(conflicts[0].slug, "same_slug");
                assert_eq!(conflicts[0].status, None);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn empty_batch_is_ok() {
        let atoms = add_drafts(&[], &HashMap::new()).unwrap();
        assert!(atoms.is_empty());
    }
}

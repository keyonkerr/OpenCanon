use crate::model::Atom;
use crate::ops::{filter_atoms, ListFilter};
use crate::Error;

/// Filter by [`ListFilter`], then keep atoms whose `body` or `id` contains
/// any keyword (trim + case-fold substring).
///
/// `title` and `tags` do not participate. Input order is preserved.
pub fn query(
    atoms: Vec<Atom>,
    keywords: &[String],
    filter: ListFilter,
) -> Result<Vec<Atom>, Error> {
    let needles = needles(keywords)?;
    Ok(filter_atoms(atoms, filter)
        .into_iter()
        .filter(|atom| matches_keyword(atom, &needles))
        .collect())
}

fn needles(keywords: &[String]) -> Result<Vec<String>, Error> {
    if keywords.is_empty() {
        return Err(Error::validation(
            0,
            Some("keyword".into()),
            "keyword must be non-empty",
        ));
    }
    let mut needles = Vec::with_capacity(keywords.len());
    for (index, keyword) in keywords.iter().enumerate() {
        let needle = keyword.trim().to_lowercase();
        if needle.is_empty() {
            return Err(Error::validation(
                index,
                Some("keyword".into()),
                "keyword must be non-empty",
            ));
        }
        needles.push(needle);
    }
    Ok(needles)
}

fn matches_keyword(atom: &Atom, needles: &[String]) -> bool {
    contains_any(&atom.body, needles) || contains_any(&atom.id, needles)
}

fn contains_any(haystack: &str, needles: &[String]) -> bool {
    let haystack = haystack.to_lowercase();
    needles
        .iter()
        .any(|needle| haystack.contains(needle.as_str()))
}

#[cfg(test)]
mod tests {
    use super::query;
    use crate::model::{Atom, Freshness, Status};
    use crate::ops::ListFilter;
    use crate::Error;

    fn atom(id: &str, status: Status, title: &str, tags: &[&str], body: &str) -> Atom {
        Atom {
            id: id.into(),
            status,
            title: title.into(),
            tags: tags.iter().map(|t| (*t).to_string()).collect(),
            freshness: Freshness::default(),
            body: body.into(),
        }
    }

    fn ids(atoms: &[Atom]) -> Vec<&str> {
        atoms.iter().map(|a| a.id.as_str()).collect()
    }

    fn sample() -> Vec<Atom> {
        vec![
            atom(
                "durability_draft",
                Status::Draft,
                "耐久草稿标题",
                &["durability"],
                "草稿正文里有恢复。",
            ),
            atom(
                "durability_daily_restore",
                Status::Active,
                "无关标题",
                &["armybreak"],
                "禁军突围中，装备耐久按日恢复。",
            ),
            atom(
                "durability_deprecated",
                Status::Deprecated,
                "耐久已下线",
                &["durability"],
                "已下线正文仍写耐久恢复。",
            ),
            atom(
                "title_only",
                Status::Active,
                "耐久只在标题",
                &["durability"],
                "这条正文完全不提那个词。",
            ),
        ]
    }

    #[test]
    fn default_is_active_body_only() {
        let hits = query(sample(), &["耐久".into()], ListFilter::Active).unwrap();
        assert_eq!(ids(&hits), vec!["durability_daily_restore"]);
    }

    #[test]
    fn all_includes_draft_and_deprecated() {
        let hits = query(sample(), &["恢复".into()], ListFilter::All).unwrap();
        assert_eq!(
            ids(&hits),
            vec![
                "durability_draft",
                "durability_daily_restore",
                "durability_deprecated",
            ]
        );
    }

    #[test]
    fn status_draft_excludes_active() {
        let hits = query(
            sample(),
            &["恢复".into()],
            ListFilter::Status(Status::Draft),
        )
        .unwrap();
        assert_eq!(ids(&hits), vec!["durability_draft"]);
    }

    #[test]
    fn title_only_does_not_match() {
        let hits = query(sample(), &["只在标题".into()], ListFilter::Active).unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn tag_only_does_not_match() {
        let hits = query(sample(), &["armybreak".into()], ListFilter::All).unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn id_substring_matches_ids_that_contain_the_word() {
        let hits = query(sample(), &["durability".into()], ListFilter::All).unwrap();
        assert_eq!(
            ids(&hits),
            vec![
                "durability_draft",
                "durability_daily_restore",
                "durability_deprecated",
            ]
        );
    }

    #[test]
    fn id_substring_matches_even_when_body_omits_the_word() {
        let hits = query(sample(), &["title".into()], ListFilter::Active).unwrap();
        assert_eq!(ids(&hits), vec!["title_only"]);
    }

    #[test]
    fn exact_id_matches_even_when_body_omits_the_word() {
        let hits = query(sample(), &["title_only".into()], ListFilter::Active).unwrap();
        assert_eq!(ids(&hits), vec!["title_only"]);
    }

    #[test]
    fn id_match_is_case_folded_and_trimmed() {
        let hits = query(sample(), &["  TITLE_ONLY  ".into()], ListFilter::Active).unwrap();
        assert_eq!(ids(&hits), vec!["title_only"]);
    }

    #[test]
    fn or_any_keyword() {
        let hits = query(
            sample(),
            &["不存在".into(), "按日".into()],
            ListFilter::Active,
        )
        .unwrap();
        assert_eq!(ids(&hits), vec!["durability_daily_restore"]);
    }

    #[test]
    fn case_insensitive_english() {
        let atoms = vec![atom(
            "en",
            Status::Active,
            "t",
            &[],
            "Durability restores daily.",
        )];
        let hits = query(atoms, &["DURABILITY".into()], ListFilter::Active).unwrap();
        assert_eq!(ids(&hits), vec!["en"]);
    }

    #[test]
    fn empty_keyword_fails() {
        let err = query(sample(), &["".into()], ListFilter::Active).unwrap_err();
        assert_eq!(
            err,
            Error::validation(0, Some("keyword".into()), "keyword must be non-empty")
        );
    }

    #[test]
    fn whitespace_keyword_fails_at_index() {
        let err = query(sample(), &["耐久".into(), "  ".into()], ListFilter::Active).unwrap_err();
        assert_eq!(
            err,
            Error::validation(1, Some("keyword".into()), "keyword must be non-empty")
        );
    }

    #[test]
    fn empty_keywords_fail() {
        let err = query(sample(), &[], ListFilter::Active).unwrap_err();
        assert_eq!(
            err,
            Error::validation(0, Some("keyword".into()), "keyword must be non-empty")
        );
    }

    #[test]
    fn preserves_input_order() {
        let hits = query(sample(), &["正文".into(), "恢复".into()], ListFilter::All).unwrap();
        assert_eq!(
            ids(&hits),
            vec![
                "durability_draft",
                "durability_daily_restore",
                "durability_deprecated",
                "title_only",
            ]
        );
    }

    #[test]
    fn trims_keyword_before_match() {
        let hits = query(sample(), &["  按日  ".into()], ListFilter::Active).unwrap();
        assert_eq!(ids(&hits), vec!["durability_daily_restore"]);
    }
}

use crate::model::{Atom, Score};

/// Only `freshness.score` changes. Identity, status, body, and other freshness keys stay.
pub fn apply_score(mut atom: Atom, score: Score) -> Atom {
    atom.freshness.score = Some(score);
    atom
}

pub fn score_unchanged(before: &Atom, after: &Atom) -> bool {
    before.freshness.score == after.freshness.score
}

#[cfg(test)]
mod tests {
    use super::{apply_score, score_unchanged};
    use crate::model::{Atom, Freshness, Score, Status};

    fn atom() -> Atom {
        Atom {
            id: "durability_daily_restore".into(),
            status: Status::Active,
            title: "t".into(),
            tags: vec!["x".into()],
            freshness: Freshness {
                last_verified: Some("2026-09-01 13:05:00".into()),
                impl_path: Some("gamesvr/DurabilityManager.java".into()),
                score: Some(Score::one()),
            },
            body: "body".into(),
        }
    }

    #[test]
    fn writes_only_score() {
        let before = atom();
        let out = apply_score(before.clone(), Score::new(0.6));
        assert_eq!(out.id, before.id);
        assert_eq!(out.status, before.status);
        assert_eq!(out.title, before.title);
        assert_eq!(out.tags, before.tags);
        assert_eq!(out.body, before.body);
        assert_eq!(out.freshness.last_verified, before.freshness.last_verified);
        assert_eq!(out.freshness.impl_path, before.freshness.impl_path);
        assert_eq!(out.freshness.score, Some(Score::new(0.6)));
        assert!(!score_unchanged(&before, &out));
        assert!(score_unchanged(
            &out,
            &apply_score(out.clone(), Score::new(0.6))
        ));
    }
}

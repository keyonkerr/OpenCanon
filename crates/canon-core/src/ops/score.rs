use crate::model::{Atom, Score};

/// Only `freshness.score` changes. Identity, status, body, and other freshness keys stay.
///
/// Write-back never raises: no existing score → `computed`; otherwise `min(old, computed)`.
pub fn apply_score(mut atom: Atom, computed: Score) -> Atom {
    let next = match atom.freshness.score {
        None => computed,
        Some(old) => min_score(old, computed),
    };
    atom.freshness.score = Some(next);
    atom
}

pub fn score_unchanged(before: &Atom, after: &Atom) -> bool {
    before.freshness.score == after.freshness.score
}

fn min_score(old: Score, computed: Score) -> Score {
    if computed.get() < old.get() {
        computed
    } else {
        old
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_score, score_unchanged};
    use crate::model::{Atom, Freshness, Score, Status};

    fn atom_with_score(score: Option<Score>) -> Atom {
        Atom {
            id: "durability_daily_restore".into(),
            status: Status::Active,
            title: "t".into(),
            tags: vec!["x".into()],
            freshness: Freshness {
                last_verified: Some("2026-09-01 13:05:00".into()),
                impl_path: "gamesvr/DurabilityManager.java".into(),
                score,
            },
            body: "body".into(),
        }
    }

    #[test]
    fn writes_only_score() {
        let before = atom_with_score(Some(Score::one()));
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

    #[test]
    fn missing_score_takes_computed() {
        let before = atom_with_score(None);
        let out = apply_score(before, Score::new(0.6));
        assert_eq!(out.freshness.score, Some(Score::new(0.6)));
    }

    #[test]
    fn zero_is_not_raised_to_floor() {
        let before = atom_with_score(Some(Score::new(0.0)));
        let out = apply_score(before, Score::new(0.6));
        assert_eq!(out.freshness.score, Some(Score::new(0.0)));
    }

    #[test]
    fn floor_is_not_raised_to_one() {
        let before = atom_with_score(Some(Score::new(0.6)));
        let out = apply_score(before, Score::one());
        assert_eq!(out.freshness.score, Some(Score::new(0.6)));
    }
}

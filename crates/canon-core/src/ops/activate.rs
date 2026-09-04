use crate::lifecycle;
use crate::model::{Atom, Score, Status, Timestamp};
use crate::Error;

/// Draft → Active, stamping `last-verified` and `score = 1`. Keeps `impl-path`.
pub fn activate(mut atom: Atom, now: Timestamp) -> Result<Atom, Error> {
    if !lifecycle::can_transition(atom.status, Status::Active) {
        return Err(Error::InvalidTransition {
            id: atom.id,
            from: atom.status,
            to: Status::Active,
        });
    }
    atom.status = Status::Active;
    atom.freshness.last_verified = Some(now.verified_stamp());
    atom.freshness.score = Some(Score::one());
    Ok(atom)
}

#[cfg(test)]
mod tests {
    use super::activate;
    use crate::model::{Atom, Freshness, Score, Status, Timestamp};
    use crate::Error;

    fn draft() -> Atom {
        Atom {
            id: "durability_daily_restore".into(),
            status: Status::Draft,
            title: "禁军突围装备耐久恢复机制".into(),
            tags: vec!["armybreak".into()],
            freshness: Freshness {
                impl_path: "gamesvr/DurabilityManager.java".into(),
                score: Some(Score::new(0.2)),
                ..Freshness::default()
            },
            body: "正文：只描述一个事实。".into(),
        }
    }

    #[test]
    fn stamps_verified_and_score_keeps_impl_path_and_identity() {
        let now = Timestamp::from_ymd_hms(2026, 9, 1, 12, 15, 0);
        let out = activate(draft(), now).unwrap();
        assert_eq!(out.status, Status::Active);
        assert_eq!(out.id, draft().id);
        assert_eq!(out.title, draft().title);
        assert_eq!(out.tags, draft().tags);
        assert_eq!(out.body, draft().body);
        assert_eq!(
            out.freshness.last_verified.as_deref(),
            Some("2026-09-01 12:15:00")
        );
        assert_eq!(out.freshness.impl_path, draft().freshness.impl_path);
        assert_eq!(out.freshness.score, Some(Score::one()));
    }

    #[test]
    fn active_cannot_activate_again() {
        let mut atom = draft();
        atom.status = Status::Active;
        let err = activate(atom, Timestamp::from_ymd_hms(2026, 9, 1, 12, 15, 0)).unwrap_err();
        match err {
            Error::InvalidTransition { from, to, .. } => {
                assert_eq!(from, Status::Active);
                assert_eq!(to, Status::Active);
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}

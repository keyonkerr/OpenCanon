mod combine;
mod gate;
mod weighted;

use crate::model::{Atom, Score, Timestamp};

pub use combine::{combine, Factor, FactorKind};

/// Implementation facts injected by the caller. Core does not read the disk.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImplSnapshot {
    pub exists: bool,
    pub changed_at: Option<Timestamp>,
    pub text: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Evaluation {
    pub score: Score,
    pub factors: Vec<Factor>,
}

pub fn has_impl_path(atom: &Atom) -> bool {
    !atom.freshness.impl_path.is_empty()
}

/// `None` when the atom has no `impl-path` (skip). Otherwise score all registered factors.
pub fn evaluate(atom: &Atom, snapshots: &[ImplSnapshot]) -> Option<Evaluation> {
    if !has_impl_path(atom) {
        return None;
    }
    let mut factors = gate::factors(atom, snapshots);
    factors.extend(weighted::factors(atom, snapshots));
    let score = combine::combine(&factors);
    Some(Evaluation { score, factors })
}

#[cfg(test)]
mod tests {
    use super::{evaluate, ImplSnapshot};
    use crate::model::{Atom, Freshness, ImplPaths, Status, Timestamp};

    fn atom(body: &str, paths: &[&str], verified: Option<&str>) -> Atom {
        Atom {
            id: "durability_daily_restore".into(),
            status: Status::Active,
            title: "t".into(),
            tags: vec![],
            freshness: Freshness {
                last_verified: verified.map(str::to_string),
                impl_path: ImplPaths::new(paths.iter().copied()),
                score: None,
            },
            body: body.into(),
        }
    }

    fn snap(exists: bool, changed: Option<Timestamp>, text: Option<&str>) -> ImplSnapshot {
        ImplSnapshot {
            exists,
            changed_at: changed,
            text: text.map(str::to_string),
        }
    }

    #[test]
    fn skip_without_impl_path() {
        let a = atom("正文", &[], Some("2026-09-01 13:05:00"));
        assert!(evaluate(&a, &[snap(true, None, Some("x"))]).is_none());
        let blank = atom("正文", &["  "], Some("2026-09-01 13:05:00"));
        assert!(evaluate(&blank, &[snap(true, None, Some("x"))]).is_none());
    }

    #[test]
    fn path_ok_impl_still_names_present() {
        let verified = Timestamp::from_ymd_hms(2026, 9, 1, 13, 5, 0);
        let a = atom(
            "耐久按日 restoreDurability。",
            &["gamesvr/DurabilityManager.java"],
            Some("2026-09-01 13:05:00"),
        );
        let out = evaluate(
            &a,
            &[snap(true, Some(verified), Some("void restoreDurability() {}"))],
        )
        .unwrap();
        assert_eq!(out.score.get(), 1.0);
    }

    #[test]
    fn impl_newer_than_verified() {
        let a = atom(
            "纯中文。",
            &["gamesvr/DurabilityManager.java"],
            Some("2026-09-01 13:05:00"),
        );
        let later = Timestamp::from_ymd_hms(2026, 9, 2, 0, 0, 0);
        let out = evaluate(&a, &[snap(true, Some(later), Some(""))]).unwrap();
        assert_eq!(out.score.get(), 0.60);
    }

    #[test]
    fn never_verified() {
        let a = atom("纯中文。", &["gamesvr/DurabilityManager.java"], None);
        let out = evaluate(&a, &[snap(true, None, Some(""))]).unwrap();
        assert_eq!(out.score.get(), 0.60);
    }

    #[test]
    fn missing_code_name_is_zero() {
        let a = atom(
            "see FooBar and max_hp",
            &["gamesvr/DurabilityManager.java"],
            Some("2026-09-01 13:05:00"),
        );
        let out = evaluate(&a, &[snap(true, None, Some("max_hp = 1"))]).unwrap();
        assert_eq!(out.score.get(), 0.0);
    }

    #[test]
    fn path_missing_is_zero() {
        let a = atom(
            "纯中文。",
            &["gamesvr/DurabilityManager.java"],
            Some("2026-09-01 13:05:00"),
        );
        let out = evaluate(&a, &[snap(false, None, None)]).unwrap();
        assert_eq!(out.score.get(), 0.0);
    }

    #[test]
    fn union_of_files_covers_body_tokens() {
        let a = atom(
            "FooBar max_hp",
            &["a.rs", "b.rs"],
            Some("2026-09-01 13:05:00"),
        );
        let verified = Timestamp::from_ymd_hms(2026, 9, 1, 13, 5, 0);
        let out = evaluate(
            &a,
            &[
                snap(true, Some(verified), Some("FooBar")),
                snap(true, Some(verified), Some("max_hp = 1")),
            ],
        )
        .unwrap();
        assert_eq!(out.score.get(), 1.0);
    }

    #[test]
    fn any_missing_file_is_zero() {
        let a = atom("纯中文。", &["a.rs", "b.rs"], Some("2026-09-01 13:05:00"));
        let out = evaluate(
            &a,
            &[snap(true, None, Some("")), snap(false, None, None)],
        )
        .unwrap();
        assert_eq!(out.score.get(), 0.0);
    }

    #[test]
    fn any_file_newer_than_verified_is_floor() {
        let a = atom("纯中文。", &["a.rs", "b.rs"], Some("2026-09-01 13:05:00"));
        let verified = Timestamp::from_ymd_hms(2026, 9, 1, 13, 5, 0);
        let later = Timestamp::from_ymd_hms(2026, 9, 2, 0, 0, 0);
        let out = evaluate(
            &a,
            &[
                snap(true, Some(verified), Some("")),
                snap(true, Some(later), Some("")),
            ],
        )
        .unwrap();
        assert_eq!(out.score.get(), 0.60);
    }
}

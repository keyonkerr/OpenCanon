use crate::model::{Atom, Timestamp};

use super::super::ImplSnapshot;

pub const ID: &str = "impl-current";

pub fn value(atom: &Atom, snapshots: &[ImplSnapshot]) -> f64 {
    let Some(raw) = atom.freshness.last_verified.as_deref() else {
        return 0.0;
    };
    let Some(verified) = Timestamp::parse_verified_stamp(raw) else {
        return 0.0;
    };
    if snapshots
        .iter()
        .any(|s| s.changed_at.is_some_and(|changed| changed > verified))
    {
        0.0
    } else {
        1.0
    }
}

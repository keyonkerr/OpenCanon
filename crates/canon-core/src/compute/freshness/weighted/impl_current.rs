use crate::model::{Atom, Timestamp};

use super::super::ImplSnapshot;

pub const ID: &str = "impl-current";

pub fn value(atom: &Atom, snapshot: &ImplSnapshot) -> f64 {
    let Some(raw) = atom.freshness.last_verified.as_deref() else {
        return 0.0;
    };
    let Some(verified) = Timestamp::parse_verified_stamp(raw) else {
        return 0.0;
    };
    match snapshot.changed_at {
        Some(changed) if changed > verified => 0.0,
        _ => 1.0,
    }
}

use crate::model::{Atom, Timestamp};

use super::super::ImplSnapshot;

pub const ID: &str = "impl-current";

/// Linear decay from 1 to 0 over this many seconds when impl is not newer.
const HORIZON_SECS: u64 = 180 * 86_400;

pub fn value(atom: &Atom, snapshots: &[ImplSnapshot], now: Timestamp) -> f64 {
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
        return 0.0;
    }
    let age = now.saturating_secs_since(verified);
    if age >= HORIZON_SECS {
        0.0
    } else {
        1.0 - (age as f64) / (HORIZON_SECS as f64)
    }
}

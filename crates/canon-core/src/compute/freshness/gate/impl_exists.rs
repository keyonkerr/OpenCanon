use super::super::ImplSnapshot;

pub const ID: &str = "impl-exists";

pub fn value(snapshots: &[ImplSnapshot]) -> f64 {
    if !snapshots.is_empty() && snapshots.iter().all(|s| s.exists) {
        1.0
    } else {
        0.0
    }
}

mod impl_exists;

use super::combine::{Factor, FactorKind};
use super::ImplSnapshot;

pub fn factors(snapshots: &[ImplSnapshot]) -> Vec<Factor> {
    vec![Factor {
        id: impl_exists::ID,
        kind: FactorKind::Gate,
        value: impl_exists::value(snapshots),
        weight: None,
    }]
}

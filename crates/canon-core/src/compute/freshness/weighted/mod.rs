mod impl_current;

use crate::model::Atom;

use super::combine::{Factor, FactorKind};
use super::ImplSnapshot;

/// Sole place weighted `id → weight` is assigned. Sum must be ≤ 1.
const WEIGHTS: &[(&str, f64)] = &[(impl_current::ID, 0.40)];

pub fn factors(atom: &Atom, snapshot: &ImplSnapshot) -> Vec<Factor> {
    vec![Factor {
        id: impl_current::ID,
        kind: FactorKind::Weighted,
        value: impl_current::value(atom, snapshot),
        weight: Some(weight(impl_current::ID)),
    }]
}

fn weight(id: &str) -> f64 {
    WEIGHTS
        .iter()
        .find(|(key, _)| *key == id)
        .map(|(_, w)| *w)
        .expect("weighted factor missing from WEIGHTS")
}

#[cfg(test)]
mod tests {
    use super::{impl_current, WEIGHTS};

    #[test]
    fn weights_match_registered_ids_and_sum_at_most_one() {
        let table: Vec<&str> = WEIGHTS.iter().map(|(id, _)| *id).collect();
        assert_eq!(table, vec![impl_current::ID]);
        let sum: f64 = WEIGHTS.iter().map(|(_, w)| *w).sum();
        assert!(sum <= 1.0, "W={sum}");
        assert!(WEIGHTS.iter().all(|(_, w)| *w >= 0.0));
    }
}

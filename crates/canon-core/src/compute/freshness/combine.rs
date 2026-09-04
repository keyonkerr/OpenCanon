use crate::model::Score;

/// How a factor participates in the total. Combine matches on this, never on `id`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactorKind {
    Gate,
    Weighted,
    Multiplier,
    Observe,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Factor {
    pub id: &'static str,
    pub kind: FactorKind,
    pub value: f64,
    pub weight: Option<f64>,
}

impl Factor {
    pub fn gate(id: &'static str, value: f64) -> Self {
        Self {
            id,
            kind: FactorKind::Gate,
            value,
            weight: None,
        }
    }

    pub fn weighted(id: &'static str, value: f64, weight: f64) -> Self {
        Self {
            id,
            kind: FactorKind::Weighted,
            value,
            weight: Some(weight),
        }
    }
}

/// Sum of registered weighted weights. Tests lock this against the table.
pub fn weighted_sum(factors: &[Factor]) -> f64 {
    factors
        .iter()
        .filter(|f| f.kind == FactorKind::Weighted)
        .map(|f| f.weight.unwrap_or(0.0))
        .sum()
}

pub fn combine(factors: &[Factor]) -> Score {
    if factors
        .iter()
        .any(|f| f.kind == FactorKind::Gate && f.value == 0.0)
    {
        return Score::new(0.0);
    }

    let w: f64 = weighted_sum(factors);
    let mut score = 1.0 - w;
    for f in factors {
        if f.kind == FactorKind::Weighted {
            score += f.weight.unwrap_or(0.0) * f.value;
        }
    }
    for f in factors {
        if f.kind == FactorKind::Multiplier {
            score *= f.value;
        }
    }
    Score::new(round2(score.clamp(0.0, 1.0)))
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::{combine, Factor, FactorKind};

    #[test]
    fn any_gate_zero_is_total_zero() {
        let factors = vec![
            Factor::gate("impl-exists", 0.0),
            Factor::gate("body-in-impl", 1.0),
            Factor::weighted("impl-current", 1.0, 0.40),
        ];
        assert_eq!(combine(&factors).get(), 0.0);
    }

    #[test]
    fn gates_pass_current_one_is_one() {
        let factors = vec![
            Factor::gate("impl-exists", 1.0),
            Factor::gate("body-in-impl", 1.0),
            Factor::weighted("impl-current", 1.0, 0.40),
        ];
        assert_eq!(combine(&factors).get(), 1.0);
    }

    #[test]
    fn gates_pass_current_zero_is_floor() {
        let factors = vec![
            Factor::gate("impl-exists", 1.0),
            Factor::gate("body-in-impl", 1.0),
            Factor::weighted("impl-current", 0.0, 0.40),
        ];
        assert_eq!(combine(&factors).get(), 0.60);
    }

    #[test]
    fn observe_does_not_change_score() {
        let mut factors = vec![
            Factor::gate("impl-exists", 1.0),
            Factor::weighted("impl-current", 1.0, 0.40),
        ];
        let base = combine(&factors);
        factors.push(Factor {
            id: "note",
            kind: FactorKind::Observe,
            value: 0.0,
            weight: None,
        });
        assert_eq!(combine(&factors), base);
    }
}

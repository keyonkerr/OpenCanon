mod body_in_impl;
mod impl_exists;

use crate::model::Atom;

use super::combine::{Factor, FactorKind};
use super::ImplSnapshot;

pub fn factors(atom: &Atom, snapshot: &ImplSnapshot) -> Vec<Factor> {
    vec![
        Factor {
            id: impl_exists::ID,
            kind: FactorKind::Gate,
            value: impl_exists::value(snapshot),
            weight: None,
        },
        Factor {
            id: body_in_impl::ID,
            kind: FactorKind::Gate,
            value: body_in_impl::value(atom, snapshot),
            weight: None,
        },
    ]
}

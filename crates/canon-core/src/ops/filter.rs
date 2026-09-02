use crate::model::{Atom, Status};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListFilter {
    /// Default: true source only.
    Active,
    Status(Status),
    All,
}

pub fn filter_atoms(atoms: Vec<Atom>, filter: ListFilter) -> Vec<Atom> {
    match filter {
        ListFilter::All => atoms,
        ListFilter::Active | ListFilter::Status(Status::Active) => atoms
            .into_iter()
            .filter(|a| a.status == Status::Active)
            .collect(),
        ListFilter::Status(status) => atoms.into_iter().filter(|a| a.status == status).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::{filter_atoms, ListFilter};
    use crate::model::{Atom, Freshness, Status};

    fn atom(id: &str, status: Status) -> Atom {
        Atom {
            id: id.into(),
            status,
            title: id.into(),
            tags: vec![],
            freshness: Freshness::default(),
            body: "b".into(),
        }
    }

    fn sample() -> Vec<Atom> {
        vec![
            atom("d", Status::Draft),
            atom("a", Status::Active),
            atom("x", Status::Deprecated),
        ]
    }

    #[test]
    fn default_and_status_active_are_true_source_only() {
        let ids = |f| {
            filter_atoms(sample(), f)
                .into_iter()
                .map(|a| a.id)
                .collect::<Vec<_>>()
        };
        assert_eq!(ids(ListFilter::Active), vec!["a".to_string()]);
        assert_eq!(
            ids(ListFilter::Status(Status::Active)),
            vec!["a".to_string()]
        );
        assert_eq!(
            ids(ListFilter::Status(Status::Draft)),
            vec!["d".to_string()]
        );
        assert_eq!(
            ids(ListFilter::All),
            vec!["d".to_string(), "a".to_string(), "x".to_string()]
        );
    }
}

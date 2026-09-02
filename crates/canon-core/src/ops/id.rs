use std::collections::{HashMap, HashSet};

use crate::model::Status;
use crate::Error;

pub enum Occupied {
    Disk(Status),
}

pub fn assign_ids<'a>(
    slugs: impl IntoIterator<Item = &'a str>,
    occupied: &HashMap<String, Occupied>,
) -> Result<Vec<String>, Error> {
    let mut reserved: HashSet<String> = HashSet::new();
    let mut conflicts = Vec::new();
    let mut ids = Vec::new();

    for (index, slug) in slugs.into_iter().enumerate() {
        if let Some(Occupied::Disk(status)) = occupied.get(slug) {
            conflicts.push(crate::error::SlugConflict {
                index,
                slug: slug.to_string(),
                status: Some(*status),
            });
            continue;
        }
        if !reserved.insert(slug.to_string()) {
            conflicts.push(crate::error::SlugConflict {
                index,
                slug: slug.to_string(),
                status: None,
            });
            continue;
        }
        ids.push(slug.to_string());
    }

    if !conflicts.is_empty() {
        return Err(Error::SlugConflict { conflicts });
    }
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{assign_ids, Occupied};
    use crate::model::Status;
    use crate::Error;

    #[test]
    fn ids_are_the_slugs_in_order() {
        let ids = assign_ids(
            ["durability_daily_restore", "durability_cap_from_table"],
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(
            ids,
            vec![
                "durability_daily_restore".to_string(),
                "durability_cap_from_table".to_string(),
            ]
        );
    }

    #[test]
    fn disk_and_in_batch_conflicts_are_all_reported() {
        let mut occupied = HashMap::new();
        occupied.insert(
            "durability_daily_restore".into(),
            Occupied::Disk(Status::Draft),
        );
        let err = assign_ids(
            ["durability_daily_restore", "fresh_slug", "fresh_slug"],
            &occupied,
        )
        .unwrap_err();
        match err {
            Error::SlugConflict { conflicts } => {
                assert_eq!(conflicts.len(), 2);
                assert_eq!(conflicts[0].index, 0);
                assert_eq!(conflicts[0].status, Some(Status::Draft));
                assert_eq!(conflicts[1].index, 2);
                assert_eq!(conflicts[1].slug, "fresh_slug");
                assert_eq!(conflicts[1].status, None);
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}

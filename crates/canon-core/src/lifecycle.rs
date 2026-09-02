use crate::model::Status;

/// Legal transitions: Draft → Active, Active → Deprecated.
pub fn can_transition(from: Status, to: Status) -> bool {
    matches!(
        (from, to),
        (Status::Draft, Status::Active) | (Status::Active, Status::Deprecated)
    )
}

#[cfg(test)]
mod tests {
    use super::can_transition;
    use crate::model::Status;

    const ALL: [Status; 3] = [Status::Draft, Status::Active, Status::Deprecated];

    #[test]
    fn only_draft_to_active_and_active_to_deprecated_are_legal() {
        for from in ALL {
            for to in ALL {
                let allowed = can_transition(from, to);
                let expected = matches!(
                    (from, to),
                    (Status::Draft, Status::Active) | (Status::Active, Status::Deprecated)
                );
                assert_eq!(allowed, expected, "{from} -> {to}");
            }
        }
    }
}

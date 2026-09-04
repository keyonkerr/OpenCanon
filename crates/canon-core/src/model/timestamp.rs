use std::cmp::Ordering;

/// Local wall-clock instant injected by the caller. Core never reads the clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl Timestamp {
    pub fn from_ymd_hms(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> Self {
        Self {
            year,
            month: month as u8,
            day: day as u8,
            hour: hour as u8,
            minute: minute as u8,
            second: second as u8,
        }
    }

    /// Compact stamp `yyyyMMddHHmmss` from the same instant as [`Self::verified_stamp`].
    pub fn id_stamp(self) -> String {
        format!(
            "{:04}{:02}{:02}{:02}{:02}{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }

    /// `YYYY-MM-DD HH:MM:SS` used by `freshness.last-verified`.
    pub fn verified_stamp(self) -> String {
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }

    /// Parse a `verified_stamp`. Trimmed; otherwise must be exact.
    pub fn parse_verified_stamp(raw: &str) -> Option<Self> {
        let t = raw.trim();
        let bytes = t.as_bytes();
        if bytes.len() != 19 {
            return None;
        }
        if bytes[4] != b'-'
            || bytes[7] != b'-'
            || bytes[10] != b' '
            || bytes[13] != b':'
            || bytes[16] != b':'
        {
            return None;
        }
        let year: i32 = t[0..4].parse().ok()?;
        let month: u32 = t[5..7].parse().ok()?;
        let day: u32 = t[8..10].parse().ok()?;
        let hour: u32 = t[11..13].parse().ok()?;
        let minute: u32 = t[14..16].parse().ok()?;
        let second: u32 = t[17..19].parse().ok()?;
        if !(1..=12).contains(&month)
            || !(1..=31).contains(&day)
            || hour > 23
            || minute > 59
            || second > 59
        {
            return None;
        }
        Some(Self::from_ymd_hms(year, month, day, hour, minute, second))
    }
}

impl PartialOrd for Timestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Timestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
        )
            .cmp(&(
                other.year,
                other.month,
                other.day,
                other.hour,
                other.minute,
                other.second,
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::Timestamp;

    #[test]
    fn formats_id_and_verified_stamps_from_the_same_instant() {
        let ts = Timestamp::from_ymd_hms(2026, 9, 1, 13, 5, 0);
        assert_eq!(ts.id_stamp(), "20260901130500");
        assert_eq!(ts.verified_stamp(), "2026-09-01 13:05:00");
        assert_eq!(
            Timestamp::parse_verified_stamp("2026-09-01 13:05:00"),
            Some(ts)
        );
        assert_eq!(
            Timestamp::parse_verified_stamp("  2026-09-01 13:05:00  "),
            Some(ts)
        );
        assert!(Timestamp::parse_verified_stamp("2026-09-01").is_none());
        assert!(ts < Timestamp::from_ymd_hms(2026, 9, 1, 13, 5, 1));
    }
}

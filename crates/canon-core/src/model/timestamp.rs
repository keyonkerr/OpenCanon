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
}

#[cfg(test)]
mod tests {
    use super::Timestamp;

    #[test]
    fn formats_id_and_verified_stamps_from_the_same_instant() {
        let ts = Timestamp::from_ymd_hms(2026, 9, 1, 13, 5, 0);
        assert_eq!(ts.id_stamp(), "20260901130500");
        assert_eq!(ts.verified_stamp(), "2026-09-01 13:05:00");
    }
}

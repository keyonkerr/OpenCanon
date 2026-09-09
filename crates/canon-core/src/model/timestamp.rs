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

    /// Seconds `self` is after `earlier`. Zero when `self` is not later.
    pub fn saturating_secs_since(self, earlier: Self) -> u64 {
        let later = self.to_epoch_secs();
        let early = earlier.to_epoch_secs();
        later.saturating_sub(early).max(0) as u64
    }

    pub fn saturating_add_days(self, days: i64) -> Self {
        Self::from_epoch_secs(
            self.to_epoch_secs()
                .saturating_add(days.saturating_mul(86_400)),
        )
    }

    fn to_epoch_secs(self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
            .saturating_mul(86_400)
            .saturating_add(i64::from(self.hour) * 3_600)
            .saturating_add(i64::from(self.minute) * 60)
            .saturating_add(i64::from(self.second))
    }

    fn from_epoch_secs(secs: i64) -> Self {
        let days = secs.div_euclid(86_400);
        let rem = secs.rem_euclid(86_400) as u32;
        let (year, month, day) = civil_from_days(days);
        let hour = rem / 3_600;
        let minute = (rem % 3_600) / 60;
        let second = rem % 60;
        Self::from_ymd_hms(year, u32::from(month), u32::from(day), hour, minute, second)
    }
}

/// Unix days from a civil date. Howard Hinnant, public domain.
fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let mut y = year;
    let m = i32::from(month);
    y -= i32::from(m <= 2);
    let era = if y >= 0 { y } else { y - 399 }.div_euclid(400);
    let yoe = (y - era * 400) as u32;
    let mp = (m + if m > 2 { -3 } else { 9 }) as u32;
    let doy = (153 * mp + 2) / 5 + u32::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    i64::from(era) * 146_097 + i64::from(doe) - 719_468
}

fn civil_from_days(mut z: i64) -> (i32, u8, u8) {
    z += 719_468;
    let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i32 + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = y + i32::from(month <= 2);
    (year, month as u8, d as u8)
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

    #[test]
    fn unix_epoch_is_zero_and_roundtrips() {
        let epoch = Timestamp::from_ymd_hms(1970, 1, 1, 0, 0, 0);
        assert_eq!(epoch.to_epoch_secs(), 0);
        assert_eq!(Timestamp::from_epoch_secs(0), epoch);
        let ts = Timestamp::from_ymd_hms(2026, 9, 1, 13, 5, 0);
        assert_eq!(Timestamp::from_epoch_secs(ts.to_epoch_secs()), ts);
    }

    #[test]
    fn saturating_secs_since_is_zero_when_not_later() {
        let a = Timestamp::from_ymd_hms(2026, 9, 1, 13, 5, 0);
        let b = Timestamp::from_ymd_hms(2026, 9, 1, 13, 5, 1);
        assert_eq!(a.saturating_secs_since(b), 0);
        assert_eq!(a.saturating_secs_since(a), 0);
        assert_eq!(b.saturating_secs_since(a), 1);
        assert_eq!(
            a.saturating_add_days(90).saturating_secs_since(a),
            90 * 86_400
        );
        assert_eq!(
            a.saturating_add_days(180).saturating_secs_since(a),
            180 * 86_400
        );
    }
}

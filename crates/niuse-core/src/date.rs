//! Civil dates without a date-library dependency.
//!
//! Memory ages and journal headings only need `YYYY-MM-DD`, so a small,
//! deterministic conversion from days since the Unix epoch is enough.

use chrono::{Datelike, Local, TimeZone};

/// A calendar date, stored as days since 1970-01-01.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    days: i64,
}

impl Date {
    pub fn from_days(days: i64) -> Self {
        Self { days }
    }

    pub fn as_days(self) -> i64 {
        self.days
    }

    /// Today in the machine's local timezone. Memory and journal dates follow
    /// the user's calendar; event timestamps remain UTC epoch milliseconds.
    pub fn today() -> Self {
        let now = Local::now().date_naive();
        Self::from_days(days_from_civil(now.year() as i64, now.month(), now.day()))
    }

    /// Local midnight at the start of this calendar day, as epoch milliseconds.
    /// Used by `stats --days` so a one-day window means the local day, not a
    /// rolling 24 hours.
    pub fn start_millis_local(self) -> u64 {
        let (year, month, day) = civil_from_days(self.days);
        let Some(date) = chrono::NaiveDate::from_ymd_opt(year as i32, month, day) else {
            return 0;
        };
        // Some zones skip 00:00 on a DST transition. Take the first valid
        // instant of the day instead of falling back to zero (all time).
        for hour in 0..=3 {
            let Some(naive) = date.and_hms_opt(hour, 0, 0) else {
                continue;
            };
            let local = Local.from_local_datetime(&naive);
            if let Some(datetime) = local.earliest().or_else(|| local.latest()) {
                return datetime.timestamp_millis().max(0) as u64;
            }
        }
        0
    }

    /// Local calendar day for an epoch-millisecond timestamp. Used by metrics
    /// trends so their labels agree with journal and memory dates.
    pub fn from_millis_local(millis: u64) -> Self {
        let local = Local
            .timestamp_millis_opt(millis as i64)
            .single()
            .unwrap_or_else(|| Local.timestamp_opt(0, 0).unwrap());
        Self::from_days(days_from_civil(
            local.year() as i64,
            local.month(),
            local.day(),
        ))
    }

    pub fn plus_days(self, days: i64) -> Self {
        Self {
            days: self.days + days,
        }
    }

    pub fn age_days(self, today: Self) -> i64 {
        today.days - self.days
    }

    /// Parse a strict `YYYY-MM-DD` date. Out-of-range values are rejected.
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();
        let mut parts = input.split('-');
        let year: i64 = parts.next()?.parse().ok()?;
        let month: u32 = parts.next()?.parse().ok()?;
        let day: u32 = parts.next()?.parse().ok()?;
        if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
            return None;
        }
        let date = Self::from_days(days_from_civil(year, month, day));
        (date.format() == input).then_some(date)
    }

    pub fn format(self) -> String {
        let (year, month, day) = civil_from_days(self.days);
        format!("{year:04}-{month:02}-{day:02}")
    }
}

impl serde::Serialize for Date {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.format())
    }
}

/// Howard Hinnant's `civil_from_days`, with the epoch shifted to 1970-01-01.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = (z - era * 146_097) as u64;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era as i64 + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_prime + 2) / 5 + 1) as u32;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = (year - era * 400) as u64;
    let month_prime = if month > 2 { month - 3 } else { month + 9 } as u64;
    let day_of_year = (153 * month_prime + 2) / 5 + day as u64 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era as i64 - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_day_start_round_trips() {
        let today = Date::today();
        assert_eq!(Date::from_millis_local(today.start_millis_local()), today);
    }

    #[test]
    fn millis_map_to_the_local_calendar_day() {
        let now = Local
            .timestamp_millis_opt(1_700_000_000_000)
            .single()
            .unwrap();
        assert_eq!(
            Date::from_millis_local(1_700_000_000_000).format(),
            format!("{:04}-{:02}-{:02}", now.year(), now.month(), now.day())
        );
    }

    #[test]
    fn today_follows_the_local_calendar() {
        let now = Local::now().date_naive();
        assert_eq!(
            Date::today().format(),
            format!("{:04}-{:02}-{:02}", now.year(), now.month(), now.day())
        );
    }

    #[test]
    fn epoch_round_trips() {
        assert_eq!(Date::from_days(0).format(), "1970-01-01");
        assert_eq!(Date::parse("1970-01-01").unwrap().as_days(), 0);
    }

    #[test]
    fn leap_day_round_trips() {
        let leap = Date::parse("2024-02-29").expect("leap day parses");
        assert_eq!(leap.format(), "2024-02-29");
        assert_eq!(Date::from_days(leap.as_days()), leap);
    }

    #[test]
    fn known_date_round_trips() {
        let date = Date::parse("2026-10-21").expect("date parses");
        assert_eq!(date.format(), "2026-10-21");
        assert_eq!(date.plus_days(1).format(), "2026-10-22");
        assert_eq!(date.plus_days(11).format(), "2026-11-01");
    }

    #[test]
    fn rejects_invalid_dates() {
        for input in [
            "2026-13-01",
            "2026-02-30",
            "2026-1-1",
            "2026-01",
            "not-a-date",
        ] {
            assert!(Date::parse(input).is_none(), "{input} should not parse");
        }
    }

    #[test]
    fn age_is_signed() {
        let today = Date::parse("2026-10-21").unwrap();
        let old = Date::parse("2026-09-21").unwrap();
        assert_eq!(old.age_days(today), 30);
        assert_eq!(today.age_days(old), -30);
    }
}

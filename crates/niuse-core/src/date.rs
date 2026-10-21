//! Civil dates without a date-library dependency.
//!
//! Memory ages and journal headings only need `YYYY-MM-DD`, so a small,
//! deterministic conversion from days since the Unix epoch is enough.

use std::time::{SystemTime, UNIX_EPOCH};

const SECONDS_PER_DAY: i64 = 86_400;

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

    /// Today in the machine's local timezone is not tracked; the day boundary
    /// follows UTC. A personal journal one day off near midnight is worse than
    /// carrying a timezone database, and the value is visibly a date.
    pub fn today() -> Self {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or(0);
        Self::from_days(seconds.div_euclid(SECONDS_PER_DAY))
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

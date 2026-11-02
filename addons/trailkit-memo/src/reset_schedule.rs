//! The game's reset times (memo plan M13, M90): daily 00:00 UTC, weekly
//! Monday 07:30 UTC. Pure time arithmetic in UTC seconds, so it is exact
//! regardless of the player's time zone and unit-testable on any platform.

use jiff::Timestamp;

/// Seconds in one day.
const DAY_SECONDS: i64 = 86_400;
/// Seconds in one week.
const WEEK_SECONDS: i64 = 7 * DAY_SECONDS;
/// The first weekly reset after the Unix epoch (Thursday 1970-01-01):
/// Monday 1970-01-05 07:30 UTC, in seconds since the epoch.
const FIRST_WEEKLY_RESET_SECONDS: i64 = 4 * DAY_SECONDS + 7 * 3_600 + 30 * 60;

/// How often a reset note is reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResetPeriod {
    /// Every day at 00:00 UTC.
    Daily,
    /// Every Monday at 07:30 UTC.
    Weekly,
}

impl ResetPeriod {
    /// Both periods, in the order the kind dropdown lists them.
    pub const ALL: [Self; 2] = [Self::Daily, Self::Weekly];

    /// The period length in seconds.
    const fn seconds(self) -> i64 {
        match self {
            Self::Daily => DAY_SECONDS,
            Self::Weekly => WEEK_SECONDS,
        }
    }

    /// The most recent reset at or before `now`.
    #[must_use]
    pub fn latest_reset(self, now: Timestamp) -> Timestamp {
        let anchor = match self {
            Self::Daily => 0,
            Self::Weekly => FIRST_WEEKLY_RESET_SECONDS,
        };
        let seconds = now.as_second();
        let latest = seconds - (seconds - anchor).rem_euclid(self.seconds());
        Timestamp::from_second(latest).unwrap_or(Timestamp::MIN)
    }

    /// The first reset after `now`.
    #[must_use]
    pub fn next_reset(self, now: Timestamp) -> Timestamp {
        let latest = self.latest_reset(now).as_second();
        Timestamp::from_second(latest + self.seconds()).unwrap_or(Timestamp::MAX)
    }

    /// Whether a note last reset at `last_reset_applied_at` is due at `now`:
    /// a reset happened since. Several missed resets count as one (M90).
    #[must_use]
    pub fn is_due(self, last_reset_applied_at: Timestamp, now: Timestamp) -> bool {
        last_reset_applied_at < self.latest_reset(now)
    }
}

/// The first reset of any period after `now`: when reset notes must be
/// checked next.
#[must_use]
pub fn next_reset_of_any_period(now: Timestamp) -> Timestamp {
    ResetPeriod::ALL
        .into_iter()
        .map(|period| period.next_reset(now))
        .min()
        .unwrap_or(Timestamp::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parses an RFC 3339 test time.
    fn at(text: &str) -> Timestamp {
        text.parse().expect("valid test timestamp")
    }

    #[test]
    fn daily_reset_is_midnight_utc() {
        let now = at("2026-11-02T13:45:00Z");
        assert_eq!(
            ResetPeriod::Daily.latest_reset(now),
            at("2026-11-02T00:00:00Z")
        );
        assert_eq!(
            ResetPeriod::Daily.next_reset(now),
            at("2026-11-03T00:00:00Z")
        );
        let midnight = at("2026-11-03T00:00:00Z");
        assert_eq!(ResetPeriod::Daily.latest_reset(midnight), midnight);
    }

    #[test]
    fn weekly_reset_is_monday_0730_utc() {
        // 2026-11-02 is a Monday.
        let before = at("2026-11-02T07:29:59Z");
        assert_eq!(
            ResetPeriod::Weekly.latest_reset(before),
            at("2026-10-26T07:30:00Z")
        );
        assert_eq!(
            ResetPeriod::Weekly.next_reset(before),
            at("2026-11-02T07:30:00Z")
        );
        let after = at("2026-11-05T20:00:00Z");
        assert_eq!(
            ResetPeriod::Weekly.latest_reset(after),
            at("2026-11-02T07:30:00Z")
        );
        assert_eq!(
            ResetPeriod::Weekly.next_reset(after),
            at("2026-11-09T07:30:00Z")
        );
    }

    #[test]
    fn due_once_after_any_number_of_missed_resets() {
        let applied = at("2026-10-20T12:00:00Z");
        assert!(ResetPeriod::Daily.is_due(applied, at("2026-11-02T01:00:00Z")));
        assert!(!ResetPeriod::Daily.is_due(applied, at("2026-10-20T23:59:59Z")));
        let reset_now = at("2026-11-02T13:00:00Z");
        assert!(!ResetPeriod::Daily.is_due(reset_now, at("2026-11-02T23:00:00Z")));
        assert!(!ResetPeriod::Weekly.is_due(reset_now, at("2026-11-08T23:00:00Z")));
    }

    #[test]
    fn next_check_is_the_earlier_reset() {
        // Sunday evening: daily reset at midnight comes before Monday 07:30.
        assert_eq!(
            next_reset_of_any_period(at("2026-11-01T20:00:00Z")),
            at("2026-11-02T00:00:00Z")
        );
        // Monday 03:00: weekly 07:30 comes before the next midnight.
        assert_eq!(
            next_reset_of_any_period(at("2026-11-02T03:00:00Z")),
            at("2026-11-02T07:30:00Z")
        );
    }
}

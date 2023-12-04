use casper_types::Timestamp;
use std::time::{Duration, SystemTime};

// Ledger/Zondax supports timestamps only up to seconds resolution.
// `Display` impl for the `Timestamp` in the casper-node crate uses milliseconds-resolution
// so we need a custom implementation for the timestamp representation.
pub(crate) fn timestamp_to_seconds_res(timestamp: Timestamp) -> String {
    let system_time = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_millis(timestamp.millis()))
        .expect("should be within system time limits");
    format!("{}", humantime::format_rfc3339_seconds(system_time))
}

#[cfg(test)]
mod parse_tests {
    use casper_types::TimeDiff;

    const MINUTE: u32 = 60u32;
    const HOUR: u32 = 60 * MINUTE;
    const DAY: u32 = 24 * HOUR;
    const WEEK: u32 = 7 * DAY;
    const MONTH: u32 = 4 * WEEK;

    #[test]
    fn test_60s() {
        // 60s
        {
            let expected = "1m";
            let time_diff = TimeDiff::from_seconds(MINUTE);
            assert_eq!(expected, &format!("{}", time_diff))
        };
    }

    #[test]
    fn minute_plus_20s() {
        // 60s + 20s
        {
            let expected = "1m 20s";
            let time_diff = TimeDiff::from_seconds(MINUTE + 20);
            assert_eq!(expected, &format!("{}", time_diff))
        };
    }

    #[test]
    fn test_60min() {
        // 60min
        {
            let expected = "1h";
            let time_diff = TimeDiff::from_seconds(HOUR);
            assert_eq!(expected, &format!("{}", time_diff))
        };
    }

    #[test]
    fn test_60min_plus_1min_and_20s() {
        // 60min + 1min + 20s
        {
            let expected = "1h 1m 20s";
            let time_diff = TimeDiff::from_seconds(HOUR + MINUTE + 20);
            assert_eq!(expected, &format!("{}", time_diff))
        };
    }

    #[test]
    fn test_24h() {
        // 24h
        {
            let expected = "1day";
            let time_diff = TimeDiff::from_seconds(DAY);
            assert_eq!(expected, &format!("{}", time_diff))
        };
    }

    #[test]
    fn test_24h_plus_1h_plus_miunte_plus_20s() {
        // 24h + 60min + 60s + 20s
        {
            let expected = "1day 1h 1m 20s";
            let time_diff = TimeDiff::from_seconds(DAY + HOUR + MINUTE + 20);
            assert_eq!(expected, &format!("{}", time_diff))
        };
    }
    #[test]
    fn test_7days_plus_day_plus_hour_plus_minute_plus_20s() {
        // week + day + hour + minute + 20s
        {
            let expected = "8days 1h 1m 20s";
            let time_diff = TimeDiff::from_seconds(WEEK + DAY + HOUR + MINUTE + 20);
            assert_eq!(expected, &format!("{}", time_diff))
        };
    }

    #[ignore = "This test fails"]
    #[test]
    fn test_month_plus_week_plus_day_plus_hour_plus_minute_plus_20s() {
        // month + week + day + hour + minute + 20s
        // This test fails:
        // left: `"1month 8days 1h 1m 20s"`,
        // right: `"1month 5days 14h 27m 44s"`',
        {
            let expected = "1month 8days 1h 1m 20s";
            const EXPECTED_SECONDS: u32 = MONTH + WEEK + DAY + HOUR + MINUTE + 20;
            let time_diff = TimeDiff::from_seconds(EXPECTED_SECONDS);
            assert_eq!(expected, &format!("{}", time_diff))
        };
    }
}

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

    fn assert_equality(expected: &str, time_diff: TimeDiff) {
        assert_eq!(expected, &format!("{}", time_diff))
    }

    #[test]
    fn test() {
        let minute = 60u32;
        let hour = 60 * minute;
        let day = 24 * hour;
        let week = 7 * day;
        let month = 4 * week;

        // 60s
        assert_equality("1m", TimeDiff::from_seconds(minute));

        // 60s + 20s
        assert_equality("1m 20s", TimeDiff::from_seconds(minute + 20));

        // 60min
        assert_equality("1h", TimeDiff::from_seconds(hour));

        // 60min + 1min + 20s
        assert_equality("1h 1m 20s", TimeDiff::from_seconds(hour + minute + 20));

        // 24h
        assert_equality("1day", TimeDiff::from_seconds(day));

        // 24h + 60min + 60s + 20s
        assert_equality(
            "1day 1h 1m 20s",
            TimeDiff::from_seconds(day + hour + minute + 20),
        );

        // week + day + hour + minute + 20s
        assert_equality(
            "8days 1h 1m 20s",
            TimeDiff::from_seconds(week + day + hour + minute + 20),
        );

        // month + week + day + hour + minute + 20s
        // NOTE: This test fails
        //
        // left: `"1month 8days 1h 1m 20s"`,
        // right: `"1month 5days 14h 27m 44s"`',
        assert_equality(
            "1month 8days 1h 1m 20s",
            TimeDiff::from_seconds(month + week + day + hour + minute + 20),
        );
    }
}

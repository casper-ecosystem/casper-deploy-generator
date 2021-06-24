use casper_node::types::Timestamp;
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

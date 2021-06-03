use std::time::{Duration, SystemTime};

use casper_node::types::Timestamp;
use casper_types::{CLValue, Key, PublicKey};
use itertools::Itertools;

/// Turn JSON representation into a string.
fn serde_value_to_str(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(value) => format!("{}", value),
        serde_json::Value::Number(num) => format!("{}", num),
        serde_json::Value::String(string) => drop_key_type_prefix(string.clone()),
        serde_json::Value::Array(arr) => {
            format!("[{}]", arr.iter().map(serde_value_to_str).join(", "))
        }
        serde_json::Value::Object(map) => map.values().map(serde_value_to_str).join(":"),
    }
}

/// Drop type prefix (if we know how).
fn drop_key_type_prefix(cl_in: String) -> String {
    let parsed_key = Key::from_formatted_str(&cl_in);
    match parsed_key {
        Ok(key) => {
            let prefix = match key {
                Key::Account(_) => "account-hash-",
                Key::Hash(_) => "hash-",
                Key::Transfer(_) => "transfer-",
                Key::DeployInfo(_) => "deploy-",
                Key::EraInfo(_) => "era-",
                Key::Balance(_) => "balance-",
                Key::Bid(_) => "bid-",
                Key::Withdraw(_) => "withdraw-",
                Key::URef(_) => {
                    // format: uref-XXXX-YYY
                    return cl_in
                        .chars()
                        .skip("uref-".len())
                        .take_while(|c| *c != '-')
                        .collect();
                }
            };
            cl_in.chars().skip(prefix.len()).collect()
        }
        Err(_) => {
            // No idea how to handle that. Return raw.
            cl_in
        }
    }
}

/// Extracts the `parsed` field from the `CLValue`
/// (which is a pair of type identifier and raw bytes).
/// It should be human-readable.
pub(crate) fn cl_value_to_string(cl_in: &CLValue) -> String {
    match serde_json::to_value(&cl_in) {
        Ok(value) => {
            let parsed = value.get("parsed").unwrap();
            let value_str = serde_value_to_str(parsed);
            value_str
        }
        Err(err) => {
            eprintln!("error when parsing CLValue to CLValueJson#Object, {}", err);
            panic!("{:?}", err)
        }
    }
}

// Ledger/Zondax supports timestamps only up to seconds resolution.
// `Display` impl for the `Timestamp` in the casper-node crate uses milliseconds-resolution
// so we need a custom implementation for the timestamp representation.
pub(crate) fn timestamp_to_seconds_res(timestamp: Timestamp) -> String {
    let system_time = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_millis(timestamp.millis()))
        .expect("should be within system time limits");
    format!("{}", humantime::format_rfc3339_seconds(system_time))
}

// `PublicKey`'s `String` representation includes a `PublicKey::<variant>` prefix.
// This method drops that prefix (and the closing ')') from the `String` representation for the Ledger.
pub(crate) fn drop_public_key_prefix(key: &PublicKey) -> String {
    let variant = match key {
        PublicKey::System => todo!(),
        PublicKey::Ed25519(_) => "Ed25519",
        PublicKey::Secp256k1(_) => "Secp256k1",
    };
    let prefix = format!("PublicKey::{}(", variant);
    let str = format!("{:?}", key);
    str.chars()
        .skip(prefix.len())
        .take_while(|c| *c != ')')
        .collect()
}

use casper_types::{CLValue, Key, PublicKey, ED25519_TAG, SECP256K1_TAG};
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
                Key::Dictionary(_) => "dictionary-",
                Key::SystemContractRegistry => "system-contract-registry-",
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
            serde_value_to_str(parsed)
        }
        Err(err) => {
            eprintln!("error when parsing CLValue to CLValueJson#Object, {}", err);
            panic!("{:?}", err)
        }
    }
}

// `PublicKey`'s `String` representation includes a `PublicKey::<variant>` prefix.
// This method drops that prefix (and the closing ')') from the `String` representation for the Ledger.
pub(crate) fn parse_public_key(key: &PublicKey) -> String {
    let key_tag = match key {
        PublicKey::System => panic!("Did not expect system key"),
        PublicKey::Ed25519(_) => format!("0{}", ED25519_TAG),
        PublicKey::Secp256k1(_) => format!("0{}", SECP256K1_TAG),
    };

    let variant = match key {
        PublicKey::System => todo!(),
        PublicKey::Ed25519(_) => "Ed25519",
        PublicKey::Secp256k1(_) => "Secp256k1",
    };
    let prefix = format!("PublicKey::{}(", variant);
    let str = format!("{:?}", key);
    let key_str: String = str
        .chars()
        .skip(prefix.len())
        .take_while(|c| *c != ')')
        .collect();

    format!("{}{}", key_tag, key_str)
}

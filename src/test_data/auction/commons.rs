use crate::sample::Sample;
use crate::test_data::commons::{prepend_label, sample_executables, sample_module_bytes};
use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_types::{runtime_args, AsymmetricType, PublicKey, RuntimeArgs, U512};

/// Generates a valid auction transaction.
pub(crate) fn valid(entrypoint: &str, ra: Vec<RuntimeArgs>) -> Vec<Sample<ExecutableDeployItem>> {
    let mut output = vec![];

    for args in ra {
        for sample in sample_executables(entrypoint, args.clone(), None, true) {
            output.push(prepend_label(sample, entrypoint));
        }

        let mut ra: RuntimeArgs = args;
        ra.insert("auction", entrypoint).unwrap();
        output.push(prepend_label(sample_module_bytes(ra), entrypoint));
    }

    output
}

/// Constructs transactions that are invalid (un)delegate deploys
/// but are valid "generic" deploys - i.e. they will still be processed by a node
/// but will not be recognized as auction commands.
pub(crate) fn invalid_delegation(entry_point: &str) -> Vec<Sample<ExecutableDeployItem>> {
    let delegator: PublicKey = PublicKey::ed25519_from_bytes([1u8; 32]).unwrap();
    let validator: PublicKey = PublicKey::ed25519_from_bytes([3u8; 32]).unwrap();
    let amount = U512::from(100000000u32);

    let valid_args = runtime_args! {
        "delegator" => delegator.clone(),
        "validator" => validator.clone(),
        "amount" => amount,
    };

    let invalid_args = {
        let missing_required_amount = runtime_args! {
            "delegator" => delegator.clone(),
            "validator" => validator.clone(),
        };

        let missing_required_delegator = runtime_args! {
            "validator" => validator.clone(),
            "amount" => amount,
        };

        let missing_required_validator = runtime_args! {
            "delegator" => delegator.clone(),
            "amount" => amount
        };

        let invalid_amount_type = runtime_args! {
            "validator" => validator,
            "delegator" => delegator,
            "amount" => 100000u32
        };

        // We're setting the "validity bit" to `true`, otherwise such transaction would
        // be rejected by the Ledger Hardware and we don't want that. dApps could be written
        // in such a way that they use similar arguments.
        vec![
            Sample::new("missing_amount", missing_required_amount, true),
            Sample::new("missing_delegator", missing_required_delegator, true),
            Sample::new("missing_validator", missing_required_validator, true),
            Sample::new("invalid_type_amount", invalid_amount_type, true),
        ]
    };

    invalid_args
        .into_iter()
        .flat_map(|sample_ra| {
            let (label, ra, valid) = sample_ra.destructure();
            sample_executables(entry_point, ra, Some(label), valid)
        })
        .chain(sample_executables(
            "invalid",
            valid_args.clone(),
            Some("invalid_entrypoint".to_string()),
            true, // Even though entrypoint is invalid, it's possible that generic transaction (non-native auction) uses similar set of arguments but changes the entrypoint. In that case, transaction MUSTN'T be invalid b/c it will get rejected by the Ledger.
        ))
        .map(|sample| prepend_label(sample, entry_point))
        .collect()
}

use crate::sample::Sample;
use casper_execution_engine::core::engine_state::executable_deploy_item::ExecutableDeployItem::{
    StoredContractByHash, StoredContractByName, StoredVersionedContractByHash,
    StoredVersionedContractByName,
};
use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_types::{
    runtime_args, ContractHash, ContractPackageHash, ContractVersion, PublicKey, RuntimeArgs, U512,
};
use rand::Rng;

pub(crate) fn sample_executables<R: Rng>(
    _rng: &mut R,
    entry_point: &str,
    ra: RuntimeArgs,
    base_label: Option<String>,
) -> Vec<Sample<ExecutableDeployItem>> {
    let contract_hash = ContractHash::new([1u8; 32]);
    let contract_package_hash = ContractPackageHash::new([1u8; 32]);
    let contract_version: ContractVersion = 1;
    let contract_name = format!("{}-contract", entry_point);
    let deploy_items = vec![
        Sample::new(
            "type:by-hash",
            StoredContractByHash {
                hash: contract_hash.clone(),
                entry_point: entry_point.to_string(),
                args: ra.clone(),
            },
            true,
        ),
        Sample::new(
            "type:by-name",
            StoredContractByName {
                name: contract_name.to_string(),
                entry_point: entry_point.to_string(),
                args: ra.clone(),
            },
            true,
        ),
        Sample::new(
            "type:versioned-by-hash",
            StoredVersionedContractByHash {
                hash: contract_package_hash.clone(),
                version: Some(contract_version),
                entry_point: entry_point.to_string(),
                args: ra.clone(),
            },
            true,
        ),
        Sample::new(
            "type:versioned-by-name",
            StoredVersionedContractByName {
                name: contract_name.to_string(),
                version: Some(contract_version),
                entry_point: entry_point.to_string(),
                args: ra.clone(),
            },
            true,
        ),
    ];

    deploy_items
        .into_iter()
        .map(|mut sample| {
            if let Some(label) = &base_label {
                sample.add_label(label.clone());
            }
            sample
        })
        .collect()
}

pub(crate) fn invalid_delegation<R: Rng>(
    rng: &mut R,
    entry_point: &str,
) -> Vec<Sample<ExecutableDeployItem>> {
    let delegator: PublicKey = PublicKey::ed25519([1u8; 32]).unwrap();
    let validator: PublicKey = PublicKey::ed25519([3u8; 32]).unwrap();
    let amount = U512::from(100000000);

    let valid_args = runtime_args! {
        "delegator" => delegator,
        "validator" => validator,
        "amount" => amount,
    };

    let missing_required_amount = runtime_args! {
        "delegator" => delegator,
        "validator" => validator,
    };

    let missing_required_delegator = runtime_args! {
        "validator" => validator,
        "amount" => amount,
    };

    let missing_required_validator = runtime_args! {
        "delegator" => delegator,
        "amount" => amount
    };

    let invalid_amount_type = runtime_args! {
        "validator" => validator,
        "delegator" => delegator,
        "amount" => 100000u32
    };

    let invalid_args = vec![
        Sample::new("missing:amount", missing_required_amount, false),
        Sample::new("missing:delegator", missing_required_delegator, false),
        Sample::new("missing:validator", missing_required_validator, false),
        Sample::new("invalid_type:amount", invalid_amount_type, false),
    ];

    invalid_args
        .into_iter()
        .flat_map(|sample_ra| {
            let (label, ra, _valid) = sample_ra.destructure();
            let mut invalid_args_executables =
                sample_executables(rng, entry_point, ra, Some(label.clone()));
            invalid_args_executables.extend(sample_executables(
                rng,
                "invalid",
                valid_args.clone(),
                Some("invalid:entrypoint".to_string()),
            ));
            invalid_args_executables
                .into_iter()
                .map(|sample_invalid_executable| {
                    let (label, sample, _valid) = sample_invalid_executable.destructure();
                    let new_label = format!("{}-{}", entry_point, label);
                    Sample::new(new_label, sample, false)
                })
        })
        .collect()
}

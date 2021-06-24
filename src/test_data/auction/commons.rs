use crate::sample::Sample;
use casper_execution_engine::core::engine_state::executable_deploy_item::ExecutableDeployItem::{
    StoredContractByHash, StoredContractByName, StoredVersionedContractByHash,
    StoredVersionedContractByName,
};
use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_types::{ContractHash, ContractPackageHash, ContractVersion, RuntimeArgs};
use rand::Rng;

pub(crate) fn sample_executables<R: Rng>(
    _rng: &mut R,
    entry_point: &str,
    ra: RuntimeArgs,
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
}

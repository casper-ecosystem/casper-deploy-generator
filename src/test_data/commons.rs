use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_types::bytesrepr::Bytes;
use casper_types::{
    ContractHash, ContractPackageHash, ContractVersion, RuntimeArgs, UREF_ADDR_LENGTH,
};

use crate::sample::Sample;

// Using provided `entry_point` and arguments, returns a vector of samples
// for each of the existing `ExecutableDeployItem` variant.
pub(crate) fn sample_executables(
    entry_point: &str,
    ra: RuntimeArgs,
    base_label: Option<String>,
    valid: bool,
) -> Vec<Sample<ExecutableDeployItem>> {
    let contract_hash = ContractHash::new([1u8; 32]);
    let contract_package_hash = ContractPackageHash::new([1u8; 32]);
    let contract_version: ContractVersion = 1;
    let contract_name = format!("{}_contract", entry_point);
    let deploy_items = vec![
        Sample::new(
            "type_by_hash",
            ExecutableDeployItem::StoredContractByHash {
                hash: contract_hash,
                entry_point: entry_point.to_string(),
                args: ra.clone(),
            },
            valid,
        ),
        Sample::new(
            "type_by_name",
            ExecutableDeployItem::StoredContractByName {
                name: contract_name.to_string(),
                entry_point: entry_point.to_string(),
                args: ra.clone(),
            },
            valid,
        ),
        Sample::new(
            "type_versioned_by_hash",
            ExecutableDeployItem::StoredVersionedContractByHash {
                hash: contract_package_hash,
                version: Some(contract_version),
                entry_point: entry_point.to_string(),
                args: ra.clone(),
            },
            valid,
        ),
        Sample::new(
            "type_versioned_by_name",
            ExecutableDeployItem::StoredVersionedContractByName {
                name: contract_name,
                version: Some(contract_version),
                entry_point: entry_point.to_string(),
                args: ra,
            },
            valid,
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

// ModuleBytes action calls are too different from other deploy variants to be included in the same generic logic.
pub(crate) fn sample_module_bytes(ra: RuntimeArgs) -> Sample<ExecutableDeployItem> {
    Sample::new(
        "type_module_bytes",
        ExecutableDeployItem::ModuleBytes {
            module_bytes: Bytes::new(),
            args: ra,
        },
        true,
    )
}

// Prepends `entrypoint` to the current label of `sample`.
pub(crate) fn prepend_label(
    sample: Sample<ExecutableDeployItem>,
    entrypoint: &str,
) -> Sample<ExecutableDeployItem> {
    let (executable_label, executable, valid) = sample.destructure();
    let label = format!("{}__{}", entrypoint, executable_label);
    Sample::new(label, executable, valid)
}

pub(crate) const UREF_ADDR: [u8; UREF_ADDR_LENGTH] = [
    74, 207, 207, 108, 104, 76, 88, 202, 246, 179, 41, 110, 58, 151, 196, 160, 74, 250, 247, 123,
    184, 117, 202, 154, 64, 164, 93, 178, 84, 233, 74, 117,
];

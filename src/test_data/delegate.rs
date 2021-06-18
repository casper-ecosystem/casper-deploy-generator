//! Sample test vectors for delegation deploys.
//!
//! Method name (entrypoint):
//! `delegate`
//!
//! Arguments:
//! | name | type |
//! |---------|---------|
//! | `delegator` | `PublicKey` |
//! | `validator` | `PublicKey` |
//! | `amount` | `U512` |

use crate::sample::Sample;
use casper_execution_engine::core::engine_state::executable_deploy_item::ExecutableDeployItem::{
    StoredContractByName, StoredVersionedContractByHash, StoredVersionedContractByName,
};
use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_types::{
    ContractHash, ContractPackageHash, ContractVersion, PublicKey, RuntimeArgs, U512,
};
use rand::Rng;

const ENTRY_POINT_NAME: &str = "delegate";
const VERSION: Option<ContractVersion> = None;

#[derive(Clone, Copy, Debug)]
struct Delegate {
    delegator: PublicKey,
    validator: PublicKey,
    amount: U512,
}

impl Delegate {
    fn new(delegator: PublicKey, validator: PublicKey, amount: U512) -> Self {
        Delegate {
            delegator,
            validator,
            amount,
        }
    }
}

impl From<Delegate> for RuntimeArgs {
    fn from(d: Delegate) -> Self {
        let mut ra = RuntimeArgs::new();
        ra.insert("delegator", d.delegator).unwrap();
        ra.insert("validator", d.validator).unwrap();
        ra.insert("amount", d.amount).unwrap();
        ra
    }
}

fn sample_executables<R: Rng>(_rng: &mut R, ra: RuntimeArgs) -> Vec<Sample<ExecutableDeployItem>> {
    let contract_hash = ContractHash::new([1u8; 32]);
    let contract_package_hash = ContractPackageHash::new([1u8; 32]);
    let entry_point = ENTRY_POINT_NAME.to_string();
    let contract_version: ContractVersion = 1;
    let contract_name = "delegation-contract";
    let deploy_items = vec![
        Sample::new(
            "type:by-hash",
            ExecutableDeployItem::StoredContractByHash {
                hash: contract_hash.clone(),
                entry_point: entry_point.clone(),
                args: ra.clone(),
            },
            true,
        ),
        Sample::new(
            "type:by-name",
            StoredContractByName {
                name: contract_name.to_string(),
                entry_point: entry_point.clone(),
                args: ra.clone(),
            },
            true,
        ),
        Sample::new(
            "type:versioned-by-hash",
            StoredVersionedContractByHash {
                hash: contract_package_hash.clone(),
                version: Some(contract_version),
                entry_point: entry_point.clone(),
                args: ra.clone(),
            },
            true,
        ),
        Sample::new(
            "type:versioned-by-name",
            StoredVersionedContractByName {
                name: contract_name.to_string(),
                version: Some(contract_version),
                entry_point,
                args: ra.clone(),
            },
            true,
        ),
    ];

    deploy_items
}

fn sample_delegations<R: Rng>(_rng: &mut R) -> Vec<Delegate> {
    let amount_min = U512::from(0u8);
    let amount_mid = U512::from(100000000);
    let amount_max = U512::MAX;
    let amounts = vec![amount_min, amount_mid, amount_max];

    let delegator: PublicKey = PublicKey::ed25519([1u8; 32]).unwrap();
    let validator: PublicKey = PublicKey::ed25519([3u8; 32]).unwrap();

    amounts
        .into_iter()
        .map(|amount| Delegate::new(delegator, validator, amount))
        .collect()
}

pub(crate) fn valid<R: Rng>(rng: &mut R) -> Vec<Sample<ExecutableDeployItem>> {
    let mut output = vec![];

    for delegation in sample_delegations(rng) {
        for sample_executable in sample_executables(rng, delegation.into()) {
            let (executable_label, executable, _) = sample_executable.destructure();
            let label = format!("delegation-{}", executable_label.clone());
            let sample = Sample::new(label, executable, true);
            output.push(sample);
        }
    }

    output
}

//! Sample test vectors for undelegation deploys.
//!
//! Method name (entrypoint):
//! `undelegate`
//!
//! Arguments:
//! | name | type |
//! |---------|---------|
//! | `delegator` | `PublicKey` |
//! | `validator` | `PublicKey` |
//! | `amount` | `U512` |

use crate::sample::Sample;
use crate::test_data::auction::commons::sample_executables;
use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_types::{runtime_args, PublicKey, RuntimeArgs, U512};
use rand::Rng;

const ENTRY_POINT_NAME: &str = "undelegate";

#[derive(Clone, Copy, Debug)]
struct Undelegate {
    delegator: PublicKey,
    validator: PublicKey,
    amount: U512,
}

impl Undelegate {
    fn new(delegator: PublicKey, validator: PublicKey, amount: U512) -> Self {
        Undelegate {
            delegator,
            validator,
            amount,
        }
    }
}

impl From<Undelegate> for RuntimeArgs {
    fn from(d: Undelegate) -> Self {
        let mut ra = RuntimeArgs::new();
        ra.insert("delegator", d.delegator).unwrap();
        ra.insert("validator", d.validator).unwrap();
        ra.insert("amount", d.amount).unwrap();
        ra
    }
}

fn sample_undelegations<R: Rng>(_rng: &mut R) -> Vec<Undelegate> {
    let amount_min = U512::from(0u8);
    let amount_mid = U512::from(100000000);
    let amount_max = U512::MAX;
    let amounts = vec![amount_min, amount_mid, amount_max];

    let delegator: PublicKey = PublicKey::ed25519([1u8; 32]).unwrap();
    let validator: PublicKey = PublicKey::ed25519([3u8; 32]).unwrap();

    amounts
        .into_iter()
        .map(|amount| Undelegate::new(delegator, validator, amount))
        .collect()
}

pub(crate) fn valid<R: Rng>(rng: &mut R) -> Vec<Sample<ExecutableDeployItem>> {
    let mut output = vec![];

    for delegation in sample_undelegations(rng) {
        for sample_executable in sample_executables(rng, ENTRY_POINT_NAME, delegation.into(), None)
        {
            let (executable_label, executable, _) = sample_executable.destructure();
            let label = format!("undelegation-{}", executable_label.clone());
            let sample = Sample::new(label, executable, true);
            output.push(sample);
        }
    }

    output
}

pub(crate) fn invalid<R: Rng>(rng: &mut R) -> Vec<Sample<ExecutableDeployItem>> {
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
                sample_executables(rng, ENTRY_POINT_NAME, ra, Some(label.clone()));
            invalid_args_executables.extend(sample_executables(
                rng,
                "invalid_entrypoint",
                valid_args.clone(),
                Some(label.clone()),
            ));
            invalid_args_executables
                .into_iter()
                .map(|sample_invalid_executable| {
                    let (label, sample, _valid) = sample_invalid_executable.destructure();
                    let new_label = format!("undelegate-{}", label);
                    Sample::new(new_label, sample, false)
                })
        })
        .collect()
}

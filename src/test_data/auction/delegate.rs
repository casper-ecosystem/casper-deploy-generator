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
use crate::test_data::auction::commons::sample_executables;
use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_types::{runtime_args, PublicKey, RuntimeArgs, U512};
use rand::Rng;

const ENTRY_POINT_NAME: &str = "delegate";

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
        for sample_executable in sample_executables(rng, ENTRY_POINT_NAME, delegation.into(), None)
        {
            let (executable_label, executable, _) = sample_executable.destructure();
            let label = format!("delegation-{}", executable_label.clone());
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
                    let new_label = format!("delegate-{}", label);
                    Sample::new(new_label, sample, false)
                })
        })
        .collect()
}

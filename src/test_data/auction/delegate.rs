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
use casper_types::{PublicKey, RuntimeArgs, U512};
use rand::Rng;

use super::commons::invalid_delegation;

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
    invalid_delegation(rng, ENTRY_POINT_NAME)
}

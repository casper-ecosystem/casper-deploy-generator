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
use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_types::{AsymmetricType, PublicKey, RuntimeArgs, U512};

use super::commons::invalid_delegation;

const ENTRY_POINT_NAME: &str = "undelegate";

#[derive(Clone, Debug)]
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

fn sample_undelegations() -> Vec<Undelegate> {
    let amount_min = U512::from(0u8);
    let amount_mid = U512::from(100000000);
    let amount_max = U512::MAX;
    let amounts = vec![amount_min, amount_mid, amount_max];

    let delegator: PublicKey = PublicKey::ed25519_from_bytes([1u8; 32]).unwrap();
    let validator: PublicKey = PublicKey::ed25519_from_bytes([3u8; 32]).unwrap();

    amounts
        .into_iter()
        .map(|amount| Undelegate::new(delegator.clone(), validator.clone(), amount))
        .collect()
}

pub(crate) fn valid() -> Vec<Sample<ExecutableDeployItem>> {
    let delegate_rargs = sample_undelegations().into_iter().map(Into::into).collect();

    super::commons::valid(ENTRY_POINT_NAME, delegate_rargs)
}

pub(crate) fn invalid() -> Vec<Sample<ExecutableDeployItem>> {
    invalid_delegation(ENTRY_POINT_NAME)
}

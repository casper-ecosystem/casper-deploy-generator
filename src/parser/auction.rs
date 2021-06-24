use casper_execution_engine::core::engine_state::ExecutableDeployItem;

use crate::{
    ledger::{Element, TxnPhase},
    parser::{
        deploy::{deploy_type, identity, parse_amount},
        runtime_args::parse_optional_arg,
    },
};

use super::deploy::is_entrypoint;

fn parse_auction_item(method: &str, item: &ExecutableDeployItem) -> Vec<Element> {
    let mut elements = vec![Element::regular("Auction", method.to_string())];
    elements.extend(
        deploy_type(TxnPhase::Session, item)
            .into_iter()
            .map(|mut e| {
                // For now, we choose to not display deploy's details for delegation.
                e.as_expert();
                e
            }),
    );
    match item {
        ExecutableDeployItem::ModuleBytes { .. } | ExecutableDeployItem::Transfer { .. } => {
            panic!("unexpected type for {}", method)
        }
        ExecutableDeployItem::StoredContractByHash { args, .. }
        | ExecutableDeployItem::StoredContractByName { args, .. }
        | ExecutableDeployItem::StoredVersionedContractByHash { args, .. }
        | ExecutableDeployItem::StoredVersionedContractByName { args, .. } => {
            // Public key of the account we're delegating from.
            let delegator_pk = parse_optional_arg(args, "delegator", false, identity);
            elements.extend(delegator_pk.into_iter());
            // Public key of the validator we're delegating to.
            let validator_pk = parse_optional_arg(args, "validator", false, identity);
            elements.extend(validator_pk.into_iter());
            // Amount we're delegating.
            elements.extend(parse_amount(args).into_iter());
        }
    };
    elements
}

pub(crate) fn parse_delegation(item: &ExecutableDeployItem) -> Vec<Element> {
    parse_auction_item("delegate", item)
}

pub(crate) fn parse_undelegation(item: &ExecutableDeployItem) -> Vec<Element> {
    parse_auction_item("undelegate", item)
}

/// Returns `true` when the deploy's entry point is *literally* _delegate_
pub(crate) fn is_delegate(item: &ExecutableDeployItem) -> bool {
    is_entrypoint(item, "delegate")
}

/// Returns `true` when the deploy's entry point is *literally* _undelegate_
pub(crate) fn is_undelegate(item: &ExecutableDeployItem) -> bool {
    is_entrypoint(item, "undelegate")
}

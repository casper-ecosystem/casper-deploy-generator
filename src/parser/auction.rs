use casper_execution_engine::core::engine_state::ExecutableDeployItem;

use crate::{
    ledger::{Element, TxnPhase},
    parser::{
        deploy::{deploy_type, identity, parse_transfer_amount},
        runtime_args::parse_optional_arg,
    },
};

use super::deploy::is_entrypoint;

fn parse_auction_item(method: &str, item: &ExecutableDeployItem) -> Vec<Element> {
    let mut elements = vec![];
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
        ExecutableDeployItem::Transfer { .. } => {
            panic!("unexpected type for {}", method)
        }
        ExecutableDeployItem::StoredContractByHash { args, .. }
        | ExecutableDeployItem::StoredContractByName { args, .. }
        | ExecutableDeployItem::StoredVersionedContractByHash { args, .. }
        | ExecutableDeployItem::StoredVersionedContractByName { args, .. }
        | ExecutableDeployItem::ModuleBytes { args, .. } => {
            // Public key of the account we're delegating from.
            let delegator_pk = parse_optional_arg(args, "delegator", "delegator", false, identity);
            elements.extend(delegator_pk.into_iter());
            // Public key of the validator we're delegating to.
            let validator_pk = parse_optional_arg(args, "validator", "validator", false, identity);
            elements.extend(validator_pk.into_iter());
            // Amount we're delegating.
            elements.extend(parse_transfer_amount(args).into_iter());
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
    is_entrypoint(item, "delegate") || has_delegate_arg(item)
}

/// Returns `true` when the deploy's entry point is *literally* _undelegate_
pub(crate) fn is_undelegate(item: &ExecutableDeployItem) -> bool {
    is_entrypoint(item, "undelegate") || has_undelegate_arg(item)
}

fn get_auction_arg(item: &ExecutableDeployItem) -> Option<String> {
    match item {
        // ModuleBytes variant does not have an entry point, it defaults to `call()`,
        // so we expect a special named argument called `auction` when detecting auction contract calls.
        ExecutableDeployItem::ModuleBytes { args, .. } => args.get("auction").map(|cl_value| {
            cl_value
                .clone()
                .into_t::<String>()
                .expect("argument should be string")
        }),
        _ => None,
    }
}

fn has_delegate_arg(item: &ExecutableDeployItem) -> bool {
    get_auction_arg(item)
        .filter(|arg_value| arg_value == "delegate")
        .is_some()
}

fn has_undelegate_arg(item: &ExecutableDeployItem) -> bool {
    get_auction_arg(item)
        .filter(|arg_value| arg_value == "undelegate")
        .is_some()
}

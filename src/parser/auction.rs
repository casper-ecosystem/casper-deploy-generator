use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_types::RuntimeArgs;

use crate::{
    ledger::{Element, TxnPhase},
    parser::deploy::{deploy_type, parse_transfer_amount},
};

use super::deploy::{
    is_entrypoint, parse_delegator, parse_new_validator, parse_old_validator, parse_validator,
};

fn parse_auction_item<'a, F>(
    method: &str,
    item: &'a ExecutableDeployItem,
    args_parser: F,
) -> Vec<Element>
where
    F: Fn(&'a RuntimeArgs) -> Vec<Element>,
{
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
            elements.extend(args_parser(args));
        }
    };
    elements
}

pub(crate) fn parse_delegation(item: &ExecutableDeployItem) -> Vec<Element> {
    let arg_parser = |args| {
        let mut elements = vec![];
        // Public key of the account we're delegating from.
        elements.extend(parse_delegator(args).into_iter());
        // Public key of the validator we're delegating to.
        elements.extend(parse_validator(args).into_iter());
        // Amount we're delegating.
        elements.extend(parse_transfer_amount(args).into_iter());
        elements
    };
    parse_auction_item("delegate", item, arg_parser)
}

pub(crate) fn parse_undelegation(item: &ExecutableDeployItem) -> Vec<Element> {
    let arg_parser = |args| {
        let mut elements = vec![];
        // Public key of the account we're delegating from.
        elements.extend(parse_delegator(args).into_iter());
        // Public key of the validator we're delegating to.
        elements.extend(parse_validator(args).into_iter());
        // Amount we're delegating.
        elements.extend(parse_transfer_amount(args).into_iter());
        elements
    };
    parse_auction_item("undelegate", item, arg_parser)
}

pub(crate) fn parse_redelegation(item: &ExecutableDeployItem) -> Vec<Element> {
    let arg_parser = |args| {
        let mut elements = vec![];
        // Public key of the account we're delegating from.
        elements.extend(parse_delegator(args).into_iter());
        // Public key of the current validator we have been redelagating to so far.
        elements.extend(parse_old_validator(args).into_iter());
        // New validator we're redelegating to.
        elements.extend(parse_new_validator(args).into_iter());
        // Amount we're delegating.
        elements.extend(parse_transfer_amount(args).into_iter());
        elements
    };
    parse_auction_item("redelegate", item, arg_parser)
}

/// Returns `true` when the deploy's entry point is *literally* _delegate_
pub(crate) fn is_delegate(item: &ExecutableDeployItem) -> bool {
    is_entrypoint(item, "delegate") || has_delegate_arg(item)
}

/// Returns `true` when the deploy's entry point is *literally* _undelegate_
pub(crate) fn is_undelegate(item: &ExecutableDeployItem) -> bool {
    is_entrypoint(item, "undelegate") || has_undelegate_arg(item)
}

/// Returns `true` when the deploy's entry point is *literally* _undelegate_
pub(crate) fn is_redelegate(item: &ExecutableDeployItem) -> bool {
    is_entrypoint(item, "redelegate") || has_redelegate_arg(item)
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

fn has_redelegate_arg(item: &ExecutableDeployItem) -> bool {
    get_auction_arg(item)
        .filter(|arg_value| arg_value == "redelegate")
        .is_some()
}

use std::collections::BTreeMap;

use crate::{
    ledger::{Element, TxnPhase},
    parser::{runtime_args::parse_optional_arg, utils::timestamp_to_seconds_res},
    utils::parse_public_key,
};
use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_hashing::Digest;
use casper_node::types::{Deploy, DeployHeader};
use casper_types::{
    bytesrepr::Bytes,
    system::mint::{self, ARG_ID, ARG_SOURCE, ARG_TARGET, ARG_TO},
    CLValue, RuntimeArgs, U512,
};
use thousands::Separable;

use super::{
    auction::{
        is_delegate, is_redelegate, is_undelegate, parse_delegation, parse_redelegation,
        parse_undelegation,
    },
    runtime_args::{parse_runtime_args, parse_transfer_args},
};

pub(crate) fn parse_deploy_header(dh: &DeployHeader) -> Vec<Element> {
    let mut elements = vec![];
    elements.push(Element::regular("chain ID", dh.chain_name().to_string()));
    elements.push(Element::regular("account", parse_public_key(dh.account())));
    elements.push(Element::expert(
        "timestamp",
        timestamp_to_seconds_res(dh.timestamp()),
    ));
    elements.push(Element::expert("ttl", format!("{}", dh.ttl())));
    elements.push(Element::expert("gas price", format!("{}", dh.gas_price())));
    elements.push(Element::expert(
        "Deps #",
        format!("{:?}", dh.dependencies().len()),
    ));
    elements
}

pub(crate) fn parse_phase(item: &ExecutableDeployItem, phase: TxnPhase) -> Vec<Element> {
    if is_delegate(item) {
        parse_delegation(item)
    } else if is_undelegate(item) {
        parse_undelegation(item)
    } else if is_redelegate(item) {
        parse_redelegation(item)
    } else {
        let mut elements: Vec<Element> = deploy_type(phase, item);
        match item {
            ExecutableDeployItem::ModuleBytes { module_bytes, args } => {
                if is_system_payment(phase, module_bytes) {
                    // The only required argument for the system payment is `amount`.
                    elements.extend(parse_fee(args).into_iter());
                } else {
                    elements.extend(parse_amount(args));
                }
                let args_sans_amount = remove_amount_arg(args.clone());
                elements.extend(parse_runtime_args(&args_sans_amount));
            }
            ExecutableDeployItem::StoredContractByHash {
                entry_point, args, ..
            } => {
                elements.push(entrypoint(entry_point));
                elements.extend(parse_amount(args));
                let args_sans_amount = remove_amount_arg(args.clone());
                elements.extend(parse_runtime_args(&args_sans_amount));
            }
            ExecutableDeployItem::StoredContractByName {
                entry_point, args, ..
            } => {
                elements.push(entrypoint(entry_point));
                elements.extend(parse_amount(args));
                let args_sans_amount = remove_amount_arg(args.clone());
                elements.extend(parse_runtime_args(&args_sans_amount));
            }
            ExecutableDeployItem::StoredVersionedContractByHash {
                entry_point, args, ..
            } => {
                elements.push(entrypoint(entry_point));
                elements.extend(parse_amount(args));
                let args_sans_amount = remove_amount_arg(args.clone());
                elements.extend(parse_runtime_args(&args_sans_amount));
            }
            ExecutableDeployItem::StoredVersionedContractByName {
                entry_point, args, ..
            } => {
                elements.push(entrypoint(entry_point));
                elements.extend(parse_amount(args));
                let args_sans_amount = remove_amount_arg(args.clone());
                elements.extend(parse_runtime_args(&args_sans_amount));
            }
            ExecutableDeployItem::Transfer { args } => {
                elements.extend(parse_transfer_args(args));
                let args_sans_transfer = remove_transfer_args(args.clone());
                elements.extend(parse_runtime_args(&args_sans_transfer));
            }
        }
        elements
    }
}

/// Returns the main elements describing the deploy:
/// – is it a payment or session code,
/// – is it a raw contract bytes, call by name, by hash, versioned, etc.?
///
/// Does NOT parse the arguments or entry points.
pub(crate) fn deploy_type(phase: TxnPhase, item: &ExecutableDeployItem) -> Vec<Element> {
    // Session|Payment :
    let phase_label = format!("{}", phase);
    match item {
        ExecutableDeployItem::ModuleBytes { module_bytes, .. } => {
            if is_system_payment(phase, module_bytes) {
                // Payment: system
                // Do nothing. For the sake of familiarity with othe system we don't diplay this for native payments,
                // as this is equivalent to the built-in payment on Ethereum and alike.
                vec![]
            } else {
                let contract_hash = format!("{:?}", Digest::hash(module_bytes.as_slice()));
                vec![
                    // Session|Payment: contract
                    Element::regular(&phase_label, "contract".to_string()),
                    // Cntrct hash: <hash of contract bytes>
                    Element::regular("Cntrct hash", contract_hash),
                ]
            }
        }
        ExecutableDeployItem::StoredContractByHash { hash, .. } => {
            vec![
                // Session|Payment: by-hash
                Element::regular(&phase_label, "by-hash".to_string()),
                // Address: <contract address>
                Element::regular("address", format!("{}", hash)),
            ]
        }
        ExecutableDeployItem::StoredContractByName { name, .. } => {
            vec![
                // Session|Payment: by-name
                Element::regular(&phase_label, "by-name".to_string()),
                // Name: <name of the contract>
                Element::regular("name", name.clone()),
            ]
        }
        ExecutableDeployItem::StoredVersionedContractByHash { hash, version, .. } => {
            vec![
                // Session|Payment: by-hash-versioned
                Element::regular(&phase_label, "by-hash-versioned".to_string()),
                // Address: <contract address>
                Element::regular("address", hash.to_string()),
                // Version: <version>
                parse_version(version),
            ]
        }
        ExecutableDeployItem::StoredVersionedContractByName { name, version, .. } => {
            vec![
                // Session|Payment: by-name-versioned
                Element::regular(&phase_label, "by-name-versioned".to_string()),
                // Name: <name of the contract>
                Element::regular("name", name.to_string()),
                // Version: <version>
                parse_version(version),
            ]
        }
        ExecutableDeployItem::Transfer { .. } => {
            // Session|Payment: native transfer
            // Do not add yet another element to Ledger panes – it's already been taken care of in `parse_deploy`,
            // where we add `Type: *` pane.
            vec![]
        }
    }
}

fn parse_version(version: &Option<u32>) -> Element {
    let version = match version {
        None => "latest".to_string(),
        Some(version) => format!("{}", version),
    };
    Element::expert("version", version)
}

// Payment is a system type of payment when the `module_bytes` are empty.
fn is_system_payment(phase: TxnPhase, module_bytes: &Bytes) -> bool {
    phase.is_payment() && module_bytes.inner_bytes().is_empty()
}

fn remove_amount_arg(args: RuntimeArgs) -> RuntimeArgs {
    let mut tree: BTreeMap<String, CLValue> = args.into();
    tree.remove(mint::ARG_AMOUNT);
    tree.into()
}

/// Removes all arguments that are used in the Transfer.
fn remove_transfer_args(args: RuntimeArgs) -> RuntimeArgs {
    let mut tree: BTreeMap<String, CLValue> = args.into();
    tree.remove(ARG_TO);
    tree.remove(ARG_SOURCE);
    tree.remove(ARG_TARGET);
    tree.remove(mint::ARG_AMOUNT);
    tree.remove(ARG_ID);
    tree.into()
}

fn format_amount(motes: U512) -> String {
    format!("{} motes", motes.separate_with_spaces())
}

pub(crate) fn parse_fee(args: &RuntimeArgs) -> Option<Element> {
    parse_motes(args, "fee")
}

pub(crate) fn parse_amount(args: &RuntimeArgs) -> Option<Element> {
    parse_motes(args, "amount")
}

fn parse_motes(args: &RuntimeArgs, ledger_label: &str) -> Option<Element> {
    let f = |amount_str: String| {
        let motes_amount = U512::from_dec_str(&amount_str).unwrap();
        format_amount(motes_amount)
    };
    parse_optional_arg(args, mint::ARG_AMOUNT, ledger_label, false, f)
}

#[cfg(test)]
mod amount {
    use casper_types::U512;

    use crate::parser::deploy::format_amount;

    #[test]
    fn amount_space_separated() {
        let one: U512 = 1u8.into();
        let expected = "1 motes".to_string();
        assert_eq!(expected, format_amount(one));
        let thousand: U512 = 1_000u32.into();
        let expected = "1 000 motes".to_string();
        assert_eq!(expected, format_amount(thousand));
        let ten_thousand: U512 = 10_000u64.into();
        let expected = "10 000 motes".to_string();
        assert_eq!(expected, format_amount(ten_thousand));
        let ten_billion: U512 = U512::from(10000000000u64);
        let expected = "10 000 000 000 motes".to_string();
        assert_eq!(expected, format_amount(ten_billion));
    }
}

pub(crate) fn identity<T>(el: T) -> T {
    el
}

pub(crate) fn parse_approvals(d: &Deploy) -> Vec<Element> {
    let approvals_count = d.approvals().len();
    vec![Element::expert(
        "Approvals #",
        format!("{}", approvals_count),
    )]
}

fn entrypoint(entry_point: &str) -> Element {
    Element::expert("entry-point", entry_point.to_string())
}

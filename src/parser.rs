use std::collections::BTreeMap;

use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_node::{
    crypto::hash,
    types::{Deploy, DeployHeader},
};
use casper_types::{
    bytesrepr::Bytes,
    system::mint::{self, ARG_ID, ARG_SOURCE, ARG_TARGET, ARG_TO},
    CLValue, RuntimeArgs, U512,
};
use thousands::Separable;

use crate::{
    ledger::{Element, TxnPhase},
    utils::{cl_value_to_string, parse_public_key, timestamp_to_seconds_res},
};

/// Parses all contract arguments into a form:
/// arg-n-name: <name>
/// arg-n-val: <val>
/// where n is the ordinal number of the argument.
fn parse_runtime_args(ra: &RuntimeArgs) -> Vec<Element> {
    let mut elements: Vec<Element> = vec![];
    let named_args: BTreeMap<String, CLValue> = ra.clone().into();
    for (idx, (name, value)) in named_args.iter().enumerate() {
        let name_label = format!("arg-{}-name", idx);
        elements.push(Element::expert(&name_label, name.to_string()));
        let value_label = format!("arg-{}-val", idx);
        let value_str = cl_value_to_string(&value);
        elements.push(Element::expert(&value_label, value_str));
    }
    elements
}

fn parse_version(version: &Option<u32>) -> Element {
    let version = match version {
        None => "latest".to_string(),
        Some(version) => format!("{}", version),
    };
    Element::expert("version", format!("{}", version))
}

fn format_amount(motes: U512) -> String {
    format!("{} motes", motes.separate_with_spaces())
}

fn parse_amount(args: &RuntimeArgs) -> Option<Element> {
    let f = |amount_str: String| {
        let motes_amount = U512::from_dec_str(&amount_str).unwrap();
        format_amount(motes_amount)
    };
    parse_optional_arg(args, mint::ARG_AMOUNT, false, f)
}

#[cfg(test)]
mod amount {
    use casper_types::U512;

    use crate::parser::format_amount;

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

fn parse_optional_arg<F: Fn(String) -> String>(
    args: &RuntimeArgs,
    key: &str,
    expert: bool,
    f: F,
) -> Option<Element> {
    match args.get(key) {
        Some(cl_value) => {
            let value = f(cl_value_to_string(cl_value));
            let element = if expert {
                Element::expert(key, value)
            } else {
                Element::regular(key, value)
            };
            Some(element)
        }
        None => None,
    }
}

fn identity<T>(el: T) -> T {
    el
}

/// Required fields for transfer are:
/// * target
/// * amount
/// * ID
/// Optional fields:
/// * source
fn parse_transfer(args: &RuntimeArgs) -> Vec<Element> {
    let mut elements: Vec<Element> = parse_optional_arg(args, ARG_TO, false, identity)
        .into_iter()
        .collect();
    elements.extend(parse_optional_arg(args, ARG_SOURCE, true, identity).into_iter());
    elements.extend(parse_optional_arg(args, ARG_TARGET, false, identity));
    elements.extend(parse_amount(args));
    elements.extend(parse_optional_arg(args, ARG_ID, true, identity).into_iter());
    elements
}

// Payment is a system type of payment when the `module_bytes` are empty.
fn is_system_payment(phase: TxnPhase, module_bytes: &Bytes) -> bool {
    phase.is_payment() && module_bytes.inner_bytes().is_empty()
}

pub(crate) fn parse_deploy_header(dh: &DeployHeader) -> Vec<Element> {
    let mut elements = vec![];
    elements.push(Element::regular("chain ID", format!("{}", dh.chain_name())));
    elements.push(Element::regular("from", parse_public_key(dh.account())));
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
    let item_type;
    let phase_args = match item {
        ExecutableDeployItem::ModuleBytes { module_bytes, args } => {
            if is_system_payment(phase, module_bytes) {
                item_type = "system".to_string();
                // The only required argument for the system payment is `amount`.
                let mut elements: Vec<Element> = parse_amount(args).into_iter().collect();
                let args_sans_amount = remove_amount_arg(args.clone());
                elements.extend(parse_runtime_args(&args_sans_amount));
                elements
            } else {
                item_type = "contract".to_string();
                let bytes = format!("{:?}", hash::hash(module_bytes.as_slice()));
                let mut elements = vec![Element::regular("Cntrct hash", bytes)];
                elements.extend(parse_runtime_args(args));
                elements
            }
        }
        ExecutableDeployItem::StoredContractByHash {
            hash,
            entry_point,
            args,
        } => {
            item_type = "by-hash".to_string();
            let mut elements = vec![Element::regular("address", format!("{}", hash))];
            elements.push(entrypoint(entry_point));
            elements.extend(parse_runtime_args(args));
            elements
        }
        ExecutableDeployItem::StoredContractByName {
            name,
            entry_point,
            args,
        } => {
            item_type = "by-name".to_string();
            let mut elements = vec![Element::regular("name", format!("{}", name))];
            elements.push(entrypoint(entry_point));
            elements.extend(parse_runtime_args(args));
            elements
        }
        ExecutableDeployItem::StoredVersionedContractByHash {
            hash,
            version,
            entry_point,
            args,
        } => {
            item_type = "by-hash-versioned".to_string();
            let mut elements = vec![Element::regular("address", format!("{}", hash))];
            elements.push(entrypoint(entry_point));
            elements.push(parse_version(version));
            elements.extend(parse_runtime_args(args));
            elements
        }
        ExecutableDeployItem::StoredVersionedContractByName {
            name,
            version,
            entry_point,
            args,
        } => {
            item_type = "by-name-versioned".to_string();
            let mut elements = vec![Element::regular("name", format!("{}", name))];
            elements.push(entrypoint(entry_point));
            elements.push(parse_version(version));
            elements.extend(parse_runtime_args(args));
            elements
        }
        ExecutableDeployItem::Transfer { args } => {
            item_type = "native transfer".to_string();
            let mut elements = parse_transfer(args);
            let args_sans_transfer = remove_transfer_args(args.clone());
            elements.extend(parse_runtime_args(&&args_sans_transfer));
            elements
        }
    };
    let phase_label = format!("{}", phase);
    let mut elements = vec![Element::regular(&phase_label, item_type)];
    elements.extend(phase_args);
    elements
}

pub(crate) fn parse_approvals(d: &Deploy) -> Vec<Element> {
    let approvals_count = d.approvals().len();
    vec![Element::regular(
        "Approvals #",
        format!("{}", approvals_count),
    )]
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

fn entrypoint(entry_point: &str) -> Element {
    Element::expert("entry-point", format!("{}", entry_point))
}

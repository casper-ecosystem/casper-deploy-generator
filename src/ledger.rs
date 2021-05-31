use std::{
    collections::BTreeMap,
    fmt::Display,
    str::FromStr,
    time::{Duration, SystemTime},
};

use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_node::{
    crypto::hash,
    types::{Deploy, DeployHeader, Timestamp},
};
use casper_types::{
    bytesrepr::ToBytes,
    system::{
        mint::{ARG_ID, ARG_SOURCE, ARG_TARGET, ARG_TO},
        standard_payment::ARG_AMOUNT,
    },
    CLValue, Key, RuntimeArgs, U512,
};

use humantime;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

const LEDGER_VIEW_NAME_COUNT: usize = 11;
const LEDGER_VIEW_TOP_COUNT: usize = 17;
const LEDGER_VIEW_BOTTOM_COUNT: usize = 17;

struct Elements(Vec<Element>);

/// Turn JSON representation into a string.
fn serde_value_to_str(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(value) => format!("{}", value),
        serde_json::Value::Number(num) => format!("{}", num),
        serde_json::Value::String(string) => drop_key_type_prefix(string.clone()),
        serde_json::Value::Array(arr) => {
            format!("[{}]", arr.iter().map(serde_value_to_str).join(", "))
        }
        serde_json::Value::Object(map) => map.values().map(serde_value_to_str).join(":"),
    }
}

/// Drop type prefix (if we know how).
fn drop_key_type_prefix(cl_in: String) -> String {
    let parsed_key = Key::from_formatted_str(&cl_in);
    match parsed_key {
        Ok(key) => {
            let prefix = match key {
                Key::Account(_) => "account-hash-",
                Key::Hash(_) => "hash-",
                Key::Transfer(_) => "transfer-",
                Key::DeployInfo(_) => "deploy-",
                Key::EraInfo(_) => "era-",
                Key::Balance(_) => "balance-",
                Key::Bid(_) => "bid-",
                Key::Withdraw(_) => "withdraw-",
                Key::URef(_) => {
                    // format: uref-XXXX-YYY
                    return cl_in
                        .chars()
                        .skip("uref-".len())
                        .take_while(|c| *c != '-')
                        .collect();
                }
            };
            cl_in.chars().skip(prefix.len()).collect()
        }
        Err(_) => {
            // No idea how to handle that. Return raw.
            cl_in
        }
    }
}

/// Extracts the `parsed` field from the `CLValue`
/// (which is a pair of type identifier and raw bytes).
/// It should be human-readable.
fn cl_value_to_string(cl_in: &CLValue) -> String {
    match serde_json::to_value(&cl_in) {
        Ok(value) => {
            let parsed = value.get("parsed").unwrap();
            let value_str = serde_value_to_str(parsed);
            value_str
        }
        Err(err) => {
            eprintln!("error when parsing CLValue to CLValueJson#Object, {}", err);
            panic!("{:?}", err)
        }
    }
}

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

fn parse_amount(args: &RuntimeArgs) -> Element {
    let amount_str = cl_value_to_string(args.get(ARG_AMOUNT).unwrap());
    let motes_amount = U512::from_str(&amount_str).unwrap();
    Element::regular("amount", format!("{:.9} motes", motes_amount))
}

fn parse_arg(args: &RuntimeArgs, key: &str, expert: bool) -> Element {
    let value = cl_value_to_string(args.get(key).unwrap());
    if expert {
        Element::expert(key, value)
    } else {
        Element::regular(key, value)
    }
}

fn parse_optional_arg(args: &RuntimeArgs, key: &str, expert: bool) -> Option<Element> {
    match args.get(key) {
        Some(cl_value) => {
            let value = cl_value_to_string(cl_value);
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

fn entrypoint(entry_point: &str) -> Element {
    Element::expert("entry-point", format!("{}", entry_point))
}

fn remove_amount_arg(args: RuntimeArgs) -> RuntimeArgs {
    let mut tree: BTreeMap<String, CLValue> = args.into();
    tree.remove(ARG_AMOUNT);
    tree.into()
}

/// Required fields for transfer are:
/// * target
/// * amount
/// * ID
/// Optional fields:
/// * to
/// * source
fn parse_transfer(args: &RuntimeArgs) -> Vec<Element> {
    let mut elements: Vec<Element> = parse_optional_arg(args, ARG_TO, false)
        .into_iter()
        .collect();
    elements.extend(parse_optional_arg(args, ARG_SOURCE, true).into_iter());
    elements.push(parse_arg(args, ARG_TARGET, false));
    elements.push(parse_amount(args));
    elements.extend(parse_optional_arg(args, ARG_ID, true).into_iter());
    elements
}

/// Removes all arguments that are used in the Transfer.
fn remove_transfer_args(args: RuntimeArgs) -> RuntimeArgs {
    let mut tree: BTreeMap<String, CLValue> = args.into();
    tree.remove(ARG_TO);
    tree.remove(ARG_SOURCE);
    tree.remove(ARG_TARGET);
    tree.remove(ARG_AMOUNT);
    tree.remove(ARG_ID);
    tree.into()
}

fn parse_phase(item: &ExecutableDeployItem, phase: TxnPhase) -> Vec<Element> {
    let item_type;
    let phase_args = match item {
        ExecutableDeployItem::ModuleBytes { module_bytes, args } => {
            if phase.is_payment() && module_bytes.inner_bytes().is_empty() {
                item_type = "system".to_string();
                let mut elements = vec![parse_amount(args)];
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

fn parse_approvals(d: &Deploy) -> Vec<Element> {
    let approvals_count = d.approvals().len();
    vec![Element::regular(
        "Approvals #",
        format!("{}", approvals_count),
    )]
}

// Ledger/Zondax supports timestamps only up to seconds resolution.
// `Display` impl for the `Timestamp` in the casper-node crate uses milliseconds-resolution
// so we need a custom implementation for the timestamp representation.
fn timestamp_to_seconds_res(timestamp: Timestamp) -> String {
    let system_time = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_millis(timestamp.millis()))
        .expect("should be within system time limits");
    format!("{}", humantime::format_rfc3339_seconds(system_time))
}

#[derive(Clone, Copy)]
enum TxnPhase {
    Payment,
    Session,
}

impl TxnPhase {
    fn is_payment(&self) -> bool {
        matches!(self, TxnPhase::Payment)
    }
}

impl Display for TxnPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxnPhase::Payment => write!(f, "Payment"),
            TxnPhase::Session => write!(f, "Execution"),
        }
    }
}

fn parse_deploy_header(dh: &DeployHeader) -> Vec<Element> {
    let mut elements = vec![];
    elements.push(Element::regular("chain ID", format!("{}", dh.chain_name())));
    elements.push(Element::regular(
        "from",
        format!("{}", dh.account().to_account_hash()),
    ));
    elements.push(Element::expert(
        "timestamp",
        timestamp_to_seconds_res(dh.timestamp()),
    ));
    elements.push(Element::expert("ttl", format!("{}", dh.ttl())));
    elements.push(Element::expert("gas price", format!("{}", dh.gas_price())));
    elements.push(Element::expert(
        "txn deps",
        format!(
            "{:?}",
            dh.dependencies()
                .iter()
                .map(|dh| dh.inner())
                .collect::<Vec<_>>()
        ),
    ));
    elements
}

impl Into<Elements> for Deploy {
    fn into(self) -> Elements {
        let mut elements = vec![];
        let deploy_type = if self.session().is_transfer() {
            "Transfer"
        } else {
            "Execute Contract"
        };
        elements.push(Element::regular("Type", format!("{}", deploy_type)));
        elements.extend(parse_deploy_header(self.header()));
        elements.extend(parse_phase(self.payment(), TxnPhase::Payment));
        elements.extend(parse_phase(self.session(), TxnPhase::Session));
        elements.extend(parse_approvals(&self));
        Elements(elements)
    }
}

#[derive(Debug)]
struct Element {
    name: String,
    value: String,
    // Whether to display in expert mode only.
    expert: bool,
}

// Capitalizes the first character.
fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

impl Element {
    fn expert(name: &str, value: String) -> Element {
        Element {
            name: capitalize_first(name),
            value,
            expert: true,
        }
    }

    fn regular(name: &str, value: String) -> Self {
        Element {
            name: capitalize_first(name),
            value,
            expert: false,
        }
    }
}

struct Ledger(Vec<Element>);

impl Ledger {
    fn from_deploy(deploy: Deploy) -> Self {
        let elements: Elements = deploy.into();
        Ledger(elements.0)
    }
}

#[derive(Default, Clone)]
struct LedgerValue {
    top: String,
    bottom: String,
}

impl LedgerValue {
    // Adds a char to the ledger value.
    // Single value is limited by the number of chars that can be
    // printed on one ledger view: 34 char total in two lines.
    // Returns whether adding char was successful.
    fn add_char(&mut self, c: char) -> bool {
        if self.top.chars().count() < LEDGER_VIEW_TOP_COUNT {
            self.top = format!("{}{}", self.top, c);
            return true;
        }
        if self.bottom.chars().count() < LEDGER_VIEW_BOTTOM_COUNT {
            self.bottom = format!("{}{}", self.bottom, c);
            return true;
        }
        false
    }
}

impl std::fmt::Display for LedgerValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.top, self.bottom)
    }
}

// Single page view representation.
// Example:
// Hash [1/2]
// 01001010101…
// 10101010101…
//
// When displayed can span multiple pages: 1/n
struct LedgerPageView {
    // Name of the panel, like hash, chain name, sender, etc.
    name: String,
    // Whether element is for expert mode only.
    expert: bool,
    values: Vec<LedgerValue>,
}

impl LedgerPageView {
    fn from_element(element: Element) -> Self {
        if element.name.chars().count() > LEDGER_VIEW_NAME_COUNT {
            panic!(
                "Name tag can only be {} elements. Tag: {}",
                LEDGER_VIEW_NAME_COUNT, element.name
            )
        }
        let value_str = format!("{}", element.value);
        let mut values = vec![];
        let mut curr_value = LedgerValue::default();
        for c in value_str.chars() {
            let added = curr_value.add_char(c);
            if !added {
                // Single ledger page can't contain more characters.
                values.push(curr_value.clone());
                curr_value = LedgerValue::default();
                assert!(curr_value.add_char(c));
            }
        }
        // Add the last view to the collection.
        values.push(curr_value.clone());

        LedgerPageView {
            name: element.name.clone(),
            expert: element.expert,
            values,
        }
    }

    // Turn the current element into printable views.
    fn to_string(&self) -> Vec<String> {
        let total_count = self.values.len();
        if total_count == 1 {
            // The whole value can fit on one screen.
            return vec![format!("{} : {}", self.name, self.values[0])];
        }
        let mut output = vec![];
        // Split value display into multiple screens.
        for (idx, value) in self.values.iter().enumerate() {
            output.push(format!(
                "{} [{}/{}] : {}",
                self.name,
                idx + 1,
                total_count,
                value
            ));
        }
        output
    }
}

struct LedgerView {
    pages: Vec<LedgerPageView>,
}

impl LedgerView {
    fn from_ledger(ledger: Ledger) -> Self {
        let pages = ledger
            .0
            .into_iter()
            .map(LedgerPageView::from_element)
            .collect();
        LedgerView { pages }
    }

    // Builds a vector of strings that follows the pattern:
    // "0 | Type : Transfer",
    // "1 | To [1/2] : 0101010101010101010101010101010101",
    // "1 | To [2/2] : 010101010101010101010101010101",
    // "2 | Amount : CSPR 24.5",
    // "3 | Id : 999",
    // "4 | Payment : "CSPR 1"
    fn to_string(&self, expert: bool) -> Vec<String> {
        let mut output = vec![];
        for (idx, page) in self
            .pages
            .iter()
            .filter(|page| if !page.expert { true } else { expert })
            .enumerate()
        {
            let pages_str: Vec<String> = page
                .to_string()
                .into_iter()
                .map(|page_str| format!("{} | {}", idx, page_str))
                .collect();
            output.extend(pages_str)
        }
        output
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct JsonRepr {
    index: usize,
    name: String,
    blob: String,
    output: Vec<String>,
    output_expert: Vec<String>,
}

pub(super) fn from_deploy(index: usize, name: &str, deploy: Deploy) -> JsonRepr {
    let blob = hex::encode(&deploy.to_bytes().unwrap());
    let ledger = Ledger::from_deploy(deploy);
    let ledger_view = LedgerView::from_ledger(ledger);
    let output = ledger_view.to_string(false);
    let output_expert = ledger_view.to_string(true);
    JsonRepr {
        index,
        name: name.to_string(),
        blob,
        output,
        output_expert,
    }
}

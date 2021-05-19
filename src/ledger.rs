use std::{collections::BTreeMap, fmt::Display, ops::Div, str::FromStr};

use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_node::types::{Deploy, DeployHeader};
use casper_types::{CLValue, Key, RuntimeArgs, U512, bytesrepr::ToBytes, system::{
        mint::{ARG_ID, ARG_SOURCE, ARG_TARGET, ARG_TO},
        standard_payment::ARG_AMOUNT,
    }};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

const LEDGER_VIEW_NAME_COUNT: usize = 11;
const LEDGER_VIEW_TOP_COUNT: usize = 17;
const LEDGER_VIEW_BOTTOM_COUNT: usize = 17;

struct Elements(Vec<Element>);

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

fn drop_key_type_prefix(cl_in: String) -> String {
    let parsed_key = Key::from_formatted_str(&cl_in);
    match parsed_key {
        Ok(key) => {
            let prefix = match key {
                Key::Account(_) => "account-hash-",
                Key::Hash(_) => "hash-",
                Key::URef(_) => {
                    // format: uref-XXXX-YYY
                    return cl_in
                        .chars()
                        .skip("uref-".len())
                        .take_while(|c| *c != '-')
                        .collect();
                }
                Key::Transfer(_) => "transfer-",
                Key::DeployInfo(_) => "deploy-",
                Key::EraInfo(_) => "era-",
                Key::Balance(_) => "balance-",
                Key::Bid(_) => "bid-",
                Key::Withdraw(_) => "withdraw-",
            };
            cl_in.chars().skip(prefix.len()).collect()
        }
        Err(_) => {
            // No idea how to handle that. Return raw.
            cl_in
        }
    }
}

/// Extracts the `parsed` field from the `CLValue`.
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

fn parse_transfer(args: &RuntimeArgs) -> Vec<Element> {
    let mut elements = args
        .get(ARG_TO)
        .map(cl_value_to_string)
        .map(|to| vec![Element::regular(ARG_TO, to)])
        .unwrap_or_default();
    elements.extend(parse_optional_arg(args, ARG_SOURCE, true).into_iter());
    elements.push(parse_arg(args, ARG_TARGET, true));
    elements.push(parse_amount(args));
    elements.extend(parse_optional_arg(args, ARG_ID, true).into_iter());
    elements
}

fn parse_phase(item: &ExecutableDeployItem, phase: TxnPhase) -> Vec<Element> {
    let item_type;
    let phase_args = match item {
        ExecutableDeployItem::ModuleBytes { module_bytes, args } => {
            match phase {
                TxnPhase::Payment => {
                    if module_bytes.inner_bytes().is_empty() {
                        item_type = "system".to_string();
                        let mut elements = vec![parse_amount(args)];
                        let args_sans_amount = remove_amount_arg(args.clone());
                        elements.extend(parse_runtime_args(&args_sans_amount));
                        elements
                    } else {
                        item_type = "contract".to_string();
                        let payment_bytes = "".to_string(); // TODO
                        let mut elements = vec![Element::expert("bytes", payment_bytes)];
                        // TODO: add module's hash
                        elements.extend(parse_runtime_args(args));
                        elements
                    }
                }
                TxnPhase::Session => {
                    item_type = "contract".to_string();
                    let payment_bytes = "".to_string(); // TODO
                    let mut elements = vec![Element::expert("bytes", payment_bytes)];
                    // TODO: add module's hash
                    elements.extend(parse_runtime_args(args));
                    elements
                }
            }
        }
        ExecutableDeployItem::StoredContractByHash {
            hash,
            entry_point,
            args,
        } => {
            item_type = "by-hash".to_string();
            let mut elements = vec![Element::expert("hash", format!("{}", hash))];
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
            let mut elements = vec![Element::expert("name", format!("{}", name))];
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
            let mut elements = vec![Element::expert("by-addr", format!("{}", hash))];
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
            let mut elements = vec![Element::expert("name", format!("{}", name))];
            elements.push(entrypoint(entry_point));
            elements.push(parse_version(version));
            elements.extend(parse_runtime_args(args));
            elements
        }
        ExecutableDeployItem::Transfer { args } => {
            item_type = "native transfer".to_string();
            parse_transfer(args)
        }
    };
    let phase_label = format!("{}", phase);
    let mut elements = vec![Element::regular(&phase_label, item_type)];
    elements.extend(phase_args);
    elements
}

#[derive(Clone, Copy)]
enum TxnPhase {
    Payment,
    Session,
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
    elements.push(Element::expert("timestamp", format!("{}", dh.timestamp())));
    elements.push(Element::expert("ttl", format!("{}", dh.ttl())));
    elements.push(Element::expert("gas price", format!("{}", dh.gas_price())));
    elements.push(Element::expert(
        "deps",
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

impl Element {
    fn expert(name: &str, value: String) -> Element {
        Element {
            name: name.to_string(),
            value,
            expert: true,
        }
    }

    fn regular(name: &str, value: String) -> Self {
        Element {
            name: name.to_string(),
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

    fn new() -> Self {
        Ledger(vec![])
    }

    fn add_view(&mut self, view: Element) {
        self.0.push(view)
    }
}

#[derive(Default, Clone)]
struct LedgerValue {
    top: String,
    bottom: String,
}

impl LedgerValue {
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

    fn to_string(&self) -> Vec<String> {
        let total_count = self.values.len();
        if total_count == 1 {
            return vec![format!("{} : {}", self.name, self.values[0])];
        }
        let mut output = vec![];
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
    // "2 | To [2/2] : 010101010101010101010101010101",
    // "3 | Amount : CSPR 24.5",
    // "4 | Id : 999",
    // "5 | Payment : "CSPR 1"
    fn to_string(&self, expert: bool) -> Vec<String> {
        let mut output = vec![];
        for (idx, page_str) in self
            .pages
            .iter()
            .filter(|page| if !page.expert { true } else { expert })
            .flat_map(|page| page.to_string())
            .enumerate()
        {
            output.push(format!("{} | {}", idx, page_str))
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
    let blob = "".to_string();
    // let blob = hex::encode(&deploy.to_bytes().unwrap());
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

#[cfg(test)]
mod ledger_view {
    use super::{Element, Ledger, LedgerView};

    #[test]
    fn to_string() {
        let type_element = Element {
            name: "Type".to_string(),
            value: "Transfer".to_string(),
            expert: false,
        };
        let to_element = Element {
            name: "To".to_string(),
            value: "0101010101010101010101010101010101010101010101010101010101010101".to_string(),
            expert: false,
        };
        let amount_element = Element {
            name: "Amount".to_string(),
            value: "CSPR 24.5".to_string(),
            expert: false,
        };
        let mut ledger = Ledger::new();
        ledger.add_view(type_element);
        ledger.add_view(to_element);
        ledger.add_view(amount_element);
        let ledger = LedgerView::from_ledger(ledger);
        for ledger_page_view in ledger.to_string(false) {
            println!("{}", ledger_page_view);
        }
    }
}

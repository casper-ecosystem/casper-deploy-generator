use std::collections::BTreeMap;

use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_node::types::{Deploy, DeployHeader};
use casper_types::{bytesrepr::ToBytes, CLValue, Key, RuntimeArgs};

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
        serde_json::Value::String(string) => shorten_cl_string(string.clone()),
        serde_json::Value::Array(arr) => {
            format!("[{}]", arr.iter().map(serde_value_to_str).join(", "))
        }
        serde_json::Value::Object(map) => map.values().map(serde_value_to_str).join(":"),
    }
}

fn shorten_cl_string(cl_in: String) -> String {
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

impl Into<Elements> for &RuntimeArgs {
    fn into(self) -> Elements {
        let mut elements: Vec<Element> = vec![];
        let named_args: BTreeMap<String, CLValue> = self.clone().into();
        for (idx, (name, value)) in named_args.iter().enumerate() {
            let name_label = format!("arg-{}-name", idx);
            elements.push(Element::expert(&name_label, name.to_string()));
            let value_label = format!("arg-{}-val", idx);
            let value_str = cl_value_to_string(&value);
            elements.push(Element::expert(&value_label, value_str));
        }
        Elements(elements)
    }
}

impl Into<Elements> for &ExecutableDeployItem {
    fn into(self) -> Elements {
        let mut elements = vec![];
        match self {
            ExecutableDeployItem::ModuleBytes { module_bytes, args } => {
                // TODO: add module's hash
                let args_elements: Elements = args.into();
                elements.extend(args_elements.0);
            }
            ExecutableDeployItem::StoredContractByHash {
                hash,
                entry_point,
                args,
            } => {
                elements.push(Element::expert("to-addr", format!("{}", hash)));
                elements.push(Element::expert("to-entry", format!("{}", entry_point)));
                let args_elements: Elements = args.into();
                elements.extend(args_elements.0);
            }
            ExecutableDeployItem::StoredContractByName {
                name,
                entry_point,
                args,
            } => {
                elements.push(Element::expert("to-name", format!("{}", name)));
                elements.push(Element::expert("to-entry", format!("{}", entry_point)));
                let args_elements: Elements = args.into();
                elements.extend(args_elements.0);
            }
            ExecutableDeployItem::StoredVersionedContractByHash {
                hash,
                version,
                entry_point,
                args,
            } => {
                elements.push(Element::expert("to-addr", format!("{}", hash)));
                elements.push(Element::expert("to-entry", format!("{}", entry_point)));
                let version = match version {
                    None => "latest".to_string(),
                    Some(version) => format!("{}", version),
                };
                elements.push(Element::expert("to-version", format!("{}", version)));
                let args_elements: Elements = args.into();
                elements.extend(args_elements.0);
            }
            ExecutableDeployItem::StoredVersionedContractByName {
                name,
                version,
                entry_point,
                args,
            } => {
                elements.push(Element::expert("to-name", format!("{}", name)));
                elements.push(Element::expert("to-entry", format!("{}", entry_point)));
                let version = match version {
                    None => "latest".to_string(),
                    Some(version) => format!("{}", version),
                };
                elements.push(Element::expert("to-version", format!("{}", version)));
                let args_elements: Elements = args.into();
                elements.extend(args_elements.0);
            }
            ExecutableDeployItem::Transfer { args } => {
                let maybe_target = args.get("target").map(cl_value_to_string);
                match maybe_target {
                    None => {}
                    Some(target) => elements.push(Element::regular("target", target)),
                }
                let maybe_amount = args.get("amount").map(cl_value_to_string);
                match maybe_amount {
                    None => {}
                    Some(amount) => elements.push(Element::regular("amount", amount)),
                }
                let maybe_id = args.get("id").map(cl_value_to_string);
                match maybe_id {
                    None => {}
                    Some(id) => elements.push(Element::regular("id", id)),
                }
            }
        }
        Elements(elements)
    }
}

impl Into<Elements> for &DeployHeader {
    fn into(self) -> Elements {
        let mut elements = vec![];
        elements.push(Element::regular(
            "chain ID",
            format!("{}", self.chain_name()),
        ));
        elements.push(Element::regular("from", format!("{}", self.account())));
        elements.push(Element::expert(
            "timestamp",
            format!("{}", self.timestamp()),
        ));
        elements.push(Element::expert("ttl", format!("{}", self.ttl())));
        elements.push(Element::expert(
            "gas price",
            format!("{}", self.gas_price()),
        ));
        elements.push(Element::expert(
            "deps",
            format!("{:?}", self.dependencies()),
        ));
        Elements(elements)
    }
}

impl Into<Elements> for Deploy {
    fn into(self) -> Elements {
        let mut elements = vec![];
        let deploy_type = if self.session().is_transfer() {
            "Transfer".to_string()
        } else {
            "Execute Contract".to_string()
        };
        elements.push(Element::regular("Type", deploy_type));
        let header_elements: Elements = self.header().into();
        elements.extend(header_elements.0);
        let payment_elements: Elements = self.payment().into();
        elements.extend(payment_elements.0);
        let session_elements: Elements = self.session().into();
        elements.extend(session_elements.0);
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

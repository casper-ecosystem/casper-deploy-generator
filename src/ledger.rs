use std::fmt::Display;

use casper_execution_engine::core::engine_state::ExecutableDeployItem::{self, *};
use casper_node::types::{Deploy, DeployHeader};
use casper_types::bytesrepr::ToBytes;

use serde::{Deserialize, Serialize};

const LEDGER_VIEW_NAME_COUNT: usize = 11;
const LEDGER_VIEW_TOP_COUNT: usize = 17;
const LEDGER_VIEW_BOTTOM_COUNT: usize = 17;

struct Elements<V>(Vec<Element<V>>);

impl Into<Elements<String>> for &ExecutableDeployItem {
    fn into(self) -> Elements<String> {
        match self {
            ExecutableDeployItem::ModuleBytes { module_bytes, args } => {}
            ExecutableDeployItem::StoredContractByHash {
                hash,
                entry_point,
                args,
            } => {}
            ExecutableDeployItem::StoredContractByName {
                name,
                entry_point,
                args,
            } => {}
            ExecutableDeployItem::StoredVersionedContractByHash {
                hash,
                version,
                entry_point,
                args,
            } => {}
            ExecutableDeployItem::StoredVersionedContractByName {
                name,
                version,
                entry_point,
                args,
            } => {}
            ExecutableDeployItem::Transfer { args } => {}
        }
        Elements(vec![])
    }
}

impl Into<Elements<String>> for &DeployHeader {
    fn into(self) -> Elements<String> {
        let mut elements = vec![];
        elements.push(Element::regular(
            "chain-name",
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
            "dependency",
            format!("{:?}", self.dependencies()),
        ));
        Elements(elements)
    }
}

impl Into<Elements<String>> for Deploy {
    fn into(self) -> Elements<String> {
        let mut elements = vec![];
        let header_elements: Elements<String> = self.header().into();
        elements.extend(header_elements.0);
        let payment_elements: Elements<String> = self.payment().into();
        elements.extend(payment_elements.0);
        let session_elements: Elements<String> = self.session().into();
        elements.extend(session_elements.0);
        Elements(elements)
    }
}

struct Element<V> {
    name: String,
    value: V,
    // Whether to display in expert mode only.
    expert: bool,
}

impl<V> Element<V> {
    fn expert(name: &str, value: V) -> Element<V> {
        Element {
            name: name.to_string(),
            value,
            expert: true,
        }
    }

    fn regular(name: &str, value: V) -> Self {
        Element {
            name: name.to_string(),
            value,
            expert: false,
        }
    }
}

struct Ledger(Vec<Element<String>>);

impl Ledger {
    fn from_deploy(deploy: Deploy) -> Self {
        let elements: Elements<String> = deploy.into();
        Ledger(elements.0)
    }

    fn new() -> Self {
        Ledger(vec![])
    }

    fn add_view(&mut self, view: Element<String>) {
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
    fn from_element<V: Display>(element: Element<V>) -> Self {
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
            .filter(|page| if expert { page.expert } else { true })
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

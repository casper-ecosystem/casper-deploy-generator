use std::fmt::Display;

use casper_node::types::Deploy;
use casper_types::bytesrepr::ToBytes;

use serde::{Deserialize, Serialize};

use crate::parser::{parse_approvals, parse_deploy_header, parse_phase};

const LEDGER_VIEW_NAME_COUNT: usize = 11;
const LEDGER_VIEW_TOP_COUNT: usize = 17;
const LEDGER_VIEW_BOTTOM_COUNT: usize = 17;

pub(crate) struct Elements(Vec<Element>);

#[derive(Clone, Copy)]
pub(crate) enum TxnPhase {
    Payment,
    Session,
}

impl TxnPhase {
    pub(crate) fn is_payment(&self) -> bool {
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
pub(crate) struct Element {
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
    pub(crate) fn expert(name: &str, value: String) -> Element {
        Element {
            name: capitalize_first(name),
            value,
            expert: true,
        }
    }

    pub(crate) fn regular(name: &str, value: String) -> Self {
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
    valid: bool,
    testnet: bool,
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
        valid: true,
        testnet: true,
        blob,
        output,
        output_expert,
    }
}

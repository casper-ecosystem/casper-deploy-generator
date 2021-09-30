use std::{fmt::Display, rc::Rc};

use casper_node::types::Deploy;
use casper_types::bytesrepr::ToBytes;

use serde::{Deserialize, Serialize};

use crate::{parser, sample::Sample};

const LEDGER_VIEW_NAME_COUNT: usize = 11;
const LEDGER_VIEW_TOP_COUNT: usize = 17;
const LEDGER_VIEW_BOTTOM_COUNT: usize = 17;

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

#[derive(Debug, Clone)]
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

    pub(crate) fn as_expert(&mut self) {
        self.expert = true;
    }
}

#[derive(Clone)]
struct Ledger(Vec<Element>);

impl Ledger {
    fn from_deploy(deploy: Deploy) -> Self {
        Ledger(parser::parse_deploy(deploy))
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
#[derive(Default, Clone)]
struct LedgerPageView {
    // Name of the panel, like hash, chain name, sender, etc.
    name: String,
    // Whether element is for expert mode only.
    expert: bool,
    values: Vec<LedgerValue>,
}

impl LedgerPageView {
    /// Parses an `Element` object (which represents a single piece of a transaction) into a Ledger representation -
    /// including chopping up the string representation of the `Element` so that they can fit on a single Ledger screen.
    fn from_element(element: Element) -> Self {
        if element.name.chars().count() > LEDGER_VIEW_NAME_COUNT {
            panic!(
                "Name tag can only be {} elements. Tag: {}",
                LEDGER_VIEW_NAME_COUNT, element.name
            )
        }
        let mut values = vec![];
        let mut curr_value = LedgerValue::default();
        for c in element.value.chars() {
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

    /// Turn the current element into printable Ledger views.
    /// Adds indexes and labels.
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

///
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

#[derive(Clone)]

pub(crate) struct LimitedLedgerConfig {
    page_limit: u8,
    on_regular: Rc<dyn Fn(&Ledger) -> Vec<String>>,
    on_expert: Rc<dyn Fn(&Ledger) -> Vec<String>>,
}

impl LimitedLedgerConfig {
    pub(crate) fn new(page_limit: u8) -> Self {
        Self {
            page_limit,
            on_regular: Rc::new(Self::deploy_complexity_notice),
            on_expert: Rc::new(Self::deploy_basic_info),
        }
    }

    fn deploy_complexity_notice(_ledger: &Ledger) -> Vec<String> {
        todo!()
    }

    fn deploy_basic_info(_ledger: &Ledger) -> Vec<String> {
        todo!()
    }
}

struct LimitedLedgerView<'a> {
    _config: &'a LimitedLedgerConfig,
    ledger: Ledger,
}

impl<'a> LimitedLedgerView<'a> {
    fn new(config: &'a LimitedLedgerConfig, ledger: Ledger) -> Self {
        Self {
            _config: config,
            ledger,
        }
    }

    fn regular(&self) -> Vec<String> {
        LedgerView::from_ledger(self.ledger.clone()).to_string(false)
    }

    fn expert(&self) -> Vec<String> {
        LedgerView::from_ledger(self.ledger.clone()).to_string(true)
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct JsonRepr {
    index: usize,
    name: String,
    valid_regular: bool,
    valid_expert: bool,
    testnet: bool,
    blob: String,
    output: Vec<String>,
    output_expert: Vec<String>,
}

pub(super) fn from_deploy(
    index: usize,
    sample_deploy: Sample<Deploy>,
    config: &LimitedLedgerConfig,
) -> JsonRepr {
    let (name, deploy, valid) = sample_deploy.destructure();
    let blob = hex::encode(&deploy.to_bytes().unwrap());
    let ledger = Ledger::from_deploy(deploy);
    let ledger_view = LimitedLedgerView::new(config, ledger);
    let output = ledger_view.regular();
    let output_expert = ledger_view.expert();
    JsonRepr {
        index,
        name,
        valid_regular: valid,
        valid_expert: valid,
        testnet: true,
        blob,
        output,
        output_expert,
    }
}

#[cfg(test)]
mod ledger_tests {
    #[test]
    fn limit_ledger_pages() {
        assert!(true)
    }
}

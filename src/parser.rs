mod auction;
mod deploy;
mod runtime_args;
mod utils;

use casper_node::types::Deploy;

use crate::{
    ledger::{Element, TxnPhase},
    parser::deploy::{parse_approvals, parse_deploy_header, parse_phase},
};

pub(crate) fn parse_deploy(d: Deploy) -> Vec<Element> {
    let mut elements = vec![];
    let deploy_type = if d.session().is_transfer() {
        "Transfer"
    } else {
        "Execute Contract"
    };
    elements.push(Element::regular("Type", format!("{}", deploy_type)));
    elements.extend(parse_deploy_header(d.header()));
    elements.extend(parse_phase(d.payment(), TxnPhase::Payment));
    elements.extend(parse_phase(d.session(), TxnPhase::Session));
    elements.extend(parse_approvals(&d));
    elements
}

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
    elements.push(Element::regular(
        "Txn hash",
        format!("{:?}", d.id().inner()),
    ));
    elements.push(deploy_type(&d));
    elements.extend(parse_deploy_header(d.header()));
    elements.extend(parse_phase(d.payment(), TxnPhase::Payment));
    elements.extend(parse_phase(d.session(), TxnPhase::Session));
    elements.extend(parse_approvals(&d));
    elements
}

fn deploy_type(d: &Deploy) -> Element {
    let dtype = if auction::is_delegate(d.session()) {
        "Delegate"
    } else if auction::is_undelegate(d.session()) {
        "Undelegate"
    } else if d.session().is_transfer() {
        "Token transfer"
    } else {
        "Contract execution"
    };
    Element::regular("Type", dtype.to_string())
}

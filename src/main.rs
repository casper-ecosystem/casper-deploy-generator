use ledger::JsonRepr;
use test_data::{invalid_samples, valid_samples};
use test_rng::TestRng;

use crate::ledger::LimitedLedgerConfig;

mod ledger;
mod parser;
mod sample;
mod test_data;
mod test_rng;
mod utils;

fn main() {
    let mut rng = TestRng::new();

    let page_limit = 15;

    let limited_ledger_config = LimitedLedgerConfig::new(page_limit);

    let data: Vec<JsonRepr> = valid_samples(&mut rng)
        .into_iter()
        .chain(invalid_samples(&mut rng).into_iter())
        .enumerate()
        .map(|(id, sample_deploy)| ledger::from_deploy(id, sample_deploy, &limited_ledger_config))
        .collect();

    println!("{}", serde_json::to_string_pretty(&data).unwrap());
}

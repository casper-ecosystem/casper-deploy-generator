use ledger::{LimitedLedgerConfig, ZondaxRepr};
use test_data::{
    generic_samples, invalid_samples, native_transfer_samples, redelegate_samples, valid_samples,
};
use test_rng::TestRng;

pub mod checksummed_hex;
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

    let data: Vec<ZondaxRepr> = valid_samples(&mut rng)
        .into_iter()
        .chain(invalid_samples(&mut rng).into_iter())
        .chain(native_transfer_samples(&mut rng).into_iter())
        .chain(redelegate_samples(&mut rng).into_iter())
        .chain(generic_samples(&mut rng).into_iter())
        .enumerate()
        .map(|(id, sample_deploy)| {
            ledger::deploy_to_json(id, sample_deploy, &limited_ledger_config)
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&data).unwrap());
}

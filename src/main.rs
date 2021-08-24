use ledger::JsonRepr;
use test_data::{invalid_samples, valid_samples};
use test_rng::TestRng;

mod ledger;
mod parser;
mod sample;
mod test_data;
mod test_rng;
mod utils;

fn main() {
    let mut rng = TestRng::new();

    let data: Vec<JsonRepr> = valid_samples(&mut rng)
        .into_iter()
        .chain(invalid_samples(&mut rng).into_iter())
        .enumerate()
        .map(|(id, sample_deploy)| {
            let (label, deploy, valid) = sample_deploy.destructure();
            ledger::from_deploy(id, valid, &label, deploy)
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&data).unwrap());
}

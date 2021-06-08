use ledger::JsonRepr;
use test_data::valid_samples;

mod ledger;
mod parser;
mod sample;
mod test_data;
mod utils;

fn main() {
    let valid_data: Vec<JsonRepr> = valid_samples()
        .into_iter()
        .enumerate()
        .map(|(id, sample_deploy)| {
            let (label, deploy) = sample_deploy.destructure();
            ledger::from_deploy(id, true, &label, deploy)
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&valid_data).unwrap());
}

use casper_node::types::Deploy;

use ledger::JsonRepr;
use test_data::valid_samples;

mod ledger;
mod parser;
mod sample;
mod test_data;
mod utils;

fn main() {
    let data: Vec<JsonRepr> = valid_samples()
        .into_iter()
        .enumerate()
        .map(|(id, sample_deploy)| {
            let (label, deploy) = sample_deploy.destructure();
            ledger::from_deploy(id, &label, deploy)
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&data).unwrap());
}

#[allow(unused)]
fn print(id: usize, deploy: Deploy) {
    println!("----- EXAMPLE NR {} BEGINNING -----\n", id);

    println!("JSON:\n");
    println!("{}\n", serde_json::to_string_pretty(&deploy).unwrap());

    println!("Ledger:\n");
    println!(
        "{}\n",
        serde_json::to_string_pretty(&ledger::from_deploy(id, "test", deploy)).unwrap()
    );

    println!("----- EXAMPLE NR {} END ----------\n", id);
}

use std::str::FromStr;

use casper_execution_engine::core::engine_state::executable_deploy_item::ExecutableDeployItem;
use casper_node::types::{Deploy, DeployHash, TimeDiff, Timestamp};
use casper_types::SecretKey;
use test_data::Sample;

use crate::ledger::JsonRepr;

mod ledger;
mod parser;
mod test_data;
mod utils;

fn main() {
    let data: Vec<JsonRepr> = sessions()
        .into_iter()
        .enumerate()
        .map(|(id, session)| {
            let sample_deploy = construct(session);
            let (label, deploy) = sample_deploy.destructure();
            ledger::from_deploy(id, &label, deploy)
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&data).unwrap());
}

fn sessions() -> Vec<Sample<ExecutableDeployItem>> {
    let mut sessions = vec![];
    sessions.extend(test_data::native_transfer::samples());
    sessions
}

fn construct(sample_session: Sample<ExecutableDeployItem>) -> Sample<Deploy> {
    let secret_key = SecretKey::ed25519([123u8; 32]);

    let standard_payment = test_data::system_payment::sample();

    let deploy = |session| {
        Deploy::new(
            Timestamp::from_str("2021-05-04T14:20:35.104Z").unwrap(),
            TimeDiff::from_seconds(60 * 30),
            2,
            vec![
                DeployHash::new([15u8; 32].into()),
                DeployHash::new([16u8; 32].into()),
            ],
            String::from("mainnet"),
            standard_payment,
            session,
            &secret_key,
        )
    };

    let mut sample_deploy = sample_session.map_sample(deploy);
    sample_deploy.add_label("payment:system".to_string());
    sample_deploy
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

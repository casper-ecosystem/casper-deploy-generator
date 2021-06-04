use std::{collections::BTreeMap, str::FromStr};

use casper_execution_engine::core::engine_state::executable_deploy_item::ExecutableDeployItem;
use casper_node::types::{Deploy, DeployHash, TimeDiff, Timestamp};
use casper_types::{
    bytesrepr::Bytes, runtime_args, AccessRights, Key, PublicKey, RuntimeArgs, SecretKey,
    TransferAddr, URef, U128, U512,
};

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
            let deploy = construct(session);
            ledger::from_deploy(id, "test", deploy)
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&data).unwrap());
}

fn sessions() -> Vec<ExecutableDeployItem> {
    let public_key: PublicKey = SecretKey::ed25519([11u8; 32]).into();
    let public_key2: PublicKey = SecretKey::ed25519([12u8; 32]).into();

    let mut sessions = vec![];
    sessions.extend(test_data::native_transfer::samples());
    sessions
}

fn construct(session: ExecutableDeployItem) -> Deploy {
    let secret_key = SecretKey::ed25519([123u8; 32]);

    let standard_payment = test_data::system_payment::sample();

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

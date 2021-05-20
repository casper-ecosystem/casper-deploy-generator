use std::{collections::BTreeMap, str::FromStr};

use casper_node::types::{Deploy, DeployHash, TimeDiff, Timestamp};
use casper_execution_engine::core::engine_state::executable_deploy_item::ExecutableDeployItem;
use casper_types::{AccessRights, Key, PublicKey, RuntimeArgs, SecretKey, TransferAddr, U128, U512, URef, bytesrepr::Bytes, bytesrepr::ToBytes, runtime_args};

mod ledger;

fn main() {
    for (id, session) in sessions().into_iter().enumerate() {
        let deploy = construct(session);
        print(id, deploy);
    }
}

fn sessions() -> Vec<ExecutableDeployItem> {
    let public_key: PublicKey = SecretKey::ed25519([11u8; 32]).into();
    let public_key2: PublicKey = SecretKey::ed25519([12u8; 32]).into();

    let mut sessions = vec![];

    let transfer = ExecutableDeployItem::Transfer { args: runtime_args!{
        "amount" => U512::from(24500000000u64),
        "target" => [1u8; 32],
        "id" => Some(999u64),
        "additional_info" => "this is transfer"
    }};
    sessions.push(transfer);

    let contract_call_by_hash = ExecutableDeployItem::StoredContractByHash {
        hash: [3u8; 32].into(),
        entry_point: String::from("please_call_me"),
        args: runtime_args! {
            "bool_arg" => true,
            "i32_arg" => -1i32,
            "i64_arg" => -2i64,
            "u8_arg" => 4u8,
            "u32_arg" => 5u32,
            "u64_arg" => 6u32,
            "u128_arg" => U128::from(7),
            "u256_arg" => U128::from(8),
            "u512_arg" => U128::from(9),
        }
    };
    sessions.push(contract_call_by_hash);

    let contract_call_by_name = ExecutableDeployItem::StoredContractByName {
        name: String::from("decentralized_exchange"),
        entry_point: String::from("trasnfer"),
        args: runtime_args! {
            "arg_string" => String::from("all_in"),
            "arg_public_key" => public_key,
            "arg_option_none" => None as Option<String>,
            "arg_option_first" => Some(10u32),
            "arg_option_second" => Some(public_key.to_account_hash()),
            "arg_account_hash" => public_key.to_account_hash()
        }
    };
    sessions.push(contract_call_by_name);

    let mut map = BTreeMap::new();
    map.insert("account_one", public_key);
    map.insert("account_two", public_key2);

    let contract_call_versioned_hash = ExecutableDeployItem::StoredVersionedContractByHash {
        hash: [3u8; 32].into(),
        version: Some(12),
        entry_point: String::from("vest_tokens"),
        args: runtime_args! {
            "arg_result_ok" => Result::<u32, String>::Ok(123u32),
            "arg_result_err" => Result::<u32, String>::Err(String::from("hard problem")),
            "arg_map" => map,
            "arg_empty_map" => BTreeMap::<u32, i64>::new(),
            "arg_tuple1" => (10u32, ),
            "arg_tuple2" => (11u32, String::from("second")),
            "arg_tuple3" => (12u32, String::from("second"), (30u8, true)),
            "arg_unit" => ()
        }
    };
    sessions.push(contract_call_versioned_hash);

    let contract_call_versioned_hash_no_version = ExecutableDeployItem::StoredVersionedContractByHash {
        hash: [3u8; 32].into(),
        version: None,
        entry_point: String::from("vest_tokens"),
        args: runtime_args! {
            "arg_list_one" => vec![10u32, 11, 12, 13],
            "arg_list_two" => vec![public_key.to_account_hash(), public_key2.to_account_hash()],
            "arg_uref" => URef::new([22u8; 32], AccessRights::READ)
        }
    };
    sessions.push(contract_call_versioned_hash_no_version);

    let contract_call_versioned_hash_no_version = ExecutableDeployItem::StoredVersionedContractByName {
        name: String::from("black_hole"),
        version: None,
        entry_point: String::from("explode"),
        args: runtime_args! {
            "arg_key_account" => Key::Account(public_key.to_account_hash()),
            "arg_key_hash" => Key::Hash([42u8; 32]),
            "arg_key_uref" => Key::URef(URef::new([23u8; 32], AccessRights::ADD_WRITE)),
            "arg_key_transfer" => Key::Transfer(TransferAddr::new([124u8; 32])),
            "arg_key_deploy" => Key::DeployInfo(casper_types::DeployHash::new([45u8; 32].into())),
            "arg_key_era_id" => Key::EraInfo(15),
            "arg_key_balance" => Key::Balance([254u8; 32]),
            "arg_key_bid" => Key::Bid(public_key.to_account_hash()),
            "arg_key_withdraw" => Key::Withdraw(public_key.to_account_hash())
        }
    };
    sessions.push(contract_call_versioned_hash_no_version);

    let deploy_contract = ExecutableDeployItem::ModuleBytes {
        module_bytes: Bytes::from(Vec::from([221u8; 1001])),
        args: runtime_args! {}
    };
    sessions.push(deploy_contract);

    sessions
}

fn construct(session: ExecutableDeployItem) -> Deploy {
    let secret_key = SecretKey::ed25519([123u8; 32]);

    let standard_payment = ExecutableDeployItem::ModuleBytes{
        module_bytes: Bytes::new(),
        args: runtime_args!{
            "amount" => U512::from(1000000000) 
        }
    };

    Deploy::new(
        Timestamp::from_str("2021-05-04T14:20:35.104Z").unwrap(),
        TimeDiff::from_seconds(60 * 30),
        2,
        vec![
            DeployHash::new([15u8; 32].into()),
            DeployHash::new([16u8; 32].into())
        ],
        String::from("mainnet"),
        standard_payment,
        session,
        &secret_key
    )
}

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

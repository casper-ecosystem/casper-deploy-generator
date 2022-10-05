use std::fmt::Debug;

use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_types::{
    account::{AccountHash, ACCOUNT_HASH_LENGTH},
    bytesrepr::{Bytes, ToBytes},
    AccessRights, AsymmetricType, CLTyped, CLValue, DeployHash, EraId, Key, NamedArg, PublicKey,
    RuntimeArgs, TransferAddr, URef, DEPLOY_HASH_LENGTH, KEY_DICTIONARY_LENGTH, KEY_HASH_LENGTH,
    TRANSFER_ADDR_LENGTH, U128, U256, U512, UREF_ADDR_LENGTH,
};
use rand::{prelude::SliceRandom, Rng};

use crate::{sample::Sample, test_data::commons::sample_executables};

use super::commons::UREF_ADDR;

pub(crate) fn valid<R: Rng>(rng: &mut R) -> Vec<Sample<ExecutableDeployItem>> {
    const ENTRYPOINT: &str = "generic-txn-entrypoint";
    let rargs: Vec<RuntimeArgs> = sample_args(rng);

    let mut output = Vec::with_capacity(rargs.len());

    for args in rargs {
        for sample in sample_executables(ENTRYPOINT, args, None, true) {
            output.push(sample)
        }
    }

    output
}

fn to_clvalue_labeled<T>(value: T) -> (String, CLValue)
where
    T: CLTyped + ToBytes + Debug,
{
    (
        format!("{:?}", T::cl_type()),
        CLValue::from_t(value).expect("create CLValue"),
    )
}

fn vec_to_clvalues<T>(value: Vec<T>) -> Vec<(String, CLValue)>
where
    T: CLTyped + ToBytes + Debug,
{
    value.into_iter().map(to_clvalue_labeled).collect()
}

#[allow(unused_parens)]
fn sample_args<R: Rng>(rng: &mut R) -> Vec<RuntimeArgs> {
    let mut named_args: Vec<NamedArg> = vec![
        vec_to_clvalues(vec![true, false]),
        vec_to_clvalues(vec![i32::MIN, 0, i32::MAX]),
        vec_to_clvalues(vec![i64::MIN, 0, i64::MAX]),
        vec_to_clvalues(vec![u8::MIN, u8::MAX]),
        vec_to_clvalues(vec![u32::MIN, u32::MAX]),
        vec_to_clvalues(vec![u64::MIN, u64::MAX]),
        vec_to_clvalues(vec![U128::zero(), U128::max_value()]),
        vec_to_clvalues(vec![U256::zero(), U256::max_value()]),
        vec_to_clvalues(vec![U512::zero(), U512::max_value()]),
        vec_to_clvalues(sample_keys()),
        vec_to_clvalues(sample_urefs()),
        vec![to_clvalue_labeled(())],
        vec_to_clvalues(vec!["sample-string"]),
        vec_to_clvalues(vec![
            PublicKey::system(),
            PublicKey::ed25519_from_bytes([1u8; 32]).unwrap(),
            PublicKey::secp256k1_from_bytes(
                hex::decode(b"026e1b7a8e3243f5ff14e825b0fde15103588bb61e6ae99084968b017118e0504f")
                    .unwrap(),
            )
            .unwrap(),
        ]),
        vec_to_clvalues(vec![Some(100u8), None]),
        vec![
            (
                "list-publickey".to_string(),
                CLValue::from_t::<Vec<PublicKey>>(vec![]).unwrap(),
            ),
            (
                "list-publickey".to_string(),
                CLValue::from_t(vec![
                    PublicKey::ed25519_from_bytes([1u8; 32]).unwrap(),
                    PublicKey::secp256k1_from_bytes(
                        hex::decode(
                            b"026e1b7a8e3243f5ff14e825b0fde15103588bb61e6ae99084968b017118e0504f",
                        )
                        .unwrap(),
                    )
                    .unwrap(),
                ])
                .unwrap(),
            ),
        ],
        vec![
            (
                "bytearray".to_string(),
                CLValue::from_t(Bytes::from(vec![])).unwrap(),
            ),
            (
                "bytearray".to_string(),
                CLValue::from_t(Bytes::from(vec![1u8; 32])).unwrap(),
            ),
            (
                "bytearray".to_string(),
                CLValue::from_t(Bytes::from(vec![1u8; 64])).unwrap(),
            ),
        ],
        vec_to_clvalues(vec![Ok(false), Err(-10i32)]),
        vec![to_clvalue_labeled((11u8))],
        vec![to_clvalue_labeled((11u8, 1111u64))],
        vec![to_clvalue_labeled((0u8, true, "tuple3"))],
        // ("map".to_string(), todo!())
    ]
    .into_iter()
    .flatten()
    .map(|(label, value)| NamedArg::new(label, value))
    .collect();

    let named_args_count = named_args.len() as u32;

    let mut out = vec![];

    for _ in 0..15 {
        named_args.shuffle(rng);
        let n = rng.gen_range(2..named_args_count);
        out.push(
            named_args
                .iter()
                .take(n as usize)
                .cloned()
                .collect::<Vec<NamedArg>>()
                .into(),
        );
    }

    out
}

fn sample_urefs() -> Vec<URef> {
    vec![
        URef::new(UREF_ADDR, AccessRights::NONE),
        URef::new(UREF_ADDR, AccessRights::READ),
        URef::new(UREF_ADDR, AccessRights::ADD),
        URef::new(UREF_ADDR, AccessRights::WRITE),
        URef::new(UREF_ADDR, AccessRights::READ_ADD),
        URef::new(UREF_ADDR, AccessRights::READ_ADD_WRITE),
        URef::new(UREF_ADDR, AccessRights::READ_WRITE),
        URef::new(UREF_ADDR, AccessRights::ADD_WRITE),
    ]
}

fn sample_keys() -> Vec<Key> {
    let account_key = casper_types::Key::Account(AccountHash::new([1u8; ACCOUNT_HASH_LENGTH]));
    let hash_key = casper_types::Key::Hash([1u8; KEY_HASH_LENGTH]);
    let balance_key = casper_types::Key::Balance([1u8; UREF_ADDR_LENGTH]);
    let bid_key = casper_types::Key::Bid(AccountHash::new([1u8; ACCOUNT_HASH_LENGTH]));
    let deploy_info_key = casper_types::Key::DeployInfo(DeployHash::new([1u8; DEPLOY_HASH_LENGTH]));
    let dictionary_key = casper_types::Key::Dictionary([1u8; KEY_DICTIONARY_LENGTH]);
    let era_info_key = casper_types::Key::EraInfo(EraId::new(0));
    let transfer_key = casper_types::Key::Transfer(TransferAddr::new([1u8; TRANSFER_ADDR_LENGTH]));
    let uref_key = casper_types::Key::URef(URef::new(
        [1u8; UREF_ADDR_LENGTH],
        AccessRights::READ_ADD_WRITE,
    ));
    let withdraw_key = casper_types::Key::Withdraw(AccountHash::new([1u8; ACCOUNT_HASH_LENGTH]));

    vec![
        account_key,
        hash_key,
        balance_key,
        bid_key,
        deploy_info_key,
        dictionary_key,
        era_info_key,
        transfer_key,
        uref_key,
        withdraw_key,
    ]
}

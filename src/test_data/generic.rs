use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_types::{
    account::{AccountHash, ACCOUNT_HASH_LENGTH},
    bytesrepr::{self, Bytes, ToBytes},
    AccessRights, AsymmetricType, CLType, CLTyped, CLValue, DeployHash, EraId, Key, NamedArg,
    PublicKey, RuntimeArgs, TransferAddr, URef, DEPLOY_HASH_LENGTH, KEY_DICTIONARY_LENGTH,
    KEY_HASH_LENGTH, TRANSFER_ADDR_LENGTH, U128, U256, U512, UREF_ADDR_LENGTH,
};
use rand::{prelude::SliceRandom, Rng};
use strum::{EnumIter, IntoEnumIterator};

use crate::{
    sample::Sample,
    test_data::commons::{sample_executables, sample_module_bytes},
};

use super::commons::UREF_ADDR;

pub(crate) fn valid<R: Rng>(rng: &mut R) -> Vec<Sample<ExecutableDeployItem>> {
    const ENTRYPOINT: &str = "generic-txn-entrypoint";
    let rargs: Vec<RuntimeArgs> = sample_args(rng);

    let mut output = Vec::with_capacity(rargs.len());

    output.push(sample_module_bytes(rargs.first().cloned().unwrap()));

    for args in rargs {
        for sample in sample_executables(ENTRYPOINT, args.clone(), None, true) {
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

#[derive(EnumIter, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum CLTypeVariant {
    /// `bool` primitive.
    Bool,
    /// `i32` primitive.
    I32,
    /// `i64` primitive.
    I64,
    /// `u8` primitive.
    U8,
    /// `u32` primitive.
    U32,
    /// `u64` primitive.
    U64,
    /// [`U128`] large unsigned integer type.
    U128,
    /// [`U256`] large unsigned integer type.
    U256,
    /// [`U512`] large unsigned integer type.
    U512,
    /// `()` primitive.
    Unit,
    /// `String` primitive.
    String,
    /// [`Key`] system type.
    Key,
    /// [`URef`] system type.
    URef,
    /// [`PublicKey`](crate::PublicKey) system type.
    PublicKey,
    /// `Option` of a `CLType`.
    Option,
    /// Variable-length list of a single `CLType` (comparable to a `Vec`).
    List,
    /// Fixed-length list of a single `CLType` (comparable to a Rust array).
    ByteArray,
    /// `Result` with `Ok` and `Err` variants of `CLType`s.
    Result,
    /// Map with keys of a single `CLType` and values of a single `CLType`.
    Map,
    /// 1-ary tuple of a `CLType`.
    Tuple1,
    /// 2-ary tuple of `CLType`s.
    Tuple2,
    /// 3-ary tuple of `CLType`s.
    Tuple3,
    /// Unspecified type.
    Any,
}

impl From<&CLType> for CLTypeVariant {
    fn from(value: &CLType) -> Self {
        match value {
            CLType::Bool => CLTypeVariant::Bool,
            CLType::I32 => CLTypeVariant::I32,
            CLType::I64 => CLTypeVariant::I64,
            CLType::U8 => CLTypeVariant::U8,
            CLType::U32 => CLTypeVariant::U32,
            CLType::U64 => CLTypeVariant::U64,
            CLType::U128 => CLTypeVariant::U128,
            CLType::U256 => CLTypeVariant::U256,
            CLType::U512 => CLTypeVariant::U512,
            CLType::Unit => CLTypeVariant::Unit,
            CLType::String => CLTypeVariant::String,
            CLType::Key => CLTypeVariant::Key,
            CLType::URef => CLTypeVariant::URef,
            CLType::PublicKey => CLTypeVariant::PublicKey,
            CLType::Option(_) => CLTypeVariant::Option,
            CLType::List(_) => CLTypeVariant::List,
            CLType::ByteArray(_) => CLTypeVariant::ByteArray,
            CLType::Result { .. } => CLTypeVariant::Result,
            CLType::Map { .. } => CLTypeVariant::Map,
            CLType::Tuple1([_]) => CLTypeVariant::Tuple1,
            CLType::Tuple2([_, _]) => CLTypeVariant::Tuple2,
            CLType::Tuple3([_, _, _]) => CLTypeVariant::Tuple3,
            CLType::Any => CLTypeVariant::Any,
        }
    }
}

fn for_all_cl_type_variants(args: &RuntimeArgs) -> impl Iterator<Item = CLTypeVariant> + '_ {
    args.named_args()
        .flat_map(|named_arg| match named_arg.cl_value().cl_type() {
            CLType::Tuple1([t1]) => {
                vec![CLTypeVariant::Tuple1, t1.as_ref().into()]
            }
            CLType::Tuple2([t1, t2]) => {
                vec![
                    CLTypeVariant::Tuple2,
                    t1.as_ref().into(),
                    t2.as_ref().into(),
                ]
            }
            CLType::Tuple3([t1, t2, t3]) => {
                vec![
                    CLTypeVariant::Tuple3,
                    t1.as_ref().into(),
                    t2.as_ref().into(),
                    t3.as_ref().into(),
                ]
            }
            CLType::Result { ok, err } => {
                vec![
                    CLTypeVariant::Result,
                    ok.as_ref().into(),
                    err.as_ref().into(),
                ]
            }
            CLType::Option(t) => {
                vec![CLTypeVariant::Option, t.as_ref().into()]
            }
            CLType::List(t) => {
                vec![CLTypeVariant::List, t.as_ref().into()]
            }
            CLType::Map { key, value } => {
                vec![
                    CLTypeVariant::Map,
                    key.as_ref().into(),
                    value.as_ref().into(),
                ]
            }
            primitive_type => {
                vec![CLTypeVariant::from(primitive_type)]
            }
        })
}

struct CustomStruct {
    value1: U512,
    value2: String,
    value3: u64,
}

impl ToBytes for CustomStruct {
    fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
        let Self {
            value1,
            value2,
            value3,
        } = self;
        let mut result = bytesrepr::allocate_buffer(self)?;
        result.extend_from_slice(&value1.to_bytes()?);
        result.extend_from_slice(&value2.to_bytes()?);
        result.extend_from_slice(&value3.to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        let Self {
            value1,
            value2,
            value3,
        } = self;
        value1.serialized_length() + value2.serialized_length() + value3.serialized_length()
    }
}

impl CLTyped for CustomStruct {
    fn cl_type() -> CLType {
        CLType::Any
    }
}
#[allow(unused_parens)]
fn sample_args<R: Rng>(rng: &mut R) -> Vec<RuntimeArgs> {
    let mut all_variants = CLTypeVariant::iter().collect::<BTreeSet<_>>();

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
        vec![to_clvalue_labeled((11u8,))],
        vec![to_clvalue_labeled((11u8, 1111u64))],
        vec![to_clvalue_labeled((0u8, true, "tuple3"))],
        vec![
            (
                "map".to_string(),
                CLValue::from_t(BTreeMap::<String, String>::new()).unwrap(),
            ),
            (
                "map".to_string(),
                CLValue::from_t({
                    let mut map = BTreeMap::<String, Bytes>::new();
                    map.insert("key".to_string(), b"value".to_vec().into());
                    map
                })
                .unwrap(),
            ),
            (
                "map".to_string(),
                CLValue::from_t({
                    let mut map = BTreeMap::<String, String>::new();
                    map.insert("key1".to_string(), "value1".to_string());
                    map.insert("key2".to_string(), "value2".to_string());
                    map.insert("key3".to_string(), "value3".to_string());
                    map
                })
                .unwrap(),
            ),
            (
                "map".to_string(),
                CLValue::from_t({
                    let mut map = BTreeMap::<String, U512>::new();
                    map.insert("foo".to_string(), U512::one());
                    map.insert("bar".to_string(), U512::zero());
                    map.insert("baz".to_string(), U512::MAX);
                    map
                })
                .unwrap(),
            ),
            (
                "bytearray".to_string(),
                CLValue::from_t({
                    let mut bytearray = [0u8; 32];
                    for (i, byte) in bytearray.iter_mut().enumerate() {
                        debug_assert!(i <= 255);
                        *byte = i as u8;
                    }
                    bytearray
                })
                .unwrap(),
            ),
            (
                "any".to_string(),
                CLValue::from_t(CustomStruct {
                    value1: U512::one(),
                    value2: "hello".to_string(),
                    value3: 42,
                })
                .unwrap(),
            ),
        ],
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

    // Ensure all CLType variants are used at least once.
    for runtime_args in &out {
        for variant in for_all_cl_type_variants(runtime_args) {
            let _ = all_variants.remove(&variant);
        }
    }

    assert_eq!(
        all_variants,
        BTreeSet::new(),
        "all variants must be visited"
    );

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
    let system_registry_key = casper_types::Key::SystemContractRegistry;
    let chainspec_registry_key = casper_types::Key::ChainspecRegistry;
    let checksum_registry_key = casper_types::Key::ChainspecRegistry;

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
        system_registry_key,
        chainspec_registry_key,
        checksum_registry_key,
    ]
}

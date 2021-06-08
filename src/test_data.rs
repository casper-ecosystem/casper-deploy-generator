use std::str::FromStr;

use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_node::types::{Deploy, DeployHash, TimeDiff, Timestamp};
use casper_types::{
    account::AccountHash, AccessRights, CLValue, Key, RuntimeArgs, SecretKey, URef, U512,
};

use rand::prelude::*;

use crate::sample::Sample;

// From the chainspec.
// 1 minute.
const MIN_TTL: TimeDiff = TimeDiff::from_seconds(60);
// 1 day.
const MAX_TTL: TimeDiff = TimeDiff::from_seconds(60 * 60 * 24);
// 1 hour.
const TTL_HOUR: TimeDiff = TimeDiff::from_seconds(60 * 60);

// From the chainspec.
const MIN_DEPS_COUNT: u8 = 0;
const MAX_DEPS_COUNT: u8 = 10;

// From the chainspec.
const MIN_APPROVALS_COUNT: u8 = 1;
const MAX_APPROVALS_COUNT: u8 = 10;

#[derive(Clone, Copy, Debug)]
struct NativeTransfer {
    target: TransferTarget,
    amount: U512,
    id: u64,
    source: Option<URef>,
}

impl NativeTransfer {
    fn new(target: TransferTarget, amount: U512, id: u64, source: Option<URef>) -> Self {
        NativeTransfer {
            target,
            amount,
            id,
            source,
        }
    }
}

impl From<NativeTransfer> for RuntimeArgs {
    fn from(nt: NativeTransfer) -> Self {
        let mut ra = RuntimeArgs::new();
        ra.insert("amount", nt.amount).unwrap();
        ra.insert("id", Some(nt.id)).unwrap();
        if nt.source.is_some() {
            ra.insert("source", nt.source).unwrap();
        }
        ra.insert_cl_value("target", nt.target.into_cl());
        ra
    }
}

#[derive(Clone, Copy, Debug)]
enum TransferTarget {
    // raw bytes representing account hash
    Bytes([u8; 32]),
    // transfer to a specific purse
    URef(URef),
    // transfer to an account.
    Key(Key),
}

impl TransferTarget {
    fn into_cl(self) -> CLValue {
        let cl_value_res = match self {
            TransferTarget::Bytes(bytes) => CLValue::from_t(bytes),
            TransferTarget::URef(uref) => CLValue::from_t(uref),
            TransferTarget::Key(key) => CLValue::from_t(key),
        };
        cl_value_res.unwrap()
    }

    fn bytes() -> TransferTarget {
        TransferTarget::Bytes([255u8; 32])
    }

    fn uref() -> TransferTarget {
        let uref = URef::new([0u8; 32], AccessRights::READ_ADD_WRITE);
        TransferTarget::URef(uref)
    }

    fn key() -> TransferTarget {
        let account_key = Key::Account(AccountHash::new([33u8; 32]));
        TransferTarget::Key(account_key)
    }

    fn label(&self) -> &str {
        match self {
            TransferTarget::Bytes(_) => "target:bytes",
            TransferTarget::URef(_) => "target:uref",
            TransferTarget::Key(_) => "target:key-account",
        }
    }
}

mod native_transfer {
    use casper_execution_engine::core::engine_state::ExecutableDeployItem;
    use casper_types::{AccessRights, URef, U512};

    use crate::{sample::Sample, test_data::TransferTarget};

    use super::NativeTransfer;

    fn native_transfer_samples(
        amounts: &[U512],
        ids: &[u64],
        targets: &[TransferTarget],
        sources: &[Option<URef>],
    ) -> Vec<Sample<NativeTransfer>> {
        let mut samples: Vec<Sample<NativeTransfer>> = vec![];

        for amount in amounts {
            for id in ids {
                for target in targets {
                    for source in sources {
                        let source_label = if source.is_none() {
                            "source:none"
                        } else {
                            "source:uref"
                        };
                        let label = format!("native_transfer-{}-{}", target.label(), source_label);
                        let nt = NativeTransfer::new(*target, *amount, *id, *source);
                        let sample = Sample::new(label, nt, true);
                        samples.push(sample);
                    }
                }
            }
        }

        samples
    }

    pub(super) fn samples() -> Vec<Sample<ExecutableDeployItem>> {
        let amount_min = U512::from(0u8);
        let amount_mid = U512::from(100000000);
        let amount_max = U512::MAX;
        let amounts = vec![amount_min, amount_mid, amount_max];
        let id_min = u64::MIN;
        let id_max = u64::MAX;
        let ids = vec![id_min, id_max];
        let targets = vec![
            TransferTarget::bytes(),
            TransferTarget::uref(),
            TransferTarget::key(),
        ];

        let access_rights = vec![
            AccessRights::READ,
            AccessRights::WRITE,
            AccessRights::ADD,
            AccessRights::READ_ADD,
            AccessRights::READ_WRITE,
            AccessRights::READ_ADD_WRITE,
        ];

        let sources: Vec<Option<URef>> = access_rights
            .into_iter()
            .map(|ar| Some(URef::new([2u8; 32], ar)))
            .chain(vec![None])
            .collect();

        native_transfer_samples(&amounts, &ids, &targets, &sources)
            .into_iter()
            .map(|s| {
                let f = |nt: NativeTransfer| ExecutableDeployItem::Transfer { args: nt.into() };
                s.map_sample(f)
            })
            .collect()
    }
}

mod system_payment {
    use casper_execution_engine::core::engine_state::ExecutableDeployItem;
    use casper_types::{bytesrepr::Bytes, runtime_args, RuntimeArgs, U512};

    pub(super) fn sample() -> ExecutableDeployItem {
        ExecutableDeployItem::ModuleBytes {
            module_bytes: Bytes::new(),
            args: runtime_args! {
                "amount" => U512::from(1000000000)
            },
        }
    }
}

fn make_deploy(
    session: Sample<ExecutableDeployItem>,
    payment: ExecutableDeployItem,
    ttl: TimeDiff,
    dependencies: Vec<DeployHash>,
    signing_keys: &[SecretKey],
) -> Sample<Deploy> {
    let (main_key, secondary_keys) = signing_keys.split_at(1);

    let deploy = |session| {
        Deploy::new(
            Timestamp::from_str("2021-05-04T14:20:35.104Z").unwrap(),
            ttl,
            2,
            dependencies,
            String::from("mainnet"),
            payment,
            session,
            &main_key[0],
        )
    };

    // Sign deploy with possibly multiple keys.
    let mut sample_deploy = session.map_sample(deploy);
    for key in secondary_keys {
        let (label, mut deploy, valid) = sample_deploy.destructure();
        deploy.sign(key);
        sample_deploy = Sample::new(label, deploy, valid);
    }
    sample_deploy
}

fn make_dependencies(count: u8) -> Vec<DeployHash> {
    if count == 0 {
        return vec![];
    }

    let mut dependencies = vec![];
    for i in 0..count {
        dependencies.push(DeployHash::new([i; 32].into()));
    }
    dependencies
}

fn random_keys(key_count: u8) -> Vec<SecretKey> {
    let mut out = vec![];
    for i in 0..key_count {
        let key = if i % 2 == 0 {
            SecretKey::ed25519([i; 32].into())
        } else {
            SecretKey::secp256k1([i; 32].into())
        };
        out.push(key);
    }
    out
}

pub(crate) fn valid_samples() -> Vec<Sample<Deploy>> {
    let mut rng = rand::thread_rng();

    let session_samples = native_transfer::samples();
    let standard_payment = system_payment::sample();

    let mut ttls = vec![MIN_TTL, TTL_HOUR, MAX_TTL];

    let mut deps_count = vec![MIN_DEPS_COUNT, 3, MAX_DEPS_COUNT];

    let mut key_count = vec![MIN_APPROVALS_COUNT, 3, MAX_APPROVALS_COUNT];

    let mut samples = vec![];

    for session in session_samples {
        key_count.shuffle(&mut rng);
        // Random signing keys count.
        let mut keys: Vec<SecretKey> = random_keys(*key_count.first().unwrap());
        // Randomize order of keys, so that both alg have chance to be the main one.
        keys.shuffle(&mut rng);

        // Random dependencies within correct limits.
        deps_count.shuffle(&mut rng);
        let dependencies = make_dependencies(deps_count.first().cloned().unwrap());

        // Pick a random TTL value.
        ttls.shuffle(&mut rng);
        let ttl = ttls.first().cloned().unwrap();

        let mut sample_deploy =
            make_deploy(session, standard_payment.clone(), ttl, dependencies, &keys);
        sample_deploy.add_label("payment:system".to_string());
        samples.push(sample_deploy);
    }
    samples
}

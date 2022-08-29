use std::str::FromStr;

use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_node::types::{Deploy, DeployHash, TimeDiff, Timestamp};
use casper_types::{
    account::AccountHash, AccessRights, AsymmetricType, CLValue, Key, PublicKey, RuntimeArgs,
    SecretKey, URef, U512,
};
use rand::{prelude::*, Rng};

use auction::{delegate, undelegate};

use crate::sample::Sample;

use self::{auction::redelegate, commons::UREF_ADDR};

mod auction;
mod commons;
mod generic;
mod native_transfer;
mod system_payment;

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

#[derive(Clone, Debug)]
struct NativeTransfer {
    target: TransferTarget,
    amount: U512,
    id: u64,
    source: TransferSource,
}

impl NativeTransfer {
    fn new(target: TransferTarget, amount: U512, id: u64, source: TransferSource) -> Self {
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
        if let TransferSource::URef(uref) = nt.source {
            ra.insert("source", uref).unwrap();
        }
        ra.insert_cl_value("target", nt.target.into_cl());
        ra
    }
}

#[derive(Clone, Debug)]
enum TransferSource {
    None,
    URef(URef),
}

impl TransferSource {
    pub fn uref(uref: URef) -> Self {
        TransferSource::URef(uref)
    }

    pub fn none() -> Self {
        TransferSource::None
    }

    pub fn label(&self) -> &str {
        match self {
            TransferSource::None => "source:none",
            TransferSource::URef(_) => "source:uref",
        }
    }
}

#[derive(Clone, Debug)]
enum TransferTarget {
    // raw bytes representing account hash
    Bytes([u8; 32]),
    // transfer to a specific purse
    URef(URef),
    // transfer to an account.
    Key(Key),
    // transfer to public key
    PublicKey(PublicKey),
}

impl TransferTarget {
    fn into_cl(self) -> CLValue {
        let cl_value_res = match self {
            TransferTarget::Bytes(bytes) => CLValue::from_t(bytes),
            TransferTarget::URef(uref) => CLValue::from_t(uref),
            TransferTarget::Key(key) => CLValue::from_t(key),
            TransferTarget::PublicKey(pk) => CLValue::from_t(pk),
        };
        cl_value_res.unwrap()
    }

    fn bytes() -> TransferTarget {
        TransferTarget::Bytes([255u8; 32])
    }

    fn uref() -> TransferTarget {
        let uref = URef::new(UREF_ADDR, AccessRights::READ_ADD_WRITE);
        TransferTarget::URef(uref)
    }

    fn key() -> TransferTarget {
        let account_key = Key::Account(
            AccountHash::from_formatted_str(
                "account-hash-45f3aa6ce2a450dd5a4f2cc4cc9054aded66de6b6cfc4ad977e7251cf94b649b",
            )
            .unwrap(),
        );
        TransferTarget::Key(account_key)
    }

    fn public_key_ed25519() -> TransferTarget {
        let public_key = PublicKey::ed25519_from_bytes(
            hex::decode(b"2bac1d0ff9240ff0b7b06d555815640497861619ca12583ddef434885416e69b")
                .unwrap(),
        )
        .unwrap();
        TransferTarget::PublicKey(public_key)
    }

    fn public_key_secp256k1() -> TransferTarget {
        let public_key = PublicKey::secp256k1_from_bytes(
            hex::decode(b"026e1b7a8e3243f5ff14e825b0fde15103588bb61e6ae99084968b017118e0504f")
                .unwrap(),
        )
        .unwrap();
        TransferTarget::PublicKey(public_key)
    }

    fn label(&self) -> String {
        match self {
            TransferTarget::Bytes(_) => "target:bytes".to_string(),
            TransferTarget::URef(_) => "target:uref".to_string(),
            TransferTarget::Key(_) => "target:key_account".to_string(),
            TransferTarget::PublicKey(pk) => {
                let variant = match pk {
                    PublicKey::Ed25519(_) => "ed25519",
                    PublicKey::Secp256k1(_) => "secp256k1",
                    PublicKey::System => panic!("unexpected key type variant"),
                };
                format!("target:{}_public_key", variant)
            }
        }
    }
}

fn make_deploy_sample(
    session: Sample<ExecutableDeployItem>,
    payment: Sample<ExecutableDeployItem>,
    ttl: TimeDiff,
    dependencies: Vec<DeployHash>,
    signing_keys: &[SecretKey],
) -> Sample<Deploy> {
    let (main_key, secondary_keys) = signing_keys.split_at(1);
    let (payment_label, payment, payment_validity) = payment.destructure();

    let make_deploy = |session| {
        Deploy::new(
            Timestamp::from_str("2021-05-04T14:20:35.104Z").unwrap(),
            ttl,
            2,
            dependencies,
            String::from("mainnet"),
            payment,
            session,
            &main_key[0],
            None,
        )
    };

    // Sign deploy with possibly multiple keys.
    let mut sample_session = session.map_sample(make_deploy);
    for key in secondary_keys {
        let (label, mut deploy, session_validity) = sample_session.destructure();
        deploy.sign(key);
        // Sample is valid iff both session part and payment parts are valid.
        sample_session = Sample::new(label, deploy, session_validity && payment_validity);
    }
    sample_session.add_label(payment_label);

    sample_session
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
            SecretKey::ed25519_from_bytes(&[i; 32]).expect("successful key construction")
        } else {
            SecretKey::secp256k1_from_bytes(&[i; 32]).expect("successful key construction")
        };
        out.push(key);
    }
    out
}

fn construct_samples<R: Rng>(
    rng: &mut R,
    session_samples: Vec<Sample<ExecutableDeployItem>>,
    payment_samples: Vec<Sample<ExecutableDeployItem>>,
) -> Vec<Sample<Deploy>> {
    let mut samples = vec![];

    // These params do not change validity of a sample.
    let mut ttls = vec![MIN_TTL, TTL_HOUR, MAX_TTL];
    let mut deps_count = vec![MIN_DEPS_COUNT, 3, MAX_DEPS_COUNT];
    let mut key_count = vec![MIN_APPROVALS_COUNT, 3, MAX_APPROVALS_COUNT];

    for session in session_samples {
        for payment in &payment_samples {
            // Random number of keys.
            key_count.shuffle(rng);
            // Random signing keys count.
            let mut keys: Vec<SecretKey> = random_keys(*key_count.first().unwrap());
            // Randomize order of keys, so that both alg have chance to be the main one.
            keys.shuffle(rng);

            // Random dependencies within correct limits.
            deps_count.shuffle(rng);
            let dependencies = make_dependencies(deps_count.first().cloned().unwrap());

            // Pick a random TTL value.
            ttls.shuffle(rng);
            let ttl = ttls.first().cloned().unwrap();

            let sample_deploy =
                make_deploy_sample(session.clone(), payment.clone(), ttl, dependencies, &keys);
            samples.push(sample_deploy);
        }
    }
    samples
}

pub(crate) fn valid_samples<R: Rng>(rng: &mut R) -> Vec<Sample<Deploy>> {
    let mut session_samples = native_transfer::valid();
    session_samples.extend(delegate::valid(rng));
    session_samples.extend(undelegate::valid(rng));
    let payment_samples = vec![system_payment::valid()];

    construct_samples(rng, session_samples, payment_samples)
}

pub(crate) fn invalid_samples<R: Rng>(rng: &mut R) -> Vec<Sample<Deploy>> {
    let mut session_samples = native_transfer::invalid();
    session_samples.extend(delegate::invalid(rng));
    session_samples.extend(undelegate::invalid(rng));
    let payment_samples = vec![system_payment::invalid(), system_payment::valid()];

    construct_samples(rng, session_samples, payment_samples)
}

pub(crate) fn redelegate_samples<R: Rng>(rng: &mut R) -> Vec<Sample<Deploy>> {
    let valid_samples = redelegate::valid(rng);
    let valid_payment_samples = vec![system_payment::valid()];

    let mut samples = construct_samples(rng, valid_samples, valid_payment_samples);
    let invalid_samples = redelegate::invalid(rng);
    let invalid_payment_samples = vec![system_payment::invalid(), system_payment::valid()];
    samples.extend(construct_samples(
        rng,
        invalid_samples,
        invalid_payment_samples,
    ));
    samples
}

pub(crate) fn generic_samples<R: Rng>(rng: &mut R) -> Vec<Sample<Deploy>> {
    let valid_samples = generic::valid(rng);
    let valid_payment_samples = vec![system_payment::valid()];

    let mut samples = construct_samples(rng, valid_samples.clone(), valid_payment_samples);

    samples.extend(construct_samples(
        rng,
        valid_samples,
        vec![system_payment::invalid()],
    ));
    samples
}

use std::str::FromStr;

use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_node::types::{Deploy, DeployHash, TimeDiff, Timestamp};
use casper_types::{
    account::AccountHash, AccessRights, CLValue, Key, RuntimeArgs, SecretKey, URef, U512,
};

use crate::sample::Sample;

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
                        let sample = Sample::new(label, nt);
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
) -> Sample<Deploy> {
    let secret_key = SecretKey::ed25519([123u8; 32]);

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
            payment,
            session,
            &secret_key,
        )
    };

    session.map_sample(deploy)
}

pub(crate) fn valid_samples() -> Vec<Sample<Deploy>> {
    let session_samples = native_transfer::samples();
    let standard_payment = system_payment::sample();

    let mut samples = vec![];
    for session in session_samples {
        let mut sample_deploy = make_deploy(session, standard_payment.clone());
        sample_deploy.add_label("payment:system".to_string());
        samples.push(sample_deploy);
    }
    samples
}

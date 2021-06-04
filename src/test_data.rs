use casper_types::{
    account::AccountHash, bytesrepr::ToBytes, runtime_args, AccessRights, CLType, CLTyped, CLValue,
    Key, RuntimeArgs, Transfer, URef, U512,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct NativeTransfer {
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
pub(crate) enum TransferTarget {
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
        TransferTarget::Bytes([1u8; 32])
    }

    fn uref() -> TransferTarget {
        let uref = URef::new([33u8; 32], AccessRights::READ_ADD_WRITE);
        TransferTarget::URef(uref)
    }

    fn key() -> TransferTarget {
        let account_key = Key::Account(AccountHash::new([33u8; 32]));
        TransferTarget::Key(account_key)
    }
}

pub(crate) mod native_transfer {
    use std::ops::Div;

    use casper_execution_engine::core::engine_state::ExecutableDeployItem;
    use casper_types::{account::AccountHash, AccessRights, Key, URef, U512};

    use crate::test_data::TransferTarget;

    use super::NativeTransfer;

    fn native_transfer_samples() -> Vec<NativeTransfer> {
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
        let sources = vec![Some(URef::new([2u8; 32], AccessRights::READ)), None];

        let mut samples: Vec<NativeTransfer> = vec![];

        for amount in &amounts {
            for id in &ids {
                for target in &targets {
                    for source in &sources {
                        let nt = NativeTransfer::new(*target, *amount, *id, *source);
                        samples.push(nt);
                    }
                }
            }
        }

        samples
    }

    pub(crate) fn samples() -> Vec<ExecutableDeployItem> {
        native_transfer_samples()
            .into_iter()
            .map(|nt| ExecutableDeployItem::Transfer { args: nt.into() })
            .collect()
    }
}

pub(crate) mod system_payment {
    use casper_execution_engine::core::engine_state::ExecutableDeployItem;
    use casper_types::{bytesrepr::Bytes, runtime_args, RuntimeArgs, U512};

    pub(crate) fn sample() -> ExecutableDeployItem {
        ExecutableDeployItem::ModuleBytes {
            module_bytes: Bytes::new(),
            args: runtime_args! {
                "amount" => U512::from(1000000000)
            },
        }
    }
}

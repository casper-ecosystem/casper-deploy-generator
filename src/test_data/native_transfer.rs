use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_types::{runtime_args, AccessRights, RuntimeArgs, URef, U512};

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
                    let nt = NativeTransfer::new(target.clone(), *amount, *id, *source);
                    let sample = Sample::new(label, nt, true);
                    samples.push(sample);
                }
            }
        }
    }

    samples
}

pub(super) fn valid() -> Vec<Sample<ExecutableDeployItem>> {
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
        TransferTarget::public_key(),
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

pub(super) fn invalid() -> Vec<Sample<ExecutableDeployItem>> {
    let missing_required_amount: RuntimeArgs = runtime_args! {
        "id" => 1u64,
        "target" => URef::new([1u8; 32], AccessRights::READ),
    };
    let missing_required_id: RuntimeArgs = runtime_args! {
        "amount" => U512::from(100000000u64),
        "target" => URef::new([1u8; 32], AccessRights::READ),
    };
    let missing_required_target: RuntimeArgs = runtime_args! {
        "amount" => U512::from(100000000u64),
        "id" => 1u64,
    };
    let invalid_amount_type: RuntimeArgs = runtime_args! {
        "amount" => 10000u64,
        "target" => URef::new([1u8; 32], AccessRights::READ),
        "id" => 1u64,
    };

    let invalid_transfer_args: Vec<Sample<RuntimeArgs>> = vec![
        Sample::new("missing:amount", missing_required_amount, false),
        Sample::new("missing:id", missing_required_id, false),
        Sample::new("missing:target", missing_required_target, false),
        Sample::new("invalid_type:amount", invalid_amount_type, false),
    ];

    invalid_transfer_args
        .into_iter()
        .map(|sample_ra| {
            let (label, ra, _valid) = sample_ra.destructure();
            let sample_invalid_transfer = ExecutableDeployItem::Transfer { args: ra.into() };
            let new_label = format!("native_transfer-{}", label);
            Sample::new(new_label, sample_invalid_transfer, false)
        })
        .collect()
}

use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_types::{runtime_args, AccessRights, RuntimeArgs, URef, U512};

use crate::{sample::Sample, test_data::TransferTarget};

use super::{commons::UREF_ADDR, NativeTransfer, TransferSource};

/// Given collection of native target inputs,
/// for every combination of them creates a `NativeTransfer` sample.
fn native_transfer_samples(
    amounts: &[U512],
    transfer_id: &[u64],
    targets: &[TransferTarget],
    sources: &[TransferSource],
) -> Vec<Sample<NativeTransfer>> {
    let mut samples: Vec<Sample<NativeTransfer>> = vec![];

    for amount in amounts {
        for id in transfer_id {
            for target in targets {
                for source in sources {
                    let label = format!("native_transfer_{}_{}", target.label(), source.label());
                    let nt = NativeTransfer::new(target.clone(), *amount, *id, source.clone());
                    let sample = Sample::new(label, nt, true);
                    samples.push(sample);
                }
            }
        }
    }

    samples
}

/// Returns valid native transfer samples.
pub(super) fn valid() -> Vec<Sample<ExecutableDeployItem>> {
    let amount_min = U512::from(0u8);
    let amount_mid = U512::from(100000000);
    let amount_max = U512::MAX;
    let amounts = vec![amount_min, amount_mid, amount_max];
    let id_min = u64::MIN;
    let id_max = u64::MAX;
    let transfer_id = vec![id_min, id_max];
    let targets = vec![
        TransferTarget::bytes(),
        TransferTarget::uref(),
        TransferTarget::key(),
        TransferTarget::public_key_secp256k1(),
        TransferTarget::public_key_ed25519(),
    ];

    let access_rights = vec![
        AccessRights::READ,
        AccessRights::WRITE,
        AccessRights::ADD,
        AccessRights::READ_ADD,
        AccessRights::READ_WRITE,
        AccessRights::READ_ADD_WRITE,
    ];

    let sources: Vec<TransferSource> = access_rights
        .into_iter()
        .map(|ar| TransferSource::uref(URef::new(UREF_ADDR, ar)))
        .chain(vec![TransferSource::none()])
        .collect();

    native_transfer_samples(&amounts, &transfer_id, &targets, &sources)
        .into_iter()
        .map(|s| {
            let (label, sample, validity) = s.destructure();
            Sample::new(
                label,
                ExecutableDeployItem::Transfer {
                    args: sample.into(),
                },
                validity,
            )
        })
        .collect()
}

/// Returns invalid native transfer samples.
pub(super) fn invalid() -> Vec<Sample<ExecutableDeployItem>> {
    let missing_required_amount: RuntimeArgs = runtime_args! {
        "id" => 1u64,
        "target" => URef::new(UREF_ADDR, AccessRights::READ),
    };
    let missing_required_id: RuntimeArgs = runtime_args! {
        "amount" => U512::from(100000000u64),
        "target" => URef::new(UREF_ADDR, AccessRights::READ),
    };
    let missing_required_target: RuntimeArgs = runtime_args! {
        "amount" => U512::from(100000000u64),
        "id" => 1u64,
    };
    let invalid_amount_type: RuntimeArgs = runtime_args! {
        "amount" => 10000u64,
        "target" => URef::new(UREF_ADDR, AccessRights::READ),
        "id" => 1u64,
    };

    let invalid_transfer_args: Vec<Sample<RuntimeArgs>> = vec![
        Sample::new("missing_amount", missing_required_amount, false),
        Sample::new("missing_id", missing_required_id, false),
        Sample::new("missing_target", missing_required_target, false),
        Sample::new("invalid_type_amount", invalid_amount_type, false),
    ];

    invalid_transfer_args
        .into_iter()
        .map(|sample_ra| {
            let (label, ra, validity) = sample_ra.destructure(); // TODO
            let sample_invalid_transfer = ExecutableDeployItem::Transfer { args: ra };
            let new_label = format!("native_transfer_{}", label);
            Sample::new(new_label, sample_invalid_transfer, validity)
        })
        .collect()
}

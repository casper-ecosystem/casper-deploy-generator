use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_types::{bytesrepr::Bytes, runtime_args, RuntimeArgs, U512};

use crate::sample::Sample;

pub(super) fn valid() -> Sample<ExecutableDeployItem> {
    let payment = ExecutableDeployItem::ModuleBytes {
        module_bytes: Bytes::new(),
        args: runtime_args! {
            "amount" => U512::from(1000000000)
        },
    };

    Sample::new("payment:system", payment, true)
}

pub(super) fn invalid() -> Sample<ExecutableDeployItem> {
    let payment = ExecutableDeployItem::ModuleBytes {
        module_bytes: Bytes::new(),
        args: runtime_args! {
            "paying" => U512::from(1000000000)
        },
    };

    Sample::new("payment:system-missing:amount", payment, false)
}

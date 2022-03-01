use crate::abi::HEVMCalls;
use bytes::Bytes;
use revm::{Database, EVMData};

pub fn apply<DB: Database>(
    _: &mut EVMData<'_, DB>,
    call: &HEVMCalls,
) -> Option<Result<Bytes, Bytes>> {
    Some(match call {
        /*HEVMCalls::ExpectRevert0(_) => {}
        HEVMCalls::ExpectRevert1(_) => {}
        HEVMCalls::ExpectEmit(_) => {}
        HEVMCalls::ExpectCall(_) => {}
        HEVMCalls::MockCall(_) => {}
        HEVMCalls::ClearMockedCalls(_) => {}*/
        _ => return None,
    })
}

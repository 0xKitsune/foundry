use crate::abi::HEVMCalls;
use bytes::Bytes;
use revm::{Database, EVMData};

pub fn apply<DB: Database>(
    _: &mut EVMData<'_, DB>,
    call: &HEVMCalls,
) -> Option<Result<Bytes, Bytes>> {
    Some(match call {
        /*HEVMCalls::Ffi(_) => {}
        HEVMCalls::GetCode(_) => {}*/
        _ => return None,
    })
}

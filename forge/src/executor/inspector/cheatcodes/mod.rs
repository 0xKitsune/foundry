/// Cheatcodes related to the execution environment.
mod env;
pub use env::Prank;
/// Assertion helpers (such as `expectEmit`)
mod expect;
/// Cheatcodes that interact with the external environment (FFI etc.)
mod ext;
/// Cheatcodes that configure the fuzzer
mod fuzz;
/// Utility cheatcodes (`sign` etc.)
mod util;

use crate::{abi::HEVMCalls, executor::CHEATCODE_ADDRESS};
use bytes::Bytes;
use ethers::{
    abi::{AbiDecode, AbiEncode},
    types::Address,
};
use revm::{CallInputs, Database, EVMData, Gas, Inspector, Return};
use std::collections::BTreeMap;

/// An inspector that handles calls to various cheatcodes, each with their own behavior.
///
/// Cheatcodes can be called by contracts during execution to modify the VM environment, such as
/// mocking addresses, signatures and altering call reverts.
pub struct Cheatcodes {
    /// Whether FFI is enabled or not
    ffi: bool,

    /// Address labels
    pub labels: BTreeMap<Address, String>,

    /// Prank information
    pub prank: Option<Prank>,
}

impl Cheatcodes {
    pub fn new(ffi: bool) -> Self {
        Self { ffi, labels: BTreeMap::new(), prank: None }
    }

    fn apply_cheatcode<DB: Database>(
        &mut self,
        data: &mut EVMData<'_, DB>,
        call: &CallInputs,
    ) -> Result<Bytes, Bytes> {
        // Decode the cheatcode call
        let decoded = HEVMCalls::decode(&call.input).map_err(|err| err.to_string().encode())?;

        // TODO: Log the opcode for the debugger
        // TODO: FFI flag
        env::apply(self, data, &decoded)
            .or_else(|| util::apply(self, data, &decoded))
            .or_else(|| expect::apply(data, &decoded))
            .or_else(|| fuzz::apply(data, &decoded))
            .or_else(|| ext::apply(data, &decoded))
            .ok_or_else(|| "Cheatcode was unhandled. This is a bug.".to_string().encode())?
    }
}

impl<DB> Inspector<DB> for Cheatcodes
where
    DB: Database,
{
    fn call(
        &mut self,
        data: &mut EVMData<'_, DB>,
        call: &CallInputs,
        _: bool,
    ) -> (Return, Gas, Bytes) {
        if call.contract == *CHEATCODE_ADDRESS {
            match self.apply_cheatcode(data, call) {
                Ok(retdata) => (Return::Return, Gas::new(0), retdata),
                Err(err) => (Return::Revert, Gas::new(0), err),
            }
        } else {
            // Apply our prank
            if let Some(prank) = &self.prank {
                if data.subroutine.depth() == prank.depth &&
                    data.env.tx.caller == prank.prank_caller
                {
                    // TODO: How do we set `tx.origin`?
                    data.env.tx.caller = prank.new_caller;
                }
            }

            (Return::Continue, Gas::new(0), Bytes::new())
        }
    }

    fn call_end(
        &mut self,
        data: &mut EVMData<'_, DB>,
        _: &CallInputs,
        remaining_gas: Gas,
        status: Return,
        retdata: Bytes,
        _: bool,
    ) -> (Return, Gas, Bytes) {
        if let Some(prank) = &self.prank {
            if prank.single_call && data.subroutine.depth() == prank.depth {
                std::mem::take(&mut self.prank);
            }
        }

        (status, remaining_gas, retdata)
    }
}

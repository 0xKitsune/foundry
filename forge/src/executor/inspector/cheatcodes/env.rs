use super::Cheatcodes;
use crate::abi::HEVMCalls;
use bytes::Bytes;
use ethers::{
    abi::AbiEncode,
    types::{Address, H256},
    utils::keccak256,
};
use revm::{Database, EVMData};

#[derive(Clone, Debug, Default)]
pub struct Prank {
    /// Address of the contract that initiated the prank
    pub prank_caller: Address,
    /// The address to assign to `msg.sender`
    pub new_caller: Address,
    /// The address to assign to `tx.origin`
    pub new_origin: Option<Address>,
    /// The depth at which the prank was called
    pub depth: u64,
    /// Whether or not the prank stops by itself after the next call
    pub single_call: bool,
}

fn prank(
    state: &mut Cheatcodes,
    prank_caller: Address,
    new_caller: Address,
    new_origin: Option<Address>,
    depth: u64,
    single_call: bool,
) -> Result<Bytes, Bytes> {
    let prank = Prank { prank_caller, new_caller, new_origin, depth, single_call };

    // TODO: Depth shenanigans
    if state.prank.is_some() {
        return Err("You have an active prank already.".to_string().encode().into())
    }

    state.prank = Some(prank);
    Ok(Bytes::new())
}

pub fn apply<DB: Database>(
    state: &mut Cheatcodes,
    data: &mut EVMData<'_, DB>,
    call: &HEVMCalls,
) -> Option<Result<Bytes, Bytes>> {
    Some(match call {
        HEVMCalls::Warp(inner) => {
            data.env.block.timestamp = inner.0;
            Ok(Bytes::new())
        }
        HEVMCalls::Roll(inner) => {
            data.env.block.number = inner.0;
            // TODO: Set blockhash somehow
            Ok(Bytes::new())
        }
        HEVMCalls::Fee(inner) => {
            data.env.block.basefee = inner.0;
            Ok(Bytes::new())
        }
        HEVMCalls::Store(inner) => {
            // TODO: Does this increase gas usage?
            data.subroutine.load_account(inner.0, data.db);
            data.subroutine.sstore(inner.0, inner.1.into(), inner.2.into(), data.db);
            Ok(Bytes::new())
        }
        HEVMCalls::Load(inner) => {
            // TODO: Does this increase gas usage?
            data.subroutine.load_account(inner.0, data.db);
            let (val, _) = data.subroutine.sload(inner.0, inner.1.into(), data.db);
            Ok(val.encode().into())
        }
        HEVMCalls::Etch(inner) => {
            let code = inner.1.clone();
            let hash = H256::from_slice(&keccak256(&code));

            // TODO: Does this increase gas usage?
            data.subroutine.load_account(inner.0, data.db);
            data.subroutine.set_code(inner.0, code.0, hash);
            Ok(Bytes::new())
        }
        HEVMCalls::Deal(inner) => {
            let who = inner.0;
            let value = inner.1;

            // TODO: Does this increase gas usage?
            data.subroutine.load_account(who, data.db);
            let balance = data.subroutine.account(inner.0).info.balance;

            // TODO: We should probably upstream a `set_balance` function
            if balance < value {
                data.subroutine.balance_add(who, value - balance);
            } else {
                data.subroutine.balance_sub(who, balance - value);
            }
            Ok(Bytes::new())
        }
        HEVMCalls::Prank0(inner) => {
            prank(state, data.env.tx.caller, inner.0, None, data.subroutine.depth(), true)
        }
        HEVMCalls::Prank1(inner) => {
            prank(state, data.env.tx.caller, inner.0, Some(inner.1), data.subroutine.depth(), true)
        }
        HEVMCalls::StartPrank0(inner) => {
            prank(state, data.env.tx.caller, inner.0, None, data.subroutine.depth(), false)
        }
        HEVMCalls::StartPrank1(inner) => {
            prank(state, data.env.tx.caller, inner.0, Some(inner.1), data.subroutine.depth(), false)
        }
        HEVMCalls::StopPrank(_) => {
            if let Some(prank) = &state.prank {
                data.env.tx.caller = prank.prank_caller;
            }
            state.prank = None;
            Ok(Bytes::new())
        }
        /*HEVMCalls::Record(_) => {}
        HEVMCalls::Accesses(inner) => {}
        HEVMCalls::MockCall(inner) => {}
        HEVMCalls::ClearMockedCalls(_) => {}*/
        _ => return None,
    })
}

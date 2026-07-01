use core::ffi::c_int;

use super::super::MpCgameExport;
use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `CG_LAST_ATTACKER` MP cgame exports vmMain ABI token.
///
/// Raven: int (*CG_LastAttacker)( void );
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:381-382`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:206-207`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:206-207`
/// Transport/call-site source: `oracle/oracle/codemp/client/cl_console.cpp:100-108`
pub struct CgLastAttacker;

impl InboundVmCall for CgLastAttacker {
    type Command = MpCgameExport;
    type Args = ();
    type Output = c_int;

    const COMMAND: MpCgameExport = MpCgameExport::CG_LAST_ATTACKER;
}

impl DecodeVmMain for CgLastAttacker {
    fn decode_vm_main(_transport: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for CgLastAttacker {
    fn encode_return(output: Self::Output) -> isize {
        output as isize
    }
}

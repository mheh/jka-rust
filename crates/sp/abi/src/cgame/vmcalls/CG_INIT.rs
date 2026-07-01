use core::ffi::c_int;

use super::super::SpCgameExport;
use abi_transport::generic::InboundVmCall;

/// `CG_INIT` SP cgame exports vmMain ABI token.
///
/// Raven: `void CG_Init( int serverCommandSequence );`
/// Enum value source: `oracle/oracle/code/client/vmachine.h:14`
/// Args source: `oracle/oracle/code/cgame/cg_main.cpp:25`, `oracle/oracle/code/cgame/cg_main.cpp:98`
/// Output source: `oracle/oracle/code/cgame/cg_main.cpp:98`
/// VM_Call/vmMain switch source: `oracle/oracle/code/client/cl_cgame.cpp:1047`,
/// `oracle/oracle/code/cgame/cg_main.cpp:94-115`
pub struct CgInit;

impl InboundVmCall for CgInit {
    type Command = SpCgameExport;
    type Args = c_int;
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_INIT;
}

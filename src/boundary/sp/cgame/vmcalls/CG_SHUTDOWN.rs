use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_SHUTDOWN` SP cgame exports vmMain boundary token.
///
/// Raven: `void CG_Shutdown( void );`
/// Enum value source: `oracle/oracle/code/client/vmachine.h:15`
/// Args source: `oracle/oracle/code/cgame/cg_main.cpp:101-103`
/// Output source: `oracle/oracle/code/cgame/cg_main.cpp:101-103`
/// VM_Main switch source: `oracle/oracle/code/cgame/cg_main.cpp:94-132`
pub struct CgShutdown;

impl InboundVmCall for CgShutdown {
    type Command = SpCgameExport;
    type Args = ();
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_SHUTDOWN;
}

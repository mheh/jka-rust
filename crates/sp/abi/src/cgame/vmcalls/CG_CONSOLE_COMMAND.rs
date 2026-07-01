use super::super::SpCgameExport;
use abi_transport::generic::InboundVmCall;
use sp_qshared::shared::qboolean;

/// `CG_CONSOLE_COMMAND` SP cgame exports vmMain ABI token.
///
/// Raven: `qboolean CG_ConsoleCommand( void );`
/// Enum value source: `oracle/oracle/code/client/vmachine.h:16`
/// Args source: `oracle/oracle/code/cgame/cg_main.cpp:26`, `oracle/oracle/code/cgame/cg_main.cpp:104`,
/// `oracle/oracle/code/cgame/cg_local.h:869`
/// Output source: `oracle/oracle/code/cgame/cg_main.cpp:105`, `oracle/oracle/code/cgame/cg_local.h:869`
/// VM_Call/vmMain switch source: `oracle/oracle/code/client/cl_cgame.cpp:1083`, `oracle/oracle/code/cgame/cg_main.cpp:94-115`
pub struct CgConsoleCommand;

impl InboundVmCall for CgConsoleCommand {
    type Command = SpCgameExport;
    type Args = ();
    type Output = qboolean;

    const COMMAND: SpCgameExport = SpCgameExport::CG_CONSOLE_COMMAND;
}

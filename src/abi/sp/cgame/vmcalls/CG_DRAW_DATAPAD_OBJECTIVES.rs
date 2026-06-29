use super::super::SpCgameExport;
use crate::abi::generic::InboundVmCall;

/// `CG_DRAW_DATAPAD_OBJECTIVES` SP cgame exports vmMain ABI token.
///
/// Raven: `void CG_DrawDataPadObjectives( const centity_t *cent );`
/// Enum value source: `oracle/oracle/code/client/vmachine.h:34`
/// Args source: `oracle/oracle/code/cgame/cg_main.cpp:81`, `oracle/oracle/code/cgame/cg_main.cpp:145`
/// Output source: `oracle/oracle/code/cgame/cg_main.cpp:145`
/// VM_Call/vmMain switch source: `oracle/oracle/code/client/cl_ui.cpp:175`,
/// `oracle/oracle/code/cgame/cg_main.cpp:137-157`
pub struct CgDrawDatapadObjectives;

impl InboundVmCall for CgDrawDatapadObjectives {
    type Command = SpCgameExport;
    type Args = ();
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_DRAW_DATAPAD_OBJECTIVES;
}

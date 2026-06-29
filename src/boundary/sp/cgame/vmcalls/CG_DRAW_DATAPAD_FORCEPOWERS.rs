use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_DRAW_DATAPAD_FORCEPOWERS` SP cgame exports vmMain boundary token.
///
/// Raven: `void CG_DrawDataPadForceSelect( void );`
/// Enum value source: `oracle/oracle/code/client/vmachine.h:37`
/// Args source: `oracle/oracle/code/cgame/cg_main.cpp:84`, `oracle/oracle/code/cgame/cg_main.cpp:171`
/// Output source: `oracle/oracle/code/cgame/cg_main.cpp:171`
/// VM_Call/vmMain switch source: `oracle/oracle/code/client/cl_ui.cpp:184`, `oracle/oracle/code/cgame/cg_main.cpp:94-175`
pub struct CgDrawDatapadForcepowers;

impl InboundVmCall for CgDrawDatapadForcepowers {
    type Command = SpCgameExport;
    type Args = ();
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_DRAW_DATAPAD_FORCEPOWERS;
}

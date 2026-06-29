use super::super::SpCgameExport;
use crate::abi::generic::InboundVmCall;

/// `CG_DRAW_DATAPAD_HUD` SP cgame exports vmMain ABI token.
///
/// Raven: Ghoul2 Insert End
/// Raven: `void CG_DrawDataPadHUD( centity_t *cent );`
/// Enum value source: `oracle/oracle/code/client/vmachine.h:33`
/// Args source: `oracle/oracle/code/cgame/cg_main.cpp:80`, `oracle/oracle/code/cgame/cg_main.cpp:141`
/// Output source: `oracle/oracle/code/cgame/cg_main.cpp:141`
/// VM_Call/vmMain switch source: `oracle/oracle/code/client/cl_ui.cpp:172`, `oracle/oracle/code/cgame/cg_main.cpp:94-115`
pub struct CgDrawDatapadHud;

impl InboundVmCall for CgDrawDatapadHud {
    type Command = SpCgameExport;
    type Args = ();
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_DRAW_DATAPAD_HUD;
}

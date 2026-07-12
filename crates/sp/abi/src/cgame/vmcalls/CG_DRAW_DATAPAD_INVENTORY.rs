use super::super::SpCgameExport;
use abi_transport::generic::InboundVmCall;

/// `CG_DRAW_DATAPAD_INVENTORY` SP cgame exports vmMain ABI token.
///
/// Raven: `void CG_DrawDataPadInventorySelect( void );`
/// Enum value source: `oracle/code/client/vmachine.h:36`
/// Args source: `oracle/code/cgame/cg_main.cpp:3620`, `oracle/code/cgame/cg_main.cpp:164`
/// Output source: `oracle/code/cgame/cg_main.cpp:164`
/// VM_Call/vmMain switch source: `oracle/code/client/cl_ui.cpp:181`, `oracle/code/cgame/cg_main.cpp:94-175`
pub struct CgDrawDatapadInventory;

impl InboundVmCall for CgDrawDatapadInventory {
    type Command = SpCgameExport;
    type Args = ();
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_DRAW_DATAPAD_INVENTORY;
}

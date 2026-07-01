use super::super::SpCgameExport;
use abi_transport::generic::InboundVmCall;

/// `CG_DRAW_DATAPAD_WEAPONS` SP cgame exports vmMain ABI token.
///
/// Raven: `void CG_DrawDataPadWeaponSelect( void );`
/// Enum value source: `oracle/oracle/code/client/vmachine.h:35`
/// Args source: `oracle/oracle/code/cgame/cg_main.cpp:83`, `oracle/oracle/code/cgame/cg_main.cpp:153`
/// Output source: `oracle/oracle/code/cgame/cg_main.cpp:157`
/// VM_Call/vmMain switch source: `oracle/oracle/code/client/cl_ui.cpp:178`,
/// `oracle/oracle/code/cgame/cg_main.cpp:153-158`
pub struct CgDrawDatapadWeapons;

impl InboundVmCall for CgDrawDatapadWeapons {
    type Command = SpCgameExport;
    type Args = ();
    type Output = ();

    const COMMAND: SpCgameExport = SpCgameExport::CG_DRAW_DATAPAD_WEAPONS;
}

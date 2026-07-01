use core::ffi::c_int;

use super::super::SpCgameExport;
use abi_transport::generic::InboundVmCall;

/// `CG_CROSSHAIR_PLAYER` SP cgame exports vmMain ABI token.
///
/// Raven: `int CG_CrosshairPlayer( void );`
/// Enum value source: `oracle/oracle/code/client/vmachine.h:18`
/// Args source: `oracle/oracle/code/cgame/cg_local.h:648`, `oracle/oracle/code/cgame/cg_main.cpp:110`
/// Output source: `oracle/oracle/code/cgame/cg_main.cpp:110`, `oracle/oracle/code/cgame/cg_main.cpp:464`,
/// `oracle/oracle/code/cgame/cg_local.h:648`
/// VM_Call/vmMain switch source: `oracle/oracle/code/cgame/cg_main.cpp:94-115`.
/// No direct engine `VM_Call` site was found in the sampled C++ transport call sites; this is a pure vmMain export.
pub struct CgCrosshairPlayer;

impl InboundVmCall for CgCrosshairPlayer {
    type Command = SpCgameExport;
    type Args = ();
    type Output = c_int;

    const COMMAND: SpCgameExport = SpCgameExport::CG_CROSSHAIR_PLAYER;
}

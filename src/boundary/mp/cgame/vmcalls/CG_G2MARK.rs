use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_G2MARK` MP cgame exports vmMain boundary token.
///
/// Raven: shared-buffer payload `TCGG2Mark` carries `shader`, `size`, `start`, and `dir`.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:410`
/// Shared-buffer source: `oracle/oracle/codemp/cgame/cg_public.h:498-504`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:252-254`
/// Args source: `oracle/oracle/codemp/client/FxSystem.h:161-170`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:252-254`
/// Transport/call-site source: `oracle/oracle/codemp/client/FxSystem.h:161-170`
pub struct CgG2mark;

impl InboundVmCall for CgG2mark {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args; payload is TCGG2Mark in cg.sharedBuffer/cl.mSharedMemory.
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_G2MARK;
}

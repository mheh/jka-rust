use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_G2MARK` MP cgame exports vmMain boundary token.
///
/// Raven: shared-buffer payload `TCGG2Mark` carries `shader`, `size`, `start`, and `dir`.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:410`
/// Shared-buffer source: `oracle/oracle/codemp/cgame/cg_public.h:498-504`
/// Args/source source: `oracle/oracle/codemp/cgame/cg_main.c:252-254`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:252-254`
/// Transport/switch source: `oracle/oracle/codemp/cgame/cg_main.c:252-254`
/// Transport/call-site source: `oracle/oracle/codemp/client/FxSystem.h:163-171`
/// Shared-buffer payload type source: `oracle/oracle/codemp/cgame/cg_public.h:498-504`
/// FIXME: create type `TCGG2Mark` in Rust from
/// `oracle/oracle/codemp/cgame/cg_public.h:498-504`.
pub struct CgG2mark;

impl InboundVmCall for CgG2mark {
    type Command = MpCgameExport;
    type Args = (); //FIXME: create type `TCGG2Mark` in Rust when payload modeling is added.
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_G2MARK;
}

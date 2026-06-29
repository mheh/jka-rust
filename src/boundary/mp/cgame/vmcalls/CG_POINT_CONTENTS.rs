use core::ffi::c_int;

use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_POINT_CONTENTS` MP cgame exports vmMain boundary token.
///
/// Raven: int CG_PointContents( const vec3_t point, int passEntityNum );
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:392-393`
/// Shared-buffer source: `oracle/oracle/codemp/cgame/cg_public.h:451-456`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:220-222`, `oracle/oracle/codemp/cgame/cg_main.c:362-366`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:220-222`, `oracle/oracle/codemp/cgame/cg_main.c:366`
/// Transport/switch source: `oracle/oracle/codemp/cgame/cg_main.c:220-222`
/// Transport/call-site source: `oracle/oracle/codemp/client/FxPrimitives.cpp:234-240`
/// Shared-buffer payload type source: `oracle/oracle/codemp/cgame/cg_public.h:451-456`
/// FIXME: create type `TCGPointContents` in Rust from
/// `oracle/oracle/codemp/cgame/cg_public.h:451-456`.
pub struct CgPointContents;

impl InboundVmCall for CgPointContents {
    type Command = MpCgameExport;
    type Args = (); //FIXME: create type `TCGPointContents` in Rust when this payload is modeled.
    type Output = c_int;

    const COMMAND: MpCgameExport = MpCgameExport::CG_POINT_CONTENTS;
}

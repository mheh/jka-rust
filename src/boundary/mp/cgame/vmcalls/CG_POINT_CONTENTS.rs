use core::ffi::c_int;

use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_POINT_CONTENTS` MP cgame exports vmMain boundary token.
///
/// Raven: int CG_PointContents( const vec3_t point, int passEntityNum );
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:392-393`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:362-366`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:362-366`
/// Transport/call-site source: `oracle/oracle/codemp/client/FxPrimitives.cpp:234-240`
pub struct CgPointContents;

impl InboundVmCall for CgPointContents {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args; payload is TCGPointContents in cg.sharedBuffer/cl.mSharedMemory.
    type Output = c_int;

    const COMMAND: MpCgameExport = MpCgameExport::CG_POINT_CONTENTS;
}

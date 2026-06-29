// Shared-buffer payload lives in cg.sharedBuffer / cl.mSharedMemory, not vmMain arg slots.
use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_GET_LERP_DATA` MP cgame exports vmMain boundary token.
///
/// Raven: shared-buffer payload TCGGetBoltData (mOrigin/mAngles/mScale outputs, mEntityNum input).
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:398`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:376-390`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:227-229`
/// Transport/call-site source: `oracle/oracle/codemp/client/FxSystem.cpp:96-102`
/// Raven note: this zeroes pitch/roll for players and some ridable vehicles on the engine side.
pub struct CgGetLerpData;

impl InboundVmCall for CgGetLerpData {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args; payload is TCGGetBoltData in cg.sharedBuffer/cl.mSharedMemory.
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_LERP_DATA;
}

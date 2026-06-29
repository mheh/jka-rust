// Shared-buffer payload lives in cg.sharedBuffer / cl.mSharedMemory, not vmMain arg slots.
use super::super::MpCgameExport;
use crate::abi::generic::InboundVmCall;

/// `CG_GET_LERP_DATA` MP cgame exports vmMain ABI token.
///
/// Raven: shared-buffer payload TCGGetBoltData (mOrigin/mAngles/mScale outputs, mEntityNum input).
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:398`
/// Shared-buffer source: `oracle/oracle/codemp/cgame/cg_public.h:458-466`
/// Args/source source: `oracle/oracle/codemp/cgame/cg_main.c:227-229`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:377-406`
/// Transport/switch source: `oracle/oracle/codemp/cgame/cg_main.c:227-229`
/// Transport/call-site source: `oracle/oracle/codemp/client/FxSystem.cpp:100-103`
/// FIXME: create type `TCGGetBoltData` in Rust from
/// `oracle/oracle/codemp/cgame/cg_public.h:458-466`.
/// Raven note: this zeroes pitch/roll for players and some ridable vehicles on the engine side.
pub struct CgGetLerpData;

impl InboundVmCall for CgGetLerpData {
    type Command = MpCgameExport;
    type Args = (); //FIXME: create type `TCGGetBoltData` in Rust when payload modeling is added.
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_LERP_DATA;
}

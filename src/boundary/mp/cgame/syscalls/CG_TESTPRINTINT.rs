use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_TESTPRINTINT` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:191`
/// Args candidate source: `oracle/oracle/codemp/cgame/cg_syscalls.c:513-514`
///
/// Raven's searched MP cgame engine switch handles shared `TRAP_TESTPRINTINT`
/// at `oracle/oracle/codemp/client/cl_cgame.cpp:678-679`, which corresponds to
/// `CGAME_TESTPRINTINT`, not this later enum token. Keep this stubbed until a
/// distinct `CG_TESTPRINTINT` transport source is found.
pub struct CgTestprintint;

impl OutboundSysCall for CgTestprintint {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_TESTPRINTINT;
}

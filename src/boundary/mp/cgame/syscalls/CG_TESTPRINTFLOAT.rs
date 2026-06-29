use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_TESTPRINTFLOAT` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:192`
/// Args candidate source: `oracle/oracle/codemp/cgame/cg_syscalls.c:517-518`
///
/// Raven's searched MP cgame engine switch handles shared `TRAP_TESTPRINTFLOAT`
/// at `oracle/oracle/codemp/client/cl_cgame.cpp:680-681`, which corresponds to
/// `CGAME_TESTPRINTFLOAT`, not this later enum token. Keep this stubbed until a
/// distinct `CG_TESTPRINTFLOAT` transport source is found.
pub struct CgTestprintfloat;

impl OutboundSysCall for CgTestprintfloat {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_TESTPRINTFLOAT;
}

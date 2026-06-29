use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_SETCOLOR`.
///
/// Raven wrapper: `syscall( CG_R_SETCOLOR, rgba );`
/// Raven transport: `re.SetColor((const float *) VMA(1));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:392-393`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:711-713`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRSetcolorArgs {
    rgba: *const f32,
}

impl CgRSetcolorArgs {
    pub const fn new(rgba: *const f32) -> Self {
        Self { rgba }
    }

    pub const fn rgba(&self) -> *const f32 {
        self.rgba
    }
}

/// `CG_R_SETCOLOR` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:140`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:392-393`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:711-713`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:711-713`
pub struct CgRSetcolor;

impl OutboundSysCall for CgRSetcolor {
    type Import = SpCgameImport;
    type Args = CgRSetcolorArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_SETCOLOR;
}

impl EncodeSysCall for CgRSetcolor {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.rgba())])
    }
}

impl DecodeSysCallReturn for CgRSetcolor {
    fn decode_return(_word: isize) -> Self::Output {}
}

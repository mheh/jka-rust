use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::vec3_t;

/// Arguments for `CGAME_PERPENDICULARVECTOR`.
///
/// Raven's engine switch casts both ABI words with `VMA`: `dst` is the
/// caller-provided `float *`/`vec3_t` output buffer, and `src` is the read-only
/// `const float *`/`vec3_t` input. The engine returns `0`; the perpendicular
/// vector is written through `dst`, so that out-buffer remains part of `Args`
/// rather than becoming `Output`.
///
/// Args source: `oracle/codemp/client/cl_cgame.cpp:672`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:673`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:671`
/// Shared trap token source: `oracle/codemp/qcommon/qcommon.h:291`
#[derive(Debug)]
pub struct CgamePerpendicularvectorArgs {
    dst: *mut vec3_t,
    src: *const vec3_t,
}

impl CgamePerpendicularvectorArgs {
    pub const fn new(dst: *mut vec3_t, src: *const vec3_t) -> Self {
        Self { dst, src }
    }

    pub const fn dst(&self) -> *mut vec3_t {
        self.dst
    }

    pub const fn src(&self) -> *const vec3_t {
        self.src
    }
}

/// `CGAME_PERPENDICULARVECTOR` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:139`
/// Args source: `oracle/codemp/client/cl_cgame.cpp:672`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:673`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:671`
/// Shared trap token source: `oracle/codemp/qcommon/qcommon.h:291`
pub struct CgamePerpendicularvector;

impl OutboundSysCall for CgamePerpendicularvector {
    type Import = MpCgameImport;
    type Args = CgamePerpendicularvectorArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_PERPENDICULARVECTOR;
}

impl EncodeSysCall for CgamePerpendicularvector {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.dst()), ptr_to_word(args.src())])
    }
}

impl DecodeSysCallReturn for CgamePerpendicularvector {
    // `PerpendicularVector` writes through `dst`; Raven returns 0 from the syscall arm.
    fn decode_return(_word: isize) -> Self::Output {}
}

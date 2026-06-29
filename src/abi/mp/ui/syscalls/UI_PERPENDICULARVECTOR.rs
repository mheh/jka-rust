use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;

/// Arguments for `UI_PERPENDICULARVECTOR`.
///
/// Raven's engine switch casts both ABI words with `VMA`: `dst` is the
/// caller-provided `float *`/`vec3_t` output buffer, and `src` is the read-only
/// `const float *`/`vec3_t` input. The engine returns `0`; the perpendicular
/// vector is written through `dst`, so that out-buffer remains part of `Args`
/// rather than becoming `Output`.
///
/// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:672`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:673`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:671`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:291`
#[derive(Debug)]
pub struct UiPerpendicularvectorArgs {
    dst: *mut vec3_t,
    src: *const vec3_t,
}

impl UiPerpendicularvectorArgs {
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

/// `UI_PERPENDICULARVECTOR` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:139`
/// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:672`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:673`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:671`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:291`
pub struct UiPerpendicularvector;

impl OutboundSysCall for UiPerpendicularvector {
    type Import = MpUiImport;
    type Args = UiPerpendicularvectorArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_PERPENDICULARVECTOR;
}

impl EncodeSysCall for UiPerpendicularvector {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.dst()), ptr_to_word(args.src())])
    }
}

impl DecodeSysCallReturn for UiPerpendicularvector {
    // `PerpendicularVector` writes through `dst`; Raven returns 0 from the syscall arm.
    fn decode_return(_word: isize) -> Self::Output {}
}

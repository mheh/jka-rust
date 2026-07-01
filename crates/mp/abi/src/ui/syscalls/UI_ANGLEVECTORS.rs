use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::vec3_t;

/// Arguments for `UI_ANGLEVECTORS`.
///
/// Raven's engine switch casts all four ABI words with `VMA`. `angles` is the
/// read-only `const float *`/`vec3_t` input, while `forward`, `right`, and `up`
/// are caller-provided `vec3_t` output buffers. The engine returns `0`; the
/// computed vectors are written through those out-pointers, so they remain part
/// of `Args` rather than becoming `Output`.
///
/// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:669`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:670`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:668`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:290`
#[derive(Debug)]
pub struct UiAnglevectorsArgs {
    angles: *const vec3_t,
    forward: *mut vec3_t,
    right: *mut vec3_t,
    up: *mut vec3_t,
}

impl UiAnglevectorsArgs {
    pub const fn new(
        angles: *const vec3_t,
        forward: *mut vec3_t,
        right: *mut vec3_t,
        up: *mut vec3_t,
    ) -> Self {
        Self {
            angles,
            forward,
            right,
            up,
        }
    }

    pub const fn angles(&self) -> *const vec3_t {
        self.angles
    }

    pub const fn forward(&self) -> *mut vec3_t {
        self.forward
    }

    pub const fn right(&self) -> *mut vec3_t {
        self.right
    }

    pub const fn up(&self) -> *mut vec3_t {
        self.up
    }
}

/// `UI_ANGLEVECTORS` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:138`
/// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:669`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:670`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:668`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:290`
pub struct UiAnglevectors;

impl OutboundSysCall for UiAnglevectors {
    type Import = MpUiImport;
    type Args = UiAnglevectorsArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_ANGLEVECTORS;
}

impl EncodeSysCall for UiAnglevectors {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.angles()),
            ptr_to_word(args.forward()),
            ptr_to_word(args.right()),
            ptr_to_word(args.up()),
        ])
    }
}

impl DecodeSysCallReturn for UiAnglevectors {
    // `AngleVectors` writes through `forward`/`right`/`up`; Raven returns 0 from the syscall arm.
    fn decode_return(_word: isize) -> Self::Output {}
}

use super::super::MpUiImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::abi::pass_float;

/// Arguments for `UI_R_ADDLIGHTTOSCENE`.
///
/// C ABI: `void trap_R_AddLightToScene(const vec3_t org, float intensity, float r, float g, float b)`.
/// Raven's client switch forwards the origin through `VMA(1)` and packs the
/// four scalar values as float words.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:182-183`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:182-183`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:968-972`
#[derive(Debug, Clone, Copy)]
pub struct UiRAddlighttosceneArgs {
    pub origin: *const f32,
    pub intensity: f32,
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl UiRAddlighttosceneArgs {
    pub const fn new(origin: *const f32, intensity: f32, red: f32, green: f32, blue: f32) -> Self {
        Self {
            origin,
            intensity,
            red,
            green,
            blue,
        }
    }
}

/// `UI_R_ADDLIGHTTOSCENE` MP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:43`
pub struct UiRAddlighttoscene;

impl OutboundSysCall for UiRAddlighttoscene {
    type Import = MpUiImport;
    type Args = UiRAddlighttosceneArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_R_ADDLIGHTTOSCENE;
}

impl EncodeSysCall for UiRAddlighttoscene {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            crate::abi::generic::ptr_to_word(args.origin),
            pass_float(args.intensity),
            pass_float(args.red),
            pass_float(args.green),
            pass_float(args.blue),
        ])
    }
}

impl DecodeSysCallReturn for UiRAddlighttoscene {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}

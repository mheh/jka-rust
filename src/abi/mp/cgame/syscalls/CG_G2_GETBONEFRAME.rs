use core::ffi::{c_char, c_int, c_void};

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qboolean;

/// Arguments for `CG_G2_GETBONEFRAME`.
///
/// Raven wrapper: `return syscall(CG_G2_GETBONEFRAME, ghoul2, boneName, currentTime, currentFrame, modelList, modelIndex);`
/// Raven transport: raw `ghoul2` in `args[1]`, `boneName` via `VMA(2)`,
/// `currentFrame` via `VMA(4)`, and `modelList` via `VMA(5)`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:880-882`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1395-1404`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2GetboneframeArgs {
    ghoul2: *mut c_void,
    bone_name: *const c_char,
    current_time: c_int,
    current_frame: *mut f32,
    model_list: *mut c_int,
    model_index: c_int,
}

impl CgG2GetboneframeArgs {
    pub const fn new(
        ghoul2: *mut c_void,
        bone_name: *const c_char,
        current_time: c_int,
        current_frame: *mut f32,
        model_list: *mut c_int,
        model_index: c_int,
    ) -> Self {
        Self {
            ghoul2,
            bone_name,
            current_time,
            current_frame,
            model_list,
            model_index,
        }
    }
}

/// `CG_G2_GETBONEFRAME` MP cgame imports syscall ABI token.
///
/// Raven: trimmed down version of GBA, so I don't have to pass all those unused args across the VM-exe border
/// Raven: `//rwwFIXMEFIXME: Just make a G2API_GetBoneFrame func too. This is dirty.`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:271`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:880-882`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:880-882`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1395-1404`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1395-1404`
pub struct CgG2Getboneframe;

impl OutboundSysCall for CgG2Getboneframe {
    type Import = MpCgameImport;
    type Args = CgG2GetboneframeArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_GETBONEFRAME;
}

impl EncodeSysCall for CgG2Getboneframe {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            ptr_to_word(args.bone_name),
            args.current_time as isize,
            ptr_to_word(args.current_frame),
            ptr_to_word(args.model_list),
            args.model_index as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgG2Getboneframe {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}

use core::ffi::{c_char, c_int, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qhandle_t;

/// Arguments for `CG_G2_INITGHOUL2MODEL`.
///
/// Raven wrapper: `return syscall(CG_G2_INITGHOUL2MODEL, ghoul2Ptr, fileName, modelIndex, customSkin, customShader, modelFlags, lodBias);`
/// Raven transport: `return G2API_InitGhoul2Model((CGhoul2Info_v **)VMA(1), (const char *)VMA(2), args[3], (qhandle_t) args[4], (qhandle_t) args[5], args[6], args[7]);`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:809-812`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2525-2526`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1324-1329`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2Initghoul2modelArgs {
    ghoul2_ptr: *mut *mut c_void,
    file_name: *const c_char,
    model_index: c_int,
    custom_skin: qhandle_t,
    custom_shader: qhandle_t,
    model_flags: c_int,
    lod_bias: c_int,
}

impl CgG2Initghoul2modelArgs {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        ghoul2_ptr: *mut *mut c_void,
        file_name: *const c_char,
        model_index: c_int,
        custom_skin: qhandle_t,
        custom_shader: qhandle_t,
        model_flags: c_int,
        lod_bias: c_int,
    ) -> Self {
        Self {
            ghoul2_ptr,
            file_name,
            model_index,
            custom_skin,
            custom_shader,
            model_flags,
            lod_bias,
        }
    }
}

/// `CG_G2_INITGHOUL2MODEL` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:263`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:809-812`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1324-1329`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1324-1329`
pub struct CgG2Initghoul2model;

impl OutboundSysCall for CgG2Initghoul2model {
    type Import = MpCgameImport;
    type Args = CgG2Initghoul2modelArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_INITGHOUL2MODEL;
}

impl EncodeSysCall for CgG2Initghoul2model {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2_ptr as *const _),
            ptr_to_word(args.file_name),
            args.model_index as isize,
            args.custom_skin as isize,
            args.custom_shader as isize,
            args.model_flags as isize,
            args.lod_bias as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgG2Initghoul2model {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}

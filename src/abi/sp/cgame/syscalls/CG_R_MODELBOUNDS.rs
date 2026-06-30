use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qhandle_t;
use crate::shared::vec3_t;

/// Arguments for `CG_R_MODELBOUNDS`.
///
/// Raven wrapper: `cgi_R_ModelBounds( qhandle_t model, vec3_t mins, vec3_t maxs )`
/// Raven transport: `re.ModelBounds( args[1], (float *) VMA(2), (float *) VMA(3) );`
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:143`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:406`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:720-722`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgRModelboundsArgs {
    model: qhandle_t,
    mins: *mut vec3_t,
    maxs: *mut vec3_t,
}

impl CgRModelboundsArgs {
    pub const fn new(model: qhandle_t, mins: *mut vec3_t, maxs: *mut vec3_t) -> Self {
        Self { model, mins, maxs }
    }

    pub const fn model(&self) -> qhandle_t {
        self.model
    }

    pub const fn mins(&self) -> *mut vec3_t {
        self.mins
    }

    pub const fn maxs(&self) -> *mut vec3_t {
        self.maxs
    }
}

/// `CG_R_MODELBOUNDS` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:143`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:406`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:720-722`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:720-722`
pub struct CgRModelbounds;

impl OutboundSysCall for CgRModelbounds {
    type Import = SpCgameImport;
    type Args = CgRModelboundsArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_MODELBOUNDS;
}

impl EncodeSysCall for CgRModelbounds {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.model() as isize,
            ptr_to_word(args.mins()),
            ptr_to_word(args.maxs()),
        ])
    }
}

impl DecodeSysCallReturn for CgRModelbounds {
    fn decode_return(_word: isize) -> Self::Output {}
}

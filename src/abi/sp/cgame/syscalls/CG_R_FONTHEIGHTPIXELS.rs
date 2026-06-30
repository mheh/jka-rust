use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::syscalls::pass_float;

/// `CG_R_FONTHEIGHTPIXELS` SP cgame imports syscall ABI token.
///
/// Arguments for `CG_R_FONTHEIGHTPIXELS`.
///
/// Raven wrapper: `return syscall( CG_R_FONTHEIGHTPIXELS, iFontIndex, PASSFLOAT(scale) );`
/// Raven transport: `return re.Font_HeightPixels( args[1], VMF(2) );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:333-334`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:669-670`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgRFontheightpixelsArgs {
    font_index: c_int,
    scale: f32,
}

impl CgRFontheightpixelsArgs {
    pub const fn new(font_index: c_int, scale: f32) -> Self {
        Self { font_index, scale }
    }
}

/// `CG_R_FONTHEIGHTPIXELS` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:125`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:333-334`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:669-670`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:669-670`
pub struct CgRFontheightpixels;

impl OutboundSysCall for CgRFontheightpixels {
    type Import = SpCgameImport;
    type Args = CgRFontheightpixelsArgs;
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_FONTHEIGHTPIXELS;
}

impl EncodeSysCall for CgRFontheightpixels {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.font_index as isize, pass_float(args.scale)])
    }
}

impl DecodeSysCallReturn for CgRFontheightpixels {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}

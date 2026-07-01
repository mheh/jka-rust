use super::super::MpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use abi_transport::pass_float;

/// Arguments for `CG_FX_DRAW_2D_EFFECTS`.
///
/// Raven wrapper: `syscall( CG_FX_DRAW_2D_EFFECTS, PASSFLOAT(screenXScale), PASSFLOAT(screenYScale) );`
/// Raven transport: `FX_Draw2DEffects ( VMF(1), VMF(2) ); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:664-666`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2406`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1145-1147`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgFxDraw2dEffectsArgs {
    screen_x_scale: f32,
    screen_y_scale: f32,
}

impl CgFxDraw2dEffectsArgs {
    pub const fn new(screen_x_scale: f32, screen_y_scale: f32) -> Self {
        Self {
            screen_x_scale,
            screen_y_scale,
        }
    }

    pub const fn screen_x_scale(&self) -> f32 {
        self.screen_x_scale
    }

    pub const fn screen_y_scale(&self) -> f32 {
        self.screen_y_scale
    }
}

/// `CG_FX_DRAW_2D_EFFECTS` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:231`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:664-666`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1145-1147`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1145-1147`
pub struct CgFxDraw2dEffects;

impl OutboundSysCall for CgFxDraw2dEffects {
    type Import = MpCgameImport;
    type Args = CgFxDraw2dEffectsArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_DRAW_2D_EFFECTS;
}

impl EncodeSysCall for CgFxDraw2dEffects {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            pass_float(args.screen_x_scale()),
            pass_float(args.screen_y_scale()),
        ])
    }
}

impl DecodeSysCallReturn for CgFxDraw2dEffects {
    fn decode_return(_word: isize) -> Self::Output {}
}

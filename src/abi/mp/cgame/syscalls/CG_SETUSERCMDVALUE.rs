use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::syscalls::pass_float;
use crate::shared::qboolean;

/// Arguments for `CG_SETUSERCMDVALUE`.
///
/// Raven's cgame wrapper sends eight payload words:
/// `stateValue`, four `PASSFLOAT` float words, `fpSel`, `invenSel`, and
/// `fighterControls`. The engine switch reads the same transport shape as
/// `args[1]`, `VMF(2)` through `VMF(5)`, `args[6]`, `args[7]`, and `args[8]`.
/// `fighterControls` is consumed by the switch as `cl_bUseFighterPitch`; it is
/// not part of the seven-argument `CL_SetUserCmdValue` call.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:494-495`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:973-975`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgSetusercmdvalueArgs {
    state_value: c_int,
    sensitivity_scale: f32,
    m_pitch_override: f32,
    m_yaw_override: f32,
    m_sensitivity_override: f32,
    fp_sel: c_int,
    inven_sel: c_int,
    fighter_controls: qboolean,
}

impl CgSetusercmdvalueArgs {
    pub const fn new(
        state_value: c_int,
        sensitivity_scale: f32,
        m_pitch_override: f32,
        m_yaw_override: f32,
        m_sensitivity_override: f32,
        fp_sel: c_int,
        inven_sel: c_int,
        fighter_controls: qboolean,
    ) -> Self {
        Self {
            state_value,
            sensitivity_scale,
            m_pitch_override,
            m_yaw_override,
            m_sensitivity_override,
            fp_sel,
            inven_sel,
            fighter_controls,
        }
    }

    pub const fn state_value(&self) -> c_int {
        self.state_value
    }

    pub const fn sensitivity_scale(&self) -> f32 {
        self.sensitivity_scale
    }

    pub const fn m_pitch_override(&self) -> f32 {
        self.m_pitch_override
    }

    pub const fn m_yaw_override(&self) -> f32 {
        self.m_yaw_override
    }

    pub const fn m_sensitivity_override(&self) -> f32 {
        self.m_sensitivity_override
    }

    pub const fn fp_sel(&self) -> c_int {
        self.fp_sel
    }

    pub const fn inven_sel(&self) -> c_int {
        self.inven_sel
    }

    pub const fn fighter_controls(&self) -> qboolean {
        self.fighter_controls
    }
}

/// `CG_SETUSERCMDVALUE` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `void trap_SetUserCmdValue(...)` forwards four floats with
/// `PASSFLOAT`. Raven switch returns `0` after updating `cl_bUseFighterPitch`
/// and calling `CL_SetUserCmdValue`.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:187`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:494-495`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:973-975`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:973-975`
pub struct CgSetusercmdvalue;

impl OutboundSysCall for CgSetusercmdvalue {
    type Import = MpCgameImport;
    type Args = CgSetusercmdvalueArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_SETUSERCMDVALUE;
}

impl EncodeSysCall for CgSetusercmdvalue {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.state_value() as isize,
            pass_float(args.sensitivity_scale()),
            pass_float(args.m_pitch_override()),
            pass_float(args.m_yaw_override()),
            pass_float(args.m_sensitivity_override()),
            args.fp_sel() as isize,
            args.inven_sel() as isize,
            args.fighter_controls() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgSetusercmdvalue {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}

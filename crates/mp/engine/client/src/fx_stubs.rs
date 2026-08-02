//! Pending-lane declarations for the FX system.
//!
//! Ticket gh#26 rules the FX subsystem design, and ticket gh#27 ports the
//! system.
//! DEC-55.2 routes the 36 cgame FX trap arms here, and porting-rules §17 puts
//! the design ruling before transcription.
//! Each function below declares the shape the `cl_cgame` arms need and panics
//! when it runs.
//! No stub here is a silent no-op.
//!
//! Four arms (`CG_FX_ADDPOLY`, `CG_FX_ADDBEZIER`, `CG_FX_ADDPRIMITIVE`, and
//! `CG_FX_ADDELECTRICITY`) pass the whole argument struct, where Raven expands
//! the struct into a field list.
//! Ticket gh#27 restores the field list when it lands the real signatures.
//!
//! Source: `oracle/codemp/client/FXExport.h`, `oracle/codemp/client/FxUtil.h`

use core::ffi::c_int;

use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::shared::add_electricity_arg::addElectricityArgStruct_t;
use mp_qshared::shared::addbezier_arg::addbezierArgStruct_t;
use mp_qshared::shared::addpoly_arg::addpolyArgStruct_t;
use mp_qshared::shared::effect_trail_arg::effectTrailArgStruct_t;
use native_math::vector::vec3_t;
use native_types::qboolean;

//TODO: Port FX_RegisterEffect
// Source: oracle/codemp/client/FXExport.h:4
pub unsafe fn FX_RegisterEffect(_file: &str) -> c_int {
    todo!("Port FX_RegisterEffect — oracle/codemp/client/FxScheduler.cpp (gh#26/gh#27)")
}

//TODO: Port FX_PlayEffect
// Source: oracle/codemp/client/FXExport.h:6
pub unsafe fn FX_PlayEffect(
    _file: &str,
    _org: *mut f32,
    _fwd: *mut f32,
    _vol: c_int,
    _rad: c_int,
) {
    todo!("Port FX_PlayEffect — oracle/codemp/client/FxScheduler.cpp (gh#26/gh#27)")
}

//TODO: Port FX_PlayEffectID
// Source: oracle/codemp/client/FXExport.h:8
pub unsafe fn FX_PlayEffectID(
    _id: c_int,
    _org: *mut f32,
    _fwd: *mut f32,
    _vol: c_int,
    _rad: c_int,
    _isPortal: qboolean,
) {
    todo!("Port FX_PlayEffectID — oracle/codemp/client/FxScheduler.cpp (gh#26/gh#27)")
}

//TODO: Port FX_PlayEntityEffectID
// Source: oracle/codemp/client/FXExport.h:9-10
pub unsafe fn FX_PlayEntityEffectID(
    _id: c_int,
    _org: *mut f32,
    _axis: *mut vec3_t,
    _boltInfo: c_int,
    _entNum: c_int,
    _vol: c_int,
    _rad: c_int,
) {
    todo!("Port FX_PlayEntityEffectID — oracle/codemp/client/FxScheduler.cpp (gh#26/gh#27)")
}

//TODO: Port FX_PlayBoltedEffectID
// Source: oracle/codemp/client/FXExport.h:11-12
pub unsafe fn FX_PlayBoltedEffectID(
    _id: c_int,
    _org: *mut f32,
    _boltInfo: c_int,
    _ghoul2: *mut core::ffi::c_void,
    _iLoopTime: c_int,
    _isRelative: qboolean,
) {
    todo!("Port FX_PlayBoltedEffectID — oracle/codemp/client/FxScheduler.cpp (gh#26/gh#27)")
}

//TODO: Port FX_AddScheduledEffects
// Source: oracle/codemp/client/FXExport.h:14
pub unsafe fn FX_AddScheduledEffects(_portal: qboolean) {
    todo!("Port FX_AddScheduledEffects — oracle/codemp/client/FxScheduler.cpp (gh#26/gh#27)")
}

//TODO: Port FX_Draw2DEffects
// Source: oracle/codemp/client/FXExport.h:15
pub unsafe fn FX_Draw2DEffects(_screenXScale: f32, _screenYScale: f32) {
    todo!("Port FX_Draw2DEffects — oracle/codemp/client/FxUtil.cpp (gh#26/gh#27)")
}

//TODO: Port FX_InitSystem
// Source: oracle/codemp/client/FXExport.h:17
pub unsafe fn FX_InitSystem(_refdef: *mut refdef_t) -> c_int {
    todo!("Port FX_InitSystem — oracle/codemp/client/FxUtil.cpp (gh#26/gh#27)")
}

//TODO: Port FX_SetRefDefFromCGame
// Source: oracle/codemp/client/FXExport.h:18
pub unsafe fn FX_SetRefDefFromCGame(_refdef: *mut refdef_t) {
    todo!("Port FX_SetRefDefFromCGame — oracle/codemp/client/FxUtil.cpp (gh#26/gh#27)")
}

//TODO: Port FX_FreeSystem
// Source: oracle/codemp/client/FXExport.h:19
pub unsafe fn FX_FreeSystem() -> c_int {
    todo!("Port FX_FreeSystem — oracle/codemp/client/FxUtil.cpp (gh#26/gh#27)")
}

//TODO: Port FX_AdjustTime
// Source: oracle/codemp/client/FXExport.h:20
pub unsafe fn FX_AdjustTime(_time: c_int) {
    todo!("Port FX_AdjustTime — oracle/codemp/client/FxUtil.cpp (gh#26/gh#27)")
}

//TODO: Port FX_Free
// Source: oracle/codemp/client/FxUtil.h:10
pub unsafe fn FX_Free(_templates: bool) -> bool {
    todo!("Port FX_Free — oracle/codemp/client/FxUtil.cpp (gh#26/gh#27)")
}

//TODO: Port FX_AddParticle
// Source: oracle/codemp/client/FxUtil.h:17-27
#[allow(clippy::too_many_arguments)]
pub unsafe fn FX_AddParticle(
    _org: vec3_t,
    _vel: vec3_t,
    _accel: vec3_t,
    _size1: f32,
    _size2: f32,
    _sizeParm: f32,
    _alpha1: f32,
    _alpha2: f32,
    _alphaParm: f32,
    _sRGB: vec3_t,
    _eRGB: vec3_t,
    _rgbParm: f32,
    _rotation: f32,
    _rotationDelta: f32,
    _min: vec3_t,
    _max: vec3_t,
    _elasticity: f32,
    _deathID: c_int,
    _impactID: c_int,
    _killTime: c_int,
    _shader: c_int,
    _flags: c_int,
) {
    todo!("Port FX_AddParticle — oracle/codemp/client/FxUtil.cpp (gh#26/gh#27)")
}

//TODO: Port FX_AddLine
// Source: oracle/codemp/client/FxUtil.h:28-35
#[allow(clippy::too_many_arguments)]
pub unsafe fn FX_AddLine(
    _start: *mut f32,
    _end: *mut f32,
    _size1: f32,
    _size2: f32,
    _sizeParm: f32,
    _alpha1: f32,
    _alpha2: f32,
    _alphaParm: f32,
    _sRGB: *mut f32,
    _eRGB: *mut f32,
    _rgbParm: f32,
    _killTime: c_int,
    _shader: c_int,
    _flags: c_int,
) {
    todo!("Port FX_AddLine — oracle/codemp/client/FxUtil.cpp (gh#26/gh#27)")
}

//TODO: Port FX_AddElectricity
// Source: oracle/codemp/client/FxUtil.h:36-43
pub unsafe fn FX_AddElectricity(_args: addElectricityArgStruct_t) {
    todo!("Port FX_AddElectricity — oracle/codemp/client/FxUtil.cpp (gh#26/gh#27)")
}

//TODO: Port FX_AddPoly
// Source: oracle/codemp/client/FxUtil.h:94-100
pub unsafe fn FX_AddPoly(_args: addpolyArgStruct_t) {
    todo!("Port FX_AddPoly — oracle/codemp/client/FxUtil.cpp (gh#26/gh#27)")
}

//TODO: Port FX_AddBezier
// Source: oracle/codemp/client/FxUtil.h:108-115
pub unsafe fn FX_AddBezier(_args: addbezierArgStruct_t) {
    todo!("Port FX_AddBezier — oracle/codemp/client/FxUtil.cpp (gh#26/gh#27)")
}

//TODO: Port FX_FeedTrail
// Source: oracle/codemp/client/FxPrimitives.cpp:2315
pub unsafe fn FX_FeedTrail(_args: *mut effectTrailArgStruct_t) {
    todo!("Port FX_FeedTrail — oracle/codemp/client/FxUtil.cpp (gh#26/gh#27)")
}

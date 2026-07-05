#![allow(non_camel_case_types)]

/// Raven `vehFieldType_t` — parse-table value-kind tag for `vehField_t`
/// entries (`vehWeaponFields`/`vehicleFields`).
///
/// Type definition source: `oracle/oracle/codemp/game/bg_vehicleLoad.c:114-129`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum vehFieldType_t {
    VF_IGNORE,
    VF_INT,
    VF_FLOAT,
    /// string on disk, pointer in memory, TAG_LEVEL
    VF_LSTRING,
    VF_VECTOR,
    VF_BOOL,
    VF_VEHTYPE,
    VF_ANIM,
    /// take string, resolve into index into VehWeaponParms
    VF_WEAPON,
    /// take the string, get the G_ModelIndex
    VF_MODEL,
    /// (cgame only) take the string, get the G_ModelIndex
    VF_MODEL_CLIENT,
    /// take the string, get the G_EffectIndex
    VF_EFFECT,
    /// (cgame only) take the string, get the index
    VF_EFFECT_CLIENT,
    /// (cgame only) take the string, call trap_R_RegisterShader
    VF_SHADER,
    /// (cgame only) take the string, call trap_R_RegisterShaderNoMip
    VF_SHADER_NOMIP,
    /// take the string, get the G_SoundIndex
    VF_SOUND,
    /// (cgame only) take the string, get the index
    VF_SOUND_CLIENT,
}

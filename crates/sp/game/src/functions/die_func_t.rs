#![allow(non_camel_case_types, non_snake_case)]

/// Raven `dieFunc_t` — enumeration of entity death callback function IDs.
///
/// Type definition source: `oracle/code/game/g_functions.h:562-588`
#[repr(i32)]
pub enum dieFunc_t {
    dieF_NULL = 0,
    //
    dieF_funcBBrushDie,
    dieF_misc_model_breakable_die,
    dieF_misc_model_cargo_die,
    dieF_func_train_die,
    dieF_player_die,
    dieF_ExplodeDeath_Wait,
    dieF_ExplodeDeath,
    dieF_func_usable_die,
    dieF_turret_die,
    dieF_funcGlassDie,
    //	dieF_laserTrapDelayedExplode,
    dieF_emplaced_gun_die,
    dieF_WP_ExplosiveDie,
    dieF_ion_cannon_die,
    dieF_maglock_die,
    dieF_camera_die,
    dieF_Mark1_die,
    dieF_Interrogator_die,
    dieF_misc_atst_die,
    dieF_misc_panel_turret_die,
    dieF_thermal_die,
    dieF_eweb_die,
}

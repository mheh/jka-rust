#![allow(non_camel_case_types, non_snake_case)]

/// Raven `painFunc_t` — enumeration of entity pain/damage callback function IDs.
///
/// Type definition source: `oracle/oracle/code/game/g_functions.h:499-530`
#[repr(i32)]
pub enum painFunc_t {
    painF_NULL = 0,
    //
    painF_funcBBrushPain,
    painF_misc_model_breakable_pain,
    painF_NPC_Pain,
    painF_station_pain,
    painF_func_usable_pain,
    painF_NPC_ATST_Pain,
    painF_NPC_ST_Pain,
    painF_NPC_Jedi_Pain,
    painF_NPC_Droid_Pain,
    painF_NPC_Probe_Pain,
    painF_NPC_MineMonster_Pain,
    painF_NPC_Howler_Pain,
    painF_NPC_Rancor_Pain,
    painF_NPC_Wampa_Pain,
    painF_NPC_SandCreature_Pain,
    painF_NPC_Seeker_Pain,
    painF_NPC_Remote_Pain,
    painF_emplaced_gun_pain,
    painF_NPC_Mark1_Pain,
    painF_NPC_GM_Pain,
    painF_NPC_Sentry_Pain,
    painF_NPC_Mark2_Pain,
    painF_PlayerPain,
    painF_GasBurst,
    painF_CrystalCratePain,
    painF_TurretPain,
    painF_eweb_pain,
}

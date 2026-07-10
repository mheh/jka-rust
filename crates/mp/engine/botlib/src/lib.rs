//! `mp_engine_botlib` crate. //TODO: Port module mp_engine_botlib

// Raven-named functions/types (`Export_BotLibSetup`, `aasworld`, …) keep their
// original casing across the ABI seam, matching `mp_game`'s crate-level policy.
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod aasfile;
pub mod be_aas_bsp;
pub mod be_aas_bspq3;
pub mod be_aas_bspq3_fns;
pub mod be_aas_cluster;
pub mod be_aas_debug;
pub mod be_aas_def;
pub mod be_aas_entity;
pub mod be_aas_main;
pub mod be_aas_move;
pub mod be_aas_optimize;
pub mod be_aas_optimize_fns;
pub mod be_aas_reach;
pub mod be_aas_reach_fns;
pub mod be_aas_route;
pub mod be_aas_route_fns;
pub mod be_aas_routealt;
pub mod be_aas_routealt_fns;
pub mod be_aas_sample;
pub mod be_aas_sample_fns;
pub mod be_ai_char;
pub mod be_ai_chat;
pub mod be_ai_chat_fns;
pub mod be_ai_gen;
pub mod be_ai_goal;
pub mod be_ai_goal_fns;
pub mod be_ai_move;
pub mod be_ai_move_fns;
pub mod be_ai_weap;
pub mod be_ai_weap_fns;
pub mod be_ai_weight;
pub mod be_ea;
pub mod be_ea_fns;
pub mod be_interface;
pub mod l_crc;
pub mod l_crc_fns;
pub mod l_libvar;
pub mod l_libvar_fns;
pub mod l_log;
pub mod l_log_fns;
pub mod l_memory;
pub mod l_memory_fns;
pub mod l_precomp;
pub mod l_precomp_fns;
pub mod l_script;
pub mod l_script_fns;
pub mod l_struct;
pub mod l_struct_fns;

/// Synthesized fork-2 owner of botlib's file-scope globals (Raven's
/// `aasworld`, `botimport`, `be_botlib_export`, `libvarlist`, `bot_developer`,
/// …), threaded by `&mut BotLib` through every ported `be_*`/`l_*` function per
/// the state-threading rule (ruling 2). Not a Raven struct: botlib's globals
/// were scattered across its translation units. Fields land during integration
/// as the merge collects the porters' references; kept empty here so the mod
/// wiring compiles as a skeleton.
#[derive(Default)]
pub struct BotLib {}

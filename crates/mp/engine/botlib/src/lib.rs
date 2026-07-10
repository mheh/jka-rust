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

use core::ffi::{c_char, c_int, c_ushort};

use mp_qshared::common::mp::botlib::bot_consolemessage_s::bot_consolemessage_t;
use mp_qshared::common::mp::botlib::botlib_import_s::botlib_import_t;
use mp_qshared::shared::limits::{MAX_CLIENTS, MAX_MODELS};
use mp_qshared::shared::MAX_QPATH;

use crate::be_aas_def::aas_s::aas_t;
use crate::be_aas_def::aas_settings_s::aas_settings_t;
use crate::be_ai_chat::bot_chatstate_s::bot_chatstate_t;
use crate::be_ai_chat::bot_ichatdata_s::bot_ichatdata_t;
use crate::be_ai_chat::bot_matchtemplate_s::bot_matchtemplate_t;
use crate::be_ai_chat::bot_randomlist_s::bot_randomlist_t;
use crate::be_ai_chat::bot_replychat_s::bot_replychat_t;
use crate::be_ai_chat::bot_synonymlist_s::bot_synonymlist_t;
use crate::be_ai_goal::bot_goalstate_s::bot_goalstate_t;
use crate::be_ai_goal::itemconfig_s::itemconfig_t;
use crate::be_ai_goal::levelitem_s::levelitem_t;
use crate::be_ai_move::bot_movestate_s::bot_movestate_t;
use crate::be_interface::botlib_globals_s::botlib_globals_t;
use crate::l_libvar::libvar_s::libvar_t;
use crate::l_script::punctuation_s::punctuation_t;
use crate::l_struct::structdef_s::structdef_t;

/// Length of Raven's `default_punctuations[]` table for the MP (non-`DOLLAR`)
/// build — 53 punctuation rows plus the trailing `{NULL, 0}` sentinel.
/// Source: `oracle/codemp/botlib/l_script.cpp:69-146`
const DEFAULT_PUNCTUATIONS_LEN: usize = 54;

/// Synthesized fork-2 owner of botlib's file-scope globals (Raven's
/// `aasworld`, `botimport`, `be_botlib_export`, `libvarlist`, `bot_developer`,
/// …), threaded by `&mut BotLib` through every ported `be_*`/`l_*` function per
/// the state-threading rule (ruling 2). Not a Raven struct: botlib's globals
/// were scattered across its translation units. Each field keeps its exact
/// Raven global name and C type (raw pointers, arrays, and `#[repr(C)]` structs
/// of those — all zero-valid, matching Raven's zero-initialized BSS globals).
pub struct BotLib {
    /// Raven `aas_settings_t aassettings`.
    /// Source: `oracle/codemp/botlib/be_aas_move.cpp:29`
    pub aassettings: aas_settings_t,
    /// Raven `aas_t aasworld`.
    /// Source: `oracle/codemp/botlib/be_aas_main.cpp` (`extern` be_aas_main.h:17)
    pub aasworld: aas_t,
    /// Raven `char basefolder[MAX_QPATH]` (MP / non-`BSPC` build).
    /// Source: `oracle/codemp/botlib/l_script.cpp:151`
    pub basefolder: [c_char; MAX_QPATH],
    /// Raven `int bot_developer` — true if developer mode is on.
    /// Source: `oracle/codemp/botlib/be_interface.cpp` (`extern` be_interface.h:36)
    pub bot_developer: c_int,
    /// Raven `bot_chatstate_t *botchatstates[MAX_CLIENTS+1]`.
    /// Source: `oracle/codemp/botlib/be_ai_chat.cpp:183`
    pub botchatstates: [*mut bot_chatstate_t; MAX_CLIENTS + 1],
    /// Raven `bot_goalstate_t *botgoalstates[MAX_CLIENTS + 1]`.
    /// Source: `oracle/codemp/botlib/be_ai_goal.cpp:163`
    pub botgoalstates: [*mut bot_goalstate_t; MAX_CLIENTS + 1],
    /// Raven `botlib_import_t botimport` — the engine import fn-ptr table.
    /// Source: `oracle/codemp/botlib/be_interface.cpp` (`extern` be_interface.h:35)
    pub botimport: botlib_import_t,
    /// Raven `botlib_globals_t botlibglobals`.
    /// Source: `oracle/codemp/botlib/be_interface.cpp` (`extern` be_interface.h:34)
    pub botlibglobals: botlib_globals_t,
    /// Raven `bot_movestate_t *botmovestates[MAX_CLIENTS+1]`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:102`
    pub botmovestates: [*mut bot_movestate_t; MAX_CLIENTS + 1],
    /// Raven `int *clusterareas`.
    /// Source: `oracle/codemp/botlib/be_aas_routealt.cpp:40`
    pub clusterareas: *mut c_int,
    /// Raven `libvar_t *cmd_grappleoff`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:97`
    pub cmd_grappleoff: *mut libvar_t,
    /// Raven `libvar_t *cmd_grappleon`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:98`
    pub cmd_grappleon: *mut libvar_t,
    /// Raven `bot_consolemessage_t *consolemessageheap`.
    /// Source: `oracle/codemp/botlib/be_ai_chat.cpp:185`
    pub consolemessageheap: *mut bot_consolemessage_t,
    /// Raven `unsigned short crctable[257]`.
    /// Source: `oracle/codemp/botlib/l_crc.cpp:33`
    pub crctable: [c_ushort; 257],
    /// Raven `punctuation_t default_punctuations[]` — the C/C++ operator table.
    /// PORT-NOTE(default_punctuations): the table's initializer content is
    /// populated at setup time, not here; the zeroed placeholder only reserves
    /// the array slot the transcribed bodies index via `.as_mut_ptr()`.
    /// Source: `oracle/codemp/botlib/l_script.cpp:69-146`
    pub default_punctuations: [punctuation_t; DEFAULT_PUNCTUATIONS_LEN],
    /// Raven `libvar_t *droppedweight`.
    /// Source: `oracle/codemp/botlib/be_ai_goal.cpp:178`
    pub droppedweight: *mut libvar_t,
    /// Raven `libvar_t *entitytypemissile`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:95`
    pub entitytypemissile: *mut libvar_t,
    /// Raven `bot_consolemessage_t *freeconsolemessages`.
    /// Source: `oracle/codemp/botlib/be_ai_chat.cpp:186`
    pub freeconsolemessages: *mut bot_consolemessage_t,
    /// Raven `levelitem_t *freelevelitems`.
    /// Source: `oracle/codemp/botlib/be_ai_goal.cpp:168`
    pub freelevelitems: *mut levelitem_t,
    /// Raven `int g_gametype`.
    /// Source: `oracle/codemp/botlib/be_ai_goal.cpp:176`
    pub g_gametype: c_int,
    /// Raven `bot_ichatdata_t *ichatdata[MAX_CLIENTS]`.
    /// Source: `oracle/codemp/botlib/be_ai_chat.cpp:181`
    pub ichatdata: [*mut bot_ichatdata_t; MAX_CLIENTS],
    /// Raven `itemconfig_t *itemconfig`.
    /// Source: `oracle/codemp/botlib/be_ai_goal.cpp:165`
    pub itemconfig: *mut itemconfig_t,
    /// Raven `structdef_t iteminfo_struct` — the item-info struct definition.
    /// PORT-NOTE(iteminfo_struct): populated at setup; zeroed placeholder here.
    /// Source: `oracle/codemp/botlib/be_ai_goal.cpp:136`
    pub iteminfo_struct: structdef_t,
    /// Raven `levelitem_t *levelitemheap`.
    /// Source: `oracle/codemp/botlib/be_ai_goal.cpp:167`
    pub levelitemheap: *mut levelitem_t,
    /// Raven `levelitem_t *levelitems`.
    /// Source: `oracle/codemp/botlib/be_ai_goal.cpp:169`
    pub levelitems: *mut levelitem_t,
    /// Raven `bot_matchtemplate_t *matchtemplates`.
    /// Source: `oracle/codemp/botlib/be_ai_chat.cpp:188`
    pub matchtemplates: *mut bot_matchtemplate_t,
    /// Raven `int modeltypes[MAX_MODELS]`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:100`
    pub modeltypes: [c_int; MAX_MODELS as usize],
    /// Raven `int numaaslinks`.
    /// Source: `oracle/codemp/botlib/be_aas_sample.cpp:48`
    pub numaaslinks: c_int,
    /// Raven `int numclusterareas`.
    /// Source: `oracle/codemp/botlib/be_aas_routealt.cpp:41`
    pub numclusterareas: c_int,
    /// Raven `int numlevelitems`.
    /// Source: `oracle/codemp/botlib/be_ai_goal.cpp:170`
    pub numlevelitems: c_int,
    /// Raven `libvar_t *offhandgrapple`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:96`
    pub offhandgrapple: *mut libvar_t,
    /// Raven `bot_randomlist_t *randomstrings`.
    /// Source: `oracle/codemp/botlib/be_ai_chat.cpp:192`
    pub randomstrings: *mut bot_randomlist_t,
    /// Raven `bot_replychat_t *replychats`.
    /// Source: `oracle/codemp/botlib/be_ai_chat.cpp:194`
    pub replychats: *mut bot_replychat_t,
    /// Raven `libvar_t *saveroutingcache`.
    /// Source: `oracle/codemp/botlib/be_aas_main.cpp:32`
    pub saveroutingcache: *mut libvar_t,
    /// Raven `libvar_t *sv_gravity`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:91`
    pub sv_gravity: *mut libvar_t,
    /// Raven `libvar_t *sv_maxbarrier`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:90`
    pub sv_maxbarrier: *mut libvar_t,
    /// Raven `libvar_t *sv_maxstep`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:89`
    pub sv_maxstep: *mut libvar_t,
    /// Raven `bot_synonymlist_t *synonyms`.
    /// Source: `oracle/codemp/botlib/be_ai_chat.cpp:190`
    pub synonyms: *mut bot_synonymlist_t,
    /// Raven `libvar_t *weapindex_bfg10k`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:93`
    pub weapindex_bfg10k: *mut libvar_t,
    /// Raven `libvar_t *weapindex_grapple`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:94`
    pub weapindex_grapple: *mut libvar_t,
    /// Raven `libvar_t *weapindex_rocketlauncher`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:92`
    pub weapindex_rocketlauncher: *mut libvar_t,
}

impl Default for BotLib {
    /// Hand-written (NOT `#[derive]`): every field is a faithful C type — raw
    /// pointers, `#[repr(C)]` structs of pointers/ints/floats, and arrays of
    /// those — none of which derive `Default`, but all of which are zero-valid
    /// (null pointers, `0`, `None` fn-ptrs), matching Raven's zero-initialized
    /// file-scope globals. Fields whose Raven definitions carry a real
    /// initializer (`default_punctuations`, `iteminfo_struct`, `crctable`) are
    /// populated at setup time, not here.
    fn default() -> Self {
        // SAFETY: all fields are zero-valid (raw pointers → null, `Option<fn>`
        // → `None`, ints/floats/arrays → 0); no field type reserves a niche.
        unsafe { core::mem::zeroed() }
    }
}

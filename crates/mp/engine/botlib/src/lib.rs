//! `mp_engine_botlib` crate. //TODO: Port module mp_engine_botlib

// Raven-named functions/types (`Export_BotLibSetup`, `aasworld`, …) keep their
// original casing across the ABI seam, matching `mp_game`'s crate-level policy.
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod aasfile;
pub mod be_aas_bsp;
pub mod be_aas_bspq3;
pub mod be_aas_bspq3_fns;
pub mod be_aas_cluster;
pub mod be_aas_cluster_fns;
pub mod be_aas_debug;
pub mod be_aas_debug_fns;
pub mod be_aas_def;
pub mod be_aas_entity;
pub mod be_aas_file_fns;
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
pub mod be_ai_char_fns;
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
pub mod be_ai_weight_fns;
pub mod be_ea;
pub mod be_ea_fns;
pub mod be_interface;
pub mod be_interface_fns;
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
use mp_qshared::common::mp::botlib::bot_input_s::bot_input_t;
use mp_qshared::common::mp::botlib::botlib_import_s::botlib_import_t;
use mp_qshared::shared::limits::{MAX_CLIENTS, MAX_MODELS};
use mp_qshared::shared::{qboolean, vec3_t, MAX_QPATH};

use crate::be_aas_bsp::be_aas_bsp_consts::MAX_EPAIRKEY;
use crate::be_aas_bspq3::be_aas_bspq3_cpp_consts::MAX_BSPENTITIES;
use crate::be_aas_bspq3::bsp_entity_t;
use crate::be_aas_def::aas_s::aas_t;
use crate::be_aas_def::aas_settings_s::aas_settings_t;
use crate::be_aas_reach::aas_lreachability_s::aas_lreachability_t;
use crate::be_aas_routealt::midrangearea_t;
use crate::be_ai_char::bot_character_s::BotCharacter;
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
use crate::be_ai_weap::bot_weaponstate_s::bot_weaponstate_t;
use crate::be_ai_weap::weaponconfig_s::weaponconfig_t;
use crate::be_interface::botlib_globals_s::botlib_globals_t;
use crate::l_libvar::libvar_s::{LibVar, LibVarHandle};
use crate::l_log::consts::MAX_LOGFILENAMESIZE;
use crate::l_precomp::define_s::Define;
use crate::l_precomp::precomp_consts::MAX_SOURCEFILES;
use crate::l_precomp::source_s::Source;
use crate::l_script::consts::{
    P_ADD, P_ADD_ASSIGN, P_ASSIGN, P_ATSIGN, P_BACKSLASH, P_BIN_AND, P_BIN_AND_ASSIGN, P_BIN_NOT,
    P_BIN_OR, P_BIN_OR_ASSIGN, P_BIN_XOR, P_BIN_XOR_ASSIGN, P_BRACECLOSE, P_BRACEOPEN, P_COLON,
    P_COMMA, P_CPP1, P_CPP2, P_DEC, P_DIV, P_DIV_ASSIGN, P_DOLLAR, P_INC, P_LOGIC_AND, P_LOGIC_EQ,
    P_LOGIC_GEQ, P_LOGIC_GREATER, P_LOGIC_LEQ, P_LOGIC_LESS, P_LOGIC_NOT, P_LOGIC_OR, P_LOGIC_UNEQ,
    P_LSHIFT, P_LSHIFT_ASSIGN, P_MOD, P_MOD_ASSIGN, P_MUL, P_MUL_ASSIGN, P_PARENTHESESCLOSE,
    P_PARENTHESESOPEN, P_PARMS, P_POINTERREF, P_PRECOMP, P_PRECOMPMERGE, P_QUESTIONMARK, P_REF,
    P_RSHIFT, P_RSHIFT_ASSIGN, P_SEMICOLON, P_SQBRACKETCLOSE, P_SQBRACKETOPEN, P_SUB, P_SUB_ASSIGN,
};
use crate::l_script::punctuation_s::Punctuation;
use crate::l_struct::structdef_s::structdef_t;

/// Raven `default_punctuations[]` — the C/C++ operator table, longest match
/// first, for the MP `DOLLAR` build (includes `$`). Raven's trailing
/// `{NULL, 0}` sentinel is dropped: slice iteration ends at the slice bound.
/// Source: `oracle/codemp/botlib/l_script.cpp:69-146`
pub const DEFAULT_PUNCTUATIONS: &[Punctuation] = &[
    // binary operators
    Punctuation { p: ">>=", n: P_RSHIFT_ASSIGN },
    Punctuation { p: "<<=", n: P_LSHIFT_ASSIGN },
    //
    Punctuation { p: "...", n: P_PARMS },
    // define merge operator
    Punctuation { p: "##", n: P_PRECOMPMERGE },
    // logic operators
    Punctuation { p: "&&", n: P_LOGIC_AND },
    Punctuation { p: "||", n: P_LOGIC_OR },
    Punctuation { p: ">=", n: P_LOGIC_GEQ },
    Punctuation { p: "<=", n: P_LOGIC_LEQ },
    Punctuation { p: "==", n: P_LOGIC_EQ },
    Punctuation { p: "!=", n: P_LOGIC_UNEQ },
    // arithmatic operators
    Punctuation { p: "*=", n: P_MUL_ASSIGN },
    Punctuation { p: "/=", n: P_DIV_ASSIGN },
    Punctuation { p: "%=", n: P_MOD_ASSIGN },
    Punctuation { p: "+=", n: P_ADD_ASSIGN },
    Punctuation { p: "-=", n: P_SUB_ASSIGN },
    Punctuation { p: "++", n: P_INC },
    Punctuation { p: "--", n: P_DEC },
    // binary operators
    Punctuation { p: "&=", n: P_BIN_AND_ASSIGN },
    Punctuation { p: "|=", n: P_BIN_OR_ASSIGN },
    Punctuation { p: "^=", n: P_BIN_XOR_ASSIGN },
    Punctuation { p: ">>", n: P_RSHIFT },
    Punctuation { p: "<<", n: P_LSHIFT },
    // reference operators
    Punctuation { p: "->", n: P_POINTERREF },
    // C++
    Punctuation { p: "::", n: P_CPP1 },
    Punctuation { p: ".*", n: P_CPP2 },
    // arithmatic operators
    Punctuation { p: "*", n: P_MUL },
    Punctuation { p: "/", n: P_DIV },
    Punctuation { p: "%", n: P_MOD },
    Punctuation { p: "+", n: P_ADD },
    Punctuation { p: "-", n: P_SUB },
    Punctuation { p: "=", n: P_ASSIGN },
    // binary operators
    Punctuation { p: "&", n: P_BIN_AND },
    Punctuation { p: "|", n: P_BIN_OR },
    Punctuation { p: "^", n: P_BIN_XOR },
    Punctuation { p: "~", n: P_BIN_NOT },
    // logic operators
    Punctuation { p: "!", n: P_LOGIC_NOT },
    Punctuation { p: ">", n: P_LOGIC_GREATER },
    Punctuation { p: "<", n: P_LOGIC_LESS },
    // reference operator
    Punctuation { p: ".", n: P_REF },
    // seperators
    Punctuation { p: ",", n: P_COMMA },
    Punctuation { p: ";", n: P_SEMICOLON },
    // label indication
    Punctuation { p: ":", n: P_COLON },
    // if statement
    Punctuation { p: "?", n: P_QUESTIONMARK },
    // embracements
    Punctuation { p: "(", n: P_PARENTHESESOPEN },
    Punctuation { p: ")", n: P_PARENTHESESCLOSE },
    Punctuation { p: "{", n: P_BRACEOPEN },
    Punctuation { p: "}", n: P_BRACECLOSE },
    Punctuation { p: "[", n: P_SQBRACKETOPEN },
    Punctuation { p: "]", n: P_SQBRACKETCLOSE },
    //
    Punctuation { p: "\\", n: P_BACKSLASH },
    // precompiler operator
    Punctuation { p: "#", n: P_PRECOMP },
    Punctuation { p: "$", n: P_DOLLAR },
    // StringEd key
    Punctuation { p: "@", n: P_ATSIGN },
];

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
    pub cmd_grappleoff: LibVarHandle,
    /// Raven `libvar_t *cmd_grappleon`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:98`
    pub cmd_grappleon: LibVarHandle,
    /// Raven `bot_consolemessage_t *consolemessageheap`.
    /// Source: `oracle/codemp/botlib/be_ai_chat.cpp:185`
    pub consolemessageheap: *mut bot_consolemessage_t,
    /// Raven `unsigned short crctable[257]`.
    /// Source: `oracle/codemp/botlib/l_crc.cpp:33`
    pub crctable: [c_ushort; 257],
    /// Raven `punctuation_t default_punctuations[]` — the C/C++ operator table,
    /// now the `DEFAULT_PUNCTUATIONS` const slice (seated in `Default`, since a
    /// reference has no zero-valid bit pattern).
    /// Source: `oracle/codemp/botlib/l_script.cpp:69-146`
    pub default_punctuations: &'static [Punctuation],
    /// Raven `libvar_t *droppedweight`.
    /// Source: `oracle/codemp/botlib/be_ai_goal.cpp:178`
    pub droppedweight: LibVarHandle,
    /// Raven `libvar_t *entitytypemissile`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:95`
    pub entitytypemissile: LibVarHandle,
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
    /// Populated at setup; zeroed placeholder here.
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
    pub offhandgrapple: LibVarHandle,
    /// Raven `bot_randomlist_t *randomstrings`.
    /// Source: `oracle/codemp/botlib/be_ai_chat.cpp:192`
    pub randomstrings: *mut bot_randomlist_t,
    /// Raven `bot_replychat_t *replychats`.
    /// Source: `oracle/codemp/botlib/be_ai_chat.cpp:194`
    pub replychats: *mut bot_replychat_t,
    /// Raven `libvar_t *saveroutingcache`.
    /// Source: `oracle/codemp/botlib/be_aas_main.cpp:32`
    pub saveroutingcache: LibVarHandle,
    /// Raven `libvar_t *sv_gravity`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:91`
    pub sv_gravity: LibVarHandle,
    /// Raven `libvar_t *sv_maxbarrier`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:90`
    pub sv_maxbarrier: LibVarHandle,
    /// Raven `libvar_t *sv_maxstep`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:89`
    pub sv_maxstep: LibVarHandle,
    /// Raven `bot_synonymlist_t *synonyms`.
    /// Source: `oracle/codemp/botlib/be_ai_chat.cpp:190`
    pub synonyms: *mut bot_synonymlist_t,
    /// Raven `libvar_t *weapindex_bfg10k`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:93`
    pub weapindex_bfg10k: LibVarHandle,
    /// Raven `libvar_t *weapindex_grapple`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:94`
    pub weapindex_grapple: LibVarHandle,
    /// Raven `libvar_t *weapindex_rocketlauncher`.
    /// Source: `oracle/codemp/botlib/be_ai_move.cpp:92`
    pub weapindex_rocketlauncher: LibVarHandle,

    // ---- Round-3 field merge: file-scope globals and function-static hoists
    // referenced by transcribed bodies but absent from the initial aggregate.
    // All zero-valid (raw pointers, ints, floats, arrays, `#[repr(C)]` structs
    // of those), matching Raven's zero-initialized globals. ----
    /// Raven `qboolean addGlobalDefine`.
    /// Source: `oracle/codemp/botlib/l_precomp.cpp:109`
    pub addGlobalDefine: qboolean,
    /// Raven `aas_lreachability_t **areareachability` — reachability links for every area.
    /// Source: `oracle/codemp/botlib/be_aas_reach.cpp:85`
    pub areareachability: *mut *mut aas_lreachability_t,
    /// Raven `bot_input_t *botinputs`.
    /// Source: `oracle/codemp/botlib/be_ea.cpp:27`
    pub botinputs: *mut bot_input_t,
    /// Raven `bsp_t bspworld` — the global id-Software BSP entity store.
    /// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:70`
    pub bspworld: bsp_t,
    /// Raven `int calcgrapplereach`.
    /// Source: `oracle/codemp/botlib/be_aas_reach.cpp:68`
    pub calcgrapplereach: c_int,
    /// Raven `campspot_t *campspots`.
    /// Source: `oracle/codemp/botlib/be_ai_goal.cpp:174`
    pub campspots: *mut campspot_t,
    /// Raven function-static `float framereachability` (hoisted from `AAS_ContinueInitReachability`).
    /// Source: `oracle/codemp/botlib/be_aas_reach.cpp:4350`
    pub framereachability: f32,
    /// Raven `define_t **globaldefines` (`DEFINEHASHING` build) — defines added
    /// to every loaded source. Redesigned (porting-rules §F17) from the malloc'd
    /// hash-chained list into an owned `Vec<Define>` arena (seated in `Default`).
    /// Source: `oracle/codemp/botlib/l_precomp.cpp:105`
    pub globaldefines: Vec<Define>,
    /// Raven function-static `unsigned short int *hidetraveltimes` (hoisted from `AAS_CreateAllRoutingCache`).
    /// Source: `oracle/codemp/botlib/be_aas_route.cpp:2067`
    pub hidetraveltimes: *mut c_ushort,
    /// Raven function-static `int lastpercentage` (hoisted from `AAS_ContinueInitReachability`).
    /// Source: `oracle/codemp/botlib/be_aas_reach.cpp:4351`
    pub lastpercentage: c_int,
    /// Raven `libvar_t *libvarlist` — the library-variable list. Redesigned
    /// (porting-rules §F17) from a malloc'd linked list into this owned arena;
    /// `LibVarHandle` indexes it. Slots are only appended or cleared wholesale,
    /// never individually removed.
    /// Source: `oracle/codemp/botlib/l_libvar.cpp:20`
    pub libvars: Vec<LibVar>,
    /// Raven `logfile_t logfile`.
    /// Source: `oracle/codemp/botlib/l_log.cpp:33`
    pub logfile: logfile_t,
    /// Raven `maplocation_t *maplocations`.
    /// Source: `oracle/codemp/botlib/be_ai_goal.cpp:172`
    pub maplocations: *mut maplocation_t,
    /// Raven `int max_routingcachesize`.
    /// Source: `oracle/codemp/botlib/be_aas_route.cpp:68`
    pub max_routingcachesize: c_int,
    /// Raven `midrangearea_t *midrangeareas`.
    /// Source: `oracle/codemp/botlib/be_aas_routealt.cpp:39`
    pub midrangeareas: *mut midrangearea_t,
    /// Raven `aas_lreachability_t *nextreachability` — next free reachability from the heap.
    /// Source: `oracle/codemp/botlib/be_aas_reach.cpp:84`
    pub nextreachability: *mut aas_lreachability_t,
    /// Raven `int nofaceflood = qtrue` — statically initialized to true (the one
    /// non-zero-initialized `BotLib` global); set in `Default::default`.
    /// Source: `oracle/codemp/botlib/be_aas_cluster.cpp:38`
    pub nofaceflood: c_int,
    /// Raven `int numareacacheupdates`.
    /// Source: `oracle/codemp/botlib/be_aas_route.cpp:63`
    pub numareacacheupdates: c_int,
    /// Raven `int numlreachabilities`.
    /// Source: `oracle/codemp/botlib/be_aas_reach.cpp:86`
    pub numlreachabilities: c_int,
    /// Raven `int numportalcacheupdates`.
    /// Source: `oracle/codemp/botlib/be_aas_route.cpp:64`
    pub numportalcacheupdates: c_int,
    /// Raven `int numtokens`.
    /// Source: `oracle/codemp/botlib/l_precomp.cpp:96`
    pub numtokens: c_int,
    /// Raven reachability-type id counters — `int reach_barrier` (jump up to a barrier)
    /// and its siblings, each a distinct reachability class id.
    /// Source: `oracle/codemp/botlib/be_aas_reach.cpp:48-66`
    pub reach_barrier: c_int,
    pub reach_elevator: c_int,
    pub reach_equalfloor: c_int,
    pub reach_funcbob: c_int,
    pub reach_grapple: c_int,
    pub reach_jump: c_int,
    pub reach_jumppad: c_int,
    pub reach_ladder: c_int,
    pub reach_rocketjump: c_int,
    pub reach_step: c_int,
    pub reach_swim: c_int,
    pub reach_teleport: c_int,
    pub reach_walk: c_int,
    pub reach_walkoffledge: c_int,
    pub reach_waterjump: c_int,
    /// Raven function-static `float reachability_delay` (hoisted from `AAS_ContinueInitReachability`).
    /// Source: `oracle/codemp/botlib/be_aas_reach.cpp:4350`
    pub reachability_delay: f32,
    /// Raven `aas_lreachability_t *reachabilityheap` — heap with reachabilities.
    /// Source: `oracle/codemp/botlib/be_aas_reach.cpp:83`
    pub reachabilityheap: *mut aas_lreachability_t,
    /// Raven `int routingcachesize`.
    /// Source: `oracle/codemp/botlib/be_aas_route.cpp:67`
    pub routingcachesize: c_int,
    /// Raven `source_t *sourceFiles[MAX_SOURCEFILES]` — redesigned
    /// (porting-rules §F17) into an owned slab keyed by the 1-based source
    /// handle; a `None` slot is free. Pre-sized to `MAX_SOURCEFILES` (seated in
    /// `Default`) so a handle indexes directly.
    /// Source: `oracle/codemp/botlib/l_precomp.cpp:3187`
    pub sourceFiles: Vec<Option<Source>>,

    // --- be_interface / be_aas_debug / be_ai_char / be_ai_weight globals ---
    /// Raven `botlib_export_t be_botlib_export` — the botlib export fn-ptr table.
    /// Source: `oracle/codemp/botlib/be_interface.cpp:41`
    pub be_botlib_export: crate::be_interface::botlib_export_s::botlib_export_t,
    /// Raven `bot_character_t *botcharacters[MAX_CLIENTS + 1]` — redesigned
    /// (porting-rules §F17) from a malloc'd handle table into an owned slab
    /// keyed by the 1-based character handle (index 0 unused = Raven's null
    /// handle); a `None` slot is free. Pre-sized to `MAX_CLIENTS + 1` (seated in
    /// `Default`) so a handle indexes directly.
    /// Source: `oracle/codemp/botlib/be_ai_char.cpp:60`
    pub botcharacters: Vec<Option<BotCharacter>>,
    /// Raven `int botlibsetup` — true when the bot library has been set up.
    /// Source: `oracle/codemp/botlib/be_interface.h:21`
    pub botlibsetup: c_int,
    /// Raven `int debuglines[MAX_DEBUGLINES]`.
    /// Source: `oracle/codemp/botlib/be_aas_debug.cpp:31`
    pub debuglines: [c_int; crate::be_aas_debug::be_aas_debug_cpp_consts::MAX_DEBUGLINES as usize],
    /// Raven `int debuglinevisible[MAX_DEBUGLINES]`.
    /// Source: `oracle/codemp/botlib/be_aas_debug.cpp:32`
    pub debuglinevisible:
        [c_int; crate::be_aas_debug::be_aas_debug_cpp_consts::MAX_DEBUGLINES as usize],
    /// Raven `static int debugpolygons[MAX_DEBUGPOLYGONS]`.
    /// Source: `oracle/codemp/botlib/be_aas_debug.cpp:35`
    pub debugpolygons:
        [c_int; crate::be_aas_debug::be_aas_debug_cpp_consts::MAX_DEBUGPOLYGONS as usize],
    /// Raven `static int index` (function-local static in `AAS_ShowReachableAreas`).
    /// Source: `oracle/codemp/botlib/be_aas_debug.cpp:671`
    pub index: c_int,
    /// Raven `static int lastareanum` (function-local static in `AAS_ShowReachableAreas`).
    /// Source: `oracle/codemp/botlib/be_aas_debug.cpp:671`
    pub lastareanum: c_int,
    /// Raven `static float lasttime` (function-local static in `AAS_ShowReachableAreas`).
    /// Source: `oracle/codemp/botlib/be_aas_debug.cpp:672`
    pub lasttime: f32,
    /// Raven `int numdebuglines`.
    /// Source: `oracle/codemp/botlib/be_aas_debug.cpp:33`
    pub numdebuglines: c_int,
    /// Raven `static aas_reachability_t reach` (function-local static in
    /// `AAS_ShowReachableAreas`).
    /// Source: `oracle/codemp/botlib/be_aas_debug.cpp:670`
    pub reach: crate::aasfile::aas_reachability_s::aas_reachability_t,
    /// Raven `weightconfig_t *weightFileList[MAX_WEIGHT_FILES]`.
    /// Source: `oracle/codemp/botlib/be_ai_weight.cpp:34`
    pub weightFileList: [*mut crate::be_ai_weight::weightconfig_s::weightconfig_t;
        crate::be_ai_weight::weightconfig_s::MAX_WEIGHT_FILES],

    // --- be_ai_weap / be_aas_file globals and function-static hoists ---
    /// Raven `bot_weaponstate_t *botweaponstates[MAX_CLIENTS+1]`.
    /// Source: `oracle/codemp/botlib/be_ai_weap.cpp:111`
    pub botweaponstates: [*mut bot_weaponstate_t; MAX_CLIENTS + 1],
    /// Raven `weaponconfig_t *weaponconfig` — the loaded weapon configuration.
    /// Source: `oracle/codemp/botlib/be_ai_weap.cpp:112`
    pub weaponconfig: *mut weaponconfig_t,
    /// Raven function-static `int AAS_WriteAASLump_offset` — cross-frame write
    /// cursor hoisted from `AAS_WriteAASLump` (ruling 3 kind 3).
    /// Source: `oracle/codemp/botlib/be_aas_file.cpp:479`
    pub AAS_WriteAASLump_offset: c_int,
}

impl Default for BotLib {
    /// Hand-written (NOT `#[derive]`): nearly every field is a faithful C type —
    /// raw pointers, `#[repr(C)]` structs of pointers/ints/floats, and arrays of
    /// those — none of which derive `Default`, but all of which are zero-valid
    /// (null pointers, `0`, `None` fn-ptrs), matching Raven's zero-initialized
    /// file-scope globals. The exceptions are the owned-collection and reference
    /// fields whose all-zero bit pattern is *invalid* — `libvars`,
    /// `globaldefines`, `sourceFiles`, `botcharacters` (a `Vec`'s `NonNull`
    /// buffer pointer cannot be null) and `default_punctuations` (a `&'static`
    /// reference cannot be null); each is written explicitly before
    /// `assume_init`. Fields whose
    /// Raven definitions carry a real initializer (`iteminfo_struct`,
    /// `crctable`) are populated at setup time, not here. The cached
    /// `LibVarHandle` fields (`sv_maxstep`, `cmd_grappleon`, …) default to
    /// `LibVarHandle(0)`, matching Raven's null pointers: they are assigned real
    /// handles at setup (`BotSetupMoveAI`/`AAS_Setup`/…) before any read.
    fn default() -> Self {
        // SAFETY: every field except the five seated below is zero-valid (raw
        // pointers → null, `Option<fn>` → `None`, ints/floats/arrays → 0,
        // `LibVarHandle` → index 0). `MaybeUninit::zeroed` never materializes a
        // zeroed `Vec`/reference as a valid value: each such slot is overwritten
        // with a real value (via `ptr::write`, so the invalid zeroed bytes are
        // not dropped) before `assume_init`.
        let mut uninit = core::mem::MaybeUninit::<Self>::zeroed();
        let ptr = uninit.as_mut_ptr();
        unsafe {
            core::ptr::write(core::ptr::addr_of_mut!((*ptr).libvars), Vec::new());
            core::ptr::write(core::ptr::addr_of_mut!((*ptr).globaldefines), Vec::new());
            core::ptr::write(
                core::ptr::addr_of_mut!((*ptr).sourceFiles),
                (0..MAX_SOURCEFILES).map(|_| None).collect(),
            );
            core::ptr::write(
                core::ptr::addr_of_mut!((*ptr).botcharacters),
                (0..=MAX_CLIENTS).map(|_| None).collect(),
            );
            core::ptr::write(
                core::ptr::addr_of_mut!((*ptr).default_punctuations),
                DEFAULT_PUNCTUATIONS,
            );
            // Raven `int nofaceflood = qtrue` — the one non-zero static initializer.
            core::ptr::addr_of_mut!((*ptr).nofaceflood).write(1);
            uninit.assume_init()
        }
    }
}

impl BotLib {
    /// Resolve a cached `LibVarHandle` to its `LibVar` slot — the arena
    /// equivalent of dereferencing Raven's cached `libvar_t *` (§B5).
    #[inline]
    pub fn libvar(&self, handle: LibVarHandle) -> &LibVar {
        &self.libvars[handle.0]
    }
}

// The four botlib-internal scratch types below (`bsp_t`, `campspot_t`,
// `maplocation_t`, `logfile_t`) back `BotLib` fields but had no ported home.
// They are non-ABI internal state, so they live here in the struct's owner file
// (the only file this merge may edit) rather than a dedicated per-type module.

/// Raven `bsp_t` — the id-Software BSP entity store backing `bspworld`.
///
/// Source: `oracle/codemp/botlib/be_aas_bspq3.cpp:56-67`
#[derive(Clone, Copy)]
pub struct bsp_t {
    /// true when bsp file is loaded
    pub loaded: c_int,
    /// entity data size
    pub entdatasize: c_int,
    pub dentdata: *mut c_char,
    /// bsp entities
    pub numentities: c_int,
    pub entities: [bsp_entity_t; MAX_BSPENTITIES as usize],
}

/// Raven `maplocation_t` — location in the map ("target_location").
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:52-58`
#[derive(Clone, Copy)]
pub struct maplocation_t {
    pub origin: vec3_t,
    pub areanum: c_int,
    pub name: [c_char; MAX_EPAIRKEY as usize],
    pub next: *mut maplocation_t,
}

/// Raven `campspot_t` — camp spot ("info_camp").
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:61-71`
#[derive(Clone, Copy)]
pub struct campspot_t {
    pub origin: vec3_t,
    pub areanum: c_int,
    pub name: [c_char; MAX_EPAIRKEY as usize],
    pub range: f32,
    pub weight: f32,
    pub wait: f32,
    pub random: f32,
    pub next: *mut campspot_t,
}

/// Raven `logfile_t` — the botlib log-file handle.
///
/// Source: `oracle/codemp/botlib/l_log.cpp:26-31`
#[derive(Clone, Copy)]
pub struct logfile_t {
    pub filename: [c_char; MAX_LOGFILENAMESIZE as usize],
    pub fp: *mut libc::FILE,
    pub numwrites: c_int,
}

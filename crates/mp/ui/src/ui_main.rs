//! `ui_main.c` — the ui module's main logic (ownerdraws, feeders, menu
//! scripts, server browser).
//!
//! Source: `oracle/codemp/ui/ui_main.c`

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_short, c_void, CStr};

use mp_abi::ui::public::ui_client_state_t::uiClientState_t;
use mp_bg::bg_channel::BgState;
use mp_bg::bg_misc::{forceMasteryPoints, BG_FindItemForHoldable, BG_FindItemForWeapon};
use mp_bg::bg_saga::{
    BG_GetClassOnBaseClass, BG_GetUIPortrait, BG_GetUIPortraitFile, BG_SiegeCountBaseClass,
    BG_SiegeFindThemeForTeam, BG_SiegeGetPairedValue, BG_SiegeGetValueGroup, BG_SiegeSetTeamTheme,
    BG_SiegeTeamClassPortrait,
};
use mp_bg::public::configstring::{CS_PLAYERS, CS_SERVERINFO};
use mp_bg::public::gametype::{
    GT_CTF, GT_CTY, GT_DUEL, GT_FFA, GT_HOLOCRON, GT_JEDIMASTER, GT_POWERDUEL, GT_SIEGE,
    GT_SINGLE_PLAYER, GT_TEAM,
};
use mp_bg::public::holdable::HI_NUM_HOLDABLE;
use mp_bg::public::team::{TEAM_BLUE, TEAM_FREE, TEAM_RED, TEAM_SPECTATOR};
use mp_bg::saga::siege_class_t::siegeClass_t;
use mp_bg::saga::siege_player_class_flags_t::siegePlayerClassFlags_t::{
    SPC_DEMOLITIONIST, SPC_HEAVY_WEAPONS, SPC_INFANTRY, SPC_JEDI, SPC_MAX, SPC_SUPPORT,
    SPC_VANGUARD,
};
use mp_bg::saga::siege_team_t::{
    siegeTeam_t, MAX_SIEGE_INFO_SIZE, SIEGETEAM_TEAM1, SIEGETEAM_TEAM2,
};
use mp_bg::weapons::weapon_t::{WP_NONE, WP_NUM_WEAPONS, WP_SABER};
use mp_qshared::common::mp::qcommon::qtime::qtime_t;
use mp_qshared::common::mp::qcommon::saber::saber_colors::{saber_colors_t, SABER_BLUE, SABER_RED};
use mp_qshared::shared::cbuf_exec::cbufExec_t;
use mp_qshared::shared::com_parse::{COM_BeginParseSession, COM_ParseExt, QSharedScratch};
use mp_qshared::shared::cvar::{
    vmCvar_t, CVAR_ARCHIVE, CVAR_INIT, CVAR_INTERNAL, CVAR_NORESTART, CVAR_ROM, CVAR_SERVERINFO,
    CVAR_TEMP,
};
use mp_qshared::shared::force_powers::{
    FORCE_DARKSIDE, FORCE_LIGHTSIDE, FP_LEVITATION, FP_SABER_DEFENSE, FP_SABER_OFFENSE,
    MAX_FORCE_RANK, NUM_FORCE_POWERS, NUM_FORCE_POWER_LEVELS,
};
use mp_qshared::shared::limits::MAX_NAME_LENGTH;
use mp_qshared::shared::q_color::{S_COLOR_RED, S_COLOR_YELLOW};
use mp_qshared::shared::q_string::{COM_Parse, COM_StripExtension};
use mp_qshared::shared::{
    colorWhite, connstate_t, fileHandle_t, pc_token_t, qhandle_t, vec4_t, AS_FAVORITES, AS_GLOBAL,
    AS_LOCAL, AS_MPLAYER, CIN_LOOP, CIN_SILENT, FS_READ, KEYCATCH_UI, MAX_CLIENTS, MAX_INFO_STRING,
    MAX_INFO_VALUE, MAX_QPATH, MAX_STRING_CHARS, MAX_TOKENLENGTH, Q3_VERSION, SCREEN_HEIGHT,
    SCREEN_WIDTH,
};
use mp_uishared::shared::display_context::DisplayContext;
use mp_uishared::shared::menu_id::MenuId;
use mp_uishared::shared::menu_system::MAX_MENUFILE;
use mp_uishared::shared::menudef::{
    FEEDER_ALLMAPS, FEEDER_CINEMATICS, FEEDER_COLORCHOICES, FEEDER_DEMOS, FEEDER_FINDPLAYER,
    FEEDER_FORCECFG, FEEDER_LANGUAGES, FEEDER_MAPS, FEEDER_MODS, FEEDER_MOVES, FEEDER_MOVES_TITLES,
    FEEDER_PLAYER_LIST, FEEDER_PLAYER_SKIN_HEAD, FEEDER_PLAYER_SKIN_LEGS, FEEDER_PLAYER_SKIN_TORSO,
    FEEDER_PLAYER_SPECIES, FEEDER_Q3HEADS, FEEDER_SABER_SINGLE_INFO, FEEDER_SABER_STAFF_INFO,
    FEEDER_SERVERS, FEEDER_SERVERSTATUS, FEEDER_SIEGE_BASE_CLASS, FEEDER_SIEGE_CLASS_FORCE,
    FEEDER_SIEGE_CLASS_INVENTORY, FEEDER_SIEGE_CLASS_WEAPONS, FEEDER_SIEGE_TEAM1,
    FEEDER_SIEGE_TEAM2, FEEDER_TEAM_LIST, ITEM_TEXTSTYLE_BLINK, ITEM_TEXTSTYLE_NORMAL,
    ITEM_TEXTSTYLE_OUTLINED, ITEM_TEXTSTYLE_OUTLINESHADOWED, ITEM_TEXTSTYLE_PULSE,
    ITEM_TEXTSTYLE_SHADOWED, ITEM_TEXTSTYLE_SHADOWEDMORE, UI_ALLMAPS_SELECTION, UI_AUTOSWITCHLIST,
    UI_BLUETEAM1, UI_BLUETEAM2, UI_BLUETEAM3, UI_BLUETEAM4, UI_BLUETEAM5, UI_BLUETEAM6,
    UI_BLUETEAM7, UI_BLUETEAM8, UI_BLUETEAMNAME, UI_BOTNAME, UI_BOTSKILL, UI_CLANCINEMATIC,
    UI_CLANLOGO, UI_CLANNAME, UI_CROSSHAIR, UI_EFFECTS, UI_FORCE_MASTERY_SET, UI_FORCE_POINTS,
    UI_FORCE_RANK, UI_FORCE_RANK_HEAL, UI_FORCE_RANK_SABERTHROW, UI_FORCE_SIDE, UI_GAMETYPE,
    UI_GLINFO, UI_HANDICAP, UI_JEDI_NONJEDI, UI_JOINGAMETYPE, UI_KEYBINDSTATUS, UI_MAPCINEMATIC,
    UI_MAPPREVIEW, UI_MAPS_SELECTION, UI_MAP_TIMETOBEAT, UI_NETFILTER, UI_NETGAMETYPE,
    UI_NETMAPCINEMATIC, UI_NETMAPPREVIEW, UI_NETSOURCE, UI_OPPONENTLOGO, UI_OPPONENTLOGO_METAL,
    UI_OPPONENTLOGO_NAME, UI_OPPONENTMODEL, UI_OPPONENT_NAME, UI_PLAYERLOGO, UI_PLAYERLOGO_METAL,
    UI_PLAYERLOGO_NAME, UI_PLAYERMODEL, UI_PREVIEWCINEMATIC, UI_REDBLUE, UI_REDTEAM1, UI_REDTEAM2,
    UI_REDTEAM3, UI_REDTEAM4, UI_REDTEAM5, UI_REDTEAM6, UI_REDTEAM7, UI_REDTEAM8, UI_REDTEAMNAME,
    UI_SELECTEDPLAYER, UI_SERVERMOTD, UI_SERVERREFRESHDATE, UI_SHOW_ANYNONTEAMGAME,
    UI_SHOW_ANYTEAMGAME, UI_SHOW_DEMOAVAILABLE, UI_SHOW_FAVORITESERVERS, UI_SHOW_FFA,
    UI_SHOW_LEADER, UI_SHOW_NETANYNONTEAMGAME, UI_SHOW_NETANYTEAMGAME, UI_SHOW_NEWBESTTIME,
    UI_SHOW_NEWHIGHSCORE, UI_SHOW_NOTFAVORITESERVERS, UI_SHOW_NOTFFA, UI_SHOW_NOTLEADER, UI_SKILL,
    UI_SKIN_COLOR, UI_STARTMAPCINEMATIC, UI_TIER, UI_TIERMAP1, UI_TIERMAP2, UI_TIERMAP3,
    UI_TIER_GAMETYPE, UI_TIER_MAPNAME, UI_TOTALFORCESTARS, UI_VERSION,
};
use mp_uishared::shared::rect_def_t::RectDef;
use mp_uishared::ui_shared::{
    Display_KeyBindPending, Int_Parse, ItemParse_asset_model_go, ItemParse_model_g2skin_go,
    Item_RunScript, LerpColor, Menu_FindItemByName, Menu_GetFocused, Menu_SetFeederSelection,
    Menu_SetItemBackground, Menu_ShowGroup, Menu_ShowItemByName, Menus_AnyFullScreenVisible,
    Menus_FindByName, PC_Color_Parse, PC_Float_Parse, PC_Int_Parse, PC_Script_Parse,
    PC_String_Parse, String_Parse, String_Report, UI_CleanupGhoul2,
};
use native_math::qmath::Com_Clamp;
use native_string::{
    atof, atoi, buf_to_string, latin1_to_string, string_to_latin1, Info_ValueForKey, Q_CleanStr,
    Q_stricmp, Q_stricmpn, Q_strncpyz,
};

use crate::keycodes::fake_ascii_t::fakeAscii_t;
use crate::local::game_type_info::GameTypeInfo;
use crate::local::map_info::{MapInfo, MAX_GAMETYPES};
use crate::local::mod_info_t::ModInfo;
use crate::local::pinglist_t::MAX_ADDRESSLENGTH;
use crate::local::player_species_info_t::{PlayerSpeciesInfo, MAX_PLAYERMODELS};
use crate::local::server_filter_s::ServerFilter;
use crate::local::server_status_info_t::{
    ServerStatusInfo, MAX_SERVERSTATUS_LINES, MAX_SERVERSTATUS_TEXT,
};
use crate::local::tier_info::MAPS_PER_TIER;
use crate::trap;
use crate::ui_atoms::{
    Com_Error, Com_Printf, UI_Cvar_VariableString, UI_DrawHandlePic, UI_FillRect,
    UI_LoadBestScores, UI_SetColor,
};
use crate::ui_force::UI_DrawForceStars;
use crate::ui_gameinfo::{UI_GetBotNameByNumber, UI_GetNumBots, MAX_MAPS};
use crate::ui_saber::{
    SaberColorToString, TranslateSaberColor, UI_SaberModelForSaber, UI_SaberProperNameForSaber,
    UI_SaberSkinForSaber, UI_SaberTypeForSaber,
};
use crate::world::ui_context::UiContext;
use crate::world::ui_cvars::UiCvars;
use crate::world::ui_main_state::MAX_SABER_HILTS;
use crate::world::ui_world::{UiWorld, MAX_FORCE_CONFIGS};

/// Raven `static const int numSkillLevels = sizeof(skillLevels) /
/// sizeof(const char*)` — `skillLevels[]` (`ui_main.c:902-908`) has 5 rows;
/// the table itself is compiled-in data that lands beside the fn that reads
/// it (PORT-NOTE, `UiMainState`), so only the derived count is needed here.
///
/// Source: `oracle/codemp/ui/ui_main.c:902-909`
const NUM_SKILL_LEVELS: c_int = 5;

/// Raven `qfiles.h` `STYLE_DROPSHADOW`/`STYLE_BLINK` font-render bits
/// (`Text_Paint`'s `iFontHandle` high bits). Already ported once as a
/// `mp_engine_qcommon` const (`qfiles/font_style.rs`), but `mp_ui` has no
/// dependency on that crate, so these stay file-local consts (same fidelity,
/// same values).
///
/// Source: `oracle/codemp/qcommon/qfiles.h:570-571`
const STYLE_DROPSHADOW: u32 = 0x8000_0000;
const STYLE_BLINK: u32 = 0x4000_0000;

/// Raven `#define MAX_Q3PLAYERMODELS 256`.
///
/// Source: `oracle/codemp/ui/ui_local.h:593`
const MAX_Q3PLAYERMODELS: usize = 256;

/// Raven's movedata sound ids — an anonymous `enum` (no tag, no typedef name),
/// so they stay plain consts at the `short` width `datpadmovedata_t.sound`
/// stores them in.
///
/// Source: `oracle/codemp/ui/ui_main.c:360-368`
const MDS_NONE: c_short = 0;
const MDS_FORCE_JUMP: c_short = 1;
const MDS_ROLL: c_short = 2;
const MDS_SABER: c_short = 3;

/// Raven's datapad move-title ids — likewise an anonymous `enum`.
///
/// Source: `oracle/codemp/ui/ui_main.c:370-379`
const MD_MOVE_TITLE_MAX: c_int = 6;

/// Raven `#define MAX_MOVES 16`.
///
/// Source: `oracle/codemp/ui/ui_main.c:404`
const MAX_MOVES: usize = 16;

/// Raven `datpadmovedata_t` — one datapad move row. Compiled-in data lands
/// beside the fns that read it (§C8), so the row type is file-local here
/// rather than a `local/` type module; Raven likewise declares it inside
/// `ui_main.c`.
///
/// Type definition source: `oracle/codemp/ui/ui_main.c:406-412`
#[doc(alias = "datpadmovedata_t")]
#[allow(dead_code)]
struct DatpadMoveData {
    title: Option<&'static str>,
    desc: Option<&'static str>,
    anim: Option<&'static str>,
    sound: c_short,
}

/// Raven `static datpadmovedata_t datapadMoveData[MD_MOVE_TITLE_MAX][MAX_MOVES]`.
///
/// Raven: Some hard coded badness. At some point maybe this should be
/// externalized to a .dat file.
/// Source: `oracle/codemp/ui/ui_main.c:414-523`
const DATAPAD_MOVE_DATA: [[DatpadMoveData; MAX_MOVES]; MD_MOVE_TITLE_MAX as usize] = [
    // Acrobatics
    [
        MOVE(
            "@MENUS_FORCE_JUMP1",
            "@MENUS_FORCE_JUMP1_DESC",
            "BOTH_FORCEJUMP1",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_FORCE_FLIP",
            "@MENUS_FORCE_FLIP_DESC",
            "BOTH_FLIP_F",
            MDS_FORCE_JUMP,
        ),
        MOVE("@MENUS_ROLL", "@MENUS_ROLL_DESC", "BOTH_ROLL_F", MDS_ROLL),
        MOVE(
            "@MENUS_BACKFLIP_OFF_WALL",
            "@MENUS_BACKFLIP_OFF_WALL_DESC",
            "BOTH_WALL_FLIP_BACK1",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_SIDEFLIP_OFF_WALL",
            "@MENUS_SIDEFLIP_OFF_WALL_DESC",
            "BOTH_WALL_FLIP_RIGHT",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_WALL_RUN",
            "@MENUS_WALL_RUN_DESC",
            "BOTH_WALL_RUN_RIGHT",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_WALL_GRAB_JUMP",
            "@MENUS_WALL_GRAB_JUMP_DESC",
            "BOTH_FORCEWALLREBOUND_FORWARD",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_RUN_UP_WALL_BACKFLIP",
            "@MENUS_RUN_UP_WALL_BACKFLIP_DESC",
            "BOTH_FORCEWALLRUNFLIP_START",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_JUMPUP_FROM_KNOCKDOWN",
            "@MENUS_JUMPUP_FROM_KNOCKDOWN_DESC",
            "BOTH_KNOCKDOWN3",
            MDS_NONE,
        ),
        MOVE(
            "@MENUS_JUMPKICK_FROM_KNOCKDOWN",
            "@MENUS_JUMPKICK_FROM_KNOCKDOWN_DESC",
            "BOTH_KNOCKDOWN2",
            MDS_NONE,
        ),
        MOVE(
            "@MENUS_ROLL_FROM_KNOCKDOWN",
            "@MENUS_ROLL_FROM_KNOCKDOWN_DESC",
            "BOTH_KNOCKDOWN1",
            MDS_NONE,
        ),
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
    ],
    // Single Saber, Fast Style
    [
        MOVE(
            "@MENUS_STAB_BACK",
            "@MENUS_STAB_BACK_DESC",
            "BOTH_A2_STABBACK1",
            MDS_SABER,
        ),
        MOVE(
            "@MENUS_LUNGE_ATTACK",
            "@MENUS_LUNGE_ATTACK_DESC",
            "BOTH_LUNGE2_B__T_",
            MDS_SABER,
        ),
        MOVE(
            "@MENUS_FAST_ATTACK_KATA",
            "@MENUS_FAST_ATTACK_KATA_DESC",
            "BOTH_A1_SPECIAL",
            MDS_SABER,
        ),
        MOVE(
            "@MENUS_ATTACK_ENEMYONGROUND",
            "@MENUS_ATTACK_ENEMYONGROUND_DESC",
            "BOTH_STABDOWN",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_CARTWHEEL",
            "@MENUS_CARTWHEEL_DESC",
            "BOTH_ARIAL_RIGHT",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_BOTH_ROLL_STAB",
            "@MENUS_BOTH_ROLL_STAB2_DESC",
            "BOTH_ROLL_STAB",
            MDS_SABER,
        ),
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
    ],
    // Single Saber, Medium Style
    [
        MOVE(
            "@MENUS_SLASH_BACK",
            "@MENUS_SLASH_BACK_DESC",
            "BOTH_ATTACK_BACK",
            MDS_SABER,
        ),
        MOVE(
            "@MENUS_FLIP_ATTACK",
            "@MENUS_FLIP_ATTACK_DESC",
            "BOTH_JUMPFLIPSLASHDOWN1",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_MEDIUM_ATTACK_KATA",
            "@MENUS_MEDIUM_ATTACK_KATA_DESC",
            "BOTH_A2_SPECIAL",
            MDS_SABER,
        ),
        MOVE(
            "@MENUS_ATTACK_ENEMYONGROUND",
            "@MENUS_ATTACK_ENEMYONGROUND_DESC",
            "BOTH_STABDOWN",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_CARTWHEEL",
            "@MENUS_CARTWHEEL_DESC",
            "BOTH_ARIAL_RIGHT",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_BOTH_ROLL_STAB",
            "@MENUS_BOTH_ROLL_STAB2_DESC",
            "BOTH_ROLL_STAB",
            MDS_SABER,
        ),
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
    ],
    // Single Saber, Strong Style
    [
        MOVE(
            "@MENUS_SLASH_BACK",
            "@MENUS_SLASH_BACK_DESC",
            "BOTH_ATTACK_BACK",
            MDS_SABER,
        ),
        MOVE(
            "@MENUS_JUMP_ATTACK",
            "@MENUS_JUMP_ATTACK_DESC",
            "BOTH_FORCELEAP2_T__B_",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_STRONG_ATTACK_KATA",
            "@MENUS_STRONG_ATTACK_KATA_DESC",
            "BOTH_A3_SPECIAL",
            MDS_SABER,
        ),
        MOVE(
            "@MENUS_ATTACK_ENEMYONGROUND",
            "@MENUS_ATTACK_ENEMYONGROUND_DESC",
            "BOTH_STABDOWN",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_CARTWHEEL",
            "@MENUS_CARTWHEEL_DESC",
            "BOTH_ARIAL_RIGHT",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_BOTH_ROLL_STAB",
            "@MENUS_BOTH_ROLL_STAB2_DESC",
            "BOTH_ROLL_STAB",
            MDS_SABER,
        ),
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
    ],
    // Dual Sabers
    [
        MOVE(
            "@MENUS_SLASH_BACK",
            "@MENUS_SLASH_BACK_DESC",
            "BOTH_ATTACK_BACK",
            MDS_SABER,
        ),
        MOVE(
            "@MENUS_FLIP_FORWARD_ATTACK",
            "@MENUS_FLIP_FORWARD_ATTACK_DESC",
            "BOTH_JUMPATTACK6",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_DUAL_SABERS_TWIRL",
            "@MENUS_DUAL_SABERS_TWIRL_DESC",
            "BOTH_SPINATTACK6",
            MDS_SABER,
        ),
        MOVE(
            "@MENUS_ATTACK_ENEMYONGROUND",
            "@MENUS_ATTACK_ENEMYONGROUND_DESC",
            "BOTH_STABDOWN_DUAL",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_DUAL_SABER_BARRIER",
            "@MENUS_DUAL_SABER_BARRIER_DESC",
            "BOTH_A6_SABERPROTECT",
            MDS_SABER,
        ),
        MOVE(
            "@MENUS_DUAL_STAB_FRONT_BACK",
            "@MENUS_DUAL_STAB_FRONT_BACK_DESC",
            "BOTH_A6_FB",
            MDS_SABER,
        ),
        MOVE(
            "@MENUS_DUAL_STAB_LEFT_RIGHT",
            "@MENUS_DUAL_STAB_LEFT_RIGHT_DESC",
            "BOTH_A6_LR",
            MDS_SABER,
        ),
        MOVE(
            "@MENUS_CARTWHEEL",
            "@MENUS_CARTWHEEL_DESC",
            "BOTH_ARIAL_RIGHT",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_BOTH_ROLL_STAB",
            "@MENUS_BOTH_ROLL_STAB_DESC",
            "BOTH_ROLL_STAB",
            MDS_SABER,
        ),
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
    ],
    // Saber Staff
    [
        MOVE(
            "@MENUS_STAB_BACK",
            "@MENUS_STAB_BACK_DESC",
            "BOTH_A2_STABBACK1",
            MDS_SABER,
        ),
        MOVE(
            "@MENUS_BACK_FLIP_ATTACK",
            "@MENUS_BACK_FLIP_ATTACK_DESC",
            "BOTH_JUMPATTACK7",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_SABER_STAFF_TWIRL",
            "@MENUS_SABER_STAFF_TWIRL_DESC",
            "BOTH_SPINATTACK7",
            MDS_SABER,
        ),
        MOVE(
            "@MENUS_ATTACK_ENEMYONGROUND",
            "@MENUS_ATTACK_ENEMYONGROUND_DESC",
            "BOTH_STABDOWN_STAFF",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_SPINNING_KATA",
            "@MENUS_SPINNING_KATA_DESC",
            "BOTH_A7_SOULCAL",
            MDS_SABER,
        ),
        MOVE(
            "@MENUS_KICK1",
            "@MENUS_KICK1_DESC",
            "BOTH_A7_KICK_F",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_JUMP_KICK",
            "@MENUS_JUMP_KICK_DESC",
            "BOTH_A7_KICK_F_AIR",
            MDS_FORCE_JUMP,
        ),
        MOVE(
            "@MENUS_BUTTERFLY_ATTACK",
            "@MENUS_BUTTERFLY_ATTACK_DESC",
            "BOTH_BUTTERFLY_FR1",
            MDS_SABER,
        ),
        MOVE(
            "@MENUS_BOTH_ROLL_STAB",
            "@MENUS_BOTH_ROLL_STAB2_DESC",
            "BOTH_ROLL_STAB",
            MDS_SABER,
        ),
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
        NO_MOVE,
    ],
];

/// One populated `DATAPAD_MOVE_DATA` row.
const fn MOVE(
    title: &'static str,
    desc: &'static str,
    anim: &'static str,
    sound: c_short,
) -> DatpadMoveData {
    DatpadMoveData {
        title: Some(title),
        desc: Some(desc),
        anim: Some(anim),
        sound,
    }
}

/// Raven's `NULL, NULL, 0, MDS_NONE` padding row.
const NO_MOVE: DatpadMoveData = DatpadMoveData {
    title: None,
    desc: None,
    anim: None,
    sound: MDS_NONE,
};

/// Raven `static const char *handicapValues[]` — compiled-in data (§C8),
/// lands beside the fns that read it. The trailing Raven `NULL` sentinel
/// becomes `None` (unread by this wave's fns, kept for fidelity).
///
/// Source: `oracle/codemp/ui/ui_main.c:1895`
const HANDICAP_VALUES: [Option<&str>; 21] = [
    Some("None"),
    Some("95"),
    Some("90"),
    Some("85"),
    Some("80"),
    Some("75"),
    Some("70"),
    Some("65"),
    Some("60"),
    Some("55"),
    Some("50"),
    Some("45"),
    Some("40"),
    Some("35"),
    Some("30"),
    Some("25"),
    Some("20"),
    Some("15"),
    Some("10"),
    Some("5"),
    None,
];

/// Raven `static const char *skillLevels[]` — compiled-in data (§C8), lands
/// beside the fns that read it. `NUM_SKILL_LEVELS` above is the derived count.
///
/// Source: `oracle/codemp/ui/ui_main.c:902-908`
const SKILL_LEVELS: [&str; NUM_SKILL_LEVELS as usize] =
    ["SKILL1", "SKILL2", "SKILL3", "SKILL4", "SKILL5"];

/// Raven `char *forceMasteryLevels[NUM_FORCE_MASTERY_LEVELS]` — `bg_misc.c`
/// compiled-in data. Ui compiles `bg_misc.c` into its own link unit
/// (`WE_ARE_IN_THE_UI`, DEC-36 addendum 11) and this table has no Rust port
/// reachable from `mp_bg` yet, so it lands beside the ui fns that read it
/// (§C8), same as `SKILL_LEVELS`/`HANDICAP_VALUES` above.
///
/// Source: `oracle/codemp/game/bg_misc.c:150-160`
const FORCE_MASTERY_LEVELS: [&str; 8] = [
    "MASTERY0", "MASTERY1", "MASTERY2", "MASTERY3", "MASTERY4", "MASTERY5", "MASTERY6", "MASTERY7",
];

/// Raven `static const serverFilter_t serverFilters[]` — compiled-in data
/// (§C8), lands beside the fn that reads it.
///
/// Source: `oracle/codemp/ui/ui_main.c:896-899`
const SERVER_FILTERS: [ServerFilter; 2] = [
    ServerFilter {
        description: "MENUS_ALL",
        basedir: "",
    },
    ServerFilter {
        description: "MENUS_JEDI_ACADEMY",
        basedir: "",
    },
];

/// Raven `#define PULSE_DIVISOR 75` — no canonical qshared home reachable
/// from this crate (same story as `STYLE_DROPSHADOW`/`STYLE_BLINK` above).
///
/// Source: `oracle/codemp/game/q_shared.h:486`
const PULSE_DIVISOR: c_int = 75;

/// Raven `#define FORCE_NONJEDI 0` — no canonical `ui_force.h` const home
/// reachable from this crate yet.
///
/// Source: `oracle/codemp/ui/ui_force.h:4`
const FORCE_NONJEDI: c_int = 0;

/// Raven `#define MAX_MODS 64` — `modList`/`demoList`/`movieList` are `Vec`s
/// here (no fixed backing array to overflow), but the cap still bounds how
/// many entries each loader keeps, matching Raven's array-size ceiling.
///
/// Source: `oracle/codemp/ui/ui_local.h:590`
const MAX_MODS: usize = 64;

/// Raven `#define MAX_DEMOS 256`.
///
/// Source: `oracle/codemp/ui/ui_local.h:591`
const MAX_DEMOS: c_int = 256;

/// Raven `#define MAX_MOVIES 256`.
///
/// Source: `oracle/codemp/ui/ui_local.h:592`
const MAX_MOVIES: c_int = 256;

// DEFERRED: UI_AnimsetAlloc — part of the ui_main.c hand-maintained animation
// fork (`bgAllAnims`/`uiNumAllAnims`/`UI_ParseAnimationFile`); DEC-36 D5 rules
// ui reuses mp_bg's animation module instead of Raven's manually synced copy
// (see `UiMainState`'s PORT-NOTE, which drops the same fork's state fields).
// Source: `oracle/codemp/ui/ui_main.c:645-651`

// DEFERRED: UI_ParseAnimationFile — same hand-maintained animation fork as
// UI_AnimsetAlloc above; its state (`uiHumanoidAnimations`, `UIPAFtext`,
// `UIPAFtextLoaded`, `bgAllAnims`, `uiNumAllAnims`) was dropped at U2 per the
// same DEC-36 D5 ruling, so there is nothing left to thread this fn's body
// through.
// Source: `oracle/codemp/ui/ui_main.c:664-863`

/// Raven `GetCRDelineatedString`.
///
/// Raven kept the result in a function-scope `static char sTemp[256]`; the
/// idiomatic port returns the owned `String` (or `None` for Raven's `NULL`
/// out-of-range return) directly instead of reusing a shared buffer.
///
/// PORT-NOTE (§19): Raven `strcpy`s the line into `char sTemp[256]` (an overrun
/// for longer lines); the owned `String` returns it untruncated.
///
/// Source: `oracle/codemp/ui/ui_main.c:954-976`
pub fn GetCRDelineatedString(
    ctx: &mut UiContext,
    psStripFileRef: &str,
    psStripStringRef: &str,
    iIndex: c_int,
) -> Option<String> {
    let psList = UI_GetStringEdString(ctx, psStripFileRef, psStripStringRef);
    let mut rest = psList.as_str();

    // Raven's `while (iIndex--)` tests before the decrement, so a negative index
    // walks to the end of the list and falls out through the OOR return.
    let mut i = iIndex;
    while i != 0 {
        match rest.find('\n') {
            Some(pos) => rest = &rest[pos + 1..],
            None => return None, // OOR
        }
        i -= 1;
    }

    let sTemp = match rest.find('\n') {
        Some(pos) => &rest[..pos],
        None => rest,
    };

    Some(sTemp.to_string())
}

/// Raven `UI_TeamName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:942-950`
pub fn UI_TeamName(team: c_int) -> &'static str {
    if team == TEAM_RED {
        "RED"
    } else if team == TEAM_BLUE {
        "BLUE"
    } else if team == TEAM_SPECTATOR {
        "SPECTATOR"
    } else {
        "FREE"
    }
}

/// Raven `AssetCache`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1009-1045`
pub fn AssetCache(ctx: &mut UiContext) {
    ctx.world.uiDC.Assets.gradientBar =
        trap::R_RegisterShaderNoMip(ctx.engine, "ui/assets/gradientbar2.tga");
    ctx.world.uiDC.Assets.fxBasePic = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_base");
    ctx.world.uiDC.Assets.fxPic[0] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_red");
    ctx.world.uiDC.Assets.fxPic[1] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_orange");
    ctx.world.uiDC.Assets.fxPic[2] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_yel");
    ctx.world.uiDC.Assets.fxPic[3] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_grn");
    ctx.world.uiDC.Assets.fxPic[4] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_blue");
    ctx.world.uiDC.Assets.fxPic[5] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_purple");
    ctx.world.uiDC.Assets.fxPic[6] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_white");
    ctx.world.uiDC.Assets.scrollBar =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/scrollbar.tga");
    ctx.world.uiDC.Assets.scrollBarArrowDown =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/scrollbar_arrow_dwn_a.tga");
    ctx.world.uiDC.Assets.scrollBarArrowUp =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/scrollbar_arrow_up_a.tga");
    ctx.world.uiDC.Assets.scrollBarArrowLeft =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/scrollbar_arrow_left.tga");
    ctx.world.uiDC.Assets.scrollBarArrowRight =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/scrollbar_arrow_right.tga");
    ctx.world.uiDC.Assets.scrollBarThumb =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/scrollbar_thumb.tga");
    ctx.world.uiDC.Assets.sliderBar = trap::R_RegisterShaderNoMip(ctx.engine, "menu/new/slider");
    ctx.world.uiDC.Assets.sliderThumb =
        trap::R_RegisterShaderNoMip(ctx.engine, "menu/new/sliderthumb");

    // Icons for various server settings.
    ctx.world.uiDC.Assets.needPass = trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/needpass");
    ctx.world.uiDC.Assets.noForce = trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/noforce");
    ctx.world.uiDC.Assets.forceRestrict =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/forcerestrict");
    ctx.world.uiDC.Assets.saberOnly =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/saberonly");
    ctx.world.uiDC.Assets.trueJedi = trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/truejedi");

    for n in 0..ctx.world.uiDC.Assets.crosshairShader.len() {
        let letter = (b'a' + n as u8) as char;
        ctx.world.uiDC.Assets.crosshairShader[n] =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("gfx/2d/crosshair{}", letter));
    }

    // trap_S_RegisterSound("sound/feedback/voc_newhighscore.wav") — Raven left
    // this call commented out.
    ctx.world.newHighScoreSound = 0;
}

/// Raven `_UI_DrawSides`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1047-1051`
pub fn _UI_DrawSides(ctx: &mut UiContext, x: f32, y: f32, w: f32, h: f32, size: f32) {
    let size = size * ctx.world.uiDC.xscale;
    let white = ctx.world.uiDC.whiteShader;
    trap::R_DrawStretchPic(ctx.engine, x, y, size, h, 0.0, 0.0, 0.0, 0.0, white);
    trap::R_DrawStretchPic(
        ctx.engine,
        x + w - size,
        y,
        size,
        h,
        0.0,
        0.0,
        0.0,
        0.0,
        white,
    );
}

/// Raven `_UI_DrawTopBottom`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1053-1057`
pub fn _UI_DrawTopBottom(ctx: &mut UiContext, x: f32, y: f32, w: f32, h: f32, size: f32) {
    let size = size * ctx.world.uiDC.yscale;
    let white = ctx.world.uiDC.whiteShader;
    trap::R_DrawStretchPic(ctx.engine, x, y, w, size, 0.0, 0.0, 0.0, 0.0, white);
    trap::R_DrawStretchPic(
        ctx.engine,
        x,
        y + h - size,
        w,
        size,
        0.0,
        0.0,
        0.0,
        0.0,
        white,
    );
}

/// Raven `_UI_DrawRect`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1065-1072`
pub fn _UI_DrawRect(
    ctx: &mut UiContext,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    size: f32,
    color: &vec4_t,
) {
    trap::R_SetColor(ctx.engine, Some(color));

    _UI_DrawTopBottom(ctx, x, y, width, height, size);
    _UI_DrawSides(ctx, x, y, width, height, size);

    trap::R_SetColor(ctx.engine, None);
}

/// Raven `MenuFontToHandle`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1075-1086`
pub fn MenuFontToHandle(world: &UiWorld, iMenuFont: c_int) -> qhandle_t {
    match iMenuFont {
        1 => world.uiDC.Assets.qhSmallFont,
        2 => world.uiDC.Assets.qhMediumFont,
        3 => world.uiDC.Assets.qhBigFont,
        4 => world.uiDC.Assets.qhSmall2Font,
        _ => world.uiDC.Assets.qhMediumFont,
    }
}

/// Raven `Text_Width`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1089-1094`
pub fn Text_Width(ctx: &UiContext, text: &str, scale: f32, iMenuFont: c_int) -> c_int {
    let iFontIndex = MenuFontToHandle(ctx.world, iMenuFont);
    trap::R_Font_StrLenPixels(ctx.engine, text, iFontIndex, scale)
}

/// Raven `Text_Height`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1096-1101`
pub fn Text_Height(ctx: &UiContext, _text: &str, scale: f32, iMenuFont: c_int) -> c_int {
    let iFontIndex = MenuFontToHandle(ctx.world, iMenuFont);
    trap::R_Font_HeightPixels(ctx.engine, iFontIndex, scale)
}

/// Raven `Text_Paint`.
///
/// PORT-NOTE: the JK2-menu-style-to-SOF2-printstring-ctrl-code `switch`
/// (`ITEM_TEXTSTYLE_*` → `STYLE_BLINK`/`STYLE_DROPSHADOW`) is transcribed as a
/// `match`; both file-local const families are defined above (no canonical
/// qshared/qcommon home reachable from this crate).
///
/// Source: `oracle/codemp/ui/ui_main.c:1103-1130`
#[allow(clippy::too_many_arguments)]
pub fn Text_Paint(
    ctx: &UiContext,
    x: f32,
    y: f32,
    scale: f32,
    color: vec4_t,
    text: &str,
    _adjust: f32,
    limit: c_int,
    style: c_int,
    iMenuFont: c_int,
) {
    let iFontIndex = MenuFontToHandle(ctx.world, iMenuFont);
    // kludge.. convert JK2 menu styles to SOF2 printstring ctrl codes...
    let iStyleOR: c_int = match style {
        ITEM_TEXTSTYLE_NORMAL => 0,                           // JK2 normal text
        ITEM_TEXTSTYLE_BLINK => STYLE_BLINK as c_int,         // JK2 fast blinking
        ITEM_TEXTSTYLE_PULSE => STYLE_BLINK as c_int,         // JK2 slow pulsing
        ITEM_TEXTSTYLE_SHADOWED => STYLE_DROPSHADOW as c_int, // JK2 drop shadow
        ITEM_TEXTSTYLE_OUTLINED => STYLE_DROPSHADOW as c_int, // JK2 drop shadow
        ITEM_TEXTSTYLE_OUTLINESHADOWED => STYLE_DROPSHADOW as c_int, // JK2 drop shadow
        ITEM_TEXTSTYLE_SHADOWEDMORE => STYLE_DROPSHADOW as c_int, // JK2 drop shadow
        _ => 0,
    };

    trap::R_Font_DrawString(
        ctx.engine,
        x as c_int,                          // int ox
        y as c_int,                          // int oy
        text,                                // const char *text
        &color,                              // paletteRGBA_c c
        iStyleOR | iFontIndex,               // const int iFontHandle
        if limit == 0 { -1 } else { limit }, // iCharLimit (-1 = none)
        scale,                               // const float scale = 1.0f
    );
}

/// Raven `UI_GetStringEdString`.
///
/// Raven kept the result in a function-scope `static char text[1024]`; the
/// idiomatic port returns the owned `String` directly instead of reusing a
/// shared buffer.
///
/// Source: `oracle/codemp/ui/ui_main.c:1249-1255`
pub fn UI_GetStringEdString(ctx: &mut UiContext, refSection: &str, refName: &str) -> String {
    let key = format!("{}_{}", refSection, refName);
    trap::SP_GetStringTextString(ctx.engine, &key, 1024).unwrap_or_default()
}

/// Raven `GetMenuBuffer`.
///
/// Raven read into a function-scope `static char buf[MAX_MENUFILE]`; the port
/// reads into a local `Vec<u8>` sized to the file length and returns the
/// decoded `String` directly (each call fully repopulates the buffer before
/// use, so there is no cross-call state to preserve).
///
/// Source: `oracle/codemp/ui/ui_main.c:1439-1461`
pub fn GetMenuBuffer(ctx: &mut UiContext, filename: &str) -> String {
    let mut f: fileHandle_t = 0;
    let len = trap::FS_FOpenFile(ctx.engine, filename, &mut f, FS_READ);
    if f == 0 {
        trap::Print(
            ctx.engine,
            &format!(
                "{}menu file not found: {}, using default\n",
                S_COLOR_RED.to_str().unwrap(),
                filename
            ),
        );
        return ctx.world.main.defaultMenu.clone().unwrap_or_default();
    }
    if len >= MAX_MENUFILE as c_int {
        trap::Print(
            ctx.engine,
            &format!(
                "{}menu file too large: {} is {}, max allowed is {}",
                S_COLOR_RED.to_str().unwrap(),
                filename,
                len,
                MAX_MENUFILE
            ),
        );
        trap::FS_FCloseFile(ctx.engine, f);
        return ctx.world.main.defaultMenu.clone().unwrap_or_default();
    }

    let mut buf = vec![0u8; len as usize];
    trap::FS_Read(ctx.engine, &mut buf, f);
    trap::FS_FCloseFile(ctx.engine, f);
    // COM_Compress(buf) — Raven left this call commented out.
    latin1_to_string(&buf)
}

/// Raven `UI_DrawCenteredPic`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1229-1234`
pub fn UI_DrawCenteredPic(ctx: &mut UiContext, image: qhandle_t, w: c_int, h: c_int) {
    let x = (SCREEN_WIDTH - w) / 2;
    let y = (SCREEN_HEIGHT - h) / 2;
    UI_DrawHandlePic(ctx, x as f32, y as f32, w as f32, h as f32, image);
}

/// Raven `_UI_Shutdown`.
///
/// PORT-NOTE: `UI_CleanupGhoul2` calls through `DisplayContext` (its ported
/// shape takes `dc: &mut dyn DisplayContext`, DEC-36 addendum 12), so this fn
/// carries a `dc` parameter alongside `ctx` even though Raven's own body has
/// no `DC->` call (see escalations).
///
/// Source: `oracle/codemp/ui/ui_main.c:1432-1435`
pub fn _UI_Shutdown(ctx: &mut UiContext, dc: &mut dyn DisplayContext) {
    trap::LAN_SaveCachedServers(ctx.engine);
    UI_CleanupGhoul2(&mut ctx.world.menus, dc);
}

/// Raven `UI_SetCapFragLimits`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1911-1922`
pub fn UI_SetCapFragLimits(ctx: &mut UiContext, uiVars: bool) {
    let cap = 5;
    let frag = 10;

    if uiVars {
        trap::Cvar_Set(ctx.engine, "ui_captureLimit", &format!("{}", cap));
        trap::Cvar_Set(ctx.engine, "ui_fragLimit", &format!("{}", frag));
    } else {
        trap::Cvar_Set(ctx.engine, "capturelimit", &format!("{}", cap));
        trap::Cvar_Set(ctx.engine, "fraglimit", &format!("{}", frag));
    }
}

/// Raven `UI_GetGameTypeName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1924-1950`
pub fn UI_GetGameTypeName(ctx: &mut UiContext, gtEnum: c_int) -> String {
    match gtEnum {
        GT_FFA => UI_GetStringEdString(ctx, "MENUS", "FREE_FOR_ALL"), //"Free For All";
        GT_HOLOCRON => UI_GetStringEdString(ctx, "MENUS", "HOLOCRON_FFA"), //"Holocron FFA";
        GT_JEDIMASTER => UI_GetStringEdString(ctx, "MENUS", "SAGA"),  //"Jedi Master";??
        GT_SINGLE_PLAYER => UI_GetStringEdString(ctx, "MENUS", "SAGA"), //"Team FFA";
        GT_DUEL => UI_GetStringEdString(ctx, "MENUS", "DUEL"),        //"Team FFA";
        GT_POWERDUEL => UI_GetStringEdString(ctx, "MENUS", "POWERDUEL"), //"Team FFA";
        GT_TEAM => UI_GetStringEdString(ctx, "MENUS", "TEAM_FFA"),    //"Team FFA";
        GT_SIEGE => UI_GetStringEdString(ctx, "MENUS", "SIEGE"),      //"Siege";
        GT_CTF => UI_GetStringEdString(ctx, "MENUS", "CAPTURE_THE_FLAG"), //"Capture the Flag";
        GT_CTY => UI_GetStringEdString(ctx, "MENUS", "CAPTURE_THE_YSALIMARI"), //"Capture the Ysalamiri";
        _ => UI_GetStringEdString(ctx, "MENUS", "SAGA"),                       //"Team FFA";
    }
}

/// Raven `UI_TeamIndexFromName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2010-2023`
pub fn UI_TeamIndexFromName(world: &UiWorld, name: &str) -> c_int {
    if !name.is_empty() {
        for (i, team) in world.teamList.iter().enumerate() {
            if Q_stricmp(name, &team.teamName) == 0 {
                return i as c_int;
            }
        }
    }

    0
}

/// Raven `UI_DrawClanLogo`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2025-2040`
pub fn UI_DrawClanLogo(ctx: &mut UiContext, rect: &RectDef, _scale: f32, color: vec4_t) {
    let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
    let i = UI_TeamIndexFromName(ctx.world, &teamName);
    if i >= 0 && (i as usize) < ctx.world.teamList.len() {
        trap::R_SetColor(ctx.engine, Some(&color));

        if ctx.world.teamList[i as usize].teamIcon == -1 {
            let imageName = ctx.world.teamList[i as usize].imageName.clone();
            ctx.world.teamList[i as usize].teamIcon =
                trap::R_RegisterShaderNoMip(ctx.engine, &imageName);
            ctx.world.teamList[i as usize].teamIcon_Metal =
                trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_metal", imageName));
            ctx.world.teamList[i as usize].teamIcon_Name =
                trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_name", imageName));
        }

        let icon = ctx.world.teamList[i as usize].teamIcon;
        UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, icon);
        trap::R_SetColor(ctx.engine, None);
    }
}

/// Raven `UI_DrawClanCinematic`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2042-2068`
pub fn UI_DrawClanCinematic(ctx: &mut UiContext, rect: &RectDef, _scale: f32, color: vec4_t) {
    let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
    let i = UI_TeamIndexFromName(ctx.world, &teamName);
    if i >= 0 && (i as usize) < ctx.world.teamList.len() {
        let idx = i as usize;

        if ctx.world.teamList[idx].cinematic >= -2 {
            if ctx.world.teamList[idx].cinematic == -1 {
                let imageName = ctx.world.teamList[idx].imageName.clone();
                ctx.world.teamList[idx].cinematic = trap::CIN_PlayCinematic(
                    ctx.engine,
                    &format!("{}.roq", imageName),
                    0,
                    0,
                    0,
                    0,
                    CIN_LOOP | CIN_SILENT,
                );
            }
            if ctx.world.teamList[idx].cinematic >= 0 {
                let cinematic = ctx.world.teamList[idx].cinematic;
                trap::CIN_RunCinematic(ctx.engine, cinematic);
                trap::CIN_SetExtents(
                    ctx.engine,
                    cinematic,
                    rect.x as c_int,
                    rect.y as c_int,
                    rect.w as c_int,
                    rect.h as c_int,
                );
                trap::CIN_DrawCinematic(ctx.engine, cinematic);
            } else {
                trap::R_SetColor(ctx.engine, Some(&color));
                let icon = ctx.world.teamList[idx].teamIcon_Metal;
                UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, icon);
                trap::R_SetColor(ctx.engine, None);
                ctx.world.teamList[idx].cinematic = -2;
            }
        } else {
            trap::R_SetColor(ctx.engine, Some(&color));
            let icon = ctx.world.teamList[idx].teamIcon;
            UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, icon);
            trap::R_SetColor(ctx.engine, None);
        }
    }
}

/// Raven `UI_DrawPreviewCinematic`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2070-2082`
pub fn UI_DrawPreviewCinematic(ctx: &mut UiContext, rect: &RectDef, _scale: f32, _color: vec4_t) {
    if ctx.world.previewMovie > -2 {
        let movie = ctx.world.movieList[ctx.world.movieIndex as usize].clone();
        ctx.world.previewMovie = trap::CIN_PlayCinematic(
            ctx.engine,
            &format!("{}.roq", movie),
            0,
            0,
            0,
            0,
            CIN_LOOP | CIN_SILENT,
        );
        if ctx.world.previewMovie >= 0 {
            trap::CIN_RunCinematic(ctx.engine, ctx.world.previewMovie);
            trap::CIN_SetExtents(
                ctx.engine,
                ctx.world.previewMovie,
                rect.x as c_int,
                rect.y as c_int,
                rect.w as c_int,
                rect.h as c_int,
            );
            trap::CIN_DrawCinematic(ctx.engine, ctx.world.previewMovie);
        } else {
            ctx.world.previewMovie = -2;
        }
    }
}

/// Raven `UI_HasSetSaberOnly`.
///
/// PORT-NOTE (§19 UB pick): Raven reads `info` through `Info_ValueForKey`
/// before `trap_GetConfigString` fills it — an uninitialized-stack read
/// (`ui_main.c:2234-2239`). This port treats the pre-fill buffer as empty,
/// matching a zeroed C automatic (`Info_ValueForKey("", ...)` returns `""`,
/// so `atoi` yields 0).
///
/// Source: `oracle/codemp/ui/ui_main.c:2232-2269`
pub fn UI_HasSetSaberOnly(ctx: &mut UiContext) -> bool {
    let empty = String::new();
    let gametype = atoi(&Info_ValueForKey(&empty, "g_gametype"));

    if gametype == GT_JEDIMASTER {
        return false;
    }

    let info =
        trap::GetConfigString(ctx.engine, CS_SERVERINFO, MAX_INFO_STRING).unwrap_or_default();

    let wDisable = if gametype == GT_DUEL || gametype == GT_POWERDUEL {
        atoi(&Info_ValueForKey(&info, "g_duelWeaponDisable"))
    } else {
        atoi(&Info_ValueForKey(&info, "g_weaponDisable"))
    };

    let mut i = 0;
    while i < WP_NUM_WEAPONS {
        if (wDisable & (1 << i)) == 0 && i != WP_SABER && i != WP_NONE {
            return false;
        }
        i += 1;
    }

    true
}

/// Raven `UI_AllForceDisabled`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2271-2289`
pub fn UI_AllForceDisabled(force: c_int) -> bool {
    if force != 0 {
        for i in 0..NUM_FORCE_POWERS {
            if force & (1 << i) == 0 {
                return false;
            }
        }
        return true;
    }

    false
}

/// Raven `UI_TrueJediEnabled`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2291-2319`
pub fn UI_TrueJediEnabled(ctx: &mut UiContext) -> bool {
    let info =
        trap::GetConfigString(ctx.engine, CS_SERVERINFO, MAX_INFO_STRING).unwrap_or_default();

    // already have serverinfo at this point for stuff below. Don't bother
    // trying to use ui_forcePowerDisable.
    let disabledForce = atoi(&Info_ValueForKey(&info, "g_forcePowerDisable"));
    let allForceDisabled = UI_AllForceDisabled(disabledForce);
    let gametype = atoi(&Info_ValueForKey(&info, "g_gametype"));
    let saberOnly = UI_HasSetSaberOnly(ctx);

    let trueJedi =
        if gametype == GT_HOLOCRON || gametype == GT_JEDIMASTER || saberOnly || allForceDisabled {
            0
        } else {
            atoi(&Info_ValueForKey(&info, "g_jediVmerc"))
        };

    trueJedi != 0
}

/// Raven `UI_SetForceDisabled`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2502-2547`
pub fn UI_SetForceDisabled(world: &mut UiWorld, force: c_int) {
    if force != 0 {
        let mut i = 0;
        while i < NUM_FORCE_POWERS {
            if force & (1 << i) != 0 {
                world.force.uiForcePowersDisabled[i as usize] = true;

                if i != FP_LEVITATION && i != FP_SABER_OFFENSE && i != FP_SABER_DEFENSE {
                    world.force.uiForcePowersRank[i as usize] = 0;
                } else if i == FP_LEVITATION {
                    world.force.uiForcePowersRank[i as usize] = 1;
                } else {
                    world.force.uiForcePowersRank[i as usize] = 3;
                }
            } else {
                world.force.uiForcePowersDisabled[i as usize] = false;
            }
            i += 1;
        }
    } else {
        let mut i = 0;
        while i < NUM_FORCE_POWERS {
            world.force.uiForcePowersDisabled[i as usize] = false;
            i += 1;
        }
    }
}

/// Raven `UI_DrawEffects`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2425-2428`
pub fn UI_DrawEffects(ctx: &mut UiContext, rect: &RectDef, _scale: f32, _color: vec4_t) {
    // PORT-NOTE (§19): Raven read a stale fixed-array slot here; guarded to a
    // skipped draw (there is no fallback shader).
    let idx = ctx.world.effectsColor;
    if idx < 0 || idx as usize >= ctx.world.force.uiSaberColorShaders.len() {
        return;
    }
    let shader = ctx.world.force.uiSaberColorShaders[idx as usize];
    UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, shader);
}

/// Raven `UI_DrawMapPreview`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2430-2452`
pub fn UI_DrawMapPreview(
    ctx: &mut UiContext,
    rect: &RectDef,
    _scale: f32,
    _color: vec4_t,
    net: bool,
) {
    let mut map = if net {
        ctx.world.cvars.ui_currentNetMap.integer
    } else {
        ctx.world.cvars.ui_currentMap.integer
    };
    if map < 0 || map > ctx.world.mapList.len() as c_int {
        if net {
            ctx.world.cvars.ui_currentNetMap.integer = 0;
            trap::Cvar_Set(ctx.engine, "ui_currentNetMap", "0");
        } else {
            ctx.world.cvars.ui_currentMap.integer = 0;
            trap::Cvar_Set(ctx.engine, "ui_currentMap", "0");
        }
        map = 0;
    }

    let idx = map as usize;
    // PORT-NOTE (§19): `map == mapCount` clears Raven's guard and reads a stale
    // fixed-array slot (levelShot 0); guarded to the unknown-map fallback below.
    if idx >= ctx.world.mapList.len() {
        let shader = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/unknownmap_mp");
        UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, shader);
        return;
    }
    if ctx.world.mapList[idx].levelShot == -1 {
        let imageName = ctx.world.mapList[idx].imageName.clone();
        ctx.world.mapList[idx].levelShot = trap::R_RegisterShaderNoMip(ctx.engine, &imageName);
    }

    if ctx.world.mapList[idx].levelShot > 0 {
        let shot = ctx.world.mapList[idx].levelShot;
        UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, shot);
    } else {
        let shader = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/unknownmap_mp");
        UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, shader);
    }
}

/// Raven `UI_DrawNetMapPreview`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2739-2746`
pub fn UI_DrawNetMapPreview(ctx: &mut UiContext, rect: &RectDef, _scale: f32, _color: vec4_t) {
    if ctx.world.serverStatus.currentServerPreview > 0 {
        let preview = ctx.world.serverStatus.currentServerPreview;
        UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, preview);
    } else {
        let shader = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/unknownmap_mp");
        UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, shader);
    }
}

/// Raven `UI_DrawTierMap`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2791-2803`
pub fn UI_DrawTierMap(ctx: &mut UiContext, rect: &RectDef, index: c_int) {
    let mut i = trap::Cvar_VariableValue(ctx.engine, "ui_currentTier") as c_int;
    if i < 0 || i as usize >= ctx.world.tierList.len() {
        i = 0;
    }

    let tierIdx = i as usize;
    let mapIdx = index as usize;
    // PORT-NOTE (§19): Raven read a stale fixed-array slot here; guarded to a
    // skipped draw (there is no fallback shader).
    if tierIdx >= ctx.world.tierList.len() || mapIdx >= MAPS_PER_TIER {
        return;
    }
    if ctx.world.tierList[tierIdx].mapHandles[mapIdx] == -1 {
        let mapName = ctx.world.tierList[tierIdx].maps[mapIdx].clone();
        ctx.world.tierList[tierIdx].mapHandles[mapIdx] =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("levelshots/{}", mapName));
    }

    let handle = ctx.world.tierList[tierIdx].mapHandles[mapIdx];
    UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, handle);
}

/// Raven `UI_EnglishMapName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2805-2813`
pub fn UI_EnglishMapName(world: &UiWorld, map: &str) -> String {
    for m in world.mapList.iter() {
        if Q_stricmp(map, &m.mapLoadName) == 0 {
            return m.mapName.clone();
        }
    }
    String::new()
}

/// Raven `UI_AIFromName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2844-2852`
pub fn UI_AIFromName(world: &UiWorld, name: &str) -> String {
    for alias in world.aliasList.iter() {
        if Q_stricmp(&alias.name, name) == 0 {
            return alias.ai.clone();
        }
    }
    "Kyle".to_string()
}

/// Raven `UI_NextOpponent`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2886-2900`
pub fn UI_NextOpponent(ctx: &mut UiContext) {
    let opponentName = UI_Cvar_VariableString(ctx, "ui_opponentName");
    let mut i = UI_TeamIndexFromName(ctx.world, &opponentName);
    let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
    let j = UI_TeamIndexFromName(ctx.world, &teamName);

    i += 1;
    if i >= ctx.world.teamList.len() as c_int {
        i = 0;
    }
    if i == j {
        i += 1;
        if i >= ctx.world.teamList.len() as c_int {
            i = 0;
        }
    }
    let name = ctx.world.teamList[i as usize].teamName.clone();
    trap::Cvar_Set(ctx.engine, "ui_opponentName", &name);
}

/// Raven `UI_PriorOpponent`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2902-2916`
pub fn UI_PriorOpponent(ctx: &mut UiContext) {
    let opponentName = UI_Cvar_VariableString(ctx, "ui_opponentName");
    let mut i = UI_TeamIndexFromName(ctx.world, &opponentName);
    let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
    let j = UI_TeamIndexFromName(ctx.world, &teamName);

    i -= 1;
    if i < 0 {
        i = ctx.world.teamList.len() as c_int - 1;
    }
    if i == j {
        i -= 1;
        if i < 0 {
            i = ctx.world.teamList.len() as c_int - 1;
        }
    }
    let name = ctx.world.teamList[i as usize].teamName.clone();
    trap::Cvar_Set(ctx.engine, "ui_opponentName", &name);
}

/// Raven `UI_DrawPlayerLogo`.
///
/// PORT-NOTE: Raven's param is `vec3_t`, but `UI_OwnerDraw` hands it a real
/// `vec4_t` and `trap_R_SetColor` reads all four floats; the port takes
/// `vec4_t` and passes it through unchanged. Same for
/// `UI_DrawPlayerLogoMetal`/`Name` and the `UI_DrawOpponentLogo*` family below.
///
/// Source: `oracle/codemp/ui/ui_main.c:2918-2930`
pub fn UI_DrawPlayerLogo(ctx: &mut UiContext, rect: &RectDef, color: vec4_t) {
    let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
    let i = UI_TeamIndexFromName(ctx.world, &teamName) as usize;
    // PORT-NOTE (§19): Raven read a stale fixed-array slot here when no team is
    // loaded; guarded to a skipped draw (there is no fallback icon).
    if i >= ctx.world.teamList.len() {
        return;
    }

    if ctx.world.teamList[i].teamIcon == -1 {
        let imageName = ctx.world.teamList[i].imageName.clone();
        ctx.world.teamList[i].teamIcon = trap::R_RegisterShaderNoMip(ctx.engine, &imageName);
        ctx.world.teamList[i].teamIcon_Metal =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_metal", imageName));
        ctx.world.teamList[i].teamIcon_Name =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_name", imageName));
    }

    trap::R_SetColor(ctx.engine, Some(&color));
    let icon = ctx.world.teamList[i].teamIcon;
    UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, icon);
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `UI_DrawPlayerLogoMetal`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2932-2943`
pub fn UI_DrawPlayerLogoMetal(ctx: &mut UiContext, rect: &RectDef, color: vec4_t) {
    let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
    let i = UI_TeamIndexFromName(ctx.world, &teamName) as usize;
    // PORT-NOTE (§19): Raven read a stale fixed-array slot here when no team is
    // loaded; guarded to a skipped draw (there is no fallback icon).
    if i >= ctx.world.teamList.len() {
        return;
    }

    if ctx.world.teamList[i].teamIcon == -1 {
        let imageName = ctx.world.teamList[i].imageName.clone();
        ctx.world.teamList[i].teamIcon = trap::R_RegisterShaderNoMip(ctx.engine, &imageName);
        ctx.world.teamList[i].teamIcon_Metal =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_metal", imageName));
        ctx.world.teamList[i].teamIcon_Name =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_name", imageName));
    }

    trap::R_SetColor(ctx.engine, Some(&color));
    let icon = ctx.world.teamList[i].teamIcon_Metal;
    UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, icon);
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `UI_DrawPlayerLogoName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2945-2956`
pub fn UI_DrawPlayerLogoName(ctx: &mut UiContext, rect: &RectDef, color: vec4_t) {
    let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
    let i = UI_TeamIndexFromName(ctx.world, &teamName) as usize;
    // PORT-NOTE (§19): Raven read a stale fixed-array slot here when no team is
    // loaded; guarded to a skipped draw (there is no fallback icon).
    if i >= ctx.world.teamList.len() {
        return;
    }

    if ctx.world.teamList[i].teamIcon == -1 {
        let imageName = ctx.world.teamList[i].imageName.clone();
        ctx.world.teamList[i].teamIcon = trap::R_RegisterShaderNoMip(ctx.engine, &imageName);
        ctx.world.teamList[i].teamIcon_Metal =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_metal", imageName));
        ctx.world.teamList[i].teamIcon_Name =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_name", imageName));
    }

    trap::R_SetColor(ctx.engine, Some(&color));
    let icon = ctx.world.teamList[i].teamIcon_Name;
    UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, icon);
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `UI_DrawOpponentLogo`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2958-2969`
pub fn UI_DrawOpponentLogo(ctx: &mut UiContext, rect: &RectDef, color: vec4_t) {
    let opponentName = UI_Cvar_VariableString(ctx, "ui_opponentName");
    let i = UI_TeamIndexFromName(ctx.world, &opponentName) as usize;
    // PORT-NOTE (§19): Raven read a stale fixed-array slot here when no team is
    // loaded; guarded to a skipped draw (there is no fallback icon).
    if i >= ctx.world.teamList.len() {
        return;
    }

    if ctx.world.teamList[i].teamIcon == -1 {
        let imageName = ctx.world.teamList[i].imageName.clone();
        ctx.world.teamList[i].teamIcon = trap::R_RegisterShaderNoMip(ctx.engine, &imageName);
        ctx.world.teamList[i].teamIcon_Metal =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_metal", imageName));
        ctx.world.teamList[i].teamIcon_Name =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_name", imageName));
    }

    trap::R_SetColor(ctx.engine, Some(&color));
    let icon = ctx.world.teamList[i].teamIcon;
    UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, icon);
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `UI_DrawOpponentLogoMetal`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2971-2982`
pub fn UI_DrawOpponentLogoMetal(ctx: &mut UiContext, rect: &RectDef, color: vec4_t) {
    let opponentName = UI_Cvar_VariableString(ctx, "ui_opponentName");
    let i = UI_TeamIndexFromName(ctx.world, &opponentName) as usize;
    // PORT-NOTE (§19): Raven read a stale fixed-array slot here when no team is
    // loaded; guarded to a skipped draw (there is no fallback icon).
    if i >= ctx.world.teamList.len() {
        return;
    }

    if ctx.world.teamList[i].teamIcon == -1 {
        let imageName = ctx.world.teamList[i].imageName.clone();
        ctx.world.teamList[i].teamIcon = trap::R_RegisterShaderNoMip(ctx.engine, &imageName);
        ctx.world.teamList[i].teamIcon_Metal =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_metal", imageName));
        ctx.world.teamList[i].teamIcon_Name =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_name", imageName));
    }

    trap::R_SetColor(ctx.engine, Some(&color));
    let icon = ctx.world.teamList[i].teamIcon_Metal;
    UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, icon);
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `UI_DrawOpponentLogoName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2984-2995`
pub fn UI_DrawOpponentLogoName(ctx: &mut UiContext, rect: &RectDef, color: vec4_t) {
    let opponentName = UI_Cvar_VariableString(ctx, "ui_opponentName");
    let i = UI_TeamIndexFromName(ctx.world, &opponentName) as usize;
    // PORT-NOTE (§19): Raven read a stale fixed-array slot here when no team is
    // loaded; guarded to a skipped draw (there is no fallback icon).
    if i >= ctx.world.teamList.len() {
        return;
    }

    if ctx.world.teamList[i].teamIcon == -1 {
        let imageName = ctx.world.teamList[i].imageName.clone();
        ctx.world.teamList[i].teamIcon = trap::R_RegisterShaderNoMip(ctx.engine, &imageName);
        ctx.world.teamList[i].teamIcon_Metal =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_metal", imageName));
        ctx.world.teamList[i].teamIcon_Name =
            trap::R_RegisterShaderNoMip(ctx.engine, &format!("{}_name", imageName));
    }

    trap::R_SetColor(ctx.engine, Some(&color));
    let icon = ctx.world.teamList[i].teamIcon_Name;
    UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, icon);
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `UI_BuildPlayerList`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3276-3336`
pub fn UI_BuildPlayerList(ctx: &mut UiContext) {
    let mut cs = uiClientState_t {
        connState: connstate_t::CA_UNINITIALIZED,
        connectPacketCount: 0,
        clientNum: 0,
        servername: [0; MAX_STRING_CHARS],
        updateInfoString: [0; MAX_STRING_CHARS],
        messageString: [0; MAX_STRING_CHARS],
    };
    trap::GetClientState(ctx.engine, &mut cs);

    let info = trap::GetConfigString(ctx.engine, CS_PLAYERS + cs.clientNum, MAX_INFO_STRING)
        .unwrap_or_default();
    ctx.world.playerNumber = cs.clientNum;
    ctx.world.teamLeader = atoi(&Info_ValueForKey(&info, "tl")) != 0;
    let team = atoi(&Info_ValueForKey(&info, "t"));

    let info =
        trap::GetConfigString(ctx.engine, CS_SERVERINFO, MAX_INFO_STRING).unwrap_or_default();
    let count = atoi(&Info_ValueForKey(&info, "sv_maxclients"));

    let mut playerNames: Vec<String> = Vec::new();
    let mut playerIndexes: Vec<c_int> = Vec::new();
    let mut teamNames: Vec<String> = Vec::new();
    let mut teamClientNums: Vec<c_int> = Vec::new();
    let mut playerTeamNumber: c_int = 0;

    let mut n = 0;
    while n < count {
        let info =
            trap::GetConfigString(ctx.engine, CS_PLAYERS + n, MAX_INFO_STRING).unwrap_or_default();

        if !info.is_empty() {
            // Raven `Q_strncpyz(..., MAX_NAME_LENGTH)` truncates the raw name
            // to MAX_NAME_LENGTH-1 bytes BEFORE Q_CleanStr; Latin-1 decoding
            // is one char per byte, so a char-truncate is byte-faithful.
            let raw_name = |info: &str| -> String {
                Info_ValueForKey(info, "n")
                    .chars()
                    .take(MAX_NAME_LENGTH - 1)
                    .collect()
            };
            playerNames.push(Q_CleanStr(&raw_name(&info)));
            playerIndexes.push(n);
            let team2 = atoi(&Info_ValueForKey(&info, "t"));
            if team2 == team && n != ctx.world.playerNumber {
                teamNames.push(Q_CleanStr(&raw_name(&info)));
                teamClientNums.push(n);
                if ctx.world.playerNumber == n {
                    playerTeamNumber = (teamNames.len() - 1) as c_int;
                }
            }
        }
        n += 1;
    }

    ctx.world.playerNames = playerNames;
    ctx.world.playerIndexes = playerIndexes;
    ctx.world.teamNames = teamNames;
    ctx.world.teamClientNums = teamClientNums;

    if !ctx.world.teamLeader {
        trap::Cvar_Set(
            ctx.engine,
            "cg_selectedPlayer",
            &format!("{}", playerTeamNumber),
        );
    }

    let mut n = trap::Cvar_VariableValue(ctx.engine, "cg_selectedPlayer") as c_int;
    if n < 0 || n > ctx.world.teamNames.len() as c_int {
        n = 0;
    }

    if n < ctx.world.teamNames.len() as c_int {
        let name = ctx.world.teamNames[n as usize].clone();
        trap::Cvar_Set(ctx.engine, "cg_selectedPlayerName", &name);
    } else {
        trap::Cvar_Set(ctx.engine, "cg_selectedPlayerName", "Everyone");
    }

    if team == 0 || team == TEAM_SPECTATOR || !ctx.world.teamLeader {
        let n = ctx.world.teamNames.len() as c_int;
        trap::Cvar_Set(ctx.engine, "cg_selectedPlayer", &format!("{}", n));
        trap::Cvar_Set(ctx.engine, "cg_selectedPlayerName", "N/A");
    }
}

/// Raven `UI_Version`.
///
/// PORT-NOTE: `uiDC.textWidth`/`uiDC.drawText` are `DisplayContext` trait
/// methods (DEC-36 D3); the trait is imported here, but `impl DisplayContext
/// for UiContext` has not landed yet (see escalations).
///
/// Source: `oracle/codemp/ui/ui_main.c:3494-3501`
pub fn UI_Version(
    dc: &mut dyn DisplayContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    iMenuFont: c_int,
) {
    let width = dc.textWidth(Q3_VERSION, scale, iMenuFont);
    dc.drawText(
        rect.x - width as f32,
        rect.y,
        scale,
        color,
        Q3_VERSION,
        0.0,
        0,
        0,
        iMenuFont,
    );
}

/// Raven `UI_OwnerDrawVisible`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3781-3891`
pub fn UI_OwnerDrawVisible(ctx: &mut UiContext, flags: c_int) -> bool {
    let mut vis = true;
    let mut flags = flags;

    while flags != 0 {
        if flags & UI_SHOW_FFA != 0 {
            let gt = trap::Cvar_VariableValue(ctx.engine, "g_gametype");
            if gt != GT_FFA as f32 && gt != GT_HOLOCRON as f32 && gt != GT_JEDIMASTER as f32 {
                vis = false;
            }
            flags &= !UI_SHOW_FFA;
        }

        if flags & UI_SHOW_NOTFFA != 0 {
            let gt = trap::Cvar_VariableValue(ctx.engine, "g_gametype");
            if gt == GT_FFA as f32 || gt == GT_HOLOCRON as f32 || gt != GT_JEDIMASTER as f32 {
                vis = false;
            }
            flags &= !UI_SHOW_NOTFFA;
        }

        if flags & UI_SHOW_LEADER != 0 {
            // these need to show when this client can give orders to a player or a group
            if !ctx.world.teamLeader {
                vis = false;
            } else {
                // if showing yourself
                let sel = ctx.world.cvars.ui_selectedPlayer.integer;
                if (sel as usize) < ctx.world.teamClientNums.len()
                    && ctx.world.teamClientNums[sel as usize] == ctx.world.playerNumber
                {
                    vis = false;
                }
            }
            flags &= !UI_SHOW_LEADER;
        }

        if flags & UI_SHOW_NOTLEADER != 0 {
            // these need to show when this client is assigning their own status or they are NOT the leader
            if ctx.world.teamLeader {
                // if not showing yourself
                let sel = ctx.world.cvars.ui_selectedPlayer.integer;
                let showing_self = (sel as usize) < ctx.world.teamClientNums.len()
                    && ctx.world.teamClientNums[sel as usize] == ctx.world.playerNumber;
                if !showing_self {
                    vis = false;
                }
            }
            flags &= !UI_SHOW_NOTLEADER;
        }

        if flags & UI_SHOW_FAVORITESERVERS != 0 {
            // this assumes you only put this type of display flag on something showing in the proper context
            if ctx.world.cvars.ui_netSource.integer != AS_FAVORITES {
                vis = false;
            }
            flags &= !UI_SHOW_FAVORITESERVERS;
        }

        if flags & UI_SHOW_NOTFAVORITESERVERS != 0 {
            // this assumes you only put this type of display flag on something showing in the proper context
            if ctx.world.cvars.ui_netSource.integer == AS_FAVORITES {
                vis = false;
            }
            flags &= !UI_SHOW_NOTFAVORITESERVERS;
        }

        if flags & UI_SHOW_ANYTEAMGAME != 0 {
            let idx = ctx.world.cvars.ui_gameType.integer as usize;
            if ctx.world.gameTypes[idx].gtEnum <= GT_TEAM {
                vis = false;
            }
            flags &= !UI_SHOW_ANYTEAMGAME;
        }

        if flags & UI_SHOW_ANYNONTEAMGAME != 0 {
            let idx = ctx.world.cvars.ui_gameType.integer as usize;
            if ctx.world.gameTypes[idx].gtEnum > GT_TEAM {
                vis = false;
            }
            flags &= !UI_SHOW_ANYNONTEAMGAME;
        }

        if flags & UI_SHOW_NETANYTEAMGAME != 0 {
            let idx = ctx.world.cvars.ui_netGameType.integer as usize;
            if ctx.world.gameTypes[idx].gtEnum <= GT_TEAM {
                vis = false;
            }
            flags &= !UI_SHOW_NETANYTEAMGAME;
        }

        if flags & UI_SHOW_NETANYNONTEAMGAME != 0 {
            let idx = ctx.world.cvars.ui_netGameType.integer as usize;
            if ctx.world.gameTypes[idx].gtEnum > GT_TEAM {
                vis = false;
            }
            flags &= !UI_SHOW_NETANYNONTEAMGAME;
        }

        if flags & UI_SHOW_NEWHIGHSCORE != 0 {
            if ctx.world.newHighScoreTime < ctx.world.uiDC.realTime {
                vis = false;
            } else if ctx.world.soundHighScore
                && trap::Cvar_VariableValue(ctx.engine, "sv_killserver") == 0.0
            {
                // wait on server to go down before playing sound
                // trap_S_StartLocalSound(uiInfo.newHighScoreSound, CHAN_ANNOUNCER);
                ctx.world.soundHighScore = false;
            }
            flags &= !UI_SHOW_NEWHIGHSCORE;
        }

        if flags & UI_SHOW_NEWBESTTIME != 0 {
            if ctx.world.newBestTime < ctx.world.uiDC.realTime {
                vis = false;
            }
            flags &= !UI_SHOW_NEWBESTTIME;
        }

        if flags & UI_SHOW_DEMOAVAILABLE != 0 {
            if !ctx.world.demoAvailable {
                vis = false;
            }
            flags &= !UI_SHOW_DEMOAVAILABLE;
        } else {
            flags = 0;
        }
    }

    vis
}

/// Raven `UI_Handicap_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3893-3911`
pub fn UI_Handicap_HandleKey(
    ctx: &mut UiContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        let mut h = trap::Cvar_VariableValue(ctx.engine, "handicap").clamp(5.0, 100.0) as c_int;
        if key == fakeAscii_t::A_MOUSE2 as c_int {
            h -= 5;
        } else {
            h += 5;
        }
        if h > 100 {
            h = 5;
        } else if h < 0 {
            h = 100;
        }
        trap::Cvar_Set(ctx.engine, "handicap", &format!("{}", h));
        return true;
    }
    false
}

/// Raven `UI_AutoSwitch_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4377-4400`
pub fn UI_AutoSwitch_HandleKey(
    ctx: &mut UiContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        let mut switchVal = trap::Cvar_VariableValue(ctx.engine, "cg_autoswitch") as c_int;

        if key == fakeAscii_t::A_MOUSE2 as c_int {
            switchVal -= 1;
        } else {
            switchVal += 1;
        }

        if switchVal < 0 {
            switchVal = 2;
        } else if switchVal >= 3 {
            switchVal = 0;
        }

        trap::Cvar_Set(ctx.engine, "cg_autoswitch", &format!("{}", switchVal));
        return true;
    }
    false
}

/// Raven `UI_Skill_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4426-4446`
pub fn UI_Skill_HandleKey(
    ctx: &mut UiContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        let mut i = trap::Cvar_VariableValue(ctx.engine, "g_spSkill") as c_int;

        if key == fakeAscii_t::A_MOUSE2 as c_int {
            i -= 1;
        } else {
            i += 1;
        }

        if i < 1 {
            i = NUM_SKILL_LEVELS;
        } else if i > NUM_SKILL_LEVELS {
            i = 1;
        }

        trap::Cvar_Set(ctx.engine, "g_spSkill", &format!("{}", i));
        return true;
    }
    false
}

/// Raven `UI_BotSkill_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4615-4630`
pub fn UI_BotSkill_HandleKey(
    world: &mut UiWorld,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        if key == fakeAscii_t::A_MOUSE2 as c_int {
            world.skillIndex -= 1;
        } else {
            world.skillIndex += 1;
        }
        if world.skillIndex >= NUM_SKILL_LEVELS {
            world.skillIndex = 0;
        } else if world.skillIndex < 0 {
            world.skillIndex = NUM_SKILL_LEVELS - 1;
        }
        return true;
    }
    false
}

/// Raven `UI_RedBlue_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4632-4638`
pub fn UI_RedBlue_HandleKey(
    world: &mut UiWorld,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        world.redBlue ^= 1;
        return true;
    }
    false
}

/// Raven `#define NUM_CROSSHAIRS 9`. No canonical qshared home ported yet, so
/// this stays a file-local const (same treatment as `AS_FAVORITES`/`CIN_LOOP`
/// above).
///
/// Source: `oracle/codemp/ui/ui_shared.h:104`
const NUM_CROSSHAIRS: c_int = 9;

/// Raven `UI_DrawCrosshair`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3262-3269`
pub fn UI_DrawCrosshair(ctx: &mut UiContext, rect: &RectDef, _scale: f32, color: vec4_t) {
    trap::R_SetColor(ctx.engine, Some(&color));
    if ctx.world.currentCrosshair < 0 || ctx.world.currentCrosshair >= NUM_CROSSHAIRS {
        ctx.world.currentCrosshair = 0;
    }
    let shader = ctx.world.uiDC.Assets.crosshairShader[ctx.world.currentCrosshair as usize];
    UI_DrawHandlePic(ctx, rect.x, rect.y, rect.w, rect.h, shader);
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `UI_Crosshair_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4640-4657`
pub fn UI_Crosshair_HandleKey(
    ctx: &mut UiContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        if key == fakeAscii_t::A_MOUSE2 as c_int {
            ctx.world.currentCrosshair -= 1;
        } else {
            ctx.world.currentCrosshair += 1;
        }

        if ctx.world.currentCrosshair >= NUM_CROSSHAIRS {
            ctx.world.currentCrosshair = 0;
        } else if ctx.world.currentCrosshair < 0 {
            ctx.world.currentCrosshair = NUM_CROSSHAIRS - 1;
        }
        trap::Cvar_Set(
            ctx.engine,
            "cg_drawCrosshair",
            &format!("{}", ctx.world.currentCrosshair),
        );
        return true;
    }
    false
}

/// Raven `UI_InSoloMenu`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4300-4320`
pub fn UI_InSoloMenu(world: &UiWorld) -> bool {
    // Get current menu (either video or ingame video, I would assume)
    let menu = Menu_GetFocused(&world.menus);

    if menu.is_none() {
        return false;
    }

    Menu_FindItemByName(&world.menus, menu, "solo_gametypefield").is_some()
}

/// Raven `UI_TeamName_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4449-4471`
pub fn UI_TeamName_HandleKey(
    ctx: &mut UiContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
    blue: bool,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        let cvarName = if blue { "ui_blueTeam" } else { "ui_redTeam" };
        let current = UI_Cvar_VariableString(ctx, cvarName);
        let mut i = UI_TeamIndexFromName(ctx.world, &current);

        if key == fakeAscii_t::A_MOUSE2 as c_int {
            i -= 1;
        } else {
            i += 1;
        }

        if i >= ctx.world.teamList.len() as c_int {
            i = 0;
        } else if i < 0 {
            i = ctx.world.teamList.len() as c_int - 1;
        }

        let name = ctx.world.teamList[i as usize].teamName.clone();
        trap::Cvar_Set(ctx.engine, cvarName, &name);

        return true;
    }
    false
}

/// Raven `UI_TeamMember_HandleKey`.
///
/// Raven's comment: 0 - None, 1 - Human, 2..NumCharacters - Bot.
///
/// Source: `oracle/codemp/ui/ui_main.c:4473-4524`
pub fn UI_TeamMember_HandleKey(
    ctx: &mut UiContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
    blue: bool,
    num: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        let cvar = if blue {
            format!("ui_blueteam{}", num)
        } else {
            format!("ui_redteam{}", num)
        };
        let mut value = trap::Cvar_VariableValue(ctx.engine, &cvar) as c_int;
        let maxcl = trap::Cvar_VariableValue(ctx.engine, "sv_maxClients") as c_int;
        let mut numval = num;

        numval *= 2;
        if blue {
            numval -= 1;
        }

        if numval > maxcl {
            return false;
        }

        if value < 1 {
            value = 1;
        }

        if key == fakeAscii_t::A_MOUSE2 as c_int {
            value -= 1;
        } else {
            value += 1;
        }

        if value >= UI_GetNumBots(ctx.world) + 2 {
            value = 1;
        } else if value < 1 {
            value = UI_GetNumBots(ctx.world) + 2 - 1;
        }

        trap::Cvar_Set(ctx.engine, &cvar, &format!("{}", value));
        return true;
    }
    false
}

/// Raven `UI_BotName_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4583-4613`
pub fn UI_BotName_HandleKey(
    world: &mut UiWorld,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        let mut value = world.botIndex;

        if key == fakeAscii_t::A_MOUSE2 as c_int {
            value -= 1;
        } else {
            value += 1;
        }

        if value >= UI_GetNumBots(world) {
            value = 0;
        } else if value < 0 {
            value = UI_GetNumBots(world) - 1;
        }
        world.botIndex = value;
        return true;
    }
    false
}

/// Raven `UI_SelectedPlayer_HandleKey`.
///
/// Raven's own body never returns `qtrue` from inside the key-match block —
/// it always falls through to the trailing `return qfalse`, transcribed
/// faithfully (porting-rules §2: no speculative behavior fix).
///
/// Source: `oracle/codemp/ui/ui_main.c:4661-4691`
pub fn UI_SelectedPlayer_HandleKey(
    ctx: &mut UiContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        UI_BuildPlayerList(ctx);
        if !ctx.world.teamLeader {
            return false;
        }

        let mut selected = trap::Cvar_VariableValue(ctx.engine, "cg_selectedPlayer") as c_int;

        if key == fakeAscii_t::A_MOUSE2 as c_int {
            selected -= 1;
        } else {
            selected += 1;
        }

        // Raven `uiInfo.myTeamCount` — folded into `teamNames.len()` (§B3).
        let myTeamCount = ctx.world.teamNames.len() as c_int;
        if selected > myTeamCount {
            selected = 0;
        } else if selected < 0 {
            selected = myTeamCount;
        }

        if selected == myTeamCount {
            trap::Cvar_Set(ctx.engine, "cg_selectedPlayerName", "Everyone");
        } else {
            let name = ctx.world.teamNames[selected as usize].clone();
            trap::Cvar_Set(ctx.engine, "cg_selectedPlayerName", &name);
        }
        trap::Cvar_Set(ctx.engine, "cg_selectedPlayer", &format!("{}", selected));
    }
    false
}

/// Raven `UI_GetValue`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4963-4965`
pub fn UI_GetValue(_ownerDraw: c_int) -> f32 {
    0.0
}

/// Raven `UI_ServersQsortCompare`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4972-4974`
pub fn UI_ServersQsortCompare(ctx: &UiContext, arg1: c_int, arg2: c_int) -> c_int {
    trap::LAN_CompareServers(
        ctx.engine,
        ctx.world.cvars.ui_netSource.integer,
        ctx.world.serverStatus.sortKey,
        ctx.world.serverStatus.sortDir,
        arg1,
        arg2,
    )
}

/// Raven `UI_ServersSort`.
///
/// PORT-NOTE: Raven sorts `displayServers` in place with `qsort` over
/// `UI_ServersQsortCompare`; the port uses `Vec::sort_by` with the same
/// trap-backed comparator (`UI_ServersQsortCompare`'s body inlined here — a
/// closure cannot re-borrow `ctx` while `displayServers` is already borrowed
/// mutably), preserving behavior without pulling in `native_sort`'s
/// libc-`qsort`-shaped API (DEC-34's canonical-qsort ruling covers
/// gameplay-visible determinism, not this UI list order).
///
/// Source: `oracle/codemp/ui/ui_main.c:4982-4992`
pub fn UI_ServersSort(ctx: &mut UiContext, column: c_int, force: bool) {
    if !force && ctx.world.serverStatus.sortKey == column {
        return;
    }

    ctx.world.serverStatus.sortKey = column;

    let engine = ctx.engine;
    let sortKey = ctx.world.serverStatus.sortKey;
    let sortDir = ctx.world.serverStatus.sortDir;
    let source = ctx.world.cvars.ui_netSource.integer;
    ctx.world
        .serverStatus
        .displayServers
        .sort_by(|a, b| trap::LAN_CompareServers(engine, source, sortKey, sortDir, *a, *b).cmp(&0));
}

/// Raven `UI_Update`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5255-5406`
pub fn UI_Update(ctx: &mut UiContext, name: &str) {
    let val = trap::Cvar_VariableValue(ctx.engine, name) as c_int;

    if Q_stricmp(name, "s_khz") == 0 {
        trap::Cmd_ExecuteText(
            ctx.engine,
            cbufExec_t::EXEC_APPEND as c_int,
            "snd_restart\n",
        );
        return;
    }

    if Q_stricmp(name, "ui_SetName") == 0 {
        let uiName = UI_Cvar_VariableString(ctx, "ui_Name");
        trap::Cvar_Set(ctx.engine, "name", &uiName);
    } else if Q_stricmp(name, "ui_setRate") == 0 {
        let rate = trap::Cvar_VariableValue(ctx.engine, "rate");
        if rate >= 5000.0 {
            trap::Cvar_Set(ctx.engine, "cl_maxpackets", "30");
            trap::Cvar_Set(ctx.engine, "cl_packetdup", "1");
        } else if rate >= 4000.0 {
            trap::Cvar_Set(ctx.engine, "cl_maxpackets", "15");
            // favor less prediction errors when there's packet loss
            trap::Cvar_Set(ctx.engine, "cl_packetdup", "2");
        } else {
            trap::Cvar_Set(ctx.engine, "cl_maxpackets", "15");
            // favor lower bandwidth
            trap::Cvar_Set(ctx.engine, "cl_packetdup", "1");
        }
    } else if Q_stricmp(name, "ui_GetName") == 0 {
        let clName = UI_Cvar_VariableString(ctx, "name");
        trap::Cvar_Set(ctx.engine, "ui_Name", &clName);
    } else if Q_stricmp(name, "ui_r_colorbits") == 0 {
        match val {
            0 => trap::Cvar_SetValue(ctx.engine, "ui_r_depthbits", 0.0),
            16 => trap::Cvar_SetValue(ctx.engine, "ui_r_depthbits", 16.0),
            32 => trap::Cvar_SetValue(ctx.engine, "ui_r_depthbits", 24.0),
            _ => {}
        }
    } else if Q_stricmp(name, "ui_r_lodbias") == 0 {
        match val {
            0 => trap::Cvar_SetValue(ctx.engine, "ui_r_subdivisions", 4.0),
            1 => trap::Cvar_SetValue(ctx.engine, "ui_r_subdivisions", 12.0),
            2 => trap::Cvar_SetValue(ctx.engine, "ui_r_subdivisions", 20.0),
            _ => {}
        }
    } else if Q_stricmp(name, "ui_r_glCustom") == 0 {
        match val {
            0 => {
                // high quality
                trap::Cvar_SetValue(ctx.engine, "ui_r_fullScreen", 1.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_subdivisions", 4.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_lodbias", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_colorbits", 32.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_depthbits", 24.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_picmip", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_mode", 4.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_texturebits", 32.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_fastSky", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_inGameVideo", 1.0);
                trap::Cvar_Set(ctx.engine, "ui_r_texturemode", "GL_LINEAR_MIPMAP_LINEAR");
            }
            1 => {
                // normal
                trap::Cvar_SetValue(ctx.engine, "ui_r_fullScreen", 1.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_subdivisions", 4.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_lodbias", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_colorbits", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_depthbits", 24.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_picmip", 1.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_mode", 3.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_texturebits", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_fastSky", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_inGameVideo", 1.0);
                trap::Cvar_Set(ctx.engine, "ui_r_texturemode", "GL_LINEAR_MIPMAP_LINEAR");
            }
            2 => {
                // fast
                trap::Cvar_SetValue(ctx.engine, "ui_r_fullScreen", 1.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_subdivisions", 12.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_lodbias", 1.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_colorbits", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_depthbits", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_picmip", 2.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_mode", 3.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_texturebits", 0.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_fastSky", 1.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_inGameVideo", 0.0);
                trap::Cvar_Set(ctx.engine, "ui_r_texturemode", "GL_LINEAR_MIPMAP_NEAREST");
            }
            3 => {
                // fastest
                trap::Cvar_SetValue(ctx.engine, "ui_r_fullScreen", 1.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_subdivisions", 20.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_lodbias", 2.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_colorbits", 16.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_depthbits", 16.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_mode", 3.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_picmip", 3.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_texturebits", 16.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_fastSky", 1.0);
                trap::Cvar_SetValue(ctx.engine, "ui_r_inGameVideo", 0.0);
                trap::Cvar_Set(ctx.engine, "ui_r_texturemode", "GL_LINEAR_MIPMAP_NEAREST");
            }
            _ => {}
        }
    } else if Q_stricmp(name, "ui_mousePitch") == 0 {
        if val == 0 {
            trap::Cvar_SetValue(ctx.engine, "m_pitch", 0.022);
        } else {
            trap::Cvar_SetValue(ctx.engine, "m_pitch", -0.022);
        }
    } else if Q_stricmp(name, "ui_mousePitchVeh") == 0 {
        if val == 0 {
            trap::Cvar_SetValue(ctx.engine, "m_pitchVeh", 0.022);
        } else {
            trap::Cvar_SetValue(ctx.engine, "m_pitchVeh", -0.022);
        }
    }
}

/// Raven `UI_UpdateSaberType`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5951-5961`
pub fn UI_UpdateSaberType(ctx: &mut UiContext) {
    let sType = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_saber_type", MAX_QPATH);

    if Q_stricmp("single", &sType) == 0 || Q_stricmp("staff", &sType) == 0 {
        trap::Cvar_Set(ctx.engine, "ui_saber2", "");
    }
}

/// Raven `UI_UpdateSaberColor`.
///
/// Source: `oracle/codemp/ui/ui_main.c:6020-6022`
pub fn UI_UpdateSaberColor(_secondSaber: bool) {}

/// Raven `UI_GetTeamColor`.
///
/// Source: `oracle/codemp/ui/ui_main.c:7509-7510`
pub fn UI_GetTeamColor(_color: &mut vec4_t) {}

/// Raven `UI_ClampMaxPlayers`.
///
/// Source: `oracle/codemp/ui/ui_main.c:7529-7557`
pub fn UI_ClampMaxPlayers(ctx: &mut UiContext) {
    let idx = ctx.world.cvars.ui_netGameType.integer as usize;

    if ctx.world.gameTypes[idx].gtEnum == GT_DUEL {
        if trap::Cvar_VariableValue(ctx.engine, "sv_maxClients") < 2.0 {
            trap::Cvar_Set(ctx.engine, "sv_maxClients", "2");
        }
    } else if ctx.world.gameTypes[idx].gtEnum == GT_POWERDUEL {
        if trap::Cvar_VariableValue(ctx.engine, "sv_maxClients") < 3.0 {
            trap::Cvar_Set(ctx.engine, "sv_maxClients", "3");
        }
    }

    // max check for all game types
    if trap::Cvar_VariableValue(ctx.engine, "sv_maxClients") > MAX_CLIENTS as f32 {
        trap::Cvar_Set(ctx.engine, "sv_maxClients", &format!("{}", MAX_CLIENTS));
    }
}

/// Raven `UI_MapCountByGameType`.
///
/// Source: `oracle/codemp/ui/ui_main.c:7599-7626`
pub fn UI_MapCountByGameType(world: &mut UiWorld, singlePlayer: bool) -> c_int {
    let mut c = 0;
    let mut game = if singlePlayer {
        world.gameTypes[world.cvars.ui_gameType.integer as usize].gtEnum
    } else {
        world.gameTypes[world.cvars.ui_netGameType.integer as usize].gtEnum
    };
    if game == GT_SINGLE_PLAYER {
        game += 1;
    }
    if game == GT_TEAM {
        game = GT_FFA;
    }
    if game == GT_HOLOCRON || game == GT_JEDIMASTER {
        game = GT_FFA;
    }

    for i in 0..world.mapList.len() {
        world.mapList[i].active = false;
        if world.mapList[i].typeBits & (1 << game) != 0 {
            if singlePlayer && (world.mapList[i].typeBits & (1 << GT_SINGLE_PLAYER)) == 0 {
                continue;
            }
            c += 1;
            world.mapList[i].active = true;
        }
    }
    c
}

/// Raven `UI_hasSkinForBase`.
///
/// Source: `oracle/codemp/ui/ui_main.c:7628-7645`
pub fn UI_hasSkinForBase(ctx: &mut UiContext, base: &str, team: &str) -> bool {
    let mut f: fileHandle_t = 0;

    let test = format!("models/players/{}/{}/lower_default.skin", base, team);
    trap::FS_FOpenFile(ctx.engine, &test, &mut f, FS_READ);
    if f != 0 {
        trap::FS_FCloseFile(ctx.engine, f);
        return true;
    }

    let test = format!(
        "models/players/characters/{}/{}/lower_default.skin",
        base, team
    );
    trap::FS_FOpenFile(ctx.engine, &test, &mut f, FS_READ);
    if f != 0 {
        trap::FS_FCloseFile(ctx.engine, f);
        return true;
    }
    false
}

/// Raven `UI_HeadCountByColor`.
///
/// Source: `oracle/codemp/ui/ui_main.c:7652-7679`
pub fn UI_HeadCountByColor(world: &UiWorld) -> c_int {
    let mut c = 0;
    let teamname = match world.main.uiSkinColor {
        TEAM_BLUE => "/blue",
        TEAM_RED => "/red",
        _ => "/default",
    };

    // Count each head with this color.
    for name in world.q3HeadNames.iter() {
        if name.contains(teamname) {
            c += 1;
        }
    }
    c
}

/// Raven `UI_InsertServerIntoDisplayList`.
///
/// PORT-NOTE: Raven's manual shift-right loop becomes `Vec::insert`
/// (identical resulting order, idiomatic shape — porting-rules §10).
///
/// Source: `oracle/codemp/ui/ui_main.c:7686-7698`
pub fn UI_InsertServerIntoDisplayList(world: &mut UiWorld, num: c_int, position: c_int) {
    if position < 0 || position as usize > world.serverStatus.displayServers.len() {
        return;
    }
    world
        .serverStatus
        .displayServers
        .insert(position as usize, num);
}

/// Raven `UI_RemoveServerFromDisplayList`.
///
/// PORT-NOTE: Raven's manual shift-left loop becomes `Vec::remove` (identical
/// resulting order, idiomatic shape — porting-rules §10).
///
/// Source: `oracle/codemp/ui/ui_main.c:7705-7717`
pub fn UI_RemoveServerFromDisplayList(world: &mut UiWorld, num: c_int) {
    if let Some(i) = world
        .serverStatus
        .displayServers
        .iter()
        .position(|&n| n == num)
    {
        world.serverStatus.displayServers.remove(i);
    }
}

/// Raven `serverStatusCvar_t` — one row of the server-status name/altName
/// rename table. Internal-only (never crosses the ABI seam), so it takes the
/// idiomatic Rust shape.
///
/// Source: `oracle/codemp/ui/ui_main.c:7878-7882`
struct ServerStatusCvar {
    name: &'static str,
    altName: &'static str,
}

/// Raven `serverStatusCvar_t serverStatusCvars[]` — compiled-in data, so it
/// lands beside the function that reads it (§C8). Raven's `{NULL, NULL}`
/// terminator row is kept as the empty-name row the loop breaks on.
///
/// Source: `oracle/codemp/ui/ui_main.c:7884-7895`
const SERVER_STATUS_CVARS: [ServerStatusCvar; 10] = [
    ServerStatusCvar {
        name: "sv_hostname",
        altName: "Name",
    },
    ServerStatusCvar {
        name: "Address",
        altName: "",
    },
    ServerStatusCvar {
        name: "gamename",
        altName: "Game name",
    },
    ServerStatusCvar {
        name: "g_gametype",
        altName: "Game type",
    },
    ServerStatusCvar {
        name: "mapname",
        altName: "Map",
    },
    ServerStatusCvar {
        name: "version",
        altName: "",
    },
    ServerStatusCvar {
        name: "protocol",
        altName: "",
    },
    ServerStatusCvar {
        name: "timelimit",
        altName: "",
    },
    ServerStatusCvar {
        name: "fraglimit",
        altName: "",
    },
    ServerStatusCvar {
        name: "",
        altName: "",
    },
];

/// Raven `UI_SortServerStatusInfo`.
///
/// PORT-NOTE: only columns 0 and 3 are swapped per Raven's original (columns
/// 1/2 are left alone), kept literal here.
///
/// Source: `oracle/codemp/ui/ui_main.c:7901-7930`
pub fn UI_SortServerStatusInfo(info: &mut ServerStatusInfo) {
    // FIXME: if "gamename" == "base" or "missionpack" then
    // replace the gametype number by FFA, CTF etc.
    let mut index = 0usize;
    for cv in SERVER_STATUS_CVARS.iter() {
        if cv.name.is_empty() {
            break;
        }
        for j in 0..info.lines.len() {
            if !info.lines[j][1].is_empty() {
                continue;
            }
            if Q_stricmp(cv.name, &info.lines[j][0]) == 0 {
                // swap lines
                let tmp1 = info.lines[index][0].clone();
                let tmp2 = info.lines[index][3].clone();
                info.lines[index][0] = info.lines[j][0].clone();
                info.lines[index][3] = info.lines[j][3].clone();
                info.lines[j][0] = tmp1;
                info.lines[j][3] = tmp2;

                if !cv.altName.is_empty() {
                    info.lines[index][0] = cv.altName.to_string();
                }
                index += 1;
            }
        }
    }
}

/// Raven `UI_JoinServer`.
///
/// PORT-NOTE: the `_XBOX` live-server/system-link branch is dead PC surface
/// (porting-rules §20) and is dropped.
///
/// Source: `oracle/codemp/ui/ui_main.c:7984-8008`
pub fn UI_JoinServer(ctx: &mut UiContext) {
    trap::Cvar_Set(ctx.engine, "cg_thirdPerson", "0");
    trap::Cvar_Set(ctx.engine, "cg_cameraOrbit", "0");
    trap::Cvar_Set(ctx.engine, "ui_singlePlayerActive", "0");

    if ctx.world.serverStatus.currentServer >= 0
        && ctx.world.serverStatus.currentServer
            < ctx.world.serverStatus.displayServers.len() as c_int
    {
        let num =
            ctx.world.serverStatus.displayServers[ctx.world.serverStatus.currentServer as usize];
        let source = ctx.world.cvars.ui_netSource.integer;
        let buff = trap::LAN_GetServerAddressString(ctx.engine, source, num, 1024);
        trap::Cmd_ExecuteText(
            ctx.engine,
            cbufExec_t::EXEC_APPEND as c_int,
            &format!("connect {}\n", buff),
        );
    }
}

/// Raven `UI_CheckServerName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:8017-8040`
pub fn UI_CheckServerName(ctx: &mut UiContext) {
    let hostname =
        trap::Cvar_VariableStringBuffer(ctx.engine, "sv_hostname", MAX_INFO_STRING as usize);

    let mut changed = false;
    let cleaned: String = hostname
        .chars()
        .map(|c| {
            if c == '\\' || c == ';' || c == '"' {
                changed = true;
                '.'
            } else {
                c
            }
        })
        .collect();

    if changed {
        trap::Cvar_Set(ctx.engine, "sv_hostname", &cleaned);
    }
}

/// Raven `stristr`.
///
/// Source: `oracle/codemp/ui/ui_main.c:8146-8157`
pub fn stristr<'a>(s: &'a str, charset: &str) -> Option<&'a str> {
    let s_bytes = s.as_bytes();
    let charset_bytes = charset.as_bytes();

    for start in 0..s_bytes.len() {
        let mut i = 0;
        while i < charset_bytes.len() && start + i < s_bytes.len() {
            if charset_bytes[i].to_ascii_uppercase() != s_bytes[start + i].to_ascii_uppercase() {
                break;
            }
            i += 1;
        }
        if i == charset_bytes.len() {
            return Some(&s[start..]);
        }
    }
    None
}

/// Raven `UI_SiegeClassNum`.
///
/// PORT-NOTE: `bgNumSiegeClasses`/`bgSiegeClasses` live on ui's own
/// [`BgState`] (`world.bg_state`, DEC-36 addendum 11 — Raven's ui link unit
/// compiled `bg_saga.c` itself). `ptr::eq` keeps Raven's
/// `&bgSiegeClasses[i] == scl` pointer-identity check.
///
/// Source: `oracle/codemp/ui/ui_main.c:8342-8356`
pub fn UI_SiegeClassNum(bg: &BgState, scl: &siegeClass_t) -> c_int {
    let mut i: c_int = 0;
    while i < bg.bgNumSiegeClasses {
        if core::ptr::eq(&bg.bgSiegeClasses[i as usize], scl) {
            return i;
        }
        i += 1;
    }
    0
}

/// Raven `UI_SelectedMap`.
///
/// Source: `oracle/codemp/ui/ui_main.c:8696-8712`
pub fn UI_SelectedMap(world: &UiWorld, index: c_int, actual: &mut c_int) -> String {
    let mut c = 0;
    *actual = 0;

    for i in 0..world.mapList.len() {
        if world.mapList[i].active {
            if c == index {
                *actual = i as c_int;
                return world.mapList[i].mapName.clone();
            } else {
                c += 1;
            }
        }
    }
    String::new()
}

/// Raven `UI_SelectedTeamHead`.
///
/// Source: `oracle/codemp/ui/ui_main.c:8719-8754`
pub fn UI_SelectedTeamHead(world: &UiWorld, index: c_int, actual: &mut c_int) -> String {
    let teamname = match world.main.uiSkinColor {
        TEAM_BLUE => "/blue",
        TEAM_RED => "/red",
        _ => "/default",
    };

    // Count each head with this color.
    let mut c = 0;
    for i in 0..world.q3HeadNames.len() {
        if world.q3HeadNames[i].contains(teamname) {
            if c == index {
                *actual = i as c_int;
                return world.q3HeadNames[i].clone();
            } else {
                c += 1;
            }
        }
    }
    String::new()
}

/// Raven `UI_GetIndexFromSelection`.
///
/// Source: `oracle/codemp/ui/ui_main.c:8757-8769`
pub fn UI_GetIndexFromSelection(world: &UiWorld, actual: c_int) -> c_int {
    let mut c = 0;
    for i in 0..world.mapList.len() {
        if world.mapList[i].active {
            if i as c_int == actual {
                return c;
            }
            c += 1;
        }
    }
    0
}

/// Raven `UI_UpdatePendingPings`.
///
/// Source: `oracle/codemp/ui/ui_main.c:8771-8778`
pub fn UI_UpdatePendingPings(ctx: &mut UiContext) {
    let source = ctx.world.cvars.ui_netSource.integer;
    trap::LAN_ResetPings(ctx.engine, source);
    ctx.world.serverStatus.refreshActive = true;
    ctx.world.serverStatus.refreshtime = ctx.world.uiDC.realTime + 1000;
}

/// Raven `UI_Pause`.
///
/// Source: `oracle/codemp/ui/ui_main.c:10171-10182`
pub fn UI_Pause(ctx: &mut UiContext, b: bool) {
    if b {
        // pause the game and set the ui keycatcher
        trap::Cvar_Set(ctx.engine, "cl_paused", "1");
        trap::Key_SetCatcher(ctx.engine, KEYCATCH_UI);
    } else {
        // unpause the game and clear the ui keycatcher
        let catcher = trap::Key_GetCatcher(ctx.engine);
        trap::Key_SetCatcher(ctx.engine, catcher & !KEYCATCH_UI);
        trap::Key_ClearStates(ctx.engine);
        trap::Cvar_Set(ctx.engine, "cl_paused", "0");
    }
}

/// Raven `UI_PlayCinematic`.
///
/// Source: `oracle/codemp/ui/ui_main.c:10184-10186`
pub fn UI_PlayCinematic(ctx: &mut UiContext, name: &str, x: f32, y: f32, w: f32, h: f32) -> c_int {
    trap::CIN_PlayCinematic(
        ctx.engine,
        name,
        x as c_int,
        y as c_int,
        w as c_int,
        h as c_int,
        CIN_LOOP | CIN_SILENT,
    )
}

/// Raven `UI_DrawCinematic`.
///
/// Source: `oracle/codemp/ui/ui_main.c:10215-10218`
pub fn UI_DrawCinematic(ctx: &mut UiContext, handle: c_int, x: f32, y: f32, w: f32, h: f32) {
    trap::CIN_SetExtents(
        ctx.engine, handle, x as c_int, y as c_int, w as c_int, h as c_int,
    );
    trap::CIN_DrawCinematic(ctx.engine, handle);
}

/// Raven `UI_RunCinematicFrame`.
///
/// Source: `oracle/codemp/ui/ui_main.c:10220-10222`
pub fn UI_RunCinematicFrame(ctx: &mut UiContext, handle: c_int) {
    trap::CIN_RunCinematic(ctx.engine, handle);
}

/// Raven `UI_LoadForceConfig_List`.
///
/// PORT-NOTE: `COM_StripExtension` is qshared's `(name: &str) -> String`
/// out-param-to-return reshape. Raven's `goto nextSearch` two-pass
/// (dark then light) becomes a `bool` flag loop that runs exactly twice
/// (porting-rules §10 — behavior preserved, shape idiomatic).
///
/// Source: `oracle/codemp/ui/ui_main.c:10231-10283`
pub fn UI_LoadForceConfig_List(ctx: &mut UiContext) {
    ctx.world.forceConfigNames.clear();
    ctx.world.forceConfigSide.clear();
    // Always reserve index 0 as the "custom" config. Raven never writes
    // `forceConfigSide[0]` (zeroed static = qfalse); the placeholder keeps the
    // side table index-aligned with `forceConfigNames`.
    ctx.world.forceConfigNames.push("Custom".to_string());
    ctx.world.forceConfigSide.push(false);

    let mut lightSearch = false;
    loop {
        let mut filelist = vec![0u8; 2048];
        let numfiles = if lightSearch {
            // search light side folder
            let n = trap::FS_GetFileList(ctx.engine, "forcecfg/light", "fcf", &mut filelist);
            ctx.world.forceConfigLightIndexBegin = ctx.world.forceConfigNames.len() as c_int - 1;
            n
        } else {
            // search dark side folder
            let n = trap::FS_GetFileList(ctx.engine, "forcecfg/dark", "fcf", &mut filelist);
            ctx.world.forceConfigDarkIndexBegin = ctx.world.forceConfigNames.len() as c_int - 1;
            n
        };

        let names = latin1_to_string(&filelist);
        // Raven walks `fileptr += filelen+1` — empty entries are consumed,
        // not skipped, so no is_empty filter (entry i must stay entry i).
        let mut fileptrs = names.split('\0');

        let mut j = 0;
        while j < numfiles && ctx.world.forceConfigNames.len() < MAX_FORCE_CONFIGS {
            let fileptr = match fileptrs.next() {
                Some(f) => f,
                None => break,
            };
            let configname = COM_StripExtension(fileptr);

            ctx.world.forceConfigSide.push(lightSearch);
            ctx.world.forceConfigNames.push(configname);
            j += 1;
        }

        if !lightSearch {
            lightSearch = true;
        } else {
            break;
        }
    }
}

/// Raven `bIsImageFile`.
///
/// PORT-NOTE: the `_XBOX` `.dds`-only path (and its `_DEBUG` png/tga
/// fallback gate) is dead PC surface (porting-rules §20) and is dropped; the
/// PC build's jpg/png/tga fallback chain is transcribed unconditionally.
///
/// Source: `oracle/codemp/ui/ui_main.c:10292-10322`
pub fn bIsImageFile(ctx: &mut UiContext, dirptr: &str, skinname: &str) -> bool {
    let mut fpath = format!("models/players/{}/icon_{}.jpg", dirptr, skinname);
    let mut f: fileHandle_t = 0;
    trap::FS_FOpenFile(ctx.engine, &fpath, &mut f, FS_READ);

    if f == 0 {
        // not there, try png
        fpath = format!("models/players/{}/icon_{}.png", dirptr, skinname);
        trap::FS_FOpenFile(ctx.engine, &fpath, &mut f, FS_READ);
    }
    if f == 0 {
        // not there, try tga
        fpath = format!("models/players/{}/icon_{}.tga", dirptr, skinname);
        trap::FS_FOpenFile(ctx.engine, &fpath, &mut f, FS_READ);
    }
    let _ = &fpath;

    if f != 0 {
        trap::FS_FCloseFile(ctx.engine, f);
        return true;
    }
    false
}

/// Raven `UI_ParseColorData`.
///
/// PORT-NOTE: `COM_BeginParseSession`/`COM_ParseExt` take qshared's
/// `QSharedScratch` (Raven's `com_lines`/`com_parsename` parse-session
/// globals) and a byte-slice cursor, so `qs` is threaded in as a parameter
/// (ui has no owned scratch home yet — see escalations). `ColorCount` is the
/// `ColorActionText`/`ColorShader` `len()` (`PlayerSpeciesInfo`'s Vec model),
/// so the shader token is held until its action block closes rather than
/// written at `[ColorCount]` and left uncounted on the failure paths.
///
/// Source: `oracle/codemp/ui/ui_main.c:10468-10508`
pub fn UI_ParseColorData(
    qs: &mut QSharedScratch,
    buf: &str,
    species: &mut PlayerSpeciesInfo,
    file: &str,
) -> bool {
    let mut p: Option<&[u8]> = Some(buf.as_bytes());
    COM_BeginParseSession(qs, file);
    species.ColorShader.clear();
    species.ColorActionText.clear();

    while p.is_some() {
        // looking for the shader
        let (token, rest) = COM_ParseExt(qs, p, true);
        p = rest;
        if token.is_empty() {
            return !species.ColorActionText.is_empty();
        }
        let shader = token;

        // looking for action block {
        let (token, rest) = COM_ParseExt(qs, p, true);
        p = rest;
        // Raven tests `token[0]` only, not the whole token.
        if !token.starts_with('{') {
            return false;
        }

        // looking for action commands
        let mut actionText = String::new();
        let (mut token, rest) = COM_ParseExt(qs, p, true);
        p = rest;
        while !token.starts_with('}') {
            if token.is_empty() {
                // EOF
                return false;
            }
            actionText.push_str(&token);
            actionText.push(' ');
            // looking for action commands or final }
            let (next, rest) = COM_ParseExt(qs, p, true);
            p = rest;
            token = next;
        }
        // next color please
        species.ColorShader.push(shader);
        species.ColorActionText.push(actionText);
    }
    true // never get here
}

/// Raven `UI_ReadableSize`.
///
/// Source: `oracle/codemp/ui/ui_main.c:11039-11054`
pub fn UI_ReadableSize(value: c_int) -> String {
    if value > 1024 * 1024 * 1024 {
        // gigs
        format!(
            "{}.{:02} GB",
            value / (1024 * 1024 * 1024),
            (value % (1024 * 1024 * 1024)) * 100 / (1024 * 1024 * 1024)
        )
    } else if value > 1024 * 1024 {
        // megs
        format!(
            "{}.{:02} MB",
            value / (1024 * 1024),
            (value % (1024 * 1024)) * 100 / (1024 * 1024)
        )
    } else if value > 1024 {
        // kilos
        format!("{} KB", value / 1024)
    } else {
        // bytes
        format!("{} bytes", value)
    }
}

/// Raven `UI_PrintTime`.
///
/// Source: `oracle/codemp/ui/ui_main.c:11057-11067`
pub fn UI_PrintTime(time: c_int) -> String {
    let time = time / 1000; // change to seconds

    if time > 3600 {
        // in the hours range
        format!("{} hr {:2} min", time / 3600, (time % 3600) / 60)
    } else if time > 60 {
        // mins
        format!("{:2} min {:2} sec", time / 60, time % 60)
    } else {
        // secs
        format!("{:2} sec", time)
    }
}

/// Raven `cvarTable_t` — one `cvarTable` row. Raven's `vmCvar_t *vmCvar`
/// pointer becomes `field`, the [`UiCvars`] member name holding that cvar
/// (Rust has no runtime field reflection; the `GAME_CVAR_TABLE` precedent in
/// `g_main.rs`).
///
/// Type definition source: `oracle/codemp/ui/ui_main.c:11278-11284`
struct UiCvarTableEntry {
    field: &'static str,
    name: &'static str,
    default: &'static str,
    flags: c_int,
}

/// Raven `cvarTable` — the compiled-in name/default/flags registration table
/// (99 rows, verbatim order). The `#ifdef _XBOX` rows are dead PC surface
/// (porting-rules §20) and are dropped.
///
/// Source: `oracle/codemp/ui/ui_main.c:11399-11532`
const UI_CVAR_TABLE: [UiCvarTableEntry; 99] = [
    UiCvarTableEntry {
        field: "ui_ffa_fraglimit",
        name: "ui_ffa_fraglimit",
        default: "20",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_ffa_timelimit",
        name: "ui_ffa_timelimit",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_selectedModelIndex",
        name: "ui_selectedModelIndex",
        default: "16",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_char_model",
        name: "ui_char_model",
        default: "jedi_tf",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_char_skin_head",
        name: "ui_char_skin_head",
        default: "head_a1",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_char_skin_torso",
        name: "ui_char_skin_torso",
        default: "torso_a1",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_char_skin_legs",
        name: "ui_char_skin_legs",
        default: "lower_a1",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_char_anim",
        name: "ui_char_anim",
        default: "BOTH_WALK1",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_saber_type",
        name: "ui_saber_type",
        default: "single",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_saber",
        name: "ui_saber",
        default: "single_1",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_saber2",
        name: "ui_saber2",
        default: "none",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_saber_color",
        name: "ui_saber_color",
        default: "yellow",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_saber2_color",
        name: "ui_saber2_color",
        default: "yellow",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_char_color_red",
        name: "ui_char_color_red",
        default: "255",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_char_color_green",
        name: "ui_char_color_green",
        default: "255",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_char_color_blue",
        name: "ui_char_color_blue",
        default: "255",
        flags: CVAR_ROM | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_PrecacheModels",
        name: "ui_PrecacheModels",
        default: "0",
        flags: CVAR_ARCHIVE,
    },
    UiCvarTableEntry {
        field: "ui_team_fraglimit",
        name: "ui_team_fraglimit",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_team_timelimit",
        name: "ui_team_timelimit",
        default: "20",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_team_friendly",
        name: "ui_team_friendly",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_ctf_capturelimit",
        name: "ui_ctf_capturelimit",
        default: "8",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_ctf_timelimit",
        name: "ui_ctf_timelimit",
        default: "30",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_ctf_friendly",
        name: "ui_ctf_friendly",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_botsFile",
        name: "g_botsFile",
        default: "",
        flags: CVAR_INIT | CVAR_ROM,
    },
    UiCvarTableEntry {
        field: "ui_spSkill",
        name: "g_spSkill",
        default: "2",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_browserMaster",
        name: "ui_browserMaster",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_browserGameType",
        name: "ui_browserGameType",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_browserSortKey",
        name: "ui_browserSortKey",
        default: "4",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_browserShowFull",
        name: "ui_browserShowFull",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_browserShowEmpty",
        name: "ui_browserShowEmpty",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_drawCrosshair",
        name: "cg_drawCrosshair",
        default: "1",
        flags: CVAR_ARCHIVE,
    },
    UiCvarTableEntry {
        field: "ui_drawCrosshairNames",
        name: "cg_drawCrosshairNames",
        default: "1",
        flags: CVAR_ARCHIVE,
    },
    UiCvarTableEntry {
        field: "ui_marks",
        name: "cg_marks",
        default: "1",
        flags: CVAR_ARCHIVE,
    },
    UiCvarTableEntry {
        field: "ui_debug",
        name: "ui_debug",
        default: "0",
        flags: CVAR_TEMP | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_initialized",
        name: "ui_initialized",
        default: "0",
        flags: CVAR_TEMP | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_opponentName",
        name: "ui_opponentName",
        default: "Rebellion",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_rankChange",
        name: "ui_rankChange",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_freeSaber",
        name: "ui_freeSaber",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_forcePowerDisable",
        name: "ui_forcePowerDisable",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_redteam",
        name: "ui_redteam",
        default: "Empire",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_blueteam",
        name: "ui_blueteam",
        default: "Rebellion",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_dedicated",
        name: "ui_dedicated",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_gameType",
        name: "ui_gametype",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_joinGameType",
        name: "ui_joinGametype",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_netGameType",
        name: "ui_netGametype",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_actualNetGameType",
        name: "ui_actualNetGametype",
        default: "3",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_redteam1",
        name: "ui_redteam1",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_redteam2",
        name: "ui_redteam2",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_redteam3",
        name: "ui_redteam3",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_redteam4",
        name: "ui_redteam4",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_redteam5",
        name: "ui_redteam5",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_redteam6",
        name: "ui_redteam6",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_redteam7",
        name: "ui_redteam7",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_redteam8",
        name: "ui_redteam8",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_blueteam1",
        name: "ui_blueteam1",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_blueteam2",
        name: "ui_blueteam2",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_blueteam3",
        name: "ui_blueteam3",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_blueteam4",
        name: "ui_blueteam4",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_blueteam5",
        name: "ui_blueteam5",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_blueteam6",
        name: "ui_blueteam6",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_blueteam7",
        name: "ui_blueteam7",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_blueteam8",
        name: "ui_blueteam8",
        default: "1",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_netSource",
        name: "ui_netSource",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_menuFiles",
        name: "ui_menuFilesMP",
        default: "ui/jampmenus.txt",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_currentMap",
        name: "ui_currentMap",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_currentNetMap",
        name: "ui_currentNetMap",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_mapIndex",
        name: "ui_mapIndex",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_currentOpponent",
        name: "ui_currentOpponent",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_selectedPlayer",
        name: "cg_selectedPlayer",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_selectedPlayerName",
        name: "cg_selectedPlayerName",
        default: "",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_lastServerRefresh_0",
        name: "ui_lastServerRefresh_0",
        default: "",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_lastServerRefresh_1",
        name: "ui_lastServerRefresh_1",
        default: "",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_lastServerRefresh_2",
        name: "ui_lastServerRefresh_2",
        default: "",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_lastServerRefresh_3",
        name: "ui_lastServerRefresh_3",
        default: "",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_singlePlayerActive",
        name: "ui_singlePlayerActive",
        default: "0",
        flags: CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreAccuracy",
        name: "ui_scoreAccuracy",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreImpressives",
        name: "ui_scoreImpressives",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreExcellents",
        name: "ui_scoreExcellents",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreCaptures",
        name: "ui_scoreCaptures",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreDefends",
        name: "ui_scoreDefends",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreAssists",
        name: "ui_scoreAssists",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreGauntlets",
        name: "ui_scoreGauntlets",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreScore",
        name: "ui_scoreScore",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scorePerfect",
        name: "ui_scorePerfect",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreTeam",
        name: "ui_scoreTeam",
        default: "0 to 0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreBase",
        name: "ui_scoreBase",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreTime",
        name: "ui_scoreTime",
        default: "00:00",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreTimeBonus",
        name: "ui_scoreTimeBonus",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreSkillBonus",
        name: "ui_scoreSkillBonus",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_scoreShutoutBonus",
        name: "ui_scoreShutoutBonus",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_fragLimit",
        name: "ui_fragLimit",
        default: "10",
        flags: CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_captureLimit",
        name: "ui_captureLimit",
        default: "5",
        flags: CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_findPlayer",
        name: "ui_findPlayer",
        default: "Kyle",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_recordSPDemo",
        name: "ui_recordSPDemo",
        default: "0",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "ui_realWarmUp",
        name: "g_warmup",
        default: "20",
        flags: CVAR_ARCHIVE,
    },
    UiCvarTableEntry {
        field: "ui_realCaptureLimit",
        name: "capturelimit",
        default: "0",
        flags: CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_NORESTART,
    },
    UiCvarTableEntry {
        field: "ui_serverStatusTimeOut",
        name: "ui_serverStatusTimeOut",
        default: "7000",
        flags: CVAR_ARCHIVE | CVAR_INTERNAL,
    },
    UiCvarTableEntry {
        field: "se_language",
        name: "se_language",
        default: "english",
        flags: CVAR_ARCHIVE | CVAR_NORESTART,
    },
    UiCvarTableEntry {
        field: "ui_bypassMainMenuLoad",
        name: "ui_bypassMainMenuLoad",
        default: "0",
        flags: CVAR_INTERNAL,
    },
];

/// Maps one `UI_CVAR_TABLE` row's `field` name to its `vmCvar_t` storage in
/// `UiCvars`, standing in for the `cv->vmCvar` pointer Raven's row carries.
impl UiCvars {
    fn field_mut(&mut self, name: &str) -> &mut vmCvar_t {
        let cvars = self;
        match name {
            "ui_ffa_fraglimit" => &mut cvars.ui_ffa_fraglimit,
            "ui_ffa_timelimit" => &mut cvars.ui_ffa_timelimit,
            "ui_selectedModelIndex" => &mut cvars.ui_selectedModelIndex,
            "ui_char_model" => &mut cvars.ui_char_model,
            "ui_char_skin_head" => &mut cvars.ui_char_skin_head,
            "ui_char_skin_torso" => &mut cvars.ui_char_skin_torso,
            "ui_char_skin_legs" => &mut cvars.ui_char_skin_legs,
            "ui_char_anim" => &mut cvars.ui_char_anim,
            "ui_saber_type" => &mut cvars.ui_saber_type,
            "ui_saber" => &mut cvars.ui_saber,
            "ui_saber2" => &mut cvars.ui_saber2,
            "ui_saber_color" => &mut cvars.ui_saber_color,
            "ui_saber2_color" => &mut cvars.ui_saber2_color,
            "ui_char_color_red" => &mut cvars.ui_char_color_red,
            "ui_char_color_green" => &mut cvars.ui_char_color_green,
            "ui_char_color_blue" => &mut cvars.ui_char_color_blue,
            "ui_PrecacheModels" => &mut cvars.ui_PrecacheModels,
            "ui_team_fraglimit" => &mut cvars.ui_team_fraglimit,
            "ui_team_timelimit" => &mut cvars.ui_team_timelimit,
            "ui_team_friendly" => &mut cvars.ui_team_friendly,
            "ui_ctf_capturelimit" => &mut cvars.ui_ctf_capturelimit,
            "ui_ctf_timelimit" => &mut cvars.ui_ctf_timelimit,
            "ui_ctf_friendly" => &mut cvars.ui_ctf_friendly,
            "ui_botsFile" => &mut cvars.ui_botsFile,
            "ui_spSkill" => &mut cvars.ui_spSkill,
            "ui_browserMaster" => &mut cvars.ui_browserMaster,
            "ui_browserGameType" => &mut cvars.ui_browserGameType,
            "ui_browserSortKey" => &mut cvars.ui_browserSortKey,
            "ui_browserShowFull" => &mut cvars.ui_browserShowFull,
            "ui_browserShowEmpty" => &mut cvars.ui_browserShowEmpty,
            "ui_drawCrosshair" => &mut cvars.ui_drawCrosshair,
            "ui_drawCrosshairNames" => &mut cvars.ui_drawCrosshairNames,
            "ui_marks" => &mut cvars.ui_marks,
            "ui_debug" => &mut cvars.ui_debug,
            "ui_initialized" => &mut cvars.ui_initialized,
            "ui_opponentName" => &mut cvars.ui_opponentName,
            "ui_rankChange" => &mut cvars.ui_rankChange,
            "ui_freeSaber" => &mut cvars.ui_freeSaber,
            "ui_forcePowerDisable" => &mut cvars.ui_forcePowerDisable,
            "ui_redteam" => &mut cvars.ui_redteam,
            "ui_blueteam" => &mut cvars.ui_blueteam,
            "ui_dedicated" => &mut cvars.ui_dedicated,
            "ui_gameType" => &mut cvars.ui_gameType,
            "ui_joinGameType" => &mut cvars.ui_joinGameType,
            "ui_netGameType" => &mut cvars.ui_netGameType,
            "ui_actualNetGameType" => &mut cvars.ui_actualNetGameType,
            "ui_redteam1" => &mut cvars.ui_redteam1,
            "ui_redteam2" => &mut cvars.ui_redteam2,
            "ui_redteam3" => &mut cvars.ui_redteam3,
            "ui_redteam4" => &mut cvars.ui_redteam4,
            "ui_redteam5" => &mut cvars.ui_redteam5,
            "ui_redteam6" => &mut cvars.ui_redteam6,
            "ui_redteam7" => &mut cvars.ui_redteam7,
            "ui_redteam8" => &mut cvars.ui_redteam8,
            "ui_blueteam1" => &mut cvars.ui_blueteam1,
            "ui_blueteam2" => &mut cvars.ui_blueteam2,
            "ui_blueteam3" => &mut cvars.ui_blueteam3,
            "ui_blueteam4" => &mut cvars.ui_blueteam4,
            "ui_blueteam5" => &mut cvars.ui_blueteam5,
            "ui_blueteam6" => &mut cvars.ui_blueteam6,
            "ui_blueteam7" => &mut cvars.ui_blueteam7,
            "ui_blueteam8" => &mut cvars.ui_blueteam8,
            "ui_netSource" => &mut cvars.ui_netSource,
            "ui_menuFiles" => &mut cvars.ui_menuFiles,
            "ui_currentMap" => &mut cvars.ui_currentMap,
            "ui_currentNetMap" => &mut cvars.ui_currentNetMap,
            "ui_mapIndex" => &mut cvars.ui_mapIndex,
            "ui_currentOpponent" => &mut cvars.ui_currentOpponent,
            "ui_selectedPlayer" => &mut cvars.ui_selectedPlayer,
            "ui_selectedPlayerName" => &mut cvars.ui_selectedPlayerName,
            "ui_lastServerRefresh_0" => &mut cvars.ui_lastServerRefresh_0,
            "ui_lastServerRefresh_1" => &mut cvars.ui_lastServerRefresh_1,
            "ui_lastServerRefresh_2" => &mut cvars.ui_lastServerRefresh_2,
            "ui_lastServerRefresh_3" => &mut cvars.ui_lastServerRefresh_3,
            "ui_singlePlayerActive" => &mut cvars.ui_singlePlayerActive,
            "ui_scoreAccuracy" => &mut cvars.ui_scoreAccuracy,
            "ui_scoreImpressives" => &mut cvars.ui_scoreImpressives,
            "ui_scoreExcellents" => &mut cvars.ui_scoreExcellents,
            "ui_scoreCaptures" => &mut cvars.ui_scoreCaptures,
            "ui_scoreDefends" => &mut cvars.ui_scoreDefends,
            "ui_scoreAssists" => &mut cvars.ui_scoreAssists,
            "ui_scoreGauntlets" => &mut cvars.ui_scoreGauntlets,
            "ui_scoreScore" => &mut cvars.ui_scoreScore,
            "ui_scorePerfect" => &mut cvars.ui_scorePerfect,
            "ui_scoreTeam" => &mut cvars.ui_scoreTeam,
            "ui_scoreBase" => &mut cvars.ui_scoreBase,
            "ui_scoreTime" => &mut cvars.ui_scoreTime,
            "ui_scoreTimeBonus" => &mut cvars.ui_scoreTimeBonus,
            "ui_scoreSkillBonus" => &mut cvars.ui_scoreSkillBonus,
            "ui_scoreShutoutBonus" => &mut cvars.ui_scoreShutoutBonus,
            "ui_fragLimit" => &mut cvars.ui_fragLimit,
            "ui_captureLimit" => &mut cvars.ui_captureLimit,
            "ui_findPlayer" => &mut cvars.ui_findPlayer,
            "ui_recordSPDemo" => &mut cvars.ui_recordSPDemo,
            "ui_realWarmUp" => &mut cvars.ui_realWarmUp,
            "ui_realCaptureLimit" => &mut cvars.ui_realCaptureLimit,
            "ui_serverStatusTimeOut" => &mut cvars.ui_serverStatusTimeOut,
            "se_language" => &mut cvars.se_language,
            "ui_bypassMainMenuLoad" => &mut cvars.ui_bypassMainMenuLoad,
            other => unreachable!("UI_CVAR_TABLE row field {other:?} has no UiCvars member"),
        }
    }
}

/// Raven `UI_RegisterCvars`.
///
/// Source: `oracle/codemp/ui/ui_main.c:11540-11547`
pub fn UI_RegisterCvars(ctx: &mut UiContext) {
    for cv in UI_CVAR_TABLE.iter() {
        trap::Cvar_Register(
            ctx.engine,
            Some(ctx.world.cvars.field_mut(cv.field)),
            cv.name,
            cv.default,
            cv.flags,
        );
    }
}

/// Raven `UI_UpdateCvars`.
///
/// Source: `oracle/codemp/ui/ui_main.c:11554-11561`
pub fn UI_UpdateCvars(ctx: &mut UiContext) {
    for cv in UI_CVAR_TABLE.iter() {
        trap::Cvar_Update(ctx.engine, ctx.world.cvars.field_mut(cv.field));
    }
}

/// Raven `UI_UpdateVideoSetup`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5473-5493`
pub fn UI_UpdateVideoSetup(ctx: &mut UiContext) {
    let r_mode = UI_Cvar_VariableString(ctx, "ui_r_mode");
    trap::Cvar_Set(ctx.engine, "r_mode", &r_mode);
    let r_fullscreen = UI_Cvar_VariableString(ctx, "ui_r_fullscreen");
    trap::Cvar_Set(ctx.engine, "r_fullscreen", &r_fullscreen);
    let r_colorbits = UI_Cvar_VariableString(ctx, "ui_r_colorbits");
    trap::Cvar_Set(ctx.engine, "r_colorbits", &r_colorbits);
    let r_lodbias = UI_Cvar_VariableString(ctx, "ui_r_lodbias");
    trap::Cvar_Set(ctx.engine, "r_lodbias", &r_lodbias);
    let r_picmip = UI_Cvar_VariableString(ctx, "ui_r_picmip");
    trap::Cvar_Set(ctx.engine, "r_picmip", &r_picmip);
    let r_texturebits = UI_Cvar_VariableString(ctx, "ui_r_texturebits");
    trap::Cvar_Set(ctx.engine, "r_texturebits", &r_texturebits);
    let r_texturemode = UI_Cvar_VariableString(ctx, "ui_r_texturemode");
    trap::Cvar_Set(ctx.engine, "r_texturemode", &r_texturemode);
    let r_detailtextures = UI_Cvar_VariableString(ctx, "ui_r_detailtextures");
    trap::Cvar_Set(ctx.engine, "r_detailtextures", &r_detailtextures);
    let r_ext_compress_textures = UI_Cvar_VariableString(ctx, "ui_r_ext_compress_textures");
    trap::Cvar_Set(
        ctx.engine,
        "r_ext_compress_textures",
        &r_ext_compress_textures,
    );
    let r_depthbits = UI_Cvar_VariableString(ctx, "ui_r_depthbits");
    trap::Cvar_Set(ctx.engine, "r_depthbits", &r_depthbits);
    let r_subdivisions = UI_Cvar_VariableString(ctx, "ui_r_subdivisions");
    trap::Cvar_Set(ctx.engine, "r_subdivisions", &r_subdivisions);
    let r_fastSky = UI_Cvar_VariableString(ctx, "ui_r_fastSky");
    trap::Cvar_Set(ctx.engine, "r_fastSky", &r_fastSky);
    let r_inGameVideo = UI_Cvar_VariableString(ctx, "ui_r_inGameVideo");
    trap::Cvar_Set(ctx.engine, "r_inGameVideo", &r_inGameVideo);
    let r_allowExtensions = UI_Cvar_VariableString(ctx, "ui_r_allowExtensions");
    trap::Cvar_Set(ctx.engine, "r_allowExtensions", &r_allowExtensions);
    let cg_shadows = UI_Cvar_VariableString(ctx, "ui_cg_shadows");
    trap::Cvar_Set(ctx.engine, "cg_shadows", &cg_shadows);
    trap::Cvar_Set(ctx.engine, "ui_r_modified", "0");

    trap::Cmd_ExecuteText(ctx.engine, cbufExec_t::EXEC_APPEND as c_int, "vid_restart;");
}

/// Raven `UI_GetVideoSetup`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5503-5542`
pub fn UI_GetVideoSetup(ctx: &mut UiContext) {
    // Make sure the cvars are registered as read only.
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_glCustom",
        "4",
        CVAR_ROM | CVAR_INTERNAL | CVAR_ARCHIVE,
    );

    trap::Cvar_Register(ctx.engine, None, "ui_r_mode", "0", CVAR_ROM | CVAR_INTERNAL);
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_fullscreen",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_colorbits",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_lodbias",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_picmip",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_texturebits",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_texturemode",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_detailtextures",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_ext_compress_textures",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_depthbits",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_subdivisions",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_fastSky",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_inGameVideo",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_allowExtensions",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_cg_shadows",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_r_modified",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );

    // Copy over the real video cvars into their temporary counterparts
    let r_mode = UI_Cvar_VariableString(ctx, "r_mode");
    trap::Cvar_Set(ctx.engine, "ui_r_mode", &r_mode);
    let r_colorbits = UI_Cvar_VariableString(ctx, "r_colorbits");
    trap::Cvar_Set(ctx.engine, "ui_r_colorbits", &r_colorbits);
    let r_fullscreen = UI_Cvar_VariableString(ctx, "r_fullscreen");
    trap::Cvar_Set(ctx.engine, "ui_r_fullscreen", &r_fullscreen);
    let r_lodbias = UI_Cvar_VariableString(ctx, "r_lodbias");
    trap::Cvar_Set(ctx.engine, "ui_r_lodbias", &r_lodbias);
    let r_picmip = UI_Cvar_VariableString(ctx, "r_picmip");
    trap::Cvar_Set(ctx.engine, "ui_r_picmip", &r_picmip);
    let r_texturebits = UI_Cvar_VariableString(ctx, "r_texturebits");
    trap::Cvar_Set(ctx.engine, "ui_r_texturebits", &r_texturebits);
    let r_texturemode = UI_Cvar_VariableString(ctx, "r_texturemode");
    trap::Cvar_Set(ctx.engine, "ui_r_texturemode", &r_texturemode);
    let r_detailtextures = UI_Cvar_VariableString(ctx, "r_detailtextures");
    trap::Cvar_Set(ctx.engine, "ui_r_detailtextures", &r_detailtextures);
    let r_ext_compress_textures = UI_Cvar_VariableString(ctx, "r_ext_compress_textures");
    trap::Cvar_Set(
        ctx.engine,
        "ui_r_ext_compress_textures",
        &r_ext_compress_textures,
    );
    let r_depthbits = UI_Cvar_VariableString(ctx, "r_depthbits");
    trap::Cvar_Set(ctx.engine, "ui_r_depthbits", &r_depthbits);
    let r_subdivisions = UI_Cvar_VariableString(ctx, "r_subdivisions");
    trap::Cvar_Set(ctx.engine, "ui_r_subdivisions", &r_subdivisions);
    let r_fastSky = UI_Cvar_VariableString(ctx, "r_fastSky");
    trap::Cvar_Set(ctx.engine, "ui_r_fastSky", &r_fastSky);
    let r_inGameVideo = UI_Cvar_VariableString(ctx, "r_inGameVideo");
    trap::Cvar_Set(ctx.engine, "ui_r_inGameVideo", &r_inGameVideo);
    let r_allowExtensions = UI_Cvar_VariableString(ctx, "r_allowExtensions");
    trap::Cvar_Set(ctx.engine, "ui_r_allowExtensions", &r_allowExtensions);
    let cg_shadows = UI_Cvar_VariableString(ctx, "cg_shadows");
    trap::Cvar_Set(ctx.engine, "ui_cg_shadows", &cg_shadows);
    trap::Cvar_Set(ctx.engine, "ui_r_modified", "0");
}

/// Raven `UI_UpdateCharacterCvars`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5575-5602`
pub fn UI_UpdateCharacterCvars(ctx: &mut UiContext) {
    let model = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_char_model", MAX_QPATH as usize);
    let head = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_char_skin_head", MAX_QPATH as usize);
    let torso =
        trap::Cvar_VariableStringBuffer(ctx.engine, "ui_char_skin_torso", MAX_QPATH as usize);
    let legs = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_char_skin_legs", MAX_QPATH as usize);

    // PORT-NOTE: Raven `Com_sprintf` into `char skin[MAX_QPATH]`.
    let skin: String = format!("{}/{}|{}|{}", model, head, torso, legs)
        .chars()
        .take(MAX_QPATH as usize - 1)
        .collect();

    trap::Cvar_Set(ctx.engine, "model", &skin);

    let char_color_red = UI_Cvar_VariableString(ctx, "ui_char_color_red");
    trap::Cvar_Set(ctx.engine, "char_color_red", &char_color_red);
    let char_color_green = UI_Cvar_VariableString(ctx, "ui_char_color_green");
    trap::Cvar_Set(ctx.engine, "char_color_green", &char_color_green);
    let char_color_blue = UI_Cvar_VariableString(ctx, "ui_char_color_blue");
    trap::Cvar_Set(ctx.engine, "char_color_blue", &char_color_blue);
    trap::Cvar_Set(ctx.engine, "ui_selectedModelIndex", "-1");
}

/// Raven `UI_GetCharacterCvars`.
///
/// PORT-NOTE: Raven's `strrchr`/`strchr` pointer walk over the mutable
/// `"model"` cvar string is transcribed as byte-offset splits over the owned
/// `String` (the delimiters `/` and `|` are single-byte ASCII, so byte offsets
/// stay char-boundary-safe under the Latin-1 discipline); the `assert(p2)`
/// guards on the second and third `|` become `.expect(..)`, matching Raven's
/// abort-on-violation behavior.
///
/// Source: `oracle/codemp/ui/ui_main.c:5604-5678`
pub fn UI_GetCharacterCvars(ctx: &mut UiContext) {
    let char_color_red = UI_Cvar_VariableString(ctx, "char_color_red");
    trap::Cvar_Set(ctx.engine, "ui_char_color_red", &char_color_red);
    let char_color_green = UI_Cvar_VariableString(ctx, "char_color_green");
    trap::Cvar_Set(ctx.engine, "ui_char_color_green", &char_color_green);
    let char_color_blue = UI_Cvar_VariableString(ctx, "char_color_blue");
    trap::Cvar_Set(ctx.engine, "ui_char_color_blue", &char_color_blue);

    let model = UI_Cvar_VariableString(ctx, "model");
    if let Some(slash) = model.rfind('/') {
        if model.contains('|') {
            // we have a multipart custom jedi
            let base = model[..slash].to_string();
            let rest = &model[slash + 1..];

            let p1 = rest
                .find('|')
                .expect("multipart custom jedi model string missing '|' separator");
            let skinhead = rest[..p1].to_string();
            let rest2 = &rest[p1 + 1..];

            let p2 = rest2
                .find('|')
                .expect("multipart custom jedi model string missing second '|' separator");
            let skintorso = rest2[..p2].to_string();
            let skinlower = rest2[p2 + 1..].to_string();

            trap::Cvar_Set(ctx.engine, "ui_char_model", &base);
            trap::Cvar_Set(ctx.engine, "ui_char_skin_head", &skinhead);
            trap::Cvar_Set(ctx.engine, "ui_char_skin_torso", &skintorso);
            trap::Cvar_Set(ctx.engine, "ui_char_skin_legs", &skinlower);

            for i in 0..ctx.world.playerSpecies.len() {
                if Q_stricmp(&base, &ctx.world.playerSpecies[i].Name) == 0 {
                    ctx.world.playerSpeciesIndex = i as c_int;
                    break;
                }
            }
            return;
        }
    }

    let model = UI_Cvar_VariableString(ctx, "ui_char_model");
    for i in 0..ctx.world.playerSpecies.len() {
        if Q_stricmp(&model, &ctx.world.playerSpecies[i].Name) == 0 {
            ctx.world.playerSpeciesIndex = i as c_int;
            return; // FOUND IT, don't fall through
        }
    }
    // nope, didn't find it.
    ctx.world.playerSpeciesIndex = 0; // jic
    let name = ctx.world.playerSpecies[ctx.world.playerSpeciesIndex as usize]
        .Name
        .clone();
    trap::Cvar_Set(ctx.engine, "ui_char_model", &name);
    trap::Cvar_Set(ctx.engine, "ui_char_skin_head", "head_a1");
    trap::Cvar_Set(ctx.engine, "ui_char_skin_torso", "torso_a1");
    trap::Cvar_Set(ctx.engine, "ui_char_skin_legs", "lower_a1");
}

/// Raven `UI_UpdateSaberCvars`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5851-5865`
pub fn UI_UpdateSaberCvars(ctx: &mut UiContext) {
    let saber1 = UI_Cvar_VariableString(ctx, "ui_saber");
    trap::Cvar_Set(ctx.engine, "saber1", &saber1);
    let saber2 = UI_Cvar_VariableString(ctx, "ui_saber2");
    trap::Cvar_Set(ctx.engine, "saber2", &saber2);

    let saber_color = UI_Cvar_VariableString(ctx, "ui_saber_color");
    let colorI = TranslateSaberColor(&saber_color, &mut ctx.world.bg_state);
    trap::Cvar_Set(ctx.engine, "color1", &format!("{}", colorI));
    let g_saber_color = UI_Cvar_VariableString(ctx, "ui_saber_color");
    trap::Cvar_Set(ctx.engine, "g_saber_color", &g_saber_color);

    let saber2_color = UI_Cvar_VariableString(ctx, "ui_saber2_color");
    let colorI = TranslateSaberColor(&saber2_color, &mut ctx.world.bg_state);
    trap::Cvar_Set(ctx.engine, "color2", &format!("{}", colorI));
    let g_saber2_color = UI_Cvar_VariableString(ctx, "ui_saber2_color");
    trap::Cvar_Set(ctx.engine, "g_saber2_color", &g_saber2_color);
}

/// Raven `UI_SetSaberBoxesandHilts`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5868-5942`
pub fn UI_SetSaberBoxesandHilts(ctx: &mut UiContext) {
    // Get current menu (either video or ingame video, I would assume)
    let menu = match Menu_GetFocused(&ctx.world.menus) {
        Some(m) => m,
        None => return,
    };

    let sType = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_saber_type", MAX_QPATH as usize);

    let mut getBig = false;

    if Q_stricmp("dual", &sType) != 0 {
        getBig = true;
    } else if Q_stricmp("staff", &sType) != 0 {
        getBig = true;
    }

    if !getBig {
        return;
    }

    if let Some(item) = Menu_FindItemByName(&ctx.world.menus, Some(menu), "box2middle") {
        let window = &mut ctx.world.menus.item_mut(item).window;
        window.rect.x = 212.0;
        window.rect.y = 126.0;
        window.rect.w = 219.0;
        window.rect.h = 44.0;
    }

    if let Some(item) = Menu_FindItemByName(&ctx.world.menus, Some(menu), "box2bottom") {
        let window = &mut ctx.world.menus.item_mut(item).window;
        window.rect.x = 212.0;
        window.rect.y = 170.0;
        window.rect.w = 219.0;
        window.rect.h = 60.0;
    }

    if let Some(item) = Menu_FindItemByName(&ctx.world.menus, Some(menu), "box3middle") {
        let window = &mut ctx.world.menus.item_mut(item).window;
        window.rect.x = 418.0;
        window.rect.y = 126.0;
        window.rect.w = 219.0;
        window.rect.h = 44.0;
    }

    if let Some(item) = Menu_FindItemByName(&ctx.world.menus, Some(menu), "box3bottom") {
        let window = &mut ctx.world.menus.item_mut(item).window;
        window.rect.x = 418.0;
        window.rect.y = 170.0;
        window.rect.w = 219.0;
        window.rect.h = 60.0;
    }
}

/// Raven `UI_GetSaberCvars`.
///
/// Source: `oracle/codemp/ui/ui_main.c:6026-6039`
pub fn UI_GetSaberCvars(ctx: &mut UiContext) {
    let saber1 = UI_Cvar_VariableString(ctx, "saber1");
    trap::Cvar_Set(ctx.engine, "ui_saber", &saber1);
    let saber2 = UI_Cvar_VariableString(ctx, "saber2");
    trap::Cvar_Set(ctx.engine, "ui_saber2", &saber2);

    let color1 = trap::Cvar_VariableValue(ctx.engine, "color1") as saber_colors_t;
    match SaberColorToString(color1) {
        Some(s) => trap::Cvar_Set(ctx.engine, "g_saber_color", s),
        None => trap::Cvar_Reset(ctx.engine, "g_saber_color"),
    }
    let color2 = trap::Cvar_VariableValue(ctx.engine, "color2") as saber_colors_t;
    match SaberColorToString(color2) {
        Some(s) => trap::Cvar_Set(ctx.engine, "g_saber2_color", s),
        None => trap::Cvar_Reset(ctx.engine, "g_saber2_color"),
    }

    let g_saber_color = UI_Cvar_VariableString(ctx, "g_saber_color");
    trap::Cvar_Set(ctx.engine, "ui_saber_color", &g_saber_color);
    let g_saber2_color = UI_Cvar_VariableString(ctx, "g_saber2_color");
    trap::Cvar_Set(ctx.engine, "ui_saber2_color", &g_saber2_color);
}

/// Raven `UI_ResetCharacterListBoxes`.
///
/// Source: `oracle/codemp/ui/ui_main.c:6087-6142`
pub fn UI_ResetCharacterListBoxes(world: &mut UiWorld) {
    let menu = match Menu_GetFocused(&world.menus) {
        Some(m) => m,
        None => return,
    };

    for name in ["headlistbox", "torsolistbox", "lowerlistbox", "colorbox"] {
        if let Some(item) = Menu_FindItemByName(&world.menus, Some(menu), name) {
            let itemDef = world.menus.item_mut(item);
            if let Some(listPtr) = itemDef.typeData.listBox_mut() {
                listPtr.cursorPos = 0;
            }
            itemDef.cursorPos = 0;
        }
    }
}

/// Raven `UI_BinaryServerInsertion`.
///
/// Source: `oracle/codemp/ui/ui_main.c:7724-7756`
pub fn UI_BinaryServerInsertion(ctx: &mut UiContext, num: c_int) {
    // use binary search to insert server
    let mut len = ctx.world.serverStatus.displayServers.len() as c_int;
    let mut mid = len;
    let mut offset: c_int = 0;
    let mut res: c_int = 0;

    while mid > 0 {
        mid = len >> 1;

        let source = ctx.world.cvars.ui_netSource.integer;
        let sortKey = ctx.world.serverStatus.sortKey;
        let sortDir = ctx.world.serverStatus.sortDir;
        let s2 = ctx.world.serverStatus.displayServers[(offset + mid) as usize];
        res = trap::LAN_CompareServers(ctx.engine, source, sortKey, sortDir, num, s2);

        // if equal
        if res == 0 {
            UI_InsertServerIntoDisplayList(ctx.world, num, offset + mid);
            return;
        }
        // if larger
        else if res == 1 {
            offset += mid;
            len -= mid;
        }
        // if smaller
        else {
            len -= mid;
        }
    }
    if res == 1 {
        offset += 1;
    }
    UI_InsertServerIntoDisplayList(ctx.world, num, offset);
}

/// Raven `UI_GetServerStatusInfo`.
///
/// PORT-NOTE: Raven walks `info->text` in place with a `char *p`, nulling
/// delimiters as it goes and storing dangling pointers into the same buffer;
/// the port walks an owned `Vec<char>` with a byte-free character cursor
/// (`pos`) and copies each resolved substring instead of aliasing the buffer.
/// `info->pings`/the raw `lines[i][0]` pointer trick for the player index is
/// replaced by formatting the index directly into the owned cell.
///
/// Source: `oracle/codemp/ui/ui_main.c:8048-8139`
pub fn UI_GetServerStatusInfo(
    ctx: &mut UiContext,
    serverAddress: &str,
    info: Option<&mut ServerStatusInfo>,
) -> bool {
    let Some(info) = info else {
        trap::LAN_ServerStatus(ctx.engine, Some(serverAddress), 0);
        return false;
    };

    *info = ServerStatusInfo::default();
    let (status, text) =
        trap::LAN_ServerStatus(ctx.engine, Some(serverAddress), MAX_SERVERSTATUS_TEXT);
    if status == 0 {
        return false;
    }

    // PORT-NOTE: Raven `Q_strncpyz` into `char address[MAX_ADDRESSLENGTH]`.
    info.address = serverAddress.chars().take(MAX_ADDRESSLENGTH - 1).collect();
    info.lines.push([
        "Address".to_string(),
        String::new(),
        String::new(),
        info.address.clone(),
    ]);

    let buf: Vec<char> = text.chars().collect();
    let find_from = |from: usize, needle: char| -> Option<usize> {
        buf[from..]
            .iter()
            .position(|&c| c == needle)
            .map(|off| from + off)
    };

    // get the cvars
    let mut pos: usize = 0;
    loop {
        if pos >= buf.len() {
            pos = buf.len();
            break;
        }
        let bs = match find_from(pos, '\\') {
            Some(i) => i,
            None => {
                pos = buf.len();
                break;
            }
        };
        let after_bs = bs + 1;
        if after_bs < buf.len() && buf[after_bs] == '\\' {
            pos = after_bs;
            break;
        }
        if after_bs >= buf.len() {
            pos = buf.len();
            break;
        }
        let key_start = after_bs;
        let bs2 = match find_from(key_start, '\\') {
            Some(i) => i,
            None => {
                pos = buf.len();
                break;
            }
        };
        let key: String = buf[key_start..bs2].iter().collect();
        let value_start = bs2 + 1;
        let value_end = find_from(value_start, '\\').unwrap_or(buf.len());
        let value: String = buf[value_start..value_end].iter().collect();

        info.lines.push([key, String::new(), String::new(), value]);
        pos = value_start;
        if info.lines.len() >= MAX_SERVERSTATUS_LINES {
            // PORT-NOTE: Raven NUL-terminates a value only on the following
            // iteration, so the cap-break leaves the last value running to the end
            // of `info->text`.
            let tail: String = buf[value_start..].iter().collect();
            if let Some(last) = info.lines.last_mut() {
                last[3] = tail;
            }
            break;
        }
    }

    // get the player list
    if info.lines.len() < MAX_SERVERSTATUS_LINES - 3 {
        // empty line
        info.lines
            .push([String::new(), String::new(), String::new(), String::new()]);
        // header
        info.lines.push([
            "num".to_string(),
            "score".to_string(),
            "ping".to_string(),
            "name".to_string(),
        ]);
        // parse players
        let mut i: c_int = 0;
        loop {
            if pos >= buf.len() {
                break;
            }
            if buf[pos] == '\\' {
                pos += 1;
            }
            if pos >= buf.len() {
                break;
            }
            let score_start = pos;
            let sp1 = match find_from(pos, ' ') {
                Some(o) => o,
                None => break,
            };
            let score: String = buf[score_start..sp1].iter().collect();
            pos = sp1 + 1;

            let ping_start = pos;
            let sp2 = match find_from(pos, ' ') {
                Some(o) => o,
                None => break,
            };
            let ping: String = buf[ping_start..sp2].iter().collect();
            pos = sp2 + 1;

            let name_start = pos;
            let name_end = find_from(name_start, '\\').unwrap_or(buf.len());
            let name: String = buf[name_start..name_end].iter().collect();

            info.lines.push([format!("{}", i), score, ping, name]);
            if info.lines.len() >= MAX_SERVERSTATUS_LINES {
                // PORT-NOTE: Raven NUL-terminates the name only after this cap
                // check, so the cap-break leaves it running to the end of
                // `info->text`.
                let tail: String = buf[name_start..].iter().collect();
                if let Some(last) = info.lines.last_mut() {
                    last[3] = tail;
                }
                break;
            }

            if name_end >= buf.len() {
                break;
            }
            pos = name_end + 1;
            i += 1;
        }
    }

    UI_SortServerStatusInfo(info);
    true
}

/// Raven `UI_StopCinematic`.
///
/// Source: `oracle/codemp/ui/ui_main.c:10188-10213`
pub fn UI_StopCinematic(ctx: &mut UiContext, handle: c_int) {
    if handle >= 0 {
        trap::CIN_StopCinematic(ctx.engine, handle);
    } else {
        let handle = handle.abs();
        if handle == UI_MAPCINEMATIC {
            let idx = ctx.world.cvars.ui_currentMap.integer as usize;
            if ctx.world.mapList[idx].cinematic >= 0 {
                let cinematic = ctx.world.mapList[idx].cinematic;
                trap::CIN_StopCinematic(ctx.engine, cinematic);
                ctx.world.mapList[idx].cinematic = -1;
            }
        } else if handle == UI_NETMAPCINEMATIC {
            if ctx.world.serverStatus.currentServerCinematic >= 0 {
                let cinematic = ctx.world.serverStatus.currentServerCinematic;
                trap::CIN_StopCinematic(ctx.engine, cinematic);
                ctx.world.serverStatus.currentServerCinematic = -1;
            }
        } else if handle == UI_CLANCINEMATIC {
            let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
            let i = UI_TeamIndexFromName(ctx.world, &teamName);
            if i >= 0 && (i as usize) < ctx.world.teamList.len() {
                let idx = i as usize;
                if ctx.world.teamList[idx].cinematic >= 0 {
                    let cinematic = ctx.world.teamList[idx].cinematic;
                    trap::CIN_StopCinematic(ctx.engine, cinematic);
                    ctx.world.teamList[idx].cinematic = -1;
                }
            }
        }
    }
}

/// Raven `UI_BuildQ3Model_List`.
///
/// PORT-NOTE: the `/*...*/`-commented-out `fpath`/`trap_FS_FOpenFile` probe
/// (superseded by `bIsImageFile`, per Raven's own comment) is dead and is
/// dropped, matching Raven's compiled-out behavior.
///
/// Source: `oracle/codemp/ui/ui_main.c:10330-10441`
pub fn UI_BuildQ3Model_List(ctx: &mut UiContext) {
    ctx.world.q3HeadNames.clear();
    ctx.world.q3HeadIcons.clear();

    // iterate directory of all player models
    let mut dirlist = vec![0u8; 2048];
    let numdirs = trap::FS_GetFileList(ctx.engine, "models/players", "/", &mut dirlist);
    let dirnames = latin1_to_string(&dirlist);
    let mut dirptrs = dirnames.split('\0');

    let mut i = 0;
    while i < numdirs && ctx.world.q3HeadNames.len() < MAX_Q3PLAYERMODELS {
        let dirptr_raw = match dirptrs.next() {
            Some(d) => d,
            None => break,
        };
        let dirptr = dirptr_raw.strip_suffix('/').unwrap_or(dirptr_raw);

        if dirptr == "." || dirptr == ".." {
            i += 1;
            continue;
        }

        let mut filelist = vec![0u8; 2048];
        let numfiles = trap::FS_GetFileList(
            ctx.engine,
            &format!("models/players/{}", dirptr),
            "skin",
            &mut filelist,
        );
        let filenames = latin1_to_string(&filelist);
        let mut fileptrs = filenames.split('\0');

        let mut j = 0;
        while j < numfiles && ctx.world.q3HeadNames.len() < MAX_Q3PLAYERMODELS {
            let fileptr = match fileptrs.next() {
                Some(f) => f,
                None => break,
            };

            let mut skinname = COM_StripExtension(fileptr);
            if let Some(k) = skinname.find('_') {
                skinname = skinname[k..].to_string();
            }

            // PORT-NOTE (§19): Raven takes `&skinname[1]` unconditionally (past the
            // terminator when the stripped name is empty); the empty case is skipped.
            if !skinname.is_empty() {
                // Raven `check = &skinname[1]` — skip exactly one character.
                let check: String = skinname.chars().skip(1).collect();
                if bIsImageFile(ctx, dirptr, &check) {
                    // if it exists
                    if skinname.starts_with('_') {
                        // change character to append properly
                        skinname.replace_range(0..1, "/");
                    }

                    let candidate = format!("{}{}", dirptr, skinname);
                    // check for dupes
                    let iconExists = ctx
                        .world
                        .q3HeadNames
                        .iter()
                        .any(|n| Q_stricmp(&candidate, n) == 0);

                    if !iconExists {
                        // PORT-NOTE: Raven `Com_sprintf` into `q3HeadNames[i][64]`.
                        let candidate: String = candidate.chars().take(63).collect();
                        ctx.world.q3HeadNames.push(candidate);
                        // rww - we are now registering them as they are drawn like the
                        // TA feeder, so as to decrease UI load time.
                        ctx.world.q3HeadIcons.push(0);
                    }
                }
            }

            if ctx.world.q3HeadNames.len() >= MAX_Q3PLAYERMODELS {
                return;
            }
            j += 1;
        }
        i += 1;
    }
}

// DEFERRED: UI_SiegeInit — its bg calls' real ported shapes
// (`BG_SiegeLoadClasses(descBuffer: *mut siegeClassDesc_t, bg: &mut BgState,
// traps: &dyn BgTraps, callbacks: &mut dyn GameCallbacks)`,
// `BG_SiegeLoadTeams(bg: &mut BgState, traps: &dyn BgTraps)`) require a
// `&dyn BgTraps` and a `&mut dyn GameCallbacks` instance; ui has neither an
// implementor nor a wiring point for either trait yet (DEC-36 addendum 11/12
// name the traits, but `impl BgTraps for ...` / `impl GameCallbacks for ...`
// have not landed in `mp_ui`), and `g_UIClassDescriptions` is an owned
// `Vec<String>` with no `*mut siegeClassDesc_t` buffer to hand `descBuffer`.
// Source: `oracle/codemp/ui/ui_main.c:10443-10460`
// Source: `crates/mp/bg/src/bg_saga.rs:1477-1492` (real `BG_SiegeLoadClasses` shape)
// Source: `crates/mp/bg/src/bg_channel/bg_traps.rs:21` (`trait BgTraps`)
// Source: `crates/mp/bg/src/bg_channel/game_callbacks.rs:21` (`trait GameCallbacks`)

/// Raven `UI_BuildPlayerModel_List`.
///
/// PORT-NOTE: the `trap_Cvar_VariableValue("fs_copyfiles") > 0` `.skin`
/// re-open/close probe (a filesystem cache-warm side effect with no
/// observable state change) is transcribed as a fire-and-discard trap pair,
/// matching Raven's own discarded `f`.
///
/// Source: `oracle/codemp/ui/ui_main.c:10515-10654`
pub fn UI_BuildPlayerModel_List(ctx: &mut UiContext, inGameLoad: bool) {
    ctx.world.playerSpecies.clear();
    ctx.world.playerSpeciesIndex = 0;

    // iterate directory of all player models
    let mut dirlist = vec![0u8; 2048];
    let numdirs = trap::FS_GetFileList(ctx.engine, "models/players", "/", &mut dirlist);
    let dirnames = latin1_to_string(&dirlist);
    let mut dirptrs = dirnames.split('\0');

    let mut i = 0;
    while i < numdirs {
        let dirptr_raw = match dirptrs.next() {
            Some(d) => d,
            None => break,
        };
        // Raven tests `dirlen` on the raw entry, then strips one trailing '/'.
        if dirptr_raw.is_empty() {
            i += 1;
            continue;
        }
        let dirptr = dirptr_raw
            .strip_suffix('/')
            .unwrap_or(dirptr_raw)
            .to_string();

        if dirptr == "." || dirptr == ".." {
            i += 1;
            continue;
        }

        let fpath = format!("models/players/{}/PlayerChoice.txt", dirptr);
        let mut f: fileHandle_t = 0;
        let filelen = trap::FS_FOpenFile(ctx.engine, &fpath, &mut f, FS_READ);

        if f != 0 {
            // PORT-NOTE (§19): Raven freads into `char buffer[2048]` (overrun for
            // larger files, embedded NUL ends the parse); the whole-file read takes
            // the defined behavior.
            let mut buffer = vec![0u8; filelen as usize];
            trap::FS_Read(ctx.engine, &mut buffer, f);
            trap::FS_FCloseFile(ctx.engine, f);
            let buffer = latin1_to_string(&buffer);
            // Raven's `buffer[filelen] = 0` NUL-terminates; a NUL inside the file
            // ends the parse there.
            let buffer = match buffer.find('\0') {
                Some(n) => buffer[..n].to_string(),
                None => buffer,
            };

            // record this species
            let mut species = PlayerSpeciesInfo {
                // PORT-NOTE: Raven `Q_strncpyz` into `playerSpecies[].Name[64]`.
                Name: dirptr.chars().take(63).collect(),
                ..PlayerSpeciesInfo::default()
            };

            if !UI_ParseColorData(&mut ctx.world.bg_state.qs, &buffer, &mut species, &fpath) {
                Com_Printf(
                    ctx,
                    &format!(
                        "{}UI_BuildPlayerModel_List: Errors parsing '{}'\n",
                        S_COLOR_RED.to_str().unwrap(),
                        fpath
                    ),
                );
            }

            let mut filelist = vec![0u8; 2048];
            let numfiles = trap::FS_GetFileList(
                ctx.engine,
                &format!("models/players/{}", dirptr),
                ".skin",
                &mut filelist,
            );
            let filenames = latin1_to_string(&filelist);
            let mut fileptrs = filenames.split('\0');

            let mut iSkinParts: c_int = 0;
            let mut j = 0;
            while j < numfiles {
                let fileptr = match fileptrs.next() {
                    Some(f) => f,
                    None => break,
                };

                if trap::Cvar_VariableValue(ctx.engine, "fs_copyfiles") > 0.0 {
                    let mut f2: fileHandle_t = 0;
                    trap::FS_FOpenFile(
                        ctx.engine,
                        &format!("models/players/{}/{}", dirptr, fileptr),
                        &mut f2,
                        FS_READ,
                    );
                    if f2 != 0 {
                        trap::FS_FCloseFile(ctx.engine, f2);
                    }
                }

                let skinname = COM_StripExtension(fileptr);

                if bIsImageFile(ctx, &dirptr, &skinname) {
                    // if it exists.
                    // PORT-NOTE: Raven `Q_strncpyz` into `Skin*Names[][16]`.
                    let stored: String = skinname.chars().take(15).collect();
                    if Q_stricmpn(&skinname, "head_", 5) == 0 {
                        if species.SkinHeadNames.len() < MAX_PLAYERMODELS {
                            species.SkinHeadNames.push(stored);
                            iSkinParts |= 1 << 0;
                        }
                    } else if Q_stricmpn(&skinname, "torso_", 6) == 0 {
                        if species.SkinTorsoNames.len() < MAX_PLAYERMODELS {
                            species.SkinTorsoNames.push(stored);
                            iSkinParts |= 1 << 1;
                        }
                    } else if Q_stricmpn(&skinname, "lower_", 6) == 0 {
                        if species.SkinLegNames.len() < MAX_PLAYERMODELS {
                            species.SkinLegNames.push(stored);
                            iSkinParts |= 1 << 2;
                        }
                    }
                }
                j += 1;
            }

            if iSkinParts != 7 {
                // didn't get a skin for each, then skip this model.
                i += 1;
                continue;
            }

            ctx.world.playerSpecies.push(species);
            if !inGameLoad && ctx.world.cvars.ui_PrecacheModels.integer != 0 {
                let mut ghoul2: *mut c_void = core::ptr::null_mut();
                let modelPath = format!("models/players/{}/model.glm", dirptr);
                let g2Model =
                    trap::G2API_InitGhoul2Model(ctx.engine, &mut ghoul2, &modelPath, 0, 0, 0, 0, 0);
                if g2Model >= 0 {
                    trap::G2API_CleanGhoul2Models(ctx.engine, &mut ghoul2);
                }
            }

            if ctx.world.playerSpecies.len() >= MAX_PLAYERMODELS {
                return;
            }
        }
        i += 1;
    }
}

/// Raven `_UI_IsFullscreen`.
///
/// Source: `oracle/codemp/ui/ui_main.c:11030-11032`
pub fn _UI_IsFullscreen(world: &UiWorld) -> bool {
    Menus_AnyFullScreenVisible(&world.menus)
}

/// Raven `UI_StopServerRefresh`.
///
/// Source: `oracle/codemp/ui/ui_main.c:11569-11588`
pub fn UI_StopServerRefresh(ctx: &mut UiContext) {
    if !ctx.world.serverStatus.refreshActive {
        // not currently refreshing
        return;
    }
    ctx.world.serverStatus.refreshActive = false;
    Com_Printf(
        ctx,
        &format!(
            "{} servers listed in browser with {} players.\n",
            ctx.world.serverStatus.displayServers.len(),
            ctx.world.serverStatus.numPlayersOnServers
        ),
    );
    let count = trap::LAN_GetServerCount(ctx.engine, ctx.world.cvars.ui_netSource.integer);
    let numDisplayServers = ctx.world.serverStatus.displayServers.len() as c_int;
    if count - numDisplayServers > 0 {
        let maxPing = trap::Cvar_VariableValue(ctx.engine, "cl_maxPing") as c_int;
        Com_Printf(
            ctx,
            &format!(
                "{} servers not listed due to filters, packet loss, or pings higher than {}\n",
                count - numDisplayServers,
                maxPing
            ),
        );
    }
}

/// Raven `GetMonthAbbrevString`.
///
/// Source: `oracle/codemp/ui/ui_main.c:979-984`
pub fn GetMonthAbbrevString(ctx: &mut UiContext, iMonth: c_int) -> String {
    let p = GetCRDelineatedString(ctx, "MP_INGAME", "MONTHS", iMonth);
    p.unwrap_or_else(|| "Jan".to_string()) // sanity
}

/// Raven `GetNetSourceString`.
///
/// Source: `oracle/codemp/ui/ui_main.c:999-1004`
pub fn GetNetSourceString(ctx: &mut UiContext, iSource: c_int) -> String {
    let p = GetCRDelineatedString(ctx, "MP_INGAME", "NET_SOURCES", iSource);
    p.unwrap_or_else(|| "??".to_string())
}

/// Raven `Text_PaintWithCursor`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1133-1157`
#[allow(clippy::too_many_arguments)]
pub fn Text_PaintWithCursor(
    ctx: &UiContext,
    x: f32,
    y: f32,
    scale: f32,
    color: vec4_t,
    text: &str,
    cursorPos: c_int,
    cursor: char,
    limit: c_int,
    style: c_int,
    iMenuFont: c_int,
) {
    Text_Paint(ctx, x, y, scale, color, text, 0.0, limit, style, iMenuFont);

    // now print the cursor as well... (excuse the braces, it's for porting
    // C++ to C)
    {
        let textLen = text.chars().count();
        let iCopyCount = if limit != 0 {
            textLen.min(limit as usize)
        } else {
            textLen
        };
        // §19: Raven's `min(iCopyCount, cursorPos)` fed a negative `cursorPos`
        // to `strncpy` as a huge size_t; clamping at 0 is the defined choice.
        let iCopyCount = iCopyCount.min(cursorPos.max(0) as usize);
        let iCopyCount = iCopyCount.min(1024);

        // copy text into temp buffer for pixel measure...
        let sTemp: String = text.chars().take(iCopyCount).collect();

        let iFontIndex = MenuFontToHandle(ctx.world, iMenuFont);
        let iNextXpos = trap::R_Font_StrLenPixels(ctx.engine, &sTemp, iFontIndex, scale);

        Text_Paint(
            ctx,
            x + iNextXpos as f32,
            y,
            scale,
            color,
            &cursor.to_string(),
            0.0,
            limit,
            style | ITEM_TEXTSTYLE_BLINK,
            iMenuFont,
        );
    }
}

/// Raven `Text_Paint_Limit`.
///
/// PORT-NOTE: Raven walks `text` through `trap_AnyLanguage_ReadCharFromString`
/// byte-by-byte into a `char sTemp[4096]`; the port mirrors this over the
/// Latin-1 byte view (`string_to_latin1`/`latin1_to_string`) instead of Rust
/// `char`s, since the wrapper's contract is byte-oriented.
///
/// Source: `oracle/codemp/ui/ui_main.c:1162-1213`
#[allow(clippy::too_many_arguments)]
pub fn Text_Paint_Limit(
    ctx: &mut UiContext,
    maxX: &mut f32,
    x: f32,
    y: f32,
    scale: f32,
    color: vec4_t,
    text: &str,
    adjust: f32,
    limit: c_int,
    iMenuFont: c_int,
) {
    // this is kinda dirty, but...
    let iFontIndex = MenuFontToHandle(ctx.world, iMenuFont);

    let iPixelLen = trap::R_Font_StrLenPixels(ctx.engine, text, iFontIndex, scale);
    if x + iPixelLen as f32 > *maxX {
        // whole text won't fit, so we need to print just the amount that
        // does... Ok, this is slow and tacky, but only called occasionally,
        // and it works...
        let bytes = string_to_latin1(text);
        let mut psText: &[u8] = &bytes;
        let mut sTemp: Vec<u8> = Vec::new();
        let mut lastGoodLen = 0usize;

        while !psText.is_empty() && psText[0] != 0 {
            // Raven's while-condition order: the pixel probe runs before the
            // `psOut` sanity bound, so the trailing probe still fires when the
            // buffer is full.
            let probe = latin1_to_string(&sTemp);
            let probeLen = trap::R_Font_StrLenPixels(ctx.engine, &probe, iFontIndex, scale);
            if x + probeLen as f32 > *maxX {
                break;
            }
            if sTemp.len() >= 4095 {
                break;
            }
            lastGoodLen = sTemp.len();

            let (uiLetter, iAdvanceCount, _isTrailingPunctuation) =
                trap::AnyLanguage_ReadCharFromString(ctx.engine, psText);
            let advance = (iAdvanceCount as usize).min(psText.len().max(1));
            psText = &psText[advance..];

            if uiLetter > 255 {
                sTemp.push((uiLetter >> 8) as u8);
                sTemp.push((uiLetter & 0xFF) as u8);
            } else {
                sTemp.push((uiLetter & 0xFF) as u8);
            }
        }
        sTemp.truncate(lastGoodLen);

        *maxX = 0.0; // feedback
        let sTemp = latin1_to_string(&sTemp);
        Text_Paint(
            ctx,
            x,
            y,
            scale,
            color,
            &sTemp,
            adjust,
            limit,
            ITEM_TEXTSTYLE_NORMAL,
            iMenuFont,
        );
    } else {
        // whole text fits fine, so print it all...
        *maxX = x + iPixelLen as f32; // feedback the next position, as the caller expects
        Text_Paint(
            ctx,
            x,
            y,
            scale,
            color,
            text,
            adjust,
            limit,
            ITEM_TEXTSTYLE_NORMAL,
            iMenuFont,
        );
    }
}

/// Raven `UI_Report`.
///
/// PORT-NOTE: `String_Report`'s ported shape takes `dc: &mut dyn
/// DisplayContext` (DEC-36 addendum 12); `UiContext` does not implement the
/// trait yet, so this fn threads `dc` in beside it (see escalations).
///
/// Source: `oracle/codemp/ui/ui_main.c:1725-1729`
pub fn UI_Report(dc: &mut dyn DisplayContext) {
    String_Report(dc);
    // Font_Report(); — Raven left this call commented out.
}

/// Raven `UI_DrawHandicap`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1897-1904`
pub fn UI_DrawHandicap(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    let h = trap::Cvar_VariableValue(ctx.engine, "handicap");
    let h = Com_Clamp(5.0, 100.0, h) as c_int;
    let i = 20 - h / 5;

    let text = HANDICAP_VALUES[i as usize].unwrap_or("");
    Text_Paint(
        ctx, rect.x, rect.y, scale, color, text, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawClanName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1906-1908`
pub fn UI_DrawClanName(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &teamName, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawGameType`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1955-1958`
pub fn UI_DrawGameType(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    let idx = ctx.world.cvars.ui_gameType.integer as usize;
    // §19: past `numGameTypes` Raven read the fixed array's zeroed spare slot.
    let gtEnum = ctx
        .world
        .gameTypes
        .get(idx)
        .map(|gt| gt.gtEnum)
        .unwrap_or_default();
    let name = UI_GetGameTypeName(ctx, gtEnum);
    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &name, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawNetGameType`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1960-1968`
pub fn UI_DrawNetGameType(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    if ctx.world.cvars.ui_netGameType.integer < 0
        || ctx.world.cvars.ui_netGameType.integer >= ctx.world.gameTypes.len() as c_int
    {
        trap::Cvar_Set(ctx.engine, "ui_netGameType", "0");
        trap::Cvar_Set(ctx.engine, "ui_actualNetGameType", "0");
    }
    let idx = ctx.world.cvars.ui_netGameType.integer as usize;
    // §19: past `numGameTypes` Raven read the fixed array's zeroed spare slot.
    let gtEnum = ctx
        .world
        .gameTypes
        .get(idx)
        .map(|gt| gt.gtEnum)
        .unwrap_or_default();
    let name = UI_GetGameTypeName(ctx, gtEnum);
    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &name, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawAutoSwitch`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1970-1996`
pub fn UI_DrawAutoSwitch(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    let switchVal = trap::Cvar_VariableValue(ctx.engine, "cg_autoswitch") as c_int;

    let switchString = match switchVal {
        2 => "AUTOSWITCH2",
        3 => "AUTOSWITCH3",
        0 => "AUTOSWITCH0",
        _ => "AUTOSWITCH1",
    };

    let stripString = UI_GetStringEdString(ctx, "MP_INGAME", switchString);

    Text_Paint(
        ctx,
        rect.x,
        rect.y,
        scale,
        color,
        &stripString,
        0.0,
        0,
        textStyle,
        iMenuFont,
    );
}

/// Raven `UI_DrawJoinGameType`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1998-2006`
pub fn UI_DrawJoinGameType(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    if ctx.world.cvars.ui_joinGameType.integer < 0
        || ctx.world.cvars.ui_joinGameType.integer > ctx.world.joinGameTypes.len() as c_int
    {
        trap::Cvar_Set(ctx.engine, "ui_joinGameType", "0");
    }

    let idx = ctx.world.cvars.ui_joinGameType.integer as usize;
    // §19: past `numJoinGameTypes` Raven read the fixed array's zeroed spare slot.
    let gtEnum = ctx
        .world
        .joinGameTypes
        .get(idx)
        .map(|gt| gt.gtEnum)
        .unwrap_or_default();
    let name = UI_GetGameTypeName(ctx, gtEnum);
    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &name, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawSkill`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2084-2091`
pub fn UI_DrawSkill(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    let mut i = trap::Cvar_VariableValue(ctx.engine, "g_spSkill") as c_int;
    if i < 1 || i > NUM_SKILL_LEVELS {
        i = 1;
    }
    let text = UI_GetStringEdString(ctx, "MP_INGAME", SKILL_LEVELS[(i - 1) as usize]);
    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &text, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawGenericNum`.
///
/// PORT-NOTE: Raven's `type` param is a Rust keyword; renamed `kind` (§C — no
/// behavior change, `kind` is unused in the body just like Raven's `type`).
///
/// Source: `oracle/codemp/ui/ui_main.c:2094-2107`
#[allow(clippy::too_many_arguments)]
pub fn UI_DrawGenericNum(
    ctx: &UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    val: c_int,
    min: c_int,
    max: c_int,
    _kind: c_int,
    iMenuFont: c_int,
) {
    // Raven computes `i` (clamped to `min`/`max`) here but never reads it —
    // the `Com_sprintf` below formats the unclamped `val` — so `i` is dead;
    // preserved as a no-op to match Raven's control flow exactly.
    let _i = if val < min || val > max { min } else { val };

    let s = format!("{}", val);
    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &s, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawForceMastery`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2109-2126`
#[allow(clippy::too_many_arguments)]
pub fn UI_DrawForceMastery(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    val: c_int,
    min: c_int,
    max: c_int,
    iMenuFont: c_int,
) {
    let mut i = val;
    if i < min {
        i = min;
    }
    if i > max {
        i = max;
    }

    let s = UI_GetStringEdString(ctx, "MP_INGAME", FORCE_MASTERY_LEVELS[i as usize]);
    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &s, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawSkinColor`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2129-2150`
#[allow(clippy::too_many_arguments)]
pub fn UI_DrawSkinColor(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    val: c_int,
    _min: c_int,
    _max: c_int,
    iMenuFont: c_int,
) {
    let s = match val {
        TEAM_RED => trap::SP_GetStringTextString(ctx.engine, "MENUS_TEAM_RED", 256),
        TEAM_BLUE => trap::SP_GetStringTextString(ctx.engine, "MENUS_TEAM_BLUE", 256),
        _ => trap::SP_GetStringTextString(ctx.engine, "MENUS_DEFAULT", 256),
    }
    .unwrap_or_default();

    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &s, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawJediNonJedi`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2321-2353`
#[allow(clippy::too_many_arguments)]
pub fn UI_DrawJediNonJedi(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    val: c_int,
    min: c_int,
    max: c_int,
    iMenuFont: c_int,
) {
    // Raven computes `i` (clamped to `min`/`max`) here but never reads it
    // afterward; preserved as a no-op to match Raven's control flow exactly.
    let _i = if val < min || val > max { min } else { val };

    let _info = trap::GetConfigString(ctx.engine, CS_SERVERINFO, MAX_INFO_VALUE as usize)
        .unwrap_or_default();

    if !UI_TrueJediEnabled(ctx) {
        // true jedi mode is not on, do not draw this button type
        return;
    }

    let s = if val == FORCE_NONJEDI {
        trap::SP_GetStringTextString(ctx.engine, "MENUS_NO", 256)
    } else {
        trap::SP_GetStringTextString(ctx.engine, "MENUS_YES", 256)
    }
    .unwrap_or_default();

    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &s, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawTeamName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2355-2361`
pub fn UI_DrawTeamName(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    blue: bool,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    let cvarName = if blue { "ui_blueTeam" } else { "ui_redTeam" };
    let name = UI_Cvar_VariableString(ctx, cvarName);
    let i = UI_TeamIndexFromName(ctx.world, &name);
    if i >= 0 && (i as usize) < ctx.world.teamList.len() {
        let teamName = ctx.world.teamList[i as usize].teamName.clone();
        let text = format!("{}: {}", if blue { "Blue" } else { "Red" }, teamName);
        Text_Paint(
            ctx, rect.x, rect.y, scale, color, &text, 0.0, 0, textStyle, iMenuFont,
        );
    }
}

/// Raven `UI_DrawTeamMember`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2363-2423`
#[allow(clippy::too_many_arguments)]
pub fn UI_DrawTeamMember(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    blue: bool,
    num: c_int,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    // 0 - None
    // 1 - Human
    // 2..NumCharacters - Bot
    let cvarName = if blue {
        format!("ui_blueteam{}", num)
    } else {
        format!("ui_redteam{}", num)
    };
    let mut value = trap::Cvar_VariableValue(ctx.engine, &cvarName) as c_int;
    let maxcl = trap::Cvar_VariableValue(ctx.engine, "sv_maxClients") as c_int;
    let mut finalColor = color;
    let mut numval = num;

    numval *= 2;

    if blue {
        numval -= 1;
    }

    if numval > maxcl {
        finalColor[0] *= 0.5;
        finalColor[1] *= 0.5;
        finalColor[2] *= 0.5;

        value = -1;
    }

    let netGameIdx = ctx.world.cvars.ui_netGameType.integer as usize;
    // §19: past `numGameTypes` Raven read the fixed array's zeroed spare slot.
    let netGameType = ctx
        .world
        .gameTypes
        .get(netGameIdx)
        .map(|gt| gt.gtEnum)
        .unwrap_or_default();
    if netGameType == GT_SIEGE && value > 1 {
        value = 1;
    }

    let text = if value <= 1 {
        if value == -1 {
            //text = "Closed";
            UI_GetStringEdString(ctx, "MENUS", "CLOSED")
        } else {
            //text = "Human";
            UI_GetStringEdString(ctx, "MENUS", "HUMAN")
        }
    } else {
        let mut value = value - 2;
        if value >= UI_GetNumBots(ctx.world) {
            value = 1;
        }
        UI_GetBotNameByNumber(ctx, value)
    };

    Text_Paint(
        ctx, rect.x, rect.y, scale, finalColor, &text, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawMapTimeToBeat`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2455-2468`
pub fn UI_DrawMapTimeToBeat(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    if ctx.world.cvars.ui_currentMap.integer < 0
        || ctx.world.cvars.ui_currentMap.integer > ctx.world.mapList.len() as c_int
    {
        ctx.world.cvars.ui_currentMap.integer = 0;
        trap::Cvar_Set(ctx.engine, "ui_currentMap", "0");
    }

    let mapIdx = ctx.world.cvars.ui_currentMap.integer as usize;
    let gtIdx = ctx.world.cvars.ui_gameType.integer as usize;
    // §19: past the live count Raven read each fixed array's zeroed spare slot.
    let gtEnum = ctx
        .world
        .gameTypes
        .get(gtIdx)
        .map(|gt| gt.gtEnum)
        .unwrap_or_default();
    let time = ctx
        .world
        .mapList
        .get(mapIdx)
        .map(|m| m.timeToBeat[gtEnum as usize])
        .unwrap_or_default();

    let minutes = time / 60;
    let seconds = time % 60;

    let text = format!("{:02}:{:02}", minutes, seconds);
    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &text, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawMapCinematic`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2472-2500`
pub fn UI_DrawMapCinematic(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    net: bool,
) {
    let mut map = if net {
        ctx.world.cvars.ui_currentNetMap.integer
    } else {
        ctx.world.cvars.ui_currentMap.integer
    };
    if map < 0 || map > ctx.world.mapList.len() as c_int {
        if net {
            ctx.world.cvars.ui_currentNetMap.integer = 0;
            trap::Cvar_Set(ctx.engine, "ui_currentNetMap", "0");
        } else {
            ctx.world.cvars.ui_currentMap.integer = 0;
            trap::Cvar_Set(ctx.engine, "ui_currentMap", "0");
        }
        map = 0;
    }

    let idx = map as usize;
    // §19: past `mapCount` Raven read the map array's zeroed spare slot (whose
    // `cinematic` 0 never reaches the write-back branches).
    let mut cinematic = ctx
        .world
        .mapList
        .get(idx)
        .map(|m| m.cinematic)
        .unwrap_or_default();
    if cinematic >= -1 {
        if cinematic == -1 {
            let loadName = ctx.world.mapList[idx].mapLoadName.clone();
            cinematic = trap::CIN_PlayCinematic(
                ctx.engine,
                &format!("{}.roq", loadName),
                0,
                0,
                0,
                0,
                CIN_LOOP | CIN_SILENT,
            );
            ctx.world.mapList[idx].cinematic = cinematic;
        }
        if cinematic >= 0 {
            trap::CIN_RunCinematic(ctx.engine, cinematic);
            trap::CIN_SetExtents(
                ctx.engine,
                cinematic,
                rect.x as c_int,
                rect.y as c_int,
                rect.w as c_int,
                rect.h as c_int,
            );
            trap::CIN_DrawCinematic(ctx.engine, cinematic);
        } else {
            ctx.world.mapList[idx].cinematic = -2;
        }
    } else {
        UI_DrawMapPreview(ctx, rect, scale, color, net);
    }
}

/// Raven `UI_DrawNetMapCinematic`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2748-2761`
pub fn UI_DrawNetMapCinematic(ctx: &mut UiContext, rect: &RectDef, scale: f32, color: vec4_t) {
    if ctx.world.cvars.ui_currentNetMap.integer < 0
        || ctx.world.cvars.ui_currentNetMap.integer > ctx.world.mapList.len() as c_int
    {
        ctx.world.cvars.ui_currentNetMap.integer = 0;
        trap::Cvar_Set(ctx.engine, "ui_currentNetMap", "0");
    }

    if ctx.world.serverStatus.currentServerCinematic >= 0 {
        let cinematic = ctx.world.serverStatus.currentServerCinematic;
        trap::CIN_RunCinematic(ctx.engine, cinematic);
        trap::CIN_SetExtents(
            ctx.engine,
            cinematic,
            rect.x as c_int,
            rect.y as c_int,
            rect.w as c_int,
            rect.h as c_int,
        );
        trap::CIN_DrawCinematic(ctx.engine, cinematic);
    } else {
        UI_DrawNetMapPreview(ctx, rect, scale, color);
    }
}

/// Raven `UI_DrawNetFilter`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2765-2779`
pub fn UI_DrawNetFilter(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    if ctx.world.cvars.ui_serverFilterType.integer < 0
        || ctx.world.cvars.ui_serverFilterType.integer > SERVER_FILTERS.len() as c_int
    {
        ctx.world.cvars.ui_serverFilterType.integer = 0;
    }

    ctx.world.main.holdSPString =
        trap::SP_GetStringTextString(ctx.engine, "MENUS_GAME", MAX_STRING_CHARS as usize)
            .unwrap_or_default();

    // §19: Raven's `> numServerFilters` bound let the index reach the array's
    // zeroed spare slot; the empty description stands in for that read.
    let description = SERVER_FILTERS
        .get(ctx.world.cvars.ui_serverFilterType.integer as usize)
        .map(|f| f.description)
        .unwrap_or("")
        .to_string();
    ctx.world.main.holdSPString2 =
        trap::SP_GetStringTextString(ctx.engine, &description, MAX_STRING_CHARS as usize)
            .unwrap_or_default();

    let text = format!(
        "{} {}",
        ctx.world.main.holdSPString, ctx.world.main.holdSPString2
    );
    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &text, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawTier`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2782-2789`
pub fn UI_DrawTier(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    let mut i = trap::Cvar_VariableValue(ctx.engine, "ui_currentTier") as c_int;
    if i < 0 || i >= ctx.world.tierList.len() as c_int {
        i = 0;
    }
    // PORT-NOTE (§19): with no tiers loaded Raven read the zeroed
    // `tierList[0]` slot (empty name); `.get` reproduces that.
    let tierName = ctx
        .world
        .tierList
        .get(i as usize)
        .map(|t| t.tierName.clone())
        .unwrap_or_default();
    let text = format!("Tier: {}", tierName);
    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &text, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawTierMapName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2815-2827`
pub fn UI_DrawTierMapName(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    let mut i = trap::Cvar_VariableValue(ctx.engine, "ui_currentTier") as c_int;
    if i < 0 || i >= ctx.world.tierList.len() as c_int {
        i = 0;
    }
    let mut j = trap::Cvar_VariableValue(ctx.engine, "ui_currentMap") as c_int;
    if j < 0 || j > MAPS_PER_TIER as c_int {
        j = 0;
    }

    // §19: Raven's `j > MAPS_PER_TIER` bound let `maps[j]` reach the struct's
    // zeroed spare slot; the empty name stands in for that read.
    let map = ctx
        .world
        .tierList
        .get(i as usize)
        .and_then(|t| t.maps.get(j as usize))
        .cloned()
        .unwrap_or_default();
    let text = UI_EnglishMapName(ctx.world, &map);
    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &text, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawTierGameType`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2829-2841`
pub fn UI_DrawTierGameType(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    let mut i = trap::Cvar_VariableValue(ctx.engine, "ui_currentTier") as c_int;
    if i < 0 || i >= ctx.world.tierList.len() as c_int {
        i = 0;
    }
    let mut j = trap::Cvar_VariableValue(ctx.engine, "ui_currentMap") as c_int;
    if j < 0 || j > MAPS_PER_TIER as c_int {
        j = 0;
    }

    // §19: Raven's `j > MAPS_PER_TIER` bound let `gameTypes[j]` reach the
    // struct's zeroed spare slot; the zeroed index stands in for that read.
    let gtIdx = ctx
        .world
        .tierList
        .get(i as usize)
        .and_then(|t| t.gameTypes.get(j as usize))
        .copied()
        .unwrap_or_default() as usize;
    let text = ctx
        .world
        .gameTypes
        .get(gtIdx)
        .map(|gt| gt.gameType.clone())
        .unwrap_or_default();
    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &text, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawAllMapsSelection`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2997-3002`
pub fn UI_DrawAllMapsSelection(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    net: bool,
    iMenuFont: c_int,
) {
    let map = if net {
        ctx.world.cvars.ui_currentNetMap.integer
    } else {
        ctx.world.cvars.ui_currentMap.integer
    };
    if map >= 0 && map < ctx.world.mapList.len() as c_int {
        let mapName = ctx.world.mapList[map as usize].mapName.clone();
        Text_Paint(
            ctx, rect.x, rect.y, scale, color, &mapName, 0.0, 0, textStyle, iMenuFont,
        );
    }
}

/// Raven `UI_DrawOpponentName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3004-3006`
pub fn UI_DrawOpponentName(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    let name = UI_Cvar_VariableString(ctx, "ui_opponentName");
    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &name, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawBotName`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3238-3247`
pub fn UI_DrawBotName(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    let mut value = ctx.world.botIndex;
    if value >= UI_GetNumBots(ctx.world) {
        value = 0;
    }
    let text = UI_GetBotNameByNumber(ctx, value);
    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &text, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawBotSkill`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3249-3255`
pub fn UI_DrawBotSkill(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    if ctx.world.skillIndex >= 0 && ctx.world.skillIndex < NUM_SKILL_LEVELS {
        let text = UI_GetStringEdString(
            ctx,
            "MP_INGAME",
            SKILL_LEVELS[ctx.world.skillIndex as usize],
        );
        Text_Paint(
            ctx, rect.x, rect.y, scale, color, &text, 0.0, 0, textStyle, iMenuFont,
        );
    }
}

/// Raven `UI_DrawRedBlue`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3257-3260`
pub fn UI_DrawRedBlue(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    let text = if ctx.world.redBlue == 0 {
        UI_GetStringEdString(ctx, "MP_INGAME", "RED")
    } else {
        UI_GetStringEdString(ctx, "MP_INGAME", "BLUE")
    };
    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &text, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawSelectedPlayer`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3339-3345`
pub fn UI_DrawSelectedPlayer(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    if ctx.world.uiDC.realTime > ctx.world.playerRefresh {
        ctx.world.playerRefresh = ctx.world.uiDC.realTime + 3000;
        UI_BuildPlayerList(ctx);
    }
    let name = UI_Cvar_VariableString(ctx, "cg_selectedPlayerName");
    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &name, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_DrawServerRefreshDate`.
///
/// PORT-NOTE: Raven feeds `holdSPString` (a localized string fetched via
/// `trap_SP_GetStringTextString`) to `va()` as a printf-style format string
/// with one `%s`/`%i`-shaped slot; the port substitutes the first `%i`
/// occurrence directly rather than a general-purpose `printf` reimplementation.
/// This assumes the localized string carries exactly one bare `%i` and no other
/// conversion — to be confirmed against the shipped .str files at the live gate.
///
/// Source: `oracle/codemp/ui/ui_main.c:3347-3369`
pub fn UI_DrawServerRefreshDate(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    if ctx.world.serverStatus.refreshActive {
        let lowLight: vec4_t = [
            0.8 * color[0],
            0.8 * color[1],
            0.8 * color[2],
            0.8 * color[3],
        ];
        let mut newColor: vec4_t = [0.0, 0.0, 0.0, 0.0];
        // Raven divides two ints, so the pulse steps in 75ms plateaus.
        let t = 0.5 + 0.5 * ((ctx.world.uiDC.realTime / PULSE_DIVISOR) as f32).sin();
        LerpColor(color, lowLight, &mut newColor, t);

        ctx.world.main.holdSPString = trap::SP_GetStringTextString(
            ctx.engine,
            "MP_INGAME_GETTINGINFOFORSERVERS",
            MAX_STRING_CHARS as usize,
        )
        .unwrap_or_default();
        let count = trap::LAN_GetServerCount(ctx.engine, ctx.world.cvars.ui_netSource.integer);
        let text = ctx
            .world
            .main
            .holdSPString
            .replacen("%i", &format!("{}", count), 1);
        Text_Paint(
            ctx, rect.x, rect.y, scale, newColor, &text, 0.0, 0, textStyle, iMenuFont,
        );
    } else {
        let cvarName = format!(
            "ui_lastServerRefresh_{}",
            ctx.world.cvars.ui_netSource.integer
        );
        let raw = UI_Cvar_VariableString(ctx, &cvarName);
        // PORT-NOTE: Raven `Q_strncpyz(buff, ..., 64)` truncates to 63 usable
        // bytes + a NUL; the owned `String` truncates to 63 chars.
        let buff: String = raw.chars().take(63).collect();

        ctx.world.main.holdSPString = trap::SP_GetStringTextString(
            ctx.engine,
            "MP_INGAME_SERVER_REFRESHTIME",
            MAX_STRING_CHARS as usize,
        )
        .unwrap_or_default();

        let text = format!("{}: {}", ctx.world.main.holdSPString, buff);
        Text_Paint(
            ctx, rect.x, rect.y, scale, color, &text, 0.0, 0, textStyle, iMenuFont,
        );
    }
}

/// Raven `UI_DrawKeyBindStatus`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3430-3438`
pub fn UI_DrawKeyBindStatus(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    if Display_KeyBindPending(&ctx.world.menus) {
        let text = UI_GetStringEdString(ctx, "MP_INGAME", "WAITING_FOR_NEW_KEY");
        Text_Paint(
            ctx, rect.x, rect.y, scale, color, &text, 0.0, 0, textStyle, iMenuFont,
        );
    } else {
        //Text_Paint(rect->x, rect->y, scale, color, "Press ENTER or CLICK to change, Press BACKSPACE to clear", 0, 0, textStyle,iMenuFont);
    }
}

/// Reads a `glconfig_t` C string field. `glconfig_t` stays the frozen ABI
/// struct (`vendor_string`/`version_string`/`renderer_string`/
/// `extensions_string` are raw `*const c_char` the engine fills via
/// `trap_GetGlconfig`); no safe string accessor exists on it yet in this
/// crate, so this helper reads the pointer through a minimal seam-confined
/// `unsafe` block (porting-rules §D11 — unsafe confined to the seam).
fn glconfig_str(p: *const c_char) -> String {
    if p.is_null() {
        return String::new();
    }
    // SAFETY: `trap_GetGlconfig` fills these engine-owned C strings before any
    // ui code reads `uiDC.glconfig`; the read is borrow-only, no ownership
    // transfer or aliasing with Rust-owned memory.
    latin1_to_string(unsafe { CStr::from_ptr(p) }.to_bytes())
}

/// Raven `UI_DrawGLInfo`.
///
/// PORT-NOTE (§19 shape note / escalation): see `glconfig_str` above for the
/// one `unsafe` read this fn needs.
///
/// Source: `oracle/codemp/ui/ui_main.c:3440-3487`
pub fn UI_DrawGLInfo(
    ctx: &UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    let glconfig = &ctx.world.uiDC.glconfig;
    let vendor = glconfig_str(glconfig.vendor_string);
    let version = glconfig_str(glconfig.version_string);
    let renderer = glconfig_str(glconfig.renderer_string);
    // Raven copies into `buff[4096]` with `Q_strncpyz`, truncating the list at
    // 4095 wire bytes — one latin-1 char each after decode.
    let extensions: String = glconfig_str(glconfig.extensions_string)
        .chars()
        .take(4095)
        .collect();

    Text_Paint(
        ctx,
        rect.x + 2.0,
        rect.y,
        scale,
        color,
        &format!("GL_VENDOR: {}", vendor),
        0.0,
        rect.w as c_int,
        textStyle,
        iMenuFont,
    );
    Text_Paint(
        ctx,
        rect.x + 2.0,
        rect.y + 15.0,
        scale,
        color,
        &format!("GL_VERSION: {}: {}", version, renderer),
        0.0,
        rect.w as c_int,
        textStyle,
        iMenuFont,
    );
    Text_Paint(
        ctx,
        rect.x + 2.0,
        rect.y + 30.0,
        scale,
        color,
        &format!(
            "GL_PIXELFORMAT: color({}-bits) Z({}-bits) stencil({}-bits)",
            glconfig.colorBits, glconfig.depthBits, glconfig.stencilBits
        ),
        0.0,
        rect.w as c_int,
        textStyle,
        iMenuFont,
    );

    // build null terminated extension strings
    //
    // PORT-NOTE: Raven tokenizes `extensions_string` in place, writing '\0' at
    // each space; `y` never changes inside that loop, so `y < rect->y + rect->h`
    // is a one-shot gate rather than a running bound. Splitting is on the
    // literal space only, and `lines[128]` caps the token count.
    let mut y: c_int = (rect.y + 45.0) as c_int;
    let lines: Vec<&str> = if (y as f32) < rect.y + rect.h {
        extensions
            .split(' ')
            .filter(|s| !s.is_empty())
            .take(128)
            .collect()
    } else {
        Vec::new()
    };

    let mut i = 0usize;
    while i < lines.len() {
        Text_Paint(
            ctx,
            rect.x + 2.0,
            y as f32,
            scale,
            color,
            lines[i],
            0.0,
            (rect.w / 2.0) as c_int,
            textStyle,
            iMenuFont,
        );
        i += 1;
        if i < lines.len() {
            Text_Paint(
                ctx,
                rect.x + rect.w / 2.0,
                y as f32,
                scale,
                color,
                lines[i],
                0.0,
                (rect.w / 2.0) as c_int,
                textStyle,
                iMenuFont,
            );
            i += 1;
        }
        y += 10;
        if (y as f32) > rect.y + rect.h - 11.0 {
            break;
        }
    }
}

/// Raven `UI_Effects_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3913-3942`
pub fn UI_Effects_HandleKey(
    ctx: &mut UiContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        if !UI_TrueJediEnabled(ctx) {
            let team = trap::Cvar_VariableValue(ctx.engine, "ui_myteam") as c_int;
            if team == TEAM_RED || team == TEAM_BLUE {
                return false;
            }
        }

        if key == fakeAscii_t::A_MOUSE2 as c_int {
            ctx.world.effectsColor -= 1;
        } else {
            ctx.world.effectsColor += 1;
        }

        if ctx.world.effectsColor > 5 {
            ctx.world.effectsColor = 0;
        } else if ctx.world.effectsColor < 0 {
            ctx.world.effectsColor = 5;
        }

        trap::Cvar_SetValue(ctx.engine, "color1", ctx.world.effectsColor as f32);
        return true;
    }
    false
}

/// Raven `UI_GameType_HandleKey`.
///
/// PORT-NOTE: `Menu_SetFeederSelection`'s ported shape takes `dc: &mut dyn
/// DisplayContext` (DEC-36 addendum 12), so this fn carries a `dc` parameter
/// alongside `ctx`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4266-4297`
pub fn UI_GameType_HandleKey(
    ctx: &mut UiContext,
    dc: &mut dyn DisplayContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
    resetMap: bool,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        let oldCount = UI_MapCountByGameType(ctx.world, true);

        // hard coded mess here
        if key == fakeAscii_t::A_MOUSE2 as c_int {
            ctx.world.cvars.ui_gameType.integer -= 1;
            if ctx.world.cvars.ui_gameType.integer == 2 {
                ctx.world.cvars.ui_gameType.integer = 1;
            } else if ctx.world.cvars.ui_gameType.integer < 2 {
                ctx.world.cvars.ui_gameType.integer = ctx.world.gameTypes.len() as c_int - 1;
            }
        } else {
            ctx.world.cvars.ui_gameType.integer += 1;
            if ctx.world.cvars.ui_gameType.integer >= ctx.world.gameTypes.len() as c_int {
                ctx.world.cvars.ui_gameType.integer = 1;
            } else if ctx.world.cvars.ui_gameType.integer == 2 {
                ctx.world.cvars.ui_gameType.integer = 3;
            }
        }

        trap::Cvar_Set(
            ctx.engine,
            "ui_gameType",
            &format!("{}", ctx.world.cvars.ui_gameType.integer),
        );
        UI_SetCapFragLimits(ctx, true);

        let gtIdx = ctx.world.cvars.ui_gameType.integer as usize;
        let mapIdx = ctx.world.cvars.ui_currentMap.integer as usize;
        // §19: out of range Raven read each fixed array's zeroed spare slot and
        // still called `UI_LoadBestScores` with the empty map name.
        let mapLoadName = ctx
            .world
            .mapList
            .get(mapIdx)
            .map(|m| m.mapLoadName.clone())
            .unwrap_or_default();
        let gtEnum = ctx
            .world
            .gameTypes
            .get(gtIdx)
            .map(|gt| gt.gtEnum)
            .unwrap_or_default();
        UI_LoadBestScores(ctx, &mapLoadName, gtEnum);

        if resetMap && oldCount != UI_MapCountByGameType(ctx.world, true) {
            trap::Cvar_Set(ctx.engine, "ui_currentMap", "0");
            Menu_SetFeederSelection(&mut ctx.world.menus, dc, None, FEEDER_MAPS, 0, None);
        }
        return true;
    }
    false
}

/// Raven `UI_NetGameType_HandleKey`.
///
/// PORT-NOTE: `_XBOX` arm dropped (retail MP never built for that target); the
/// `Menu_SetFeederSelection` `dc` parameter follows the same DEC-36 addendum
/// 12 shape as `UI_GameType_HandleKey` above.
///
/// Source: `oracle/codemp/ui/ui_main.c:4322-4375`
pub fn UI_NetGameType_HandleKey(
    ctx: &mut UiContext,
    dc: &mut dyn DisplayContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        if key == fakeAscii_t::A_MOUSE2 as c_int {
            ctx.world.cvars.ui_netGameType.integer -= 1;
            if UI_InSoloMenu(ctx.world) {
                let idx = ctx.world.cvars.ui_netGameType.integer as usize;
                if idx < ctx.world.gameTypes.len() && ctx.world.gameTypes[idx].gtEnum == GT_SIEGE {
                    ctx.world.cvars.ui_netGameType.integer -= 1;
                }
            }
        } else {
            ctx.world.cvars.ui_netGameType.integer += 1;
            if UI_InSoloMenu(ctx.world) {
                let idx = ctx.world.cvars.ui_netGameType.integer as usize;
                if idx < ctx.world.gameTypes.len() && ctx.world.gameTypes[idx].gtEnum == GT_SIEGE {
                    ctx.world.cvars.ui_netGameType.integer += 1;
                }
            }
        }

        if ctx.world.cvars.ui_netGameType.integer < 0 {
            ctx.world.cvars.ui_netGameType.integer = ctx.world.gameTypes.len() as c_int - 1;
        } else if ctx.world.cvars.ui_netGameType.integer >= ctx.world.gameTypes.len() as c_int {
            ctx.world.cvars.ui_netGameType.integer = 0;
        }

        trap::Cvar_Set(
            ctx.engine,
            "ui_netGameType",
            &format!("{}", ctx.world.cvars.ui_netGameType.integer),
        );
        let gtIdx = ctx.world.cvars.ui_netGameType.integer as usize;
        let gtEnum = if gtIdx < ctx.world.gameTypes.len() {
            ctx.world.gameTypes[gtIdx].gtEnum
        } else {
            0
        };
        trap::Cvar_Set(ctx.engine, "ui_actualnetGameType", &format!("{}", gtEnum));
        trap::Cvar_Set(ctx.engine, "ui_currentNetMap", "0");
        UI_MapCountByGameType(ctx.world, false);
        Menu_SetFeederSelection(&mut ctx.world.menus, dc, None, FEEDER_ALLMAPS, 0, None);
        return true;
    }
    false
}

/// Raven `UI_OpponentName_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4571-4581`
pub fn UI_OpponentName_HandleKey(
    ctx: &mut UiContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        if key == fakeAscii_t::A_MOUSE2 as c_int {
            UI_PriorOpponent(ctx);
        } else {
            UI_NextOpponent(ctx);
        }
        return true;
    }
    false
}

/// Raven `UI_LoadMods`.
///
/// PORT-NOTE: `String_Alloc`'s pool-intern role is dropped — `ModInfo` owns
/// `modName`/`modDescr` as plain `String`s (Class C), matching the rest of
/// `UiWorld`'s owned-string convention.
///
/// Source: `oracle/codemp/ui/ui_main.c:5039-5062`
pub fn UI_LoadMods(ctx: &mut UiContext) {
    ctx.world.modList.clear();

    let mut dirlist = vec![0u8; 2048];
    let numdirs = trap::FS_GetFileList(ctx.engine, "$modlist", "", &mut dirlist);

    let mut offset = 0usize;
    let mut i = 0;
    while i < numdirs {
        // §19: Raven walked the buffer on the returned count alone, so an
        // overclaiming count over-read past it; the cursor is clamped instead.
        offset = offset.min(dirlist.len());
        let name_end = dirlist[offset..]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(dirlist.len() - offset);
        let modName = latin1_to_string(&dirlist[offset..offset + name_end]);

        let desc_start = (offset + name_end + 1).min(dirlist.len());
        let desc_end = dirlist[desc_start..]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(dirlist.len() - desc_start);
        let modDescr = latin1_to_string(&dirlist[desc_start..desc_start + desc_end]);

        ctx.world.modList.push(ModInfo { modName, modDescr });

        offset = desc_start + desc_end + 1;
        i += 1;
        if ctx.world.modList.len() >= MAX_MODS {
            break;
        }
    }
}

/// Raven `UI_LoadMovies`.
///
/// PORT-NOTE: the `.roq` suffix check and `Q_strupr` both operate byte-wise on
/// the raw NUL-separated listbuf before Latin-1 decoding (`eq_ignore_ascii_case`
/// / `to_ascii_uppercase` on the decoded owned `String`), so a
/// multi-byte-after-decode char never shifts the byte offsets the loop walks.
/// Raven's `Q_strupr` folds only `a`-`z`, so the case mapping stays ASCII-only.
/// `String_Alloc`'s pool-intern role is dropped, same as `UI_LoadMods` above.
///
/// Source: `oracle/codemp/ui/ui_main.c:5070-5093`
pub fn UI_LoadMovies(ctx: &mut UiContext) {
    ctx.world.movieList.clear();

    let mut movielist = vec![0u8; 4096];
    let count = trap::FS_GetFileList(ctx.engine, "video", "roq", &mut movielist);

    if count != 0 {
        let count = if count > MAX_MOVIES {
            MAX_MOVIES
        } else {
            count
        };

        let mut offset = 0usize;
        let mut i = 0;
        while i < count {
            // §19: cursor clamped against an overclaiming count, as `UI_LoadMods`.
            offset = offset.min(movielist.len());
            let end = movielist[offset..]
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(movielist.len() - offset);
            let mut raw_end = end;
            if end >= 4 && movielist[offset + end - 4..offset + end].eq_ignore_ascii_case(b".roq") {
                raw_end = end - 4;
            }
            let name = latin1_to_string(&movielist[offset..offset + raw_end]).to_ascii_uppercase();
            ctx.world.movieList.push(name);

            offset += end + 1;
            i += 1;
        }
    }
}

/// Raven `UI_LoadDemos`.
///
/// PORT-NOTE: same byte-wise suffix-strip / decode-once shape as
/// `UI_LoadMovies` above (the extension length varies with the protocol
/// number here, so the strip width is computed per call instead of the fixed
/// `4`).
///
/// Source: `oracle/codemp/ui/ui_main.c:5102-5130`
pub fn UI_LoadDemos(ctx: &mut UiContext) {
    let protocol = trap::Cvar_VariableValue(ctx.engine, "protocol") as c_int;
    let demoExt = format!("dm_{}", protocol);

    ctx.world.demoList.clear();

    let mut demolist = vec![0u8; 4096];
    let count = trap::FS_GetFileList(ctx.engine, "demos", &demoExt, &mut demolist);

    let demoExt = format!(".dm_{}", protocol);
    let ext_len = demoExt.len();

    if count != 0 {
        let count = if count > MAX_DEMOS { MAX_DEMOS } else { count };

        let mut offset = 0usize;
        let mut i = 0;
        while i < count {
            // §19: cursor clamped against an overclaiming count, as `UI_LoadMods`.
            offset = offset.min(demolist.len());
            let end = demolist[offset..]
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(demolist.len() - offset);
            let mut raw_end = end;
            if end >= ext_len
                && demolist[offset + end - ext_len..offset + end]
                    .eq_ignore_ascii_case(demoExt.as_bytes())
            {
                raw_end = end - ext_len;
            }
            let name = latin1_to_string(&demolist[offset..offset + raw_end]).to_ascii_uppercase();
            ctx.world.demoList.push(name);

            offset += end + 1;
            i += 1;
        }
    }
}

/// Raven `UI_SetNextMap`.
///
/// PORT-NOTE: `Menu_SetFeederSelection`'s `dc` parameter follows the same
/// DEC-36 addendum 12 shape noted on `UI_GameType_HandleKey` above; this fn
/// makes no engine trap calls, so it takes `world`/`dc` rather than `ctx`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5133-5142`
pub fn UI_SetNextMap(
    world: &mut UiWorld,
    dc: &mut dyn DisplayContext,
    actual: c_int,
    index: c_int,
) -> bool {
    let mut i = actual + 1;
    while i < world.mapList.len() as c_int {
        if world.mapList[i as usize].active {
            Menu_SetFeederSelection(
                &mut world.menus,
                dc,
                None,
                FEEDER_MAPS,
                index + 1,
                Some("skirmish"),
            );
            return true;
        }
        i += 1;
    }
    false
}

/// Raven `UI_BuildServerDisplayList`.
///
/// PORT-NOTE: `force` stays `c_int`, not `bool` — Raven's body compares it
/// against the literal `2` (a tri-state "refresh but don't reset" caller
/// convention), which a `bool` cannot represent.
///
/// Source: `oracle/codemp/ui/ui_main.c:7763-7876`
pub fn UI_BuildServerDisplayList(ctx: &mut UiContext, dc: &mut dyn DisplayContext, force: c_int) {
    let mut force = force;

    if !(force != 0 || ctx.world.uiDC.realTime > ctx.world.serverStatus.nextDisplayRefresh) {
        return;
    }
    // if we shouldn't reset
    if force == 2 {
        force = 0;
    }

    // do motd updates here too
    // The motd is Latin-1-decoded free text, so `strlen`'s wire-byte count is
    // the decoded `char` count, not the UTF-8 byte length.
    let motd = trap::Cvar_VariableStringBuffer(ctx.engine, "cl_motdString", MAX_STRING_CHARS);
    let mut len = motd.chars().count();
    let motd = if len == 0 {
        "Welcome to Jedi Academy MP!".to_string()
    } else {
        motd
    };
    len = motd.chars().count();
    ctx.world.serverStatus.motd = motd;
    if len as c_int != ctx.world.serverStatus.motdLen {
        ctx.world.serverStatus.motdLen = len as c_int;
        ctx.world.serverStatus.motdWidth = -1;
    }

    if force != 0 {
        ctx.world.scratch.UI_BuildServerDisplayList_numinvisible = 0;
        // clear number of displayed servers
        ctx.world.serverStatus.displayServers.clear();
        ctx.world.serverStatus.numPlayersOnServers = 0;
        // set list box index to zero
        Menu_SetFeederSelection(&mut ctx.world.menus, dc, None, FEEDER_SERVERS, 0, None);
        // mark all servers as visible so we store ping updates for them
        trap::LAN_MarkServerVisible(ctx.engine, ctx.world.cvars.ui_netSource.integer, -1, true);
    }

    // get the server count (comes from the master)
    let count = trap::LAN_GetServerCount(ctx.engine, ctx.world.cvars.ui_netSource.integer);
    if count == -1 || (ctx.world.cvars.ui_netSource.integer == AS_LOCAL && count == 0) {
        // still waiting on a response from the master
        ctx.world.serverStatus.displayServers.clear();
        ctx.world.serverStatus.numPlayersOnServers = 0;
        ctx.world.serverStatus.nextDisplayRefresh = ctx.world.uiDC.realTime + 500;
        return;
    }

    let mut i = 0;
    while i < count {
        // if we already got info for this server
        if trap::LAN_ServerIsVisible(ctx.engine, ctx.world.cvars.ui_netSource.integer, i) == 0 {
            i += 1;
            continue;
        }
        // get the ping for this server
        let ping = trap::LAN_GetServerPing(ctx.engine, ctx.world.cvars.ui_netSource.integer, i);
        if ping > 0 || ctx.world.cvars.ui_netSource.integer == AS_FAVORITES {
            let info = trap::LAN_GetServerInfo(
                ctx.engine,
                ctx.world.cvars.ui_netSource.integer,
                i,
                MAX_STRING_CHARS,
            );

            let clients = atoi(&Info_ValueForKey(&info, "clients"));
            ctx.world.serverStatus.numPlayersOnServers += clients;

            if ctx.world.cvars.ui_browserShowEmpty.integer == 0 && clients == 0 {
                trap::LAN_MarkServerVisible(
                    ctx.engine,
                    ctx.world.cvars.ui_netSource.integer,
                    i,
                    false,
                );
                i += 1;
                continue;
            }

            if ctx.world.cvars.ui_browserShowFull.integer == 0 {
                let maxClients = atoi(&Info_ValueForKey(&info, "sv_maxclients"));
                if clients == maxClients {
                    trap::LAN_MarkServerVisible(
                        ctx.engine,
                        ctx.world.cvars.ui_netSource.integer,
                        i,
                        false,
                    );
                    i += 1;
                    continue;
                }
            }

            let joinIdx = ctx.world.cvars.ui_joinGameType.integer as usize;
            // §19: past `numJoinGameTypes` Raven read the fixed array's zeroed
            // spare slot, so the filter compared against gametype 0.
            let joinGtEnum = ctx
                .world
                .joinGameTypes
                .get(joinIdx)
                .map(|gt| gt.gtEnum)
                .unwrap_or_default();
            if joinGtEnum != -1 {
                let game = atoi(&Info_ValueForKey(&info, "gametype"));
                if game != joinGtEnum {
                    trap::LAN_MarkServerVisible(
                        ctx.engine,
                        ctx.world.cvars.ui_netSource.integer,
                        i,
                        false,
                    );
                    i += 1;
                    continue;
                }
            }

            if ctx.world.cvars.ui_serverFilterType.integer > 0 {
                let filterIdx = ctx.world.cvars.ui_serverFilterType.integer as usize;
                if filterIdx < SERVER_FILTERS.len()
                    && Q_stricmp(
                        &Info_ValueForKey(&info, "game"),
                        SERVER_FILTERS[filterIdx].basedir,
                    ) != 0
                {
                    trap::LAN_MarkServerVisible(
                        ctx.engine,
                        ctx.world.cvars.ui_netSource.integer,
                        i,
                        false,
                    );
                    i += 1;
                    continue;
                }
            }
            // make sure we never add a favorite server twice
            if ctx.world.cvars.ui_netSource.integer == AS_FAVORITES {
                UI_RemoveServerFromDisplayList(ctx.world, i);
            }
            // insert the server into the list
            UI_BinaryServerInsertion(ctx, i);
            // done with this server
            if ping > 0 {
                trap::LAN_MarkServerVisible(
                    ctx.engine,
                    ctx.world.cvars.ui_netSource.integer,
                    i,
                    false,
                );
                ctx.world.scratch.UI_BuildServerDisplayList_numinvisible += 1;
            }
        }
        i += 1;
    }

    ctx.world.serverStatus.refreshtime = ctx.world.uiDC.realTime;
}

/// Raven `UI_BuildServerStatus`.
///
/// PORT-NOTE: `UI_GetServerStatusInfo`'s ported shape takes both `ctx` and a
/// separate `info: Option<&mut ServerStatusInfo>`; Raven aliased its own
/// `uiInfo.serverStatusInfo` as that out-param (trivial in C), which the
/// borrow checker forbids passing through `ctx` twice, so the value is taken
/// out of `ctx.world` for the call and put back after.
///
/// Source: `oracle/codemp/ui/ui_main.c:8314-8340`
pub fn UI_BuildServerStatus(ctx: &mut UiContext, dc: &mut dyn DisplayContext, force: bool) {
    if ctx.world.nextFindPlayerRefresh != 0 {
        return;
    }
    if !force {
        if ctx.world.nextServerStatusRefresh == 0
            || ctx.world.nextServerStatusRefresh > ctx.world.uiDC.realTime
        {
            return;
        }
    } else {
        Menu_SetFeederSelection(&mut ctx.world.menus, dc, None, FEEDER_SERVERSTATUS, 0, None);
        ctx.world.serverStatusInfo.lines.clear();
        // reset all server status requests
        trap::LAN_ServerStatus(ctx.engine, None, 0);
    }
    if ctx.world.serverStatus.currentServer < 0
        || ctx.world.serverStatus.currentServer
            > ctx.world.serverStatus.displayServers.len() as c_int
        || ctx.world.serverStatus.displayServers.is_empty()
    {
        return;
    }

    let addr = ctx.world.serverStatusAddress.clone();
    let mut info_local = core::mem::take(&mut ctx.world.serverStatusInfo);
    let ok = UI_GetServerStatusInfo(ctx, &addr, Some(&mut info_local));
    ctx.world.serverStatusInfo = info_local;

    if ok {
        ctx.world.nextServerStatusRefresh = 0;
        UI_GetServerStatusInfo(ctx, &addr, None);
    } else {
        ctx.world.nextServerStatusRefresh = ctx.world.uiDC.realTime + 500;
    }
}

/// Raven `UI_SetSiegeTeams`.
///
/// PORT-NOTE (§B5 / escalation): `BG_SiegeFindThemeForTeam` returns a raw
/// `*mut siegeTeam_t` into `bg_state.bgSiegeTeams`; `UiMainState.siegeTeam1`/
/// `siegeTeam2` carry the table index instead (per that field's own doc
/// comment). The pointer is converted to an index by address arithmetic
/// against the table's base pointer — a safe `usize` computation, no pointer
/// dereference — matching the same convention the rest of the port uses for
/// "pointer into an owned Rust table" fields (§B5). See escalations.
///
/// PORT-NOTE: `BG_SiegeSetTeamTheme`'s ported shape takes `themeName: *mut
/// c_char`; `Q_strncpyz` (a safe `&mut [c_char]` writer) builds the buffer, so
/// no `unsafe` is needed to pass its raw pointer through (porting-rules: raw
/// pointers pass through calls without deref).
///
/// Source: `oracle/codemp/ui/ui_main.c:8358-8468`
pub fn UI_SetSiegeTeams(ctx: &mut UiContext, dc: &mut dyn DisplayContext) {
    let mut mapname: Option<String> = None;
    let mut info = String::new();

    // Get the map name from the server info.
    if let Some(cfg) = trap::GetConfigString(ctx.engine, CS_SERVERINFO, MAX_INFO_STRING) {
        info = cfg;
        mapname = Some(Info_ValueForKey(&info, "mapname"));
    }

    let mapname = match mapname {
        Some(m) if !m.is_empty() => m,
        _ => return,
    };

    let gametype = atoi(&Info_ValueForKey(&info, "g_gametype"));

    // If the server we are connected to is not siege we cannot choose a class anyway.
    if gametype != GT_SIEGE {
        return;
    }

    // Raven's `Com_sprintf` into `levelname[MAX_QPATH]` truncates at 63 chars.
    let levelname: String = format!("maps/{}.siege", mapname)
        .chars()
        .take(MAX_QPATH - 1)
        .collect();
    if levelname.is_empty() {
        return;
    }

    let mut f: fileHandle_t = 0;
    let len = trap::FS_FOpenFile(ctx.engine, &levelname, &mut f, FS_READ);

    if f == 0 || len >= MAX_SIEGE_INFO_SIZE {
        return;
    }

    trap::FS_Read(
        ctx.engine,
        &mut ctx.world.bg_state.siege_info[..len as usize],
        f,
    );
    ctx.world.bg_state.siege_info[len as usize] = 0; // ensure null terminated

    trap::FS_FCloseFile(ctx.engine, f);

    // Found the .siege file.
    let siege_info = buf_to_string(&ctx.world.bg_state.siege_info);

    let (team1, team2) = match BG_SiegeGetValueGroup(&siege_info, "Teams") {
        Some(teams) => {
            let buf = trap::Cvar_VariableStringBuffer(ctx.engine, "cg_siegeTeam1", 1024);
            let t1 = if !buf.is_empty() && Q_stricmp(&buf, "none") != 0 {
                buf
            } else {
                BG_SiegeGetPairedValue(&teams, "team1").unwrap_or_default()
            };

            let buf = trap::Cvar_VariableStringBuffer(ctx.engine, "cg_siegeTeam2", 1024);
            let t2 = if !buf.is_empty() && Q_stricmp(&buf, "none") != 0 {
                buf
            } else {
                BG_SiegeGetPairedValue(&teams, "team2").unwrap_or_default()
            };

            (t1, t2)
        }
        None => return,
    };

    // Set the team themes so we know what classes to make available for selection.
    if let Some(teamInfo) = BG_SiegeGetValueGroup(&siege_info, &team1) {
        if let Some(btime) = BG_SiegeGetPairedValue(&teamInfo, "UseTeam") {
            let mut buf: [c_char; 1024] = [0; 1024];
            Q_strncpyz(&mut buf, &btime, 1024);
            BG_SiegeSetTeamTheme(SIEGETEAM_TEAM1, buf.as_mut_ptr(), &mut ctx.world.bg_state);
        }
    }
    if let Some(teamInfo) = BG_SiegeGetValueGroup(&siege_info, &team2) {
        if let Some(btime) = BG_SiegeGetPairedValue(&teamInfo, "UseTeam") {
            let mut buf: [c_char; 1024] = [0; 1024];
            Q_strncpyz(&mut buf, &btime, 1024);
            BG_SiegeSetTeamTheme(SIEGETEAM_TEAM2, buf.as_mut_ptr(), &mut ctx.world.bg_state);
        }
    }

    let siegeTeam1_ptr = BG_SiegeFindThemeForTeam(SIEGETEAM_TEAM1, &ctx.world.bg_state);
    let siegeTeam2_ptr = BG_SiegeFindThemeForTeam(SIEGETEAM_TEAM2, &ctx.world.bg_state);

    let base = ctx.world.bg_state.bgSiegeTeams.as_ptr() as usize;
    let elem_size = core::mem::size_of::<siegeTeam_t>();
    let idx1 = (!siegeTeam1_ptr.is_null()).then(|| (siegeTeam1_ptr as usize - base) / elem_size);
    let idx2 = (!siegeTeam2_ptr.is_null()).then(|| (siegeTeam2_ptr as usize - base) / elem_size);

    ctx.world.main.siegeTeam1 = idx1;
    ctx.world.main.siegeTeam2 = idx2;

    // set the default description for the default selection
    let classes_empty = match idx1 {
        Some(i) => ctx.world.bg_state.bgSiegeTeams[i].classes[0].is_null(),
        None => true,
    };
    if idx1.is_none() || classes_empty {
        Com_Error(ctx, "Error loading teams in UI");
    }

    Menu_SetFeederSelection(&mut ctx.world.menus, dc, None, FEEDER_SIEGE_TEAM1, 0, None);
    Menu_SetFeederSelection(&mut ctx.world.menus, dc, None, FEEDER_SIEGE_TEAM2, -1, None);
}

/// Raven `Text_PaintCenter`.
///
/// Source: `oracle/codemp/ui/ui_main.c:11069-11072`
pub fn Text_PaintCenter(
    ctx: &UiContext,
    x: f32,
    y: f32,
    scale: f32,
    color: vec4_t,
    text: &str,
    _adjust: f32,
    iMenuFont: c_int,
) {
    let len = Text_Width(ctx, text, scale, iMenuFont);
    Text_Paint(
        ctx,
        x - (len / 2) as f32,
        y,
        scale,
        color,
        text,
        0.0,
        0,
        ITEM_TEXTSTYLE_SHADOWEDMORE,
        iMenuFont,
    );
}

/// Raven `UI_DrawForceSide`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2152-2230`
#[allow(clippy::too_many_arguments)]
pub fn UI_DrawForceSide(
    ctx: &mut UiContext,
    dc: &mut dyn DisplayContext,
    rect: &RectDef,
    scale: f32,
    color: &mut vec4_t,
    textStyle: c_int,
    val: c_int,
    _min: c_int,
    _max: c_int,
    iMenuFont: c_int,
) {
    let info = trap::GetConfigString(ctx.engine, CS_SERVERINFO, MAX_INFO_VALUE).unwrap_or_default();

    if atoi(&Info_ValueForKey(&info, "g_forceBasedTeams")) != 0 {
        let myteam = trap::Cvar_VariableValue(ctx.engine, "ui_myteam") as c_int;
        match myteam {
            TEAM_RED => {
                ctx.world.force.uiForceSide = FORCE_DARKSIDE;
                color[0] = 0.2;
                color[1] = 0.2;
                color[2] = 0.2;
            }
            TEAM_BLUE => {
                ctx.world.force.uiForceSide = FORCE_LIGHTSIDE;
                color[0] = 0.2;
                color[1] = 0.2;
                color[2] = 0.2;
            }
            _ => {}
        }
    }

    let s;
    if val == FORCE_LIGHTSIDE {
        s = trap::SP_GetStringTextString(ctx.engine, "MENUS_FORCEDESC_LIGHT", 256)
            .unwrap_or_default();
        if let Some(menu) = Menus_FindByName(&ctx.world.menus, "forcealloc") {
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "lightpowers", true);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "darkpowers", false);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "darkpowers_team", false);
            // (ui_gameType.integer >= GT_TEAM))
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "lightpowers_team", true);
        }
        if let Some(menu) = Menus_FindByName(&ctx.world.menus, "ingame_playerforce") {
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "lightpowers", true);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "darkpowers", false);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "darkpowers_team", false);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "lightpowers_team", true);
        }
    } else {
        s = trap::SP_GetStringTextString(ctx.engine, "MENUS_FORCEDESC_DARK", 256)
            .unwrap_or_default();
        if let Some(menu) = Menus_FindByName(&ctx.world.menus, "forcealloc") {
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "lightpowers", false);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "lightpowers_team", false);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "darkpowers", true);
            // (ui_gameType.integer >= GT_TEAM))
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "darkpowers_team", true);
        }
        if let Some(menu) = Menus_FindByName(&ctx.world.menus, "ingame_playerforce") {
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "lightpowers", false);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "lightpowers_team", false);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "darkpowers", true);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "darkpowers_team", true);
        }
    }

    Text_Paint(
        ctx, rect.x, rect.y, scale, *color, &s, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UpdateBotButtons`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2549-2571`
pub fn UpdateBotButtons(world: &mut UiWorld, dc: &mut dyn DisplayContext) {
    let menu = match Menu_GetFocused(&world.menus) {
        Some(m) => m,
        None => return,
    };

    if world.gameTypes[world.cvars.ui_netGameType.integer as usize].gtEnum == GT_SIEGE {
        Menu_ShowItemByName(&mut world.menus, dc, menu, "humanbotfield", false);
        Menu_ShowItemByName(&mut world.menus, dc, menu, "humanbotnonfield", true);
    } else {
        Menu_ShowItemByName(&mut world.menus, dc, menu, "humanbotfield", true);
        Menu_ShowItemByName(&mut world.menus, dc, menu, "humanbotnonfield", false);
    }
}

/// Raven `UpdateForceStatus`.
///
/// Currently we don't make a distinction between those that wish to play Jedi
/// of lower than maximum skill.
///
/// Source: `oracle/codemp/ui/ui_main.c:2573-2723`
pub fn UpdateForceStatus(ctx: &mut UiContext, dc: &mut dyn DisplayContext) {
    if let Some(menu) = Menus_FindByName(&ctx.world.menus, "ingame_player") {
        let info =
            trap::GetConfigString(ctx.engine, CS_SERVERINFO, MAX_INFO_STRING).unwrap_or_default();

        // already have serverinfo at this point for stuff below. Don't
        // bother trying to use ui_forcePowerDisable.
        let disabledForce = atoi(&Info_ValueForKey(&info, "g_forcePowerDisable"));
        let allForceDisabled = UI_AllForceDisabled(disabledForce);
        let trueJedi = UI_TrueJediEnabled(ctx);

        if !trueJedi || allForceDisabled {
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "jedinonjedi", false);
        } else {
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "jedinonjedi", true);
        }
        if allForceDisabled || (trueJedi && ctx.world.force.uiJediNonJedi == FORCE_NONJEDI) {
            // No force stuff.
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "noforce", true);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "yesforce", false);
            // We don't want the saber explanation to say "configure saber
            // attack 1" since we can't.
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "sabernoneconfigme", false);
        } else {
            UI_SetForceDisabled(ctx.world, disabledForce);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "noforce", false);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "yesforce", true);
        }

        // Moved this to happen after it's done with force power disabling
        // stuff.
        if ctx.world.force.uiForcePowersRank[FP_SABER_OFFENSE as usize] > 0
            || ctx.world.cvars.ui_freeSaber.integer != 0
        {
            // Show lightsaber stuff.
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "nosaber", false);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "yessaber", true);
        } else {
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "nosaber", true);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "yessaber", false);
        }

        // The leftmost button should be "apply" unless you are in spectator,
        // where you can join any team.
        let myteam = trap::Cvar_VariableValue(ctx.engine, "ui_myteam") as c_int;
        if myteam != TEAM_SPECTATOR {
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "playerapply", true);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "playerforcejoin", false);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "playerforcered", true);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "playerforceblue", true);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "playerforcespectate", true);
        } else {
            // Set or reset buttons based on choices.
            if atoi(&Info_ValueForKey(&info, "g_gametype")) >= GT_TEAM {
                // This is a team-based game.
                Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "playerforcespectate", true);

                // This is disabled, always show both sides from spectator.
                Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "playerforcered", true);
                Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "playerforceblue", true);
            } else {
                Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "playerforcered", false);
                Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "playerforceblue", false);
            }

            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "playerapply", false);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "playerforcejoin", true);
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, "playerforcespectate", true);
        }
    }

    if !UI_TrueJediEnabled(ctx) {
        // Take the current team and force a skin color based on it.
        let myteam = trap::Cvar_VariableValue(ctx.engine, "ui_myteam") as c_int;
        match myteam {
            TEAM_RED => {
                ctx.world.main.uiSkinColor = TEAM_RED;
                ctx.world.effectsColor = SABER_RED;
            }
            TEAM_BLUE => {
                ctx.world.main.uiSkinColor = TEAM_BLUE;
                ctx.world.effectsColor = SABER_BLUE;
            }
            _ => {
                let info = trap::GetConfigString(ctx.engine, CS_SERVERINFO, MAX_INFO_STRING)
                    .unwrap_or_default();

                if atoi(&Info_ValueForKey(&info, "g_gametype")) >= GT_TEAM {
                    ctx.world.main.uiSkinColor = mp_bg::public::team::TEAM_FREE;
                } else {
                    // A bit of a hack so non-team games will remember which
                    // skin set you chose in the player menu.
                    ctx.world.main.uiSkinColor = ctx.world.main.uiHoldSkinColor;
                }
            }
        }
    }
}

/// Raven `UI_DrawNetSource`.
///
/// Source: `oracle/codemp/ui/ui_main.c:2727-2737`
pub fn UI_DrawNetSource(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    if ctx.world.cvars.ui_netSource.integer < 0
        || ctx.world.cvars.ui_netSource.integer > ctx.world.gameTypes.len() as c_int
    {
        ctx.world.cvars.ui_netSource.integer = 0;
    }

    ctx.world.main.holdSPString =
        trap::SP_GetStringTextString(ctx.engine, "MENUS_SOURCE", MAX_STRING_CHARS)
            .unwrap_or_default();
    let netSourceIdx = ctx.world.cvars.ui_netSource.integer;
    let netSource = GetNetSourceString(ctx, netSourceIdx);
    let s = format!("{} {}", ctx.world.main.holdSPString, netSource);
    Text_Paint(
        ctx, rect.x, rect.y, scale, color, &s, 0.0, 0, textStyle, iMenuFont,
    );
}

/// Raven `UI_OwnerDrawWidth`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3008-3236`
#[allow(clippy::too_many_lines)]
pub fn UI_OwnerDrawWidth(ctx: &mut UiContext, ownerDraw: c_int, scale: f32) -> c_int {
    let mut s: Option<String> = None;

    match ownerDraw {
        UI_HANDICAP => {
            let h =
                Com_Clamp(5.0, 100.0, trap::Cvar_VariableValue(ctx.engine, "handicap")) as c_int;
            let i = 20 - h / 5;
            s = HANDICAP_VALUES[i as usize].map(|v| v.to_string());
        }
        UI_SKIN_COLOR => {
            let skinColor = ctx.world.main.uiSkinColor;
            s = Some(match skinColor {
                TEAM_RED => UI_GetStringEdString(ctx, "MENUS", "TEAM_RED"),
                TEAM_BLUE => UI_GetStringEdString(ctx, "MENUS", "TEAM_BLUE"),
                _ => UI_GetStringEdString(ctx, "MENUS", "DEFAULT"),
            });
        }
        UI_FORCE_SIDE => {
            let mut i = ctx.world.force.uiForceSide;
            if i < 1 || i > 2 {
                i = 1;
            }
            s = Some(if i == FORCE_LIGHTSIDE {
                UI_GetStringEdString(ctx, "MENUS", "FORCEDESC_LIGHT")
            } else {
                UI_GetStringEdString(ctx, "MENUS", "FORCEDESC_DARK")
            });
        }
        UI_JEDI_NONJEDI => {
            let mut i = ctx.world.force.uiJediNonJedi;
            if i < 0 || i > 1 {
                i = 0;
            }
            s = Some(if i == FORCE_NONJEDI {
                UI_GetStringEdString(ctx, "MENUS", "NO")
            } else {
                UI_GetStringEdString(ctx, "MENUS", "YES")
            });
        }
        UI_FORCE_RANK => {
            let mut i = ctx.world.force.uiForceRank;
            if i < 1 || i > MAX_FORCE_RANK {
                i = 1;
            }
            s = Some(UI_GetStringEdString(
                ctx,
                "MP_INGAME",
                FORCE_MASTERY_LEVELS[i as usize],
            ));
        }
        UI_FORCE_RANK_HEAL..=UI_FORCE_RANK_SABERTHROW => {
            // this will give us the index as long as UI_FORCE_RANK is always
            // one below the first force rank index
            let findex = (ownerDraw - UI_FORCE_RANK) - 1;
            s = Some(format!(
                "{}",
                ctx.world.force.uiForcePowersRank[findex as usize]
            ));
        }
        UI_CLANNAME => {
            s = Some(UI_Cvar_VariableString(ctx, "ui_teamName"));
        }
        UI_GAMETYPE => {
            let idx = ctx.world.cvars.ui_gameType.integer as usize;
            // §19: past `numGameTypes` Raven read the fixed array's zeroed spare slot.
            s = Some(
                ctx.world
                    .gameTypes
                    .get(idx)
                    .map(|gt| gt.gameType.clone())
                    .unwrap_or_default(),
            );
        }
        UI_SKILL => {
            let mut i = trap::Cvar_VariableValue(ctx.engine, "g_spSkill") as c_int;
            if i < 1 || i > NUM_SKILL_LEVELS {
                i = 1;
            }
            s = Some(UI_GetStringEdString(
                ctx,
                "MP_INGAME",
                SKILL_LEVELS[(i - 1) as usize],
            ));
        }
        UI_BLUETEAMNAME => {
            let blueTeam = UI_Cvar_VariableString(ctx, "ui_blueTeam");
            let i = UI_TeamIndexFromName(ctx.world, &blueTeam);
            if i >= 0 && (i as usize) < ctx.world.teamList.len() {
                let label = UI_GetStringEdString(ctx, "MENUS", "TEAM_BLUE");
                s = Some(format!(
                    "{}: {}",
                    label, ctx.world.teamList[i as usize].teamName
                ));
            }
        }
        UI_REDTEAMNAME => {
            let redTeam = UI_Cvar_VariableString(ctx, "ui_redTeam");
            let i = UI_TeamIndexFromName(ctx.world, &redTeam);
            if i >= 0 && (i as usize) < ctx.world.teamList.len() {
                let label = UI_GetStringEdString(ctx, "MENUS", "TEAM_RED");
                s = Some(format!(
                    "{}: {}",
                    label, ctx.world.teamList[i as usize].teamName
                ));
            }
        }
        UI_BLUETEAM1 | UI_BLUETEAM2 | UI_BLUETEAM3 | UI_BLUETEAM4 | UI_BLUETEAM5 | UI_BLUETEAM6
        | UI_BLUETEAM7 | UI_BLUETEAM8 => {
            let iUse = if ownerDraw <= UI_BLUETEAM5 {
                ownerDraw - UI_BLUETEAM1 + 1
            } else {
                // unpleasant hack because I don't want to move up all the
                // UI_BLAHTEAM# defines
                ownerDraw - 274
            };
            let mut value =
                trap::Cvar_VariableValue(ctx.engine, &format!("ui_blueteam{}", iUse)) as c_int;
            let text = if value <= 1 {
                "Human".to_string()
            } else {
                value -= 2;
                if value as usize >= ctx.world.aliasList.len() {
                    value = 1;
                }
                ctx.world.aliasList[value as usize].name.clone()
            };
            s = Some(format!("{}. {}", iUse, text));
        }
        UI_REDTEAM1 | UI_REDTEAM2 | UI_REDTEAM3 | UI_REDTEAM4 | UI_REDTEAM5 | UI_REDTEAM6
        | UI_REDTEAM7 | UI_REDTEAM8 => {
            let iUse = if ownerDraw <= UI_REDTEAM5 {
                ownerDraw - UI_REDTEAM1 + 1
            } else {
                ownerDraw - 277
            };
            let mut value =
                trap::Cvar_VariableValue(ctx.engine, &format!("ui_redteam{}", iUse)) as c_int;
            let text = if value <= 1 {
                "Human".to_string()
            } else {
                value -= 2;
                if value as usize >= ctx.world.aliasList.len() {
                    value = 1;
                }
                ctx.world.aliasList[value as usize].name.clone()
            };
            s = Some(format!("{}. {}", iUse, text));
        }
        UI_NETSOURCE => {
            if ctx.world.cvars.ui_netSource.integer < 0
                || ctx.world.cvars.ui_netSource.integer > ctx.world.joinGameTypes.len() as c_int
            {
                ctx.world.cvars.ui_netSource.integer = 0;
            }
            ctx.world.main.holdSPString =
                trap::SP_GetStringTextString(ctx.engine, "MENUS_SOURCE", MAX_STRING_CHARS)
                    .unwrap_or_default();
            let idx = ctx.world.cvars.ui_netSource.integer;
            let netSource = GetNetSourceString(ctx, idx);
            s = Some(format!("{} {}", ctx.world.main.holdSPString, netSource));
        }
        UI_NETFILTER => {
            if ctx.world.cvars.ui_serverFilterType.integer < 0
                || ctx.world.cvars.ui_serverFilterType.integer > SERVER_FILTERS.len() as c_int
            {
                ctx.world.cvars.ui_serverFilterType.integer = 0;
            }
            ctx.world.main.holdSPString =
                trap::SP_GetStringTextString(ctx.engine, "MENUS_GAME", MAX_STRING_CHARS)
                    .unwrap_or_default();
            let filterIdx = ctx.world.cvars.ui_serverFilterType.integer as usize;
            // §19: Raven's `>` guard lets `== numServerFilters` read one past the table.
            let description = SERVER_FILTERS
                .get(filterIdx)
                .map(|f| f.description)
                .unwrap_or("");
            ctx.world.main.holdSPString2 =
                trap::SP_GetStringTextString(ctx.engine, description, MAX_STRING_CHARS)
                    .unwrap_or_default();

            s = Some(format!(
                "{} {}",
                ctx.world.main.holdSPString, ctx.world.main.holdSPString2
            ));
        }
        UI_TIER | UI_TIER_MAPNAME | UI_TIER_GAMETYPE | UI_ALLMAPS_SELECTION | UI_OPPONENT_NAME => {}
        UI_KEYBINDSTATUS => {
            if Display_KeyBindPending(&ctx.world.menus) {
                s = Some(UI_GetStringEdString(
                    ctx,
                    "MP_INGAME",
                    "WAITING_FOR_NEW_KEY",
                ));
            }
            // else: Raven leaves the "Press ENTER or CLICK..." line commented
            // out — no fallback text.
        }
        UI_SERVERREFRESHDATE => {
            let netSourceIdx = ctx.world.cvars.ui_netSource.integer;
            s = Some(UI_Cvar_VariableString(
                ctx,
                &format!("ui_lastServerRefresh_{}", netSourceIdx),
            ));
        }
        _ => {}
    }

    if let Some(text) = s {
        Text_Width(ctx, &text, scale, 0)
    } else {
        0
    }
}

/// Raven `UI_DrawServerMOTD`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3371-3428`
pub fn UI_DrawServerMOTD(
    ctx: &mut UiContext,
    rect: &RectDef,
    scale: f32,
    color: vec4_t,
    iMenuFont: c_int,
) {
    if ctx.world.serverStatus.motdLen != 0 {
        if ctx.world.serverStatus.motdWidth == -1 {
            ctx.world.serverStatus.motdWidth = 0;
            ctx.world.serverStatus.motdPaintX = rect.x as c_int + 1;
            ctx.world.serverStatus.motdPaintX2 = -1;
        }

        if ctx.world.serverStatus.motdOffset > ctx.world.serverStatus.motdLen {
            ctx.world.serverStatus.motdOffset = 0;
            ctx.world.serverStatus.motdPaintX = rect.x as c_int + 1;
            ctx.world.serverStatus.motdPaintX2 = -1;
        }

        if ctx.world.uiDC.realTime > ctx.world.serverStatus.motdTime {
            ctx.world.serverStatus.motdTime = ctx.world.uiDC.realTime + 10;
            if ctx.world.serverStatus.motdPaintX <= rect.x as c_int + 2 {
                if ctx.world.serverStatus.motdOffset < ctx.world.serverStatus.motdLen {
                    let offset = ctx.world.serverStatus.motdOffset as usize;
                    let remaining = ctx.world.serverStatus.motd[offset..].to_string();
                    let width = Text_Width(ctx, &remaining, scale, 1);
                    ctx.world.serverStatus.motdPaintX += width - 1;
                    ctx.world.serverStatus.motdOffset += 1;
                } else {
                    ctx.world.serverStatus.motdOffset = 0;
                    if ctx.world.serverStatus.motdPaintX2 >= 0 {
                        ctx.world.serverStatus.motdPaintX = ctx.world.serverStatus.motdPaintX2;
                    } else {
                        ctx.world.serverStatus.motdPaintX = rect.x as c_int + rect.w as c_int - 2;
                    }
                    ctx.world.serverStatus.motdPaintX2 = -1;
                }
            } else {
                ctx.world.serverStatus.motdPaintX -= 2;
                if ctx.world.serverStatus.motdPaintX2 >= 0 {
                    ctx.world.serverStatus.motdPaintX2 -= 2;
                }
            }
        }

        let paintX = ctx.world.serverStatus.motdPaintX as f32;
        let y = rect.y + rect.h - 3.0;
        let mut maxX = rect.x + rect.w - 2.0;
        let offset = ctx.world.serverStatus.motdOffset as usize;
        let text = ctx.world.serverStatus.motd[offset..].to_string();
        Text_Paint_Limit(
            ctx, &mut maxX, paintX, y, scale, color, &text, 0.0, 0, iMenuFont,
        );
        if ctx.world.serverStatus.motdPaintX2 >= 0 {
            let paintX2 = ctx.world.serverStatus.motdPaintX2 as f32;
            let motd = ctx.world.serverStatus.motd.clone();
            let motdOffset = ctx.world.serverStatus.motdOffset;
            let mut maxX2 = rect.x + rect.w - 2.0;
            Text_Paint_Limit(
                ctx, &mut maxX2, paintX2, y, scale, color, &motd, 0.0, motdOffset, iMenuFont,
            );
        }
        if ctx.world.serverStatus.motdOffset != 0 && maxX > 0.0 {
            // if we have an offset (we are skipping the first part of the
            // string) and we fit the string
            if ctx.world.serverStatus.motdPaintX2 == -1 {
                ctx.world.serverStatus.motdPaintX2 = rect.x as c_int + rect.w as c_int - 2;
            }
        } else {
            ctx.world.serverStatus.motdPaintX2 = -1;
        }
    }
}

/// Raven `UI_JoinGameType_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4402-4422`
pub fn UI_JoinGameType_HandleKey(
    ctx: &mut UiContext,
    dc: &mut dyn DisplayContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        if key == fakeAscii_t::A_MOUSE2 as c_int {
            ctx.world.cvars.ui_joinGameType.integer -= 1;
        } else {
            ctx.world.cvars.ui_joinGameType.integer += 1;
        }

        if ctx.world.cvars.ui_joinGameType.integer < 0 {
            ctx.world.cvars.ui_joinGameType.integer = ctx.world.joinGameTypes.len() as c_int - 1;
        } else if ctx.world.cvars.ui_joinGameType.integer >= ctx.world.joinGameTypes.len() as c_int
        {
            ctx.world.cvars.ui_joinGameType.integer = 0;
        }

        let idx = ctx.world.cvars.ui_joinGameType.integer;
        trap::Cvar_Set(ctx.engine, "ui_joinGameType", &format!("{}", idx));
        UI_BuildServerDisplayList(ctx, dc, 1);
        return true;
    }
    false
}

/// Raven `UI_NetFilter_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4551-4569`
pub fn UI_NetFilter_HandleKey(
    ctx: &mut UiContext,
    dc: &mut dyn DisplayContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        if key == fakeAscii_t::A_MOUSE2 as c_int {
            ctx.world.cvars.ui_serverFilterType.integer -= 1;
        } else {
            ctx.world.cvars.ui_serverFilterType.integer += 1;
        }

        if ctx.world.cvars.ui_serverFilterType.integer >= SERVER_FILTERS.len() as c_int {
            ctx.world.cvars.ui_serverFilterType.integer = 0;
        } else if ctx.world.cvars.ui_serverFilterType.integer < 0 {
            ctx.world.cvars.ui_serverFilterType.integer = SERVER_FILTERS.len() as c_int - 1;
        }
        UI_BuildServerDisplayList(ctx, dc, 1);
        return true;
    }
    false
}

/// Raven `UI_StartSkirmish`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5145-5253`
#[allow(clippy::too_many_lines)]
pub fn UI_StartSkirmish(ctx: &mut UiContext, dc: &mut dyn DisplayContext, next: bool) {
    let temp = trap::Cvar_VariableValue(ctx.engine, "g_gametype") as c_int;
    trap::Cvar_Set(ctx.engine, "ui_gameType", &format!("{}", temp));

    if next {
        let index = trap::Cvar_VariableValue(ctx.engine, "ui_mapIndex") as c_int;
        let _ = UI_MapCountByGameType(ctx.world, true);
        let mut actual = 0;
        let _ = UI_SelectedMap(ctx.world, index, &mut actual);
        if UI_SetNextMap(ctx.world, dc, actual, index) {
            // handled
        } else {
            let mut special = 0.0_f32;
            UI_GameType_HandleKey(
                ctx,
                dc,
                0,
                &mut special,
                fakeAscii_t::A_MOUSE1 as c_int,
                false,
            );
            let _ = UI_MapCountByGameType(ctx.world, true);
            Menu_SetFeederSelection(
                &mut ctx.world.menus,
                dc,
                None,
                FEEDER_MAPS,
                0,
                Some("skirmish"),
            );
        }
    }

    let g = ctx.world.gameTypes[ctx.world.cvars.ui_gameType.integer as usize].gtEnum;
    trap::Cvar_SetValue(ctx.engine, "g_gametype", g as f32);
    let mapLoadName = ctx.world.mapList[ctx.world.cvars.ui_currentMap.integer as usize]
        .mapLoadName
        .clone();
    trap::Cmd_ExecuteText(
        ctx.engine,
        cbufExec_t::EXEC_APPEND as c_int,
        &format!("wait ; wait ; map {}\n", mapLoadName),
    );
    let skill = trap::Cvar_VariableValue(ctx.engine, "g_spSkill");
    let mapName = ctx.world.mapList[ctx.world.cvars.ui_currentMap.integer as usize]
        .mapName
        .clone();
    trap::Cvar_Set(ctx.engine, "ui_scoreMap", &mapName);

    let opponentName = UI_Cvar_VariableString(ctx, "ui_opponentName");
    let mut k = UI_TeamIndexFromName(ctx.world, &opponentName);

    trap::Cvar_Set(ctx.engine, "ui_singlePlayerActive", "1");

    // set up sp overrides, will be replaced on postgame
    let temp = trap::Cvar_VariableValue(ctx.engine, "capturelimit") as c_int;
    trap::Cvar_Set(ctx.engine, "ui_saveCaptureLimit", &format!("{}", temp));
    let temp = trap::Cvar_VariableValue(ctx.engine, "fraglimit") as c_int;
    trap::Cvar_Set(ctx.engine, "ui_saveFragLimit", &format!("{}", temp));
    let temp = trap::Cvar_VariableValue(ctx.engine, "duel_fraglimit") as c_int;
    trap::Cvar_Set(ctx.engine, "ui_saveDuelLimit", &format!("{}", temp));

    UI_SetCapFragLimits(ctx, false);

    let temp = trap::Cvar_VariableValue(ctx.engine, "cg_drawTimer") as c_int;
    trap::Cvar_Set(ctx.engine, "ui_drawTimer", &format!("{}", temp));
    let temp = trap::Cvar_VariableValue(ctx.engine, "g_doWarmup") as c_int;
    trap::Cvar_Set(ctx.engine, "ui_doWarmup", &format!("{}", temp));
    let temp = trap::Cvar_VariableValue(ctx.engine, "g_friendlyFire") as c_int;
    trap::Cvar_Set(ctx.engine, "ui_friendlyFire", &format!("{}", temp));
    let temp = trap::Cvar_VariableValue(ctx.engine, "sv_maxClients") as c_int;
    trap::Cvar_Set(ctx.engine, "ui_maxClients", &format!("{}", temp));
    let temp = trap::Cvar_VariableValue(ctx.engine, "g_warmup") as c_int;
    trap::Cvar_Set(ctx.engine, "ui_Warmup", &format!("{}", temp));
    let temp = trap::Cvar_VariableValue(ctx.engine, "sv_pure") as c_int;
    trap::Cvar_Set(ctx.engine, "ui_pure", &format!("{}", temp));

    trap::Cvar_Set(ctx.engine, "cg_cameraOrbit", "0");
    trap::Cvar_Set(ctx.engine, "cg_thirdPerson", "0");
    trap::Cvar_Set(ctx.engine, "cg_drawTimer", "1");
    trap::Cvar_Set(ctx.engine, "g_doWarmup", "1");
    trap::Cvar_Set(ctx.engine, "g_warmup", "15");
    trap::Cvar_Set(ctx.engine, "sv_pure", "0");
    trap::Cvar_Set(ctx.engine, "g_friendlyFire", "0");
    // g_redTeam/g_blueTeam sets stay commented out, matching Raven.

    if trap::Cvar_VariableValue(ctx.engine, "ui_recordSPDemo") != 0.0 {
        let mapLoadName = ctx.world.mapList[ctx.world.cvars.ui_currentMap.integer as usize]
            .mapLoadName
            .clone();
        let buff = format!("{}_{}", mapLoadName, g);
        trap::Cvar_Set(ctx.engine, "ui_recordSPDemoName", &buff);
    }

    let mut delay: c_int = 500;

    if g == GT_DUEL || g == GT_POWERDUEL {
        let cur = ctx.world.cvars.ui_currentMap.integer as usize;
        let temp = ctx.world.mapList[cur].teamMembers * 2;
        trap::Cvar_Set(ctx.engine, "sv_maxClients", &format!("{}", temp));
        let opponentName = ctx.world.mapList[cur].opponentName.clone();
        // Raven's literal is two adjacent string constants that concatenate
        // to `"wait ; addbot %s %f , %i \n"` (the stray `""` is Raven's own
        // formatting, preserved here).
        let buff = format!("wait ; addbot {} {:.6} , {} \n", opponentName, skill, delay);
        trap::Cmd_ExecuteText(ctx.engine, cbufExec_t::EXEC_APPEND as c_int, &buff);
    } else if g == GT_HOLOCRON || g == GT_JEDIMASTER {
        let cur = ctx.world.cvars.ui_currentMap.integer as usize;
        let teamMembers = ctx.world.mapList[cur].teamMembers;
        let temp = teamMembers * 2;
        trap::Cvar_Set(ctx.engine, "sv_maxClients", &format!("{}", temp));
        for i in 0..teamMembers {
            let name = ctx.world.teamList[k as usize].teamMembers[i as usize].clone();
            let ai = UI_AIFromName(ctx.world, &name);
            let color = if g == GT_HOLOCRON { "" } else { "Blue" };
            let buff = format!(
                "addbot \"{}\" {:.6} {} {} {}\n",
                ai, skill, color, delay, name
            );
            trap::Cmd_ExecuteText(ctx.engine, cbufExec_t::EXEC_APPEND as c_int, &buff);
            delay += 500;
        }
        let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
        k = UI_TeamIndexFromName(ctx.world, &teamName);
        for i in 0..(teamMembers - 1) {
            let name = ctx.world.teamList[k as usize].teamMembers[i as usize].clone();
            let ai = UI_AIFromName(ctx.world, &name);
            let color = if g == GT_HOLOCRON { "" } else { "Red" };
            let buff = format!(
                "addbot \"{}\" {:.6} {} {} {}\n",
                ai, skill, color, delay, name
            );
            trap::Cmd_ExecuteText(ctx.engine, cbufExec_t::EXEC_APPEND as c_int, &buff);
            delay += 500;
        }
    } else {
        let cur = ctx.world.cvars.ui_currentMap.integer as usize;
        let teamMembers = ctx.world.mapList[cur].teamMembers;
        let temp = teamMembers * 2;
        trap::Cvar_Set(ctx.engine, "sv_maxClients", &format!("{}", temp));
        for i in 0..teamMembers {
            let name = ctx.world.teamList[k as usize].teamMembers[i as usize].clone();
            let ai = UI_AIFromName(ctx.world, &name);
            let color = if g == GT_FFA { "" } else { "Blue" };
            let buff = format!(
                "addbot \"{}\" {:.6} {} {} {}\n",
                ai, skill, color, delay, name
            );
            trap::Cmd_ExecuteText(ctx.engine, cbufExec_t::EXEC_APPEND as c_int, &buff);
            delay += 500;
        }
        let teamName = UI_Cvar_VariableString(ctx, "ui_teamName");
        k = UI_TeamIndexFromName(ctx.world, &teamName);
        for i in 0..(teamMembers - 1) {
            let name = ctx.world.teamList[k as usize].teamMembers[i as usize].clone();
            let ai = UI_AIFromName(ctx.world, &name);
            let color = if g == GT_FFA { "" } else { "Red" };
            let buff = format!(
                "addbot \"{}\" {:.6} {} {} {}\n",
                ai, skill, color, delay, name
            );
            trap::Cmd_ExecuteText(ctx.engine, cbufExec_t::EXEC_APPEND as c_int, &buff);
            delay += 500;
        }
    }
    if g >= GT_TEAM {
        trap::Cmd_ExecuteText(
            ctx.engine,
            cbufExec_t::EXEC_APPEND as c_int,
            "wait 5; team Red\n",
        );
    }
}

/// Raven `UI_SetBotButton`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5546-5572`
pub fn UI_SetBotButton(ctx: &mut UiContext, dc: &mut dyn DisplayContext) {
    let gameType = trap::Cvar_VariableValue(ctx.engine, "g_gametype") as c_int;
    let server = trap::Cvar_VariableValue(ctx.engine, "sv_running") as c_int;
    let name = "addBot";

    // If in siege or a client, don't show add bot button.
    if gameType == GT_SIEGE || server == 0 {
        // If it's not siege, don't worry about it.
        let menu = match Menu_GetFocused(&ctx.world.menus) {
            Some(m) => m,
            None => return,
        };

        if Menu_FindItemByName(&ctx.world.menus, Some(menu), name).is_some() {
            Menu_ShowItemByName(&mut ctx.world.menus, dc, menu, name, false);
        }
    }
}

/// Raven `UI_SetSiegeObjectiveGraphicPos`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5680-5718`
pub fn UI_SetSiegeObjectiveGraphicPos(
    ctx: &mut UiContext,
    menu: MenuId,
    itemName: &str,
    cvarName: &str,
) {
    let item = match Menu_FindItemByName(&ctx.world.menus, Some(menu), itemName) {
        Some(i) => i,
        None => return,
    };

    // get cvar data
    let cvarBuf = trap::Cvar_VariableStringBuffer(ctx.engine, cvarName, 1024);
    let mut p: &str = &cvarBuf;
    let mut holdVal = String::new();

    if String_Parse(&mut p, &mut holdVal) {
        let x = atof(&holdVal) as f32;
        if String_Parse(&mut p, &mut holdVal) {
            let y = atof(&holdVal) as f32;
            if String_Parse(&mut p, &mut holdVal) {
                let w = atof(&holdVal) as f32;
                if String_Parse(&mut p, &mut holdVal) {
                    let h = atof(&holdVal) as f32;

                    let it = ctx.world.menus.item_mut(item);
                    it.window.rectClient.x = x;
                    it.window.rectClient.y = y;
                    it.window.rectClient.w = w;
                    it.window.rectClient.h = h;

                    it.window.rect.x = it.window.rectClient.x;
                    it.window.rect.y = it.window.rectClient.y;
                    it.window.rect.w = it.window.rectClient.w;
                    it.window.rect.h = it.window.rectClient.h;
                }
            }
        }
    }
}

/// Raven `UI_UpdateCharacterSkin`.
///
/// Source: `oracle/codemp/ui/ui_main.c:6048-6085`
pub fn UI_UpdateCharacterSkin(ctx: &mut UiContext, dc: &mut dyn DisplayContext) {
    let menu = match Menu_GetFocused(&ctx.world.menus) {
        Some(m) => m,
        None => return,
    };

    let item = match Menu_FindItemByName(&ctx.world.menus, Some(menu), "character") {
        Some(i) => i,
        None => {
            let menuName = ctx
                .world
                .menus
                .menu(menu)
                .window
                .name
                .clone()
                .unwrap_or_default();
            Com_Error(
                ctx,
                &format!(
                    "UI_UpdateCharacterSkin: Could not find item (character) in menu ({})",
                    menuName
                ),
            );
            return;
        }
    };

    let model = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_char_model", MAX_QPATH);
    let head = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_char_skin_head", MAX_QPATH);
    let torso = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_char_skin_torso", MAX_QPATH);
    let legs = trap::Cvar_VariableStringBuffer(ctx.engine, "ui_char_skin_legs", MAX_QPATH);

    let skin = format!("models/players/{}/|{}|{}|{}", model, head, torso, legs);

    ItemParse_model_g2skin_go(&mut ctx.world.menus, dc, item, Some(&skin));
}

/// Raven `UI_SiegeClassCnt`.
///
/// Source: `oracle/codemp/ui/ui_main.c:7516-7527`
pub fn UI_SiegeClassCnt(ctx: &mut UiContext, dc: &mut dyn DisplayContext, team: c_int) {
    UI_SetSiegeTeams(ctx, dc);

    let infantry = BG_SiegeCountBaseClass(team, 0, &ctx.world.bg_state);
    trap::Cvar_Set(ctx.engine, "ui_infantry_cnt", &format!("{}", infantry));
    let vanguard = BG_SiegeCountBaseClass(team, 1, &ctx.world.bg_state);
    trap::Cvar_Set(ctx.engine, "ui_vanguard_cnt", &format!("{}", vanguard));
    let support = BG_SiegeCountBaseClass(team, 2, &ctx.world.bg_state);
    trap::Cvar_Set(ctx.engine, "ui_support_cnt", &format!("{}", support));
    let jedi = BG_SiegeCountBaseClass(team, 3, &ctx.world.bg_state);
    trap::Cvar_Set(ctx.engine, "ui_jedi_cnt", &format!("{}", jedi));
    let demo = BG_SiegeCountBaseClass(team, 4, &ctx.world.bg_state);
    trap::Cvar_Set(ctx.engine, "ui_demo_cnt", &format!("{}", demo));
    let heavy = BG_SiegeCountBaseClass(team, 5, &ctx.world.bg_state);
    trap::Cvar_Set(ctx.engine, "ui_heavy_cnt", &format!("{}", heavy));
}

/// Raven `UI_UpdateSiegeStatusIcons`.
///
/// Source: `oracle/codemp/ui/ui_main.c:7559-7592`
pub fn UI_UpdateSiegeStatusIcons(world: &mut UiWorld, dc: &mut dyn DisplayContext) {
    let menu = match Menu_GetFocused(&world.menus) {
        Some(m) => m,
        None => return,
    };

    for i in 0..7 {
        Menu_SetItemBackground(
            &mut world.menus,
            dc,
            Some(menu),
            &format!("wpnicon0{}", i),
            &format!("*ui_class_weapon{}", i),
        );
    }

    for i in 0..7 {
        Menu_SetItemBackground(
            &mut world.menus,
            dc,
            Some(menu),
            &format!("itemicon0{}", i),
            &format!("*ui_class_item{}", i),
        );
    }

    for i in 0..10 {
        Menu_SetItemBackground(
            &mut world.menus,
            dc,
            Some(menu),
            &format!("forceicon0{}", i),
            &format!("*ui_class_power{}", i),
        );
    }

    for i in 10..15 {
        Menu_SetItemBackground(
            &mut world.menus,
            dc,
            Some(menu),
            &format!("forceicon{}", i),
            &format!("*ui_class_power{}", i),
        );
    }
}

/// Raven `UI_FeederCount`.
///
/// Source: `oracle/codemp/ui/ui_main.c:8475-8694`
#[allow(clippy::too_many_lines)]
pub fn UI_FeederCount(ctx: &mut UiContext, dc: &mut dyn DisplayContext, feederID: f32) -> c_int {
    match feederID as c_int {
        FEEDER_SABER_SINGLE_INFO => {
            let mut count = 0;
            for i in 0..MAX_SABER_HILTS {
                match ctx.world.main.saberSingleHiltInfo.get(i) {
                    Some(s) if !s.is_empty() => count += 1,
                    _ => break,
                }
            }
            return count;
        }
        FEEDER_SABER_STAFF_INFO => {
            let mut count = 0;
            for i in 0..MAX_SABER_HILTS {
                match ctx.world.main.saberStaffHiltInfo.get(i) {
                    Some(s) if !s.is_empty() => count += 1,
                    _ => break,
                }
            }
            return count;
        }
        FEEDER_Q3HEADS => return UI_HeadCountByColor(ctx.world),
        FEEDER_SIEGE_TEAM1 => {
            if ctx.world.main.siegeTeam1.is_none() {
                UI_SetSiegeTeams(ctx, dc);
                if ctx.world.main.siegeTeam1.is_none() {
                    return 0;
                }
            }
            let idx = ctx.world.main.siegeTeam1.unwrap();
            return ctx.world.bg_state.bgSiegeTeams[idx].numClasses;
        }
        FEEDER_SIEGE_TEAM2 => {
            if ctx.world.main.siegeTeam2.is_none() {
                UI_SetSiegeTeams(ctx, dc);
                if ctx.world.main.siegeTeam2.is_none() {
                    return 0;
                }
            }
            let idx = ctx.world.main.siegeTeam2.unwrap();
            return ctx.world.bg_state.bgSiegeTeams[idx].numClasses;
        }
        FEEDER_FORCECFG => {
            return if ctx.world.force.uiForceSide == FORCE_LIGHTSIDE {
                ctx.world.forceConfigNames.len() as c_int - ctx.world.forceConfigLightIndexBegin
            } else {
                ctx.world.forceConfigLightIndexBegin + 1
            };
        }
        FEEDER_CINEMATICS => return ctx.world.movieList.len() as c_int,
        FEEDER_MAPS | FEEDER_ALLMAPS => {
            let single = feederID as c_int == FEEDER_MAPS;
            return UI_MapCountByGameType(ctx.world, single);
        }
        FEEDER_SERVERS => return ctx.world.serverStatus.displayServers.len() as c_int,
        FEEDER_SERVERSTATUS => return ctx.world.serverStatusInfo.lines.len() as c_int,
        FEEDER_FINDPLAYER => return ctx.world.foundPlayerServerAddresses.len() as c_int,
        FEEDER_PLAYER_LIST => {
            if ctx.world.uiDC.realTime > ctx.world.playerRefresh {
                ctx.world.playerRefresh = ctx.world.uiDC.realTime + 3000;
                UI_BuildPlayerList(ctx);
            }
            return ctx.world.playerNames.len() as c_int;
        }
        FEEDER_TEAM_LIST => {
            if ctx.world.uiDC.realTime > ctx.world.playerRefresh {
                ctx.world.playerRefresh = ctx.world.uiDC.realTime + 3000;
                UI_BuildPlayerList(ctx);
            }
            return ctx.world.teamNames.len() as c_int;
        }
        FEEDER_MODS => return ctx.world.modList.len() as c_int,
        FEEDER_DEMOS => return ctx.world.demoList.len() as c_int,
        FEEDER_MOVES => {
            let mut count = 0;
            for i in 0..MAX_MOVES {
                if DATAPAD_MOVE_DATA[ctx.world.movesTitleIndex as usize][i]
                    .title
                    .is_some()
                {
                    count += 1;
                }
            }
            return count;
        }
        FEEDER_MOVES_TITLES => return MD_MOVE_TITLE_MAX,
        FEEDER_PLAYER_SPECIES => return ctx.world.playerSpecies.len() as c_int,
        FEEDER_PLAYER_SKIN_HEAD => {
            let idx = ctx.world.playerSpeciesIndex as usize;
            return ctx.world.playerSpecies[idx].SkinHeadNames.len() as c_int;
        }
        FEEDER_PLAYER_SKIN_TORSO => {
            let idx = ctx.world.playerSpeciesIndex as usize;
            return ctx.world.playerSpecies[idx].SkinTorsoNames.len() as c_int;
        }
        FEEDER_PLAYER_SKIN_LEGS => {
            let idx = ctx.world.playerSpeciesIndex as usize;
            return ctx.world.playerSpecies[idx].SkinLegNames.len() as c_int;
        }
        FEEDER_COLORCHOICES => {
            let idx = ctx.world.playerSpeciesIndex as usize;
            return ctx.world.playerSpecies[idx].ColorShader.len() as c_int;
        }
        FEEDER_SIEGE_BASE_CLASS => {
            let team = trap::Cvar_VariableValue(ctx.engine, "ui_team") as c_int;
            let baseClass = trap::Cvar_VariableValue(ctx.engine, "ui_siege_class") as c_int;

            if team == SIEGETEAM_TEAM1 || team == SIEGETEAM_TEAM2 {
                // Is it a valid base class?
                if baseClass >= SPC_INFANTRY as c_int && baseClass < SPC_MAX as c_int {
                    return BG_SiegeCountBaseClass(team, baseClass as i16, &ctx.world.bg_state);
                }
            }
            return 0;
        }
        // Get the count of weapons.
        FEEDER_SIEGE_CLASS_WEAPONS => {
            let mut count = 0;
            for i in 0..WP_NUM_WEAPONS {
                let info = trap::Cvar_VariableStringBuffer(
                    ctx.engine,
                    &format!("ui_class_weapon{}", i),
                    MAX_STRING_CHARS,
                );
                if Q_stricmp(&info, "gfx/2d/select") != 0 {
                    count += 1;
                }
            }
            return count;
        }
        // Get the count of inventory.
        FEEDER_SIEGE_CLASS_INVENTORY => {
            let mut count = 0;
            for i in 0..HI_NUM_HOLDABLE {
                let info = trap::Cvar_VariableStringBuffer(
                    ctx.engine,
                    &format!("ui_class_item{}", i),
                    MAX_STRING_CHARS,
                );
                // A hack so health and ammo dispenser icons don't show up.
                if Q_stricmp(&info, "gfx/2d/select") != 0
                    && Q_stricmp(&info, "gfx/hud/i_icon_healthdisp") != 0
                    && Q_stricmp(&info, "gfx/hud/i_icon_ammodisp") != 0
                {
                    count += 1;
                }
            }
            return count;
        }
        // Get the count of force powers.
        FEEDER_SIEGE_CLASS_FORCE => {
            let mut count = 0;
            for i in 0..NUM_FORCE_POWERS {
                let info = trap::Cvar_VariableStringBuffer(
                    ctx.engine,
                    &format!("ui_class_power{}", i),
                    MAX_STRING_CHARS,
                );
                if Q_stricmp(&info, "gfx/2d/select") != 0 {
                    count += 1;
                }
            }
            return count;
        }
        _ => {}
    }

    0
}

/// Raven `UI_FeederItemImage`.
///
/// Source: `oracle/codemp/ui/ui_main.c:9174-9434`
#[allow(clippy::too_many_lines)]
pub fn UI_FeederItemImage(
    ctx: &mut UiContext,
    dc: &mut dyn DisplayContext,
    feederID: f32,
    index: c_int,
) -> qhandle_t {
    let feeder = feederID as c_int;

    if feeder == FEEDER_SABER_SINGLE_INFO {
        return 0;
    } else if feeder == FEEDER_SABER_STAFF_INFO {
        return 0;
    } else if feeder == FEEDER_Q3HEADS {
        let mut actual = 0;
        let _ = UI_SelectedTeamHead(ctx.world, index, &mut actual);
        let index = actual;

        if index >= 0 && (index as usize) < ctx.world.q3HeadNames.len() {
            // we want it to load them as it draws them, like the TA feeder
            let selModel = trap::Cvar_VariableValue(ctx.engine, "ui_selectedModelIndex") as c_int;

            if selModel != -1 && ctx.world.q3SelectedHead != selModel {
                ctx.world.q3SelectedHead = selModel;
                Menu_SetFeederSelection(
                    &mut ctx.world.menus,
                    dc,
                    None,
                    FEEDER_Q3HEADS,
                    selModel,
                    None,
                );
            }

            if ctx.world.q3HeadIcons[index as usize] == 0 {
                // this isn't the best way of doing this I guess, but I didn't
                // want a whole seperate string array for storing shader
                // names. I can't just replace q3HeadNames with the shader
                // name, because we print what's in q3HeadNames and the icon
                // name would look funny.
                //
                // PORT-NOTE (§19 UB pick): Raven scans backward from the end
                // of the name for '/' with no lower bound — a name with no
                // slash underflows the C loop (UB). `rfind` returning `None`
                // (no slash) falls back to index 0, a defined substitute.
                let headName = ctx.world.q3HeadNames[index as usize].clone();
                let slash = headName.rfind('/').unwrap_or(0);
                let skinPlace = &headName[slash.saturating_add(1).min(headName.len())..];

                // now, build a full path out of what's in q3HeadNames, into
                // iconNameFromSkinName
                let full = format!("models/players/{}", headName);
                let slash2 = full.rfind('/').unwrap_or(0);
                let mut iconNameFromSkinName = full[..=slash2].to_string();
                iconNameFromSkinName.push_str("icon_");
                iconNameFromSkinName.push_str(skinPlace);

                // and now we are ready to register (thankfully this will only
                // happen once)
                ctx.world.q3HeadIcons[index as usize] =
                    trap::R_RegisterShaderNoMip(ctx.engine, &iconNameFromSkinName);
            }
            return ctx.world.q3HeadIcons[index as usize];
        }
    } else if feeder == FEEDER_SIEGE_TEAM1 {
        if ctx.world.main.siegeTeam1.is_none() {
            UI_SetSiegeTeams(ctx, dc);
            if ctx.world.main.siegeTeam1.is_none() {
                return 0;
            }
        }
        let idx = ctx.world.main.siegeTeam1.unwrap();
        return BG_SiegeTeamClassPortrait(idx, index, &ctx.world.bg_state);
    } else if feeder == FEEDER_SIEGE_TEAM2 {
        if ctx.world.main.siegeTeam2.is_none() {
            UI_SetSiegeTeams(ctx, dc);
            if ctx.world.main.siegeTeam2.is_none() {
                return 0;
            }
        }
        let idx = ctx.world.main.siegeTeam2.unwrap();
        return BG_SiegeTeamClassPortrait(idx, index, &ctx.world.bg_state);
    } else if feeder == FEEDER_ALLMAPS || feeder == FEEDER_MAPS {
        let mut actual = 0;
        let _ = UI_SelectedMap(ctx.world, index, &mut actual);
        let index = actual;
        if index >= 0 && (index as usize) < ctx.world.mapList.len() {
            if ctx.world.mapList[index as usize].levelShot == -1 {
                let imageName = ctx.world.mapList[index as usize].imageName.clone();
                ctx.world.mapList[index as usize].levelShot =
                    trap::R_RegisterShaderNoMip(ctx.engine, &imageName);
            }
            return ctx.world.mapList[index as usize].levelShot;
        }
    } else if feeder == FEEDER_PLAYER_SKIN_HEAD {
        let speciesIdx = ctx.world.playerSpeciesIndex as usize;
        if index >= 0 && (index as usize) < ctx.world.playerSpecies[speciesIdx].SkinHeadNames.len()
        {
            let name = ctx.world.playerSpecies[speciesIdx].Name.clone();
            let skin = ctx.world.playerSpecies[speciesIdx].SkinHeadNames[index as usize].clone();
            return trap::R_RegisterShaderNoMip(
                ctx.engine,
                &format!("models/players/{}/icon_{}", name, skin),
            );
        }
    } else if feeder == FEEDER_PLAYER_SKIN_TORSO {
        let speciesIdx = ctx.world.playerSpeciesIndex as usize;
        if index >= 0 && (index as usize) < ctx.world.playerSpecies[speciesIdx].SkinTorsoNames.len()
        {
            let name = ctx.world.playerSpecies[speciesIdx].Name.clone();
            let skin = ctx.world.playerSpecies[speciesIdx].SkinTorsoNames[index as usize].clone();
            return trap::R_RegisterShaderNoMip(
                ctx.engine,
                &format!("models/players/{}/icon_{}", name, skin),
            );
        }
    } else if feeder == FEEDER_PLAYER_SKIN_LEGS {
        let speciesIdx = ctx.world.playerSpeciesIndex as usize;
        if index >= 0 && (index as usize) < ctx.world.playerSpecies[speciesIdx].SkinLegNames.len() {
            let name = ctx.world.playerSpecies[speciesIdx].Name.clone();
            let skin = ctx.world.playerSpecies[speciesIdx].SkinLegNames[index as usize].clone();
            return trap::R_RegisterShaderNoMip(
                ctx.engine,
                &format!("models/players/{}/icon_{}", name, skin),
            );
        }
    } else if feeder == FEEDER_COLORCHOICES {
        let speciesIdx = ctx.world.playerSpeciesIndex as usize;
        if index >= 0 && (index as usize) < ctx.world.playerSpecies[speciesIdx].ColorShader.len() {
            let shader = ctx.world.playerSpecies[speciesIdx].ColorShader[index as usize].clone();
            return trap::R_RegisterShaderNoMip(ctx.engine, &shader);
        }
    } else if feeder == FEEDER_SIEGE_BASE_CLASS {
        let team = trap::Cvar_VariableValue(ctx.engine, "ui_team") as c_int;
        let baseClass = trap::Cvar_VariableValue(ctx.engine, "ui_siege_class") as c_int;

        if team == SIEGETEAM_TEAM1 || team == SIEGETEAM_TEAM2 {
            // Is it a valid base class?
            if baseClass >= SPC_INFANTRY as c_int && baseClass < SPC_MAX as c_int && index >= 0 {
                return BG_GetUIPortrait(team, baseClass as i16, index as i16, &ctx.world.bg_state);
            }
        }
    } else if feeder == FEEDER_SIEGE_CLASS_WEAPONS {
        let mut validCnt = 0;
        for i in 0..WP_NUM_WEAPONS {
            let info = trap::Cvar_VariableStringBuffer(
                ctx.engine,
                &format!("ui_class_weapon{}", i),
                MAX_STRING_CHARS,
            );
            if Q_stricmp(&info, "gfx/2d/select") != 0 {
                if validCnt == index {
                    return trap::R_RegisterShaderNoMip(ctx.engine, &info);
                }
                validCnt += 1;
            }
        }
    } else if feeder == FEEDER_SIEGE_CLASS_INVENTORY {
        let mut validCnt = 0;
        for i in 0..HI_NUM_HOLDABLE {
            let info = trap::Cvar_VariableStringBuffer(
                ctx.engine,
                &format!("ui_class_item{}", i),
                MAX_STRING_CHARS,
            );
            // A hack so health and ammo dispenser icons don't show up.
            if Q_stricmp(&info, "gfx/2d/select") != 0
                && Q_stricmp(&info, "gfx/hud/i_icon_healthdisp") != 0
                && Q_stricmp(&info, "gfx/hud/i_icon_ammodisp") != 0
            {
                if validCnt == index {
                    return trap::R_RegisterShaderNoMip(ctx.engine, &info);
                }
                validCnt += 1;
            }
        }
    } else if feeder == FEEDER_SIEGE_CLASS_FORCE {
        let mut slotI: c_int = 0;
        let mut validCnt = 0;

        if let Some(menu) = Menu_GetFocused(&ctx.world.menus) {
            if let Some(item) =
                Menu_FindItemByName(&ctx.world.menus, Some(menu), "base_class_force_feed")
            {
                if let Some(listPtr) = ctx.world.menus.item(item).typeData.listBox() {
                    slotI = listPtr.startPos;
                }
            }
        }

        for i in 0..NUM_FORCE_POWERS {
            let info = trap::Cvar_VariableStringBuffer(
                ctx.engine,
                &format!("ui_class_power{}", i),
                MAX_STRING_CHARS,
            );
            if Q_stricmp(&info, "gfx/2d/select") != 0 {
                if validCnt == index {
                    let info2 = trap::Cvar_VariableStringBuffer(
                        ctx.engine,
                        &format!("ui_class_powerlevel{}", validCnt),
                        MAX_STRING_CHARS,
                    );
                    trap::Cvar_Set(
                        ctx.engine,
                        &format!("ui_class_powerlevelslot{}", index - slotI),
                        &info2,
                    );
                    return trap::R_RegisterShaderNoMip(ctx.engine, &info);
                }
                validCnt += 1;
            }
        }
    }

    0
}

/// Raven `GameType_Parse`.
///
/// Source: `oracle/codemp/ui/ui_main.c:9997-10056`
pub fn GameType_Parse(ctx: &mut UiContext, p: &mut &str, join: bool) -> bool {
    let (token, rest) = COM_Parse(p, true);
    *p = rest;

    if !token.starts_with('{') {
        return false;
    }

    if join {
        ctx.world.joinGameTypes.clear();
    } else {
        ctx.world.gameTypes.clear();
    }

    loop {
        let (token, rest) = COM_Parse(p, true);
        *p = rest;

        if Q_stricmp(&token, "}") == 0 {
            return true;
        }

        if token.is_empty() {
            return false;
        }

        if token.starts_with('{') {
            // two tokens per line, character name and sex
            let mut gameType = String::new();
            let mut gtEnum: c_int = 0;
            if !String_Parse(p, &mut gameType) || !Int_Parse(p, &mut gtEnum) {
                return false;
            }

            if join {
                if ctx.world.joinGameTypes.len() < MAX_GAMETYPES {
                    ctx.world
                        .joinGameTypes
                        .push(GameTypeInfo { gameType, gtEnum });
                } else {
                    Com_Printf(ctx, "Too many net game types, last one replace!\n");
                }
            } else if ctx.world.gameTypes.len() < MAX_GAMETYPES {
                ctx.world.gameTypes.push(GameTypeInfo { gameType, gtEnum });
            } else {
                Com_Printf(ctx, "Too many game types, last one replace!\n");
            }

            let (token2, rest2) = COM_Parse(p, true);
            *p = rest2;
            if !token2.starts_with('}') {
                return false;
            }
        }
    }
}

/// Raven `MapList_Parse`.
///
/// Source: `oracle/codemp/ui/ui_main.c:10058-10120`
pub fn MapList_Parse(ctx: &mut UiContext, p: &mut &str) -> bool {
    let (token, rest) = COM_Parse(p, true);
    *p = rest;

    if !token.starts_with('{') {
        return false;
    }

    ctx.world.mapList.clear();

    loop {
        let (token, rest) = COM_Parse(p, true);
        *p = rest;

        if Q_stricmp(&token, "}") == 0 {
            return true;
        }

        if token.is_empty() {
            return false;
        }

        if token.starts_with('{') {
            let mut mapName = String::new();
            let mut mapLoadName = String::new();
            let mut teamMembers: c_int = 0;
            if !String_Parse(p, &mut mapName)
                || !String_Parse(p, &mut mapLoadName)
                || !Int_Parse(p, &mut teamMembers)
            {
                return false;
            }

            let mut opponentName = String::new();
            if !String_Parse(p, &mut opponentName) {
                return false;
            }

            let mut typeBits: c_int = 0;
            let mut timeToBeat = [0; MAX_GAMETYPES];

            loop {
                let (token2, rest2) = COM_Parse(p, true);
                *p = rest2;
                match token2.as_bytes().first().copied() {
                    Some(b @ b'0'..=b'9') => {
                        let digit = (b - b'0') as usize;
                        typeBits |= 1 << digit;
                        let mut time: c_int = 0;
                        if !Int_Parse(p, &mut time) {
                            return false;
                        }
                        if digit < timeToBeat.len() {
                            timeToBeat[digit] = time;
                        }
                    }
                    _ => break,
                }
            }

            let levelShotName = format!("levelshots/{}_small", mapLoadName);
            let levelShot = trap::R_RegisterShaderNoMip(ctx.engine, &levelShotName);

            if ctx.world.mapList.len() < MAX_MAPS {
                ctx.world.mapList.push(MapInfo {
                    mapName,
                    mapLoadName,
                    opponentName,
                    teamMembers,
                    typeBits,
                    cinematic: -1,
                    timeToBeat,
                    levelShot,
                    ..Default::default()
                });
            } else {
                Com_Printf(ctx, "Too many maps, last one replaced!\n");
            }
        }
    }
}

/// Raven `UI_DisplayDownloadInfo`.
///
/// Source: `oracle/codemp/ui/ui_main.c:11075-11163`
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub fn UI_DisplayDownloadInfo(
    ctx: &mut UiContext,
    downloadName: &str,
    centerPoint: f32,
    yStart: f32,
    scale: f32,
    iMenuFont: c_int,
) {
    let colorLtGreyAlpha: vec4_t = [0.0, 0.0, 0.0, 0.5];

    UI_FillRect(
        ctx,
        0.0,
        0.0,
        SCREEN_WIDTH as f32,
        SCREEN_HEIGHT as f32,
        &colorLtGreyAlpha,
    );

    // "Downloading:"
    let sDownLoading = GetCRDelineatedString(ctx, "MENUS", "DOWNLOAD_STUFF", 0).unwrap_or_default();
    // "Estimated time left:"
    let sEstimatedTimeLeft =
        GetCRDelineatedString(ctx, "MENUS", "DOWNLOAD_STUFF", 1).unwrap_or_default();
    // "Transfer rate:"
    let sTransferRate =
        GetCRDelineatedString(ctx, "MENUS", "DOWNLOAD_STUFF", 2).unwrap_or_default();
    // "of"
    let sOf = GetCRDelineatedString(ctx, "MENUS", "DOWNLOAD_STUFF", 3).unwrap_or_default();
    // "copied"
    let sCopied = GetCRDelineatedString(ctx, "MENUS", "DOWNLOAD_STUFF", 4).unwrap_or_default();
    // "sec."
    let sSec = GetCRDelineatedString(ctx, "MENUS", "DOWNLOAD_STUFF", 5).unwrap_or_default();

    let downloadSize = trap::Cvar_VariableValue(ctx.engine, "cl_downloadSize") as c_int;
    let downloadCount = trap::Cvar_VariableValue(ctx.engine, "cl_downloadCount") as c_int;
    let downloadTime = trap::Cvar_VariableValue(ctx.engine, "cl_downloadTime") as c_int;

    let leftWidth: f32 = 320.0;

    UI_SetColor(ctx, Some(&colorWhite));

    Text_PaintCenter(
        ctx,
        centerPoint,
        yStart + 112.0,
        scale,
        colorWhite,
        &sDownLoading,
        0.0,
        iMenuFont,
    );
    Text_PaintCenter(
        ctx,
        centerPoint,
        yStart + 192.0,
        scale,
        colorWhite,
        &sEstimatedTimeLeft,
        0.0,
        iMenuFont,
    );
    Text_PaintCenter(
        ctx,
        centerPoint,
        yStart + 248.0,
        scale,
        colorWhite,
        &sTransferRate,
        0.0,
        iMenuFont,
    );

    let s = if downloadSize > 0 {
        format!("{} ({}%)", downloadName, downloadCount * 100 / downloadSize)
    } else {
        downloadName.to_string()
    };

    Text_PaintCenter(
        ctx,
        centerPoint,
        yStart + 136.0,
        scale,
        colorWhite,
        &s,
        0.0,
        iMenuFont,
    );

    let dlSizeBuf = UI_ReadableSize(downloadCount);
    let totalSizeBuf = UI_ReadableSize(downloadSize);

    if downloadCount < 4096 || downloadTime == 0 {
        Text_PaintCenter(
            ctx,
            leftWidth,
            yStart + 216.0,
            scale,
            colorWhite,
            "estimating",
            0.0,
            iMenuFont,
        );
        Text_PaintCenter(
            ctx,
            leftWidth,
            yStart + 160.0,
            scale,
            colorWhite,
            &format!("({} {} {} {})", dlSizeBuf, sOf, totalSizeBuf, sCopied),
            0.0,
            iMenuFont,
        );
    } else {
        let xferRate = if (ctx.world.uiDC.realTime - downloadTime) / 1000 != 0 {
            downloadCount / ((ctx.world.uiDC.realTime - downloadTime) / 1000)
        } else {
            0
        };
        let xferRateBuf = UI_ReadableSize(xferRate);

        // Extrapolate estimated completion time.
        if downloadSize != 0 && xferRate != 0 {
            let n = downloadSize / xferRate; // estimated time for entire d/l in secs

            // We do it in K (/1024) because we'd overflow around 4MB.
            let dlTimeBuf =
                UI_PrintTime((n - (((downloadCount / 1024) * n) / (downloadSize / 1024))) * 1000);

            Text_PaintCenter(
                ctx,
                leftWidth,
                yStart + 216.0,
                scale,
                colorWhite,
                &dlTimeBuf,
                0.0,
                iMenuFont,
            );
            Text_PaintCenter(
                ctx,
                leftWidth,
                yStart + 160.0,
                scale,
                colorWhite,
                &format!("({} {} {} {})", dlSizeBuf, sOf, totalSizeBuf, sCopied),
                0.0,
                iMenuFont,
            );
        } else {
            Text_PaintCenter(
                ctx,
                leftWidth,
                yStart + 216.0,
                scale,
                colorWhite,
                "estimating",
                0.0,
                iMenuFont,
            );
            if downloadSize != 0 {
                Text_PaintCenter(
                    ctx,
                    leftWidth,
                    yStart + 160.0,
                    scale,
                    colorWhite,
                    &format!("({} {} {} {})", dlSizeBuf, sOf, totalSizeBuf, sCopied),
                    0.0,
                    iMenuFont,
                );
            } else {
                Text_PaintCenter(
                    ctx,
                    leftWidth,
                    yStart + 160.0,
                    scale,
                    colorWhite,
                    &format!("({} {})", dlSizeBuf, sCopied),
                    0.0,
                    iMenuFont,
                );
            }
        }

        if xferRate != 0 {
            Text_PaintCenter(
                ctx,
                leftWidth,
                yStart + 272.0,
                scale,
                colorWhite,
                &format!("{}/{}", xferRateBuf, sSec),
                0.0,
                iMenuFont,
            );
        }
    }
}

/// Raven `UI_DoServerRefresh`.
///
/// Source: `oracle/codemp/ui/ui_main.c:11596-11632`
pub fn UI_DoServerRefresh(ctx: &mut UiContext, dc: &mut dyn DisplayContext) {
    let mut wait = false;

    if !ctx.world.serverStatus.refreshActive {
        return;
    }
    if ctx.world.cvars.ui_netSource.integer != AS_FAVORITES {
        if ctx.world.cvars.ui_netSource.integer == AS_LOCAL {
            if trap::LAN_GetServerCount(ctx.engine, ctx.world.cvars.ui_netSource.integer) == 0 {
                wait = true;
            }
        } else if trap::LAN_GetServerCount(ctx.engine, ctx.world.cvars.ui_netSource.integer) < 0 {
            wait = true;
        }
    }

    if ctx.world.uiDC.realTime < ctx.world.serverStatus.refreshtime && wait {
        return;
    }

    // if still trying to retrieve pings
    if trap::LAN_UpdateVisiblePings(ctx.engine, ctx.world.cvars.ui_netSource.integer) {
        ctx.world.serverStatus.refreshtime = ctx.world.uiDC.realTime + 1000;
    } else if !wait {
        // get the last servers in the list
        UI_BuildServerDisplayList(ctx, dc, 2);
        // stop the refresh
        UI_StopServerRefresh(ctx);
    }
    //
    UI_BuildServerDisplayList(ctx, dc, 0);
}

/// Raven `UI_StartServerRefresh`.
///
/// Source: `oracle/codemp/ui/ui_main.c:11639-11688`
pub fn UI_StartServerRefresh(ctx: &mut UiContext, full: bool) {
    let mut q = qtime_t {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
    };
    trap::RealTime(ctx.engine, &mut q);
    let netSourceIdx = ctx.world.cvars.ui_netSource.integer;
    let month = GetMonthAbbrevString(ctx, q.tm_mon);
    let value = format!(
        "{}-{}, {} @ {}:{:2}",
        month,
        q.tm_mday,
        1900 + q.tm_year,
        q.tm_hour,
        q.tm_min
    );
    trap::Cvar_Set(
        ctx.engine,
        &format!("ui_lastServerRefresh_{}", netSourceIdx),
        &value,
    );

    if !full {
        UI_UpdatePendingPings(ctx);
        return;
    }

    ctx.world.serverStatus.refreshActive = true;
    ctx.world.serverStatus.nextDisplayRefresh = ctx.world.uiDC.realTime + 1000;
    // clear number of displayed servers
    ctx.world.serverStatus.displayServers.clear();
    ctx.world.serverStatus.numPlayersOnServers = 0;
    // mark all servers as visible so we store ping updates for them
    trap::LAN_MarkServerVisible(ctx.engine, ctx.world.cvars.ui_netSource.integer, -1, true);
    // reset all the pings
    trap::LAN_ResetPings(ctx.engine, ctx.world.cvars.ui_netSource.integer);
    //
    if ctx.world.cvars.ui_netSource.integer == AS_LOCAL {
        trap::Cmd_ExecuteText(ctx.engine, cbufExec_t::EXEC_NOW as c_int, "localservers\n");
        ctx.world.serverStatus.refreshtime = ctx.world.uiDC.realTime + 1000;
        return;
    }

    ctx.world.serverStatus.refreshtime = ctx.world.uiDC.realTime + 5000;
    // Optimatch is handled elsewhere (retail excludes _XBOX).
    if ctx.world.cvars.ui_netSource.integer == AS_GLOBAL
        || ctx.world.cvars.ui_netSource.integer == AS_MPLAYER
    {
        let i = if ctx.world.cvars.ui_netSource.integer == AS_GLOBAL {
            0
        } else {
            1
        };

        let ptr = UI_Cvar_VariableString(ctx, "debug_protocol");
        if !ptr.is_empty() {
            trap::Cmd_ExecuteText(
                ctx.engine,
                cbufExec_t::EXEC_NOW as c_int,
                &format!("globalservers {} {}\n", i, ptr),
            );
        } else {
            let protocol = trap::Cvar_VariableValue(ctx.engine, "protocol") as c_int;
            trap::Cmd_ExecuteText(
                ctx.engine,
                cbufExec_t::EXEC_NOW as c_int,
                &format!("globalservers {} {}\n", i, protocol),
            );
        }
    }
}

/// Raven `SORT_HOST`/`SORT_MAP`/`SORT_CLIENTS`/`SORT_GAME`/`SORT_PING` —
/// server-browser column ids `UI_FeederItemText`'s `FEEDER_SERVERS` switches
/// on.
///
/// Source: `oracle/codemp/ui/ui_public.h:210-214`
const SORT_HOST: c_int = 0;
const SORT_MAP: c_int = 1;
const SORT_CLIENTS: c_int = 2;
const SORT_GAME: c_int = 3;
const SORT_PING: c_int = 4;

/// Raven `static const int numNetSources` — hard-coded to 3 (the commented-out
/// `netSources[]`/`sizeof` derivation above it is dead).
///
/// Source: `oracle/codemp/ui/ui_main.c:998`
const NUM_NET_SOURCES: c_int = 3;

/// Raven `static char *netnames[]`.
///
/// PORT-NOTE: the trailing Raven `NULL` sentinel (never reached — `nettype`
/// only carries 0-2) becomes `""`.
/// Source: `oracle/codemp/ui/ui_main.c:930-935`
const NETNAMES: [&str; 4] = ["???", "UDP", "IPX", ""];

/// Raven `static const char *teamArenaGameTypes[]`.
///
/// Source: `oracle/codemp/ui/ui_main.c:913-924`
const TEAM_ARENA_GAME_TYPES: [&str; 11] = [
    "FFA",
    "Holocron",
    "JediMaster",
    "Duel",
    "PowerDuel",
    "SP",
    "Team FFA",
    "Siege",
    "CTF",
    "CTY",
    "TeamTournament",
];

/// Raven `static int const numTeamArenaGameTypes`.
///
/// Source: `oracle/codemp/ui/ui_main.c:926`
const NUM_TEAM_ARENA_GAME_TYPES: c_int = TEAM_ARENA_GAME_TYPES.len() as c_int;

/// Raven `char *datapadMoveTitleData[MD_MOVE_TITLE_MAX]`.
///
/// Source: `oracle/codemp/ui/ui_main.c:384-391`
const DATAPAD_MOVE_TITLE_DATA: [&str; MD_MOVE_TITLE_MAX as usize] = [
    "@MENUS_ACROBATICS",
    "@MENUS_SINGLE_FAST",
    "@MENUS_SINGLE_MEDIUM",
    "@MENUS_SINGLE_STRONG",
    "@MENUS_DUAL_SABERS",
    "@MENUS_SABER_STAFF",
];

/// Raven `char *HolocronIcons[]` (`oracle/codemp/cgame/holocronicons.h`,
/// `#include`d by `ui_main.c:22`) — indexed by `forcePowers_t`.
///
/// Source: `oracle/codemp/cgame/holocronicons.h:4-22`
const HOLOCRON_ICONS: [&str; NUM_FORCE_POWERS as usize] = [
    "gfx/mp/f_icon_lt_heal",       // FP_HEAL
    "gfx/mp/f_icon_levitation",    // FP_LEVITATION
    "gfx/mp/f_icon_speed",         // FP_SPEED
    "gfx/mp/f_icon_push",          // FP_PUSH
    "gfx/mp/f_icon_pull",          // FP_PULL
    "gfx/mp/f_icon_lt_telepathy",  // FP_TELEPATHY
    "gfx/mp/f_icon_dk_grip",       // FP_GRIP
    "gfx/mp/f_icon_dk_l1",         // FP_LIGHTNING
    "gfx/mp/f_icon_dk_rage",       // FP_RAGE
    "gfx/mp/f_icon_lt_protect",    // FP_PROTECT
    "gfx/mp/f_icon_lt_absorb",     // FP_ABSORB
    "gfx/mp/f_icon_lt_healother",  // FP_TEAM_HEAL
    "gfx/mp/f_icon_dk_forceother", // FP_TEAM_FORCE
    "gfx/mp/f_icon_dk_drain",      // FP_DRAIN
    "gfx/mp/f_icon_sight",         // FP_SEE
    "gfx/mp/f_icon_saber_attack",  // FP_SABER_OFFENSE
    "gfx/mp/f_icon_saber_defend",  // FP_SABER_DEFENSE
    "gfx/mp/f_icon_saber_throw",   // FP_SABERTHROW
];

/// Decode a `pc_token_t.string` fixed buffer into an owned `String` — the
/// `Asset_Parse` token-comparison helper (Raven compares `token.string`
/// directly; the port's `pc_token_t.string` is a byte buffer, not a Rust
/// string).
fn pc_token_str(token: &pc_token_t) -> String {
    buf_to_string(&token.string.iter().map(|&c| c as u8).collect::<Vec<u8>>())
}

/// Raven `Asset_Parse`.
///
/// Source: `oracle/codemp/ui/ui_main.c:1463-1722`
#[allow(clippy::too_many_lines)]
pub fn Asset_Parse(ctx: &mut UiContext, dc: &mut dyn DisplayContext, handle: c_int) -> bool {
    let mut token = pc_token_t {
        type_: 0,
        subtype: 0,
        intvalue: 0,
        floatvalue: 0.0,
        string: [0; MAX_TOKENLENGTH],
    };

    if !trap::PC_ReadToken(ctx.engine, handle, &mut token) {
        return false;
    }
    if Q_stricmp(&pc_token_str(&token), "{") != 0 {
        return false;
    }

    loop {
        token = pc_token_t {
            type_: 0,
            subtype: 0,
            intvalue: 0,
            floatvalue: 0.0,
            string: [0; MAX_TOKENLENGTH],
        };

        if !trap::PC_ReadToken(ctx.engine, handle, &mut token) {
            return false;
        }

        let tokenStr = pc_token_str(&token);

        if Q_stricmp(&tokenStr, "}") == 0 {
            return true;
        }

        // font
        if Q_stricmp(&tokenStr, "font") == 0 {
            let mut pointSize = 0;
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token)
                || !PC_Int_Parse(dc, handle, &mut pointSize)
            {
                return false;
            }
            ctx.world.uiDC.Assets.qhMediumFont =
                trap::R_RegisterFont(ctx.engine, &pc_token_str(&token));
            ctx.world.uiDC.Assets.fontRegistered = true;
            continue;
        }

        if Q_stricmp(&tokenStr, "smallFont") == 0 {
            let mut pointSize = 0;
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token)
                || !PC_Int_Parse(dc, handle, &mut pointSize)
            {
                return false;
            }
            ctx.world.uiDC.Assets.qhSmallFont =
                trap::R_RegisterFont(ctx.engine, &pc_token_str(&token));
            continue;
        }

        if Q_stricmp(&tokenStr, "small2Font") == 0 {
            let mut pointSize = 0;
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token)
                || !PC_Int_Parse(dc, handle, &mut pointSize)
            {
                return false;
            }
            ctx.world.uiDC.Assets.qhSmall2Font =
                trap::R_RegisterFont(ctx.engine, &pc_token_str(&token));
            continue;
        }

        if Q_stricmp(&tokenStr, "bigFont") == 0 {
            let mut pointSize = 0;
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token)
                || !PC_Int_Parse(dc, handle, &mut pointSize)
            {
                return false;
            }
            ctx.world.uiDC.Assets.qhBigFont =
                trap::R_RegisterFont(ctx.engine, &pc_token_str(&token));
            continue;
        }

        if Q_stricmp(&tokenStr, "cursor") == 0 {
            let mut cursorStr = String::new();
            if !PC_String_Parse(dc, handle, &mut cursorStr) {
                // Raven passes `S_COLOR_YELLOW` as the FORMAT string, so retail
                // prints only "^3". Source: `oracle/codemp/ui/ui_main.c:1528`
                Com_Printf(ctx, S_COLOR_YELLOW.to_str().unwrap());
                return false;
            }
            ctx.world.uiDC.Assets.cursorStr = cursorStr.clone();
            ctx.world.uiDC.Assets.cursor = trap::R_RegisterShaderNoMip(ctx.engine, &cursorStr);
            continue;
        }

        // gradientbar
        if Q_stricmp(&tokenStr, "gradientbar") == 0 {
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token) {
                return false;
            }
            ctx.world.uiDC.Assets.gradientBar =
                trap::R_RegisterShaderNoMip(ctx.engine, &pc_token_str(&token));
            continue;
        }

        // enterMenuSound
        if Q_stricmp(&tokenStr, "menuEnterSound") == 0 {
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token) {
                return false;
            }
            ctx.world.uiDC.Assets.menuEnterSound =
                trap::S_RegisterSound(ctx.engine, &pc_token_str(&token));
            continue;
        }

        // exitMenuSound
        if Q_stricmp(&tokenStr, "menuExitSound") == 0 {
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token) {
                return false;
            }
            ctx.world.uiDC.Assets.menuExitSound =
                trap::S_RegisterSound(ctx.engine, &pc_token_str(&token));
            continue;
        }

        // itemFocusSound
        if Q_stricmp(&tokenStr, "itemFocusSound") == 0 {
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token) {
                return false;
            }
            ctx.world.uiDC.Assets.itemFocusSound =
                trap::S_RegisterSound(ctx.engine, &pc_token_str(&token));
            continue;
        }

        // menuBuzzSound
        if Q_stricmp(&tokenStr, "menuBuzzSound") == 0 {
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token) {
                return false;
            }
            ctx.world.uiDC.Assets.menuBuzzSound =
                trap::S_RegisterSound(ctx.engine, &pc_token_str(&token));
            continue;
        }

        if Q_stricmp(&tokenStr, "fadeClamp") == 0 {
            if !PC_Float_Parse(dc, handle, &mut ctx.world.uiDC.Assets.fadeClamp) {
                return false;
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "fadeCycle") == 0 {
            if !PC_Int_Parse(dc, handle, &mut ctx.world.uiDC.Assets.fadeCycle) {
                return false;
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "fadeAmount") == 0 {
            if !PC_Float_Parse(dc, handle, &mut ctx.world.uiDC.Assets.fadeAmount) {
                return false;
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "shadowX") == 0 {
            if !PC_Float_Parse(dc, handle, &mut ctx.world.uiDC.Assets.shadowX) {
                return false;
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "shadowY") == 0 {
            if !PC_Float_Parse(dc, handle, &mut ctx.world.uiDC.Assets.shadowY) {
                return false;
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "shadowColor") == 0 {
            if !PC_Color_Parse(dc, handle, &mut ctx.world.uiDC.Assets.shadowColor) {
                return false;
            }
            ctx.world.uiDC.Assets.shadowFadeClamp = ctx.world.uiDC.Assets.shadowColor[3];
            continue;
        }

        if Q_stricmp(&tokenStr, "moveRollSound") == 0 {
            if trap::PC_ReadToken(ctx.engine, handle, &mut token) {
                ctx.world.uiDC.Assets.moveRollSound =
                    trap::S_RegisterSound(ctx.engine, &pc_token_str(&token));
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "moveJumpSound") == 0 {
            if trap::PC_ReadToken(ctx.engine, handle, &mut token) {
                ctx.world.uiDC.Assets.moveJumpSound =
                    trap::S_RegisterSound(ctx.engine, &pc_token_str(&token));
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "datapadmoveSaberSound1") == 0 {
            if trap::PC_ReadToken(ctx.engine, handle, &mut token) {
                ctx.world.uiDC.Assets.datapadmoveSaberSound1 =
                    trap::S_RegisterSound(ctx.engine, &pc_token_str(&token));
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "datapadmoveSaberSound2") == 0 {
            if trap::PC_ReadToken(ctx.engine, handle, &mut token) {
                ctx.world.uiDC.Assets.datapadmoveSaberSound2 =
                    trap::S_RegisterSound(ctx.engine, &pc_token_str(&token));
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "datapadmoveSaberSound3") == 0 {
            if trap::PC_ReadToken(ctx.engine, handle, &mut token) {
                ctx.world.uiDC.Assets.datapadmoveSaberSound3 =
                    trap::S_RegisterSound(ctx.engine, &pc_token_str(&token));
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "datapadmoveSaberSound4") == 0 {
            if trap::PC_ReadToken(ctx.engine, handle, &mut token) {
                ctx.world.uiDC.Assets.datapadmoveSaberSound4 =
                    trap::S_RegisterSound(ctx.engine, &pc_token_str(&token));
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "datapadmoveSaberSound5") == 0 {
            if trap::PC_ReadToken(ctx.engine, handle, &mut token) {
                ctx.world.uiDC.Assets.datapadmoveSaberSound5 =
                    trap::S_RegisterSound(ctx.engine, &pc_token_str(&token));
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "datapadmoveSaberSound6") == 0 {
            if trap::PC_ReadToken(ctx.engine, handle, &mut token) {
                ctx.world.uiDC.Assets.datapadmoveSaberSound6 =
                    trap::S_RegisterSound(ctx.engine, &pc_token_str(&token));
            }
            continue;
        }

        // precaching various sound files used in the menus
        if Q_stricmp(&tokenStr, "precacheSound") == 0 {
            let mut tempStr = String::new();
            if PC_Script_Parse(dc, handle, &mut tempStr) {
                let mut p: &str = &tempStr;
                loop {
                    let (soundFile, rest) = COM_Parse(p, false);
                    p = rest;
                    if !soundFile.is_empty() && !soundFile.starts_with(';') {
                        trap::S_RegisterSound(ctx.engine, &soundFile);
                    }
                    if soundFile.is_empty() {
                        break;
                    }
                }
            }
            continue;
        }
    }
}

/// Raven `UI_OwnerDraw`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3510-3779`
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
pub fn UI_OwnerDraw(
    ctx: &mut UiContext,
    dc: &mut dyn DisplayContext,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    text_x: f32,
    text_y: f32,
    ownerDraw: c_int,
    _ownerDrawFlags: c_int,
    _align: c_int,
    _special: f32,
    scale: f32,
    mut color: vec4_t,
    _shader: qhandle_t,
    textStyle: c_int,
    iMenuFont: c_int,
) {
    let rect = RectDef {
        x: x + text_x,
        y: y + text_y,
        w,
        h,
    };

    match ownerDraw {
        UI_HANDICAP => {
            UI_DrawHandicap(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_SKIN_COLOR => {
            let uiSkinColor = ctx.world.main.uiSkinColor;
            UI_DrawSkinColor(
                ctx,
                &rect,
                scale,
                color,
                textStyle,
                uiSkinColor,
                TEAM_FREE,
                TEAM_BLUE,
                iMenuFont,
            );
        }
        UI_FORCE_SIDE => {
            let uiForceSide = ctx.world.force.uiForceSide;
            UI_DrawForceSide(
                ctx,
                dc,
                &rect,
                scale,
                &mut color,
                textStyle,
                uiForceSide,
                1,
                2,
                iMenuFont,
            );
        }
        UI_JEDI_NONJEDI => {
            let uiJediNonJedi = ctx.world.force.uiJediNonJedi;
            UI_DrawJediNonJedi(
                ctx,
                &rect,
                scale,
                color,
                textStyle,
                uiJediNonJedi,
                0,
                1,
                iMenuFont,
            );
        }
        UI_FORCE_POINTS => {
            let uiForceAvailable = ctx.world.force.uiForceAvailable;
            UI_DrawGenericNum(
                ctx,
                &rect,
                scale,
                color,
                textStyle,
                uiForceAvailable,
                1,
                forceMasteryPoints[MAX_FORCE_RANK as usize],
                ownerDraw,
                iMenuFont,
            );
        }
        UI_FORCE_MASTERY_SET | UI_FORCE_RANK => {
            let uiForceRank = ctx.world.force.uiForceRank;
            UI_DrawForceMastery(
                ctx,
                &rect,
                scale,
                color,
                textStyle,
                uiForceRank,
                0,
                MAX_FORCE_RANK,
                iMenuFont,
            );
        }
        UI_FORCE_RANK_HEAL..=UI_FORCE_RANK_SABERTHROW => {
            // this will give us the index as long as UI_FORCE_RANK is always
            // one below the first force rank index
            let findex = (ownerDraw - UI_FORCE_RANK) - 1;
            let darkLight = ctx.world.force.uiForcePowerDarkLight[findex as usize];
            if darkLight != 0 && ctx.world.force.uiForceSide != darkLight {
                color[0] *= 0.5;
                color[1] *= 0.5;
                color[2] *= 0.5;
            }
            let drawRank = ctx.world.force.uiForcePowersRank[findex as usize];
            UI_DrawForceStars(
                ctx,
                &rect,
                scale,
                &color,
                textStyle,
                findex,
                drawRank,
                0,
                NUM_FORCE_POWER_LEVELS - 1,
            );
        }
        UI_EFFECTS => {
            UI_DrawEffects(ctx, &rect, scale, color);
        }
        // PORT-NOTE (§20 dead surface, D7): `UI_DrawPlayerModel` is dead
        // (Raven leaves this call commented out).
        UI_PLAYERMODEL => {}
        UI_CLANNAME => {
            UI_DrawClanName(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_CLANLOGO => {
            UI_DrawClanLogo(ctx, &rect, scale, color);
        }
        UI_CLANCINEMATIC => {
            UI_DrawClanCinematic(ctx, &rect, scale, color);
        }
        UI_PREVIEWCINEMATIC => {
            UI_DrawPreviewCinematic(ctx, &rect, scale, color);
        }
        UI_GAMETYPE => {
            UI_DrawGameType(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_NETGAMETYPE => {
            UI_DrawNetGameType(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_AUTOSWITCHLIST => {
            UI_DrawAutoSwitch(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_JOINGAMETYPE => {
            UI_DrawJoinGameType(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_MAPPREVIEW => {
            UI_DrawMapPreview(ctx, &rect, scale, color, true);
        }
        UI_MAP_TIMETOBEAT => {
            UI_DrawMapTimeToBeat(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_MAPCINEMATIC => {
            UI_DrawMapCinematic(ctx, &rect, scale, color, false);
        }
        UI_STARTMAPCINEMATIC => {
            UI_DrawMapCinematic(ctx, &rect, scale, color, true);
        }
        UI_SKILL => {
            UI_DrawSkill(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        // Raven leaves `UI_DrawTotalForceStars` commented out.
        UI_TOTALFORCESTARS => {}
        UI_BLUETEAMNAME => {
            UI_DrawTeamName(ctx, &rect, scale, color, true, textStyle, iMenuFont);
        }
        UI_REDTEAMNAME => {
            UI_DrawTeamName(ctx, &rect, scale, color, false, textStyle, iMenuFont);
        }
        UI_BLUETEAM1 | UI_BLUETEAM2 | UI_BLUETEAM3 | UI_BLUETEAM4 | UI_BLUETEAM5 | UI_BLUETEAM6
        | UI_BLUETEAM7 | UI_BLUETEAM8 => {
            let iUse = if ownerDraw <= UI_BLUETEAM5 {
                ownerDraw - UI_BLUETEAM1 + 1
            } else {
                // unpleasant hack because I don't want to move up all the
                // UI_BLAHTEAM# defines
                ownerDraw - 274
            };
            UI_DrawTeamMember(ctx, &rect, scale, color, true, iUse, textStyle, iMenuFont);
        }
        UI_REDTEAM1 | UI_REDTEAM2 | UI_REDTEAM3 | UI_REDTEAM4 | UI_REDTEAM5 | UI_REDTEAM6
        | UI_REDTEAM7 | UI_REDTEAM8 => {
            let iUse = if ownerDraw <= UI_REDTEAM5 {
                ownerDraw - UI_REDTEAM1 + 1
            } else {
                ownerDraw - 277
            };
            UI_DrawTeamMember(ctx, &rect, scale, color, false, iUse, textStyle, iMenuFont);
        }
        UI_NETSOURCE => {
            UI_DrawNetSource(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_NETMAPPREVIEW => {
            UI_DrawNetMapPreview(ctx, &rect, scale, color);
        }
        UI_NETMAPCINEMATIC => {
            UI_DrawNetMapCinematic(ctx, &rect, scale, color);
        }
        UI_NETFILTER => {
            UI_DrawNetFilter(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_TIER => {
            UI_DrawTier(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        // PORT-NOTE (§20 dead surface, D7): `UI_DrawOpponent` is dead (Raven
        // leaves this call commented out).
        UI_OPPONENTMODEL => {}
        UI_TIERMAP1 => {
            UI_DrawTierMap(ctx, &rect, 0);
        }
        UI_TIERMAP2 => {
            UI_DrawTierMap(ctx, &rect, 1);
        }
        UI_TIERMAP3 => {
            UI_DrawTierMap(ctx, &rect, 2);
        }
        UI_PLAYERLOGO => {
            UI_DrawPlayerLogo(ctx, &rect, color);
        }
        UI_PLAYERLOGO_METAL => {
            UI_DrawPlayerLogoMetal(ctx, &rect, color);
        }
        UI_PLAYERLOGO_NAME => {
            UI_DrawPlayerLogoName(ctx, &rect, color);
        }
        UI_OPPONENTLOGO => {
            UI_DrawOpponentLogo(ctx, &rect, color);
        }
        UI_OPPONENTLOGO_METAL => {
            UI_DrawOpponentLogoMetal(ctx, &rect, color);
        }
        UI_OPPONENTLOGO_NAME => {
            UI_DrawOpponentLogoName(ctx, &rect, color);
        }
        UI_TIER_MAPNAME => {
            UI_DrawTierMapName(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_TIER_GAMETYPE => {
            UI_DrawTierGameType(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_ALLMAPS_SELECTION => {
            UI_DrawAllMapsSelection(ctx, &rect, scale, color, textStyle, true, iMenuFont);
        }
        UI_MAPS_SELECTION => {
            UI_DrawAllMapsSelection(ctx, &rect, scale, color, textStyle, false, iMenuFont);
        }
        UI_OPPONENT_NAME => {
            UI_DrawOpponentName(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_BOTNAME => {
            UI_DrawBotName(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_BOTSKILL => {
            UI_DrawBotSkill(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_REDBLUE => {
            UI_DrawRedBlue(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_CROSSHAIR => {
            UI_DrawCrosshair(ctx, &rect, scale, color);
        }
        UI_SELECTEDPLAYER => {
            UI_DrawSelectedPlayer(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_SERVERREFRESHDATE => {
            UI_DrawServerRefreshDate(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_SERVERMOTD => {
            UI_DrawServerMOTD(ctx, &rect, scale, color, iMenuFont);
        }
        UI_GLINFO => {
            UI_DrawGLInfo(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_KEYBINDSTATUS => {
            UI_DrawKeyBindStatus(ctx, &rect, scale, color, textStyle, iMenuFont);
        }
        UI_VERSION => {
            UI_Version(dc, &rect, scale, color, iMenuFont);
        }
        _ => {}
    }
}

/// Raven `UI_Chat_Main_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3949-3996`
pub fn UI_Chat_Main_HandleKey(
    world: &mut UiWorld,
    dc: &mut dyn DisplayContext,
    key: c_int,
) -> bool {
    let menu = match Menu_GetFocused(&world.menus) {
        Some(m) => m,
        None => return false,
    };

    let item = if key == fakeAscii_t::A_1 as c_int || key == fakeAscii_t::A_PLING as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "attack")
    } else if key == fakeAscii_t::A_2 as c_int || key == fakeAscii_t::A_AT as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "defend")
    } else if key == fakeAscii_t::A_3 as c_int || key == fakeAscii_t::A_HASH as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "request")
    } else if key == fakeAscii_t::A_4 as c_int || key == fakeAscii_t::A_STRING as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "reply")
    } else if key == fakeAscii_t::A_5 as c_int || key == fakeAscii_t::A_PERCENT as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "spot")
    } else if key == fakeAscii_t::A_6 as c_int || key == fakeAscii_t::A_CARET as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "tactics")
    } else {
        return false;
    };

    if let Some(item) = item {
        let action = world.menus.item(item).action.clone();
        Item_RunScript(&mut world.menus, dc, item, &action);
    }

    true
}

/// Raven `UI_Chat_Attack_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:3999-4034`
pub fn UI_Chat_Attack_HandleKey(
    world: &mut UiWorld,
    dc: &mut dyn DisplayContext,
    key: c_int,
) -> bool {
    let menu = match Menu_GetFocused(&world.menus) {
        Some(m) => m,
        None => return false,
    };

    let item = if key == fakeAscii_t::A_1 as c_int || key == fakeAscii_t::A_PLING as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "att_01")
    } else if key == fakeAscii_t::A_2 as c_int || key == fakeAscii_t::A_AT as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "att_02")
    } else if key == fakeAscii_t::A_3 as c_int || key == fakeAscii_t::A_HASH as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "att_03")
    } else {
        return false;
    };

    if let Some(item) = item {
        let action = world.menus.item(item).action.clone();
        Item_RunScript(&mut world.menus, dc, item, &action);
    }

    true
}

/// Raven `UI_Chat_Defend_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4037-4076`
pub fn UI_Chat_Defend_HandleKey(
    world: &mut UiWorld,
    dc: &mut dyn DisplayContext,
    key: c_int,
) -> bool {
    let menu = match Menu_GetFocused(&world.menus) {
        Some(m) => m,
        None => return false,
    };

    let item = if key == fakeAscii_t::A_1 as c_int || key == fakeAscii_t::A_PLING as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "def_01")
    } else if key == fakeAscii_t::A_2 as c_int || key == fakeAscii_t::A_AT as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "def_02")
    } else if key == fakeAscii_t::A_3 as c_int || key == fakeAscii_t::A_HASH as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "def_03")
    } else if key == fakeAscii_t::A_4 as c_int || key == fakeAscii_t::A_STRING as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "def_04")
    } else {
        return false;
    };

    if let Some(item) = item {
        let action = world.menus.item(item).action.clone();
        Item_RunScript(&mut world.menus, dc, item, &action);
    }

    true
}

/// Raven `UI_Chat_Request_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4079-4126`
pub fn UI_Chat_Request_HandleKey(
    world: &mut UiWorld,
    dc: &mut dyn DisplayContext,
    key: c_int,
) -> bool {
    let menu = match Menu_GetFocused(&world.menus) {
        Some(m) => m,
        None => return false,
    };

    let item = if key == fakeAscii_t::A_1 as c_int || key == fakeAscii_t::A_PLING as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "req_01")
    } else if key == fakeAscii_t::A_2 as c_int || key == fakeAscii_t::A_AT as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "req_02")
    } else if key == fakeAscii_t::A_3 as c_int || key == fakeAscii_t::A_HASH as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "req_03")
    } else if key == fakeAscii_t::A_4 as c_int || key == fakeAscii_t::A_STRING as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "req_04")
    } else if key == fakeAscii_t::A_5 as c_int || key == fakeAscii_t::A_PERCENT as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "req_05")
    } else if key == fakeAscii_t::A_6 as c_int || key == fakeAscii_t::A_CARET as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "req_06")
    } else {
        return false;
    };

    if let Some(item) = item {
        let action = world.menus.item(item).action.clone();
        Item_RunScript(&mut world.menus, dc, item, &action);
    }

    true
}

/// Raven `UI_Chat_Reply_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4129-4172`
pub fn UI_Chat_Reply_HandleKey(
    world: &mut UiWorld,
    dc: &mut dyn DisplayContext,
    key: c_int,
) -> bool {
    let menu = match Menu_GetFocused(&world.menus) {
        Some(m) => m,
        None => return false,
    };

    let item = if key == fakeAscii_t::A_1 as c_int || key == fakeAscii_t::A_PLING as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "rep_01")
    } else if key == fakeAscii_t::A_2 as c_int || key == fakeAscii_t::A_AT as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "rep_02")
    } else if key == fakeAscii_t::A_3 as c_int || key == fakeAscii_t::A_HASH as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "rep_03")
    } else if key == fakeAscii_t::A_4 as c_int || key == fakeAscii_t::A_STRING as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "rep_04")
    } else if key == fakeAscii_t::A_5 as c_int || key == fakeAscii_t::A_PERCENT as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "rep_05")
    } else {
        return false;
    };

    if let Some(item) = item {
        let action = world.menus.item(item).action.clone();
        Item_RunScript(&mut world.menus, dc, item, &action);
    }

    true
}

/// Raven `UI_Chat_Spot_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4175-4214`
pub fn UI_Chat_Spot_HandleKey(
    world: &mut UiWorld,
    dc: &mut dyn DisplayContext,
    key: c_int,
) -> bool {
    let menu = match Menu_GetFocused(&world.menus) {
        Some(m) => m,
        None => return false,
    };

    let item = if key == fakeAscii_t::A_1 as c_int || key == fakeAscii_t::A_PLING as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "spot_01")
    } else if key == fakeAscii_t::A_2 as c_int || key == fakeAscii_t::A_AT as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "spot_02")
    } else if key == fakeAscii_t::A_3 as c_int || key == fakeAscii_t::A_HASH as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "spot_03")
    } else if key == fakeAscii_t::A_4 as c_int || key == fakeAscii_t::A_STRING as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "spot_04")
    } else {
        return false;
    };

    if let Some(item) = item {
        let action = world.menus.item(item).action.clone();
        Item_RunScript(&mut world.menus, dc, item, &action);
    }

    true
}

/// Raven `UI_Chat_Tactical_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4217-4264`
pub fn UI_Chat_Tactical_HandleKey(
    world: &mut UiWorld,
    dc: &mut dyn DisplayContext,
    key: c_int,
) -> bool {
    let menu = match Menu_GetFocused(&world.menus) {
        Some(m) => m,
        None => return false,
    };

    let item = if key == fakeAscii_t::A_1 as c_int || key == fakeAscii_t::A_PLING as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "tac_01")
    } else if key == fakeAscii_t::A_2 as c_int || key == fakeAscii_t::A_AT as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "tac_02")
    } else if key == fakeAscii_t::A_3 as c_int || key == fakeAscii_t::A_HASH as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "tac_03")
    } else if key == fakeAscii_t::A_4 as c_int || key == fakeAscii_t::A_STRING as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "tac_04")
    } else if key == fakeAscii_t::A_5 as c_int || key == fakeAscii_t::A_PERCENT as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "tac_05")
    } else if key == fakeAscii_t::A_6 as c_int || key == fakeAscii_t::A_CARET as c_int {
        Menu_FindItemByName(&world.menus, Some(menu), "tac_06")
    } else {
        return false;
    };

    if let Some(item) = item {
        let action = world.menus.item(item).action.clone();
        Item_RunScript(&mut world.menus, dc, item, &action);
    }

    true
}

/// Raven `UI_NetSource_HandleKey`.
///
/// Source: `oracle/codemp/ui/ui_main.c:4526-4549`
pub fn UI_NetSource_HandleKey(
    ctx: &mut UiContext,
    dc: &mut dyn DisplayContext,
    _flags: c_int,
    _special: &mut f32,
    key: c_int,
) -> bool {
    if key == fakeAscii_t::A_MOUSE1 as c_int
        || key == fakeAscii_t::A_MOUSE2 as c_int
        || key == fakeAscii_t::A_ENTER as c_int
        || key == fakeAscii_t::A_KP_ENTER as c_int
    {
        if key == fakeAscii_t::A_MOUSE2 as c_int {
            ctx.world.cvars.ui_netSource.integer -= 1;
        } else {
            ctx.world.cvars.ui_netSource.integer += 1;
        }

        if ctx.world.cvars.ui_netSource.integer >= NUM_NET_SOURCES {
            ctx.world.cvars.ui_netSource.integer = 0;
        } else if ctx.world.cvars.ui_netSource.integer < 0 {
            ctx.world.cvars.ui_netSource.integer = NUM_NET_SOURCES - 1;
        }

        UI_BuildServerDisplayList(ctx, dc, 1);
        if ctx.world.cvars.ui_netSource.integer != AS_GLOBAL {
            UI_StartServerRefresh(ctx, true);
        }
        trap::Cvar_Set(
            ctx.engine,
            "ui_netSource",
            &format!("{}", ctx.world.cvars.ui_netSource.integer),
        );
        return true;
    }
    false
}

/// Raven `UI_FindCurrentSiegeTeamClass`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5720-5803`
pub fn UI_FindCurrentSiegeTeamClass(ctx: &mut UiContext, dc: &mut dyn DisplayContext) {
    let myTeam = trap::Cvar_VariableValue(ctx.engine, "ui_myteam") as c_int;

    let menu = match Menu_GetFocused(&ctx.world.menus) {
        Some(m) => m,
        None => return,
    };

    if myTeam != TEAM_RED && myTeam != TEAM_BLUE {
        return;
    }

    // If the player is on a team,
    if myTeam == TEAM_RED {
        if let Some(item) = Menu_FindItemByName(&ctx.world.menus, Some(menu), "onteam1") {
            let action = ctx.world.menus.item(item).action.clone();
            Item_RunScript(&mut ctx.world.menus, dc, item, &action);
        }
    } else if myTeam == TEAM_BLUE {
        if let Some(item) = Menu_FindItemByName(&ctx.world.menus, Some(menu), "onteam2") {
            let action = ctx.world.menus.item(item).action.clone();
            Item_RunScript(&mut ctx.world.menus, dc, item, &action);
        }
    }

    let baseClass = trap::Cvar_VariableValue(ctx.engine, "ui_siege_class") as c_int;

    // Find correct class button and activate it.
    let itemname = if baseClass == SPC_INFANTRY as c_int {
        "class1_button"
    } else if baseClass == SPC_HEAVY_WEAPONS as c_int {
        "class2_button"
    } else if baseClass == SPC_DEMOLITIONIST as c_int {
        "class3_button"
    } else if baseClass == SPC_VANGUARD as c_int {
        "class4_button"
    } else if baseClass == SPC_SUPPORT as c_int {
        "class5_button"
    } else if baseClass == SPC_JEDI as c_int {
        "class6_button"
    } else {
        return;
    };

    if let Some(item) = Menu_FindItemByName(&ctx.world.menus, Some(menu), itemname) {
        let action = ctx.world.menus.item(item).action.clone();
        Item_RunScript(&mut ctx.world.menus, dc, item, &action);
    }
}

/// Raven `UI_UpdateSiegeObjectiveGraphics`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5805-5847`
pub fn UI_UpdateSiegeObjectiveGraphics(ctx: &mut UiContext, dc: &mut dyn DisplayContext) {
    let menu = match Menu_GetFocused(&ctx.world.menus) {
        Some(m) => m,
        None => return,
    };

    // Hiding a bunch of fields because the opening section of the siege menu
    // was getting too long
    Menu_ShowGroup(&mut ctx.world.menus, dc, menu, "class_button", false);
    Menu_ShowGroup(&mut ctx.world.menus, dc, menu, "class_count", false);
    Menu_ShowGroup(&mut ctx.world.menus, dc, menu, "feeders", false);
    Menu_ShowGroup(&mut ctx.world.menus, dc, menu, "classdescription", false);
    Menu_ShowGroup(&mut ctx.world.menus, dc, menu, "minidesc", false);
    Menu_ShowGroup(&mut ctx.world.menus, dc, menu, "obj_longdesc", false);
    Menu_ShowGroup(&mut ctx.world.menus, dc, menu, "objective_pic", false);
    Menu_ShowGroup(&mut ctx.world.menus, dc, menu, "stats", false);
    Menu_ShowGroup(&mut ctx.world.menus, dc, menu, "forcepowerlevel", false);

    // Get objective icons for each team
    for teamI in 1..3 {
        for objI in 1..8 {
            Menu_SetItemBackground(
                &mut ctx.world.menus,
                dc,
                Some(menu),
                &format!("tm{}_icon{}", teamI, objI),
                &format!("*team{}_objective{}_mapicon", teamI, objI),
            );
            Menu_SetItemBackground(
                &mut ctx.world.menus,
                dc,
                Some(menu),
                &format!("tm{}_l_icon{}", teamI, objI),
                &format!("*team{}_objective{}_mapicon", teamI, objI),
            );
        }
    }

    // Now get their placement on the map
    for teamI in 1..3 {
        for objI in 1..8 {
            UI_SetSiegeObjectiveGraphicPos(
                ctx,
                menu,
                &format!("tm{}_icon{}", teamI, objI),
                &format!("team{}_objective{}_mappos", teamI, objI),
            );
        }
    }
}

/// Raven `UI_UpdateSaberHilt`.
///
/// Source: `oracle/codemp/ui/ui_main.c:5963-6018`
pub fn UI_UpdateSaberHilt(ctx: &mut UiContext, dc: &mut dyn DisplayContext, secondSaber: bool) {
    let menu = match Menu_GetFocused(&ctx.world.menus) {
        // Get current menu (either video or ingame video, I would assume)
        Some(m) => m,
        None => return,
    };

    let (itemName, saberCvarName) = if secondSaber {
        ("saber2", "ui_saber2")
    } else {
        ("saber", "ui_saber")
    };

    let item = match Menu_FindItemByName(&ctx.world.menus, Some(menu), itemName) {
        Some(i) => i,
        None => {
            let menuName = ctx
                .world
                .menus
                .menu(menu)
                .window
                .name
                .clone()
                .unwrap_or_default();
            Com_Error(
                ctx,
                &format!(
                    "UI_UpdateSaberHilt: Could not find item ({}) in menu ({})",
                    itemName, menuName
                ),
            );
            return;
        }
    };

    let model = trap::Cvar_VariableStringBuffer(ctx.engine, saberCvarName, MAX_QPATH);

    // §19 divergence: Raven's `item->text = model` aliases the stack buffer and
    // dangles after return (`oracle/codemp/ui/ui_main.c:6001`); the clone is the defined pick.
    ctx.world.menus.item_mut(item).text = Some(model.clone());
    // read this from the sabers.cfg
    if let Some(modelPath) = UI_SaberModelForSaber(ctx, &model) {
        // successfully found a model
        let mut animRunLength = 0;
        ItemParse_asset_model_go(
            &mut ctx.world.menus,
            dc,
            item,
            &modelPath,
            &mut animRunLength,
        ); // set the model
           // get the customSkin, if any
        if let Some(skinPath) = UI_SaberSkinForSaber(ctx, &model) {
            ItemParse_model_g2skin_go(&mut ctx.world.menus, dc, item, Some(&skinPath));
        // apply the skin
        } else {
            ItemParse_model_g2skin_go(&mut ctx.world.menus, dc, item, None); // apply the skin
        }
    }
}

/// Clamp `s` to `max` characters, the `Com_sprintf` truncation Raven's fixed
/// staging buffers impose (`hostname[1024]`, `clientBuff[32]`); the Latin-1
/// decode makes one source byte one `char`.
/// Source: `oracle/codemp/ui/ui_main.c:8783-8784`
fn feeder_buf_clamp(s: String, max: usize) -> String {
    if s.chars().count() > max {
        s.chars().take(max).collect()
    } else {
        s
    }
}

/// Raven `UI_FeederItemText`.
///
/// PORT-NOTE: Raven's `static char info[MAX_STRING_CHARS]` staging buffer
/// persists as `world.scratch.UI_FeederItemText_info`, so the FEEDER_SERVERS
/// `lastColumn`/`lastTime` guard reuses the previously fetched (possibly
/// stale) server info exactly as retail does.
///
/// Source: `oracle/codemp/ui/ui_main.c:8780-9171`
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
pub fn UI_FeederItemText(
    ctx: &mut UiContext,
    feederID: f32,
    index: c_int,
    column: c_int,
    handle1: &mut qhandle_t,
    handle2: &mut qhandle_t,
    handle3: &mut qhandle_t,
) -> String {
    let feeder = feederID as c_int;
    *handle1 = -1;
    *handle2 = -1;
    *handle3 = -1;

    if feeder == FEEDER_SABER_SINGLE_INFO {
        let name = ctx
            .world
            .main
            .saberSingleHiltInfo
            .get(index as usize)
            .cloned()
            .unwrap_or_default();
        return UI_SaberProperNameForSaber(ctx, &name).unwrap_or_default();
    } else if feeder == FEEDER_SABER_STAFF_INFO {
        let name = ctx
            .world
            .main
            .saberStaffHiltInfo
            .get(index as usize)
            .cloned()
            .unwrap_or_default();
        return UI_SaberProperNameForSaber(ctx, &name).unwrap_or_default();
    } else if feeder == FEEDER_Q3HEADS {
        let mut actual = 0;
        return UI_SelectedTeamHead(ctx.world, index, &mut actual);
    } else if feeder == FEEDER_SIEGE_TEAM1 {
        // nothing I guess, the description part can cover this
        return String::new();
    } else if feeder == FEEDER_SIEGE_TEAM2 {
        // nothing I guess, the description part can cover this
        return String::new();
    } else if feeder == FEEDER_FORCECFG {
        if index >= 0 && index < ctx.world.forceConfigNames.len() as c_int {
            if index == 0 {
                // always show "custom"
                return ctx.world.forceConfigNames[index as usize].clone();
            } else if ctx.world.force.uiForceSide == FORCE_LIGHTSIDE {
                let idx = index + ctx.world.forceConfigLightIndexBegin;
                if idx < 0 || idx >= ctx.world.forceConfigNames.len() as c_int {
                    return String::new();
                }
                return ctx.world.forceConfigNames[idx as usize].clone();
            } else if ctx.world.force.uiForceSide == FORCE_DARKSIDE {
                let idx = index + ctx.world.forceConfigDarkIndexBegin;
                if idx < 0 {
                    return String::new();
                }
                if idx > ctx.world.forceConfigLightIndexBegin {
                    // dark gets read in before light
                    return String::new();
                }
                if idx >= ctx.world.forceConfigNames.len() as c_int {
                    return String::new();
                }
                return ctx.world.forceConfigNames[idx as usize].clone();
            } else {
                return String::new();
            }
        }
    } else if feeder == FEEDER_MAPS || feeder == FEEDER_ALLMAPS {
        let mut actual = 0;
        return UI_SelectedMap(ctx.world, index, &mut actual);
    } else if feeder == FEEDER_SERVERS {
        if index >= 0 && index < ctx.world.serverStatus.displayServers.len() as c_int {
            let serverNum = ctx.world.serverStatus.displayServers[index as usize];
            if ctx.world.scratch.UI_FeederItemText_lastColumn != column
                || ctx.world.scratch.UI_FeederItemText_lastTime > ctx.world.uiDC.realTime + 5000
            {
                ctx.world.scratch.UI_FeederItemText_info = trap::LAN_GetServerInfo(
                    ctx.engine,
                    ctx.world.cvars.ui_netSource.integer,
                    serverNum,
                    MAX_STRING_CHARS,
                );
                ctx.world.scratch.UI_FeederItemText_lastColumn = column;
                ctx.world.scratch.UI_FeederItemText_lastTime = ctx.world.uiDC.realTime;
            }
            let info = ctx.world.scratch.UI_FeederItemText_info.clone();

            let ping = atoi(&Info_ValueForKey(&info, "ping"));
            // if ping == -1: Raven's "do a server refresh" branch is
            // commented out — no-op.

            match column {
                SORT_HOST => {
                    if ping <= 0 {
                        return Info_ValueForKey(&info, "addr");
                    } else {
                        if atoi(&Info_ValueForKey(&info, "needpass")) != 0 {
                            *handle3 = ctx.world.uiDC.Assets.needPass;
                        }
                        let gametype = atoi(&Info_ValueForKey(&info, "gametype"));
                        if gametype != GT_JEDIMASTER {
                            let mut saberOnly = true;
                            let mut allForceDisabled = false;

                            let restrictedForce = atoi(&Info_ValueForKey(&info, "fdisable"));
                            if UI_AllForceDisabled(restrictedForce) {
                                // all force powers are disabled
                                allForceDisabled = true;
                                *handle2 = ctx.world.uiDC.Assets.noForce;
                            } else if restrictedForce != 0 {
                                // at least one force power is disabled
                                *handle2 = ctx.world.uiDC.Assets.forceRestrict;
                            }

                            let wDisable = atoi(&Info_ValueForKey(&info, "wdisable"));
                            let mut i = 0;
                            while i < WP_NUM_WEAPONS {
                                if (wDisable & (1 << i)) == 0 && i != WP_SABER && i != WP_NONE {
                                    saberOnly = false;
                                }
                                i += 1;
                            }
                            if saberOnly {
                                *handle1 = ctx.world.uiDC.Assets.saberOnly;
                            } else if atoi(&Info_ValueForKey(&info, "truejedi")) != 0
                                && gametype != GT_HOLOCRON
                                && gametype != GT_JEDIMASTER
                                && !saberOnly
                                && !allForceDisabled
                            {
                                // truejedi is on and allowed in this mode
                                *handle1 = ctx.world.uiDC.Assets.trueJedi;
                            }
                        }
                        if ctx.world.cvars.ui_netSource.integer == AS_LOCAL {
                            let nettype = atoi(&Info_ValueForKey(&info, "nettype"));
                            // §19 divergence: Raven's `netnames[atoi(...)]` is an
                            // unchecked read (OOB on a hostile `nettype`); the bounds-checked
                            // fetch with an empty fallback is the defined pick.
                            let netname =
                                NETNAMES.get(nettype as usize).copied().unwrap_or_default();
                            return feeder_buf_clamp(
                                format!("{} [{}]", Info_ValueForKey(&info, "hostname"), netname),
                                1023,
                            );
                        } else if atoi(&Info_ValueForKey(&info, "sv_allowAnonymous")) != 0 {
                            // anonymous server
                            return feeder_buf_clamp(
                                format!("(A) {}", Info_ValueForKey(&info, "hostname")),
                                1023,
                            );
                        } else {
                            return feeder_buf_clamp(Info_ValueForKey(&info, "hostname"), 1023);
                        }
                    }
                }
                SORT_MAP => return Info_ValueForKey(&info, "mapname"),
                SORT_CLIENTS => {
                    return feeder_buf_clamp(
                        format!(
                            "{} ({})",
                            Info_ValueForKey(&info, "clients"),
                            Info_ValueForKey(&info, "sv_maxclients")
                        ),
                        31,
                    );
                }
                SORT_GAME => {
                    let game = atoi(&Info_ValueForKey(&info, "gametype"));
                    // Raven writes "Inactive" then immediately overwrites it
                    // with "Unknown" on the out-of-range path — the final
                    // value is always "Unknown".
                    return if game >= 0 && game < NUM_TEAM_ARENA_GAME_TYPES {
                        TEAM_ARENA_GAME_TYPES[game as usize].to_string()
                    } else {
                        "Unknown".to_string()
                    };
                }
                SORT_PING => {
                    return if ping <= 0 {
                        "...".to_string()
                    } else {
                        Info_ValueForKey(&info, "ping")
                    };
                }
                _ => {}
            }
        }
    } else if feeder == FEEDER_SERVERSTATUS {
        if index >= 0 && (index as usize) < ctx.world.serverStatusInfo.lines.len() {
            if column >= 0 && column < 4 {
                return ctx.world.serverStatusInfo.lines[index as usize][column as usize].clone();
            }
        }
    } else if feeder == FEEDER_FINDPLAYER {
        if index >= 0 && (index as usize) < ctx.world.foundPlayerServerNames.len() {
            return ctx.world.foundPlayerServerNames[index as usize].clone();
        }
    } else if feeder == FEEDER_PLAYER_LIST {
        if index >= 0 && (index as usize) < ctx.world.playerNames.len() {
            return ctx.world.playerNames[index as usize].clone();
        }
    } else if feeder == FEEDER_TEAM_LIST {
        if index >= 0 && (index as usize) < ctx.world.teamNames.len() {
            return ctx.world.teamNames[index as usize].clone();
        }
    } else if feeder == FEEDER_MODS {
        if index >= 0 && (index as usize) < ctx.world.modList.len() {
            let m = &ctx.world.modList[index as usize];
            if !m.modDescr.is_empty() {
                return m.modDescr.clone();
            } else {
                return m.modName.clone();
            }
        }
    } else if feeder == FEEDER_CINEMATICS {
        if index >= 0 && (index as usize) < ctx.world.movieList.len() {
            return ctx.world.movieList[index as usize].clone();
        }
    } else if feeder == FEEDER_DEMOS {
        if index >= 0 && (index as usize) < ctx.world.demoList.len() {
            return ctx.world.demoList[index as usize].clone();
        }
    } else if feeder == FEEDER_MOVES {
        return DATAPAD_MOVE_DATA[ctx.world.movesTitleIndex as usize][index as usize]
            .title
            .map(|s| s.to_string())
            .unwrap_or_default();
    } else if feeder == FEEDER_MOVES_TITLES {
        return DATAPAD_MOVE_TITLE_DATA[index as usize].to_string();
    } else if feeder == FEEDER_PLAYER_SPECIES {
        return ctx.world.playerSpecies[index as usize].Name.clone();
    } else if feeder == FEEDER_LANGUAGES {
        return String::new();
    } else if feeder == FEEDER_COLORCHOICES {
        let speciesIdx = ctx.world.playerSpeciesIndex as usize;
        if index >= 0 && (index as usize) < ctx.world.playerSpecies[speciesIdx].ColorShader.len() {
            let shader = ctx.world.playerSpecies[speciesIdx].ColorShader[index as usize].clone();
            *handle1 = trap::R_RegisterShaderNoMip(ctx.engine, &shader);
            return shader;
        }
    } else if feeder == FEEDER_PLAYER_SKIN_HEAD {
        let speciesIdx = ctx.world.playerSpeciesIndex as usize;
        if index >= 0 && (index as usize) < ctx.world.playerSpecies[speciesIdx].SkinHeadNames.len()
        {
            let name = ctx.world.playerSpecies[speciesIdx].Name.clone();
            let skin = ctx.world.playerSpecies[speciesIdx].SkinHeadNames[index as usize].clone();
            *handle1 = trap::R_RegisterShaderNoMip(
                ctx.engine,
                &format!("models/players/{}/icon_{}", name, skin),
            );
            return skin;
        }
    } else if feeder == FEEDER_PLAYER_SKIN_TORSO {
        let speciesIdx = ctx.world.playerSpeciesIndex as usize;
        if index >= 0 && (index as usize) < ctx.world.playerSpecies[speciesIdx].SkinTorsoNames.len()
        {
            let name = ctx.world.playerSpecies[speciesIdx].Name.clone();
            let skin = ctx.world.playerSpecies[speciesIdx].SkinTorsoNames[index as usize].clone();
            *handle1 = trap::R_RegisterShaderNoMip(
                ctx.engine,
                &format!("models/players/{}/icon_{}", name, skin),
            );
            return skin;
        }
    } else if feeder == FEEDER_PLAYER_SKIN_LEGS {
        let speciesIdx = ctx.world.playerSpeciesIndex as usize;
        if index >= 0 && (index as usize) < ctx.world.playerSpecies[speciesIdx].SkinLegNames.len() {
            let name = ctx.world.playerSpecies[speciesIdx].Name.clone();
            let skin = ctx.world.playerSpecies[speciesIdx].SkinLegNames[index as usize].clone();
            *handle1 = trap::R_RegisterShaderNoMip(
                ctx.engine,
                &format!("models/players/{}/icon_{}", name, skin),
            );
            return skin;
        }
    } else if feeder == FEEDER_SIEGE_BASE_CLASS {
        return String::new();
    } else if feeder == FEEDER_SIEGE_CLASS_WEAPONS {
        return String::new();
    }
    // PORT-NOTE: the `#ifdef _XBOX` feeders (`FEEDER_XBL_*`) are non-retail
    // MP surface — dropped, not compiled in the retail build.

    String::new()
}

/// Raven `UI_SiegeSetCvarsForClass`.
///
/// Raven: called every time a class is selected from a feeder, sets info for
/// shaders to be displayed in the menu about the class -rww
///
/// PORT-NOTE: Raven passed the `siegeClass_t *` itself; the port threads its
/// index into `world.bg_state.bgSiegeClasses` (§B5) so no reference into
/// `bg_state` aliases `ctx` across a `&mut ctx` use.
///
/// Source: `oracle/codemp/ui/ui_main.c:9441-9592`
#[allow(clippy::too_many_lines)]
pub fn UI_SiegeSetCvarsForClass(ctx: &mut UiContext, sclIndex: Option<usize>) {
    // let's clear the things out first
    for i in 0..WP_NUM_WEAPONS {
        trap::Cvar_Set(
            ctx.engine,
            &format!("ui_class_weapon{}", i),
            "gfx/2d/select",
        );
    }
    // now for inventory items
    for i in 0..HI_NUM_HOLDABLE {
        trap::Cvar_Set(ctx.engine, &format!("ui_class_item{}", i), "gfx/2d/select");
    }
    // now for force powers
    for i in 0..NUM_FORCE_POWERS {
        trap::Cvar_Set(ctx.engine, &format!("ui_class_power{}", i), "gfx/2d/select");
    }

    // now health and armor
    trap::Cvar_Set(ctx.engine, "ui_class_health", "0");
    trap::Cvar_Set(ctx.engine, "ui_class_armor", "0");

    trap::Cvar_Set(ctx.engine, "ui_class_icon", "");

    let sclIndex = match sclIndex {
        // no select?
        Some(sclIndex) => sclIndex,
        None => return,
    };

    // The class's fields are read out up front; the borrow of `bg_state` ends
    // here, so nothing below aliases it.
    let scl = &ctx.world.bg_state.bgSiegeClasses[sclIndex];
    let weapons = scl.weapons;
    let saber1 = scl.saber1.clone();
    let saber2 = scl.saber2.clone();
    let invenItems = scl.invenItems;
    let forcePowerLevels = scl.forcePowerLevels;
    let maxhealth = scl.maxhealth;
    let maxarmor = scl.maxarmor;
    let speed = scl.speed;
    let classShader = scl.classShader;

    // set cvars for which weaps we have
    let mut i = 0;
    let mut count: c_int = 0;
    trap::Cvar_Set(ctx.engine, &format!("ui_class_weapondesc{}", count), " "); // Blank it out to start with
    while i < WP_NUM_WEAPONS {
        if weapons & (1 << i) != 0 {
            if i == WP_SABER {
                // we want to see what kind of saber they have, and set the
                // cvar based on that
                let saberType = if !saber1.is_empty() && !saber2.is_empty() {
                    "gfx/hud/w_icon_duallightsaber".to_string()
                    // fixme: need saber data access on ui to determine if
                    // staff, "gfx/hud/w_icon_saberstaff"
                } else if !saber1.is_empty() {
                    match UI_SaberTypeForSaber(ctx, &saber1) {
                        Some(buf) if Q_stricmp(&buf, "SABER_STAFF") == 0 => {
                            "gfx/hud/w_icon_saberstaff".to_string()
                        }
                        _ => "gfx/hud/w_icon_lightsaber".to_string(),
                    }
                } else {
                    "gfx/hud/w_icon_lightsaber".to_string()
                };

                trap::Cvar_Set(ctx.engine, &format!("ui_class_weapon{}", count), &saberType);
                trap::Cvar_Set(
                    ctx.engine,
                    &format!("ui_class_weapondesc{}", count),
                    "@MENUS_AN_ELEGANT_WEAPON_FOR",
                );
                count += 1;
                trap::Cvar_Set(ctx.engine, &format!("ui_class_weapondesc{}", count), " ");
            // Blank it out to start with
            } else {
                let item = BG_FindItemForWeapon(i);
                trap::Cvar_Set(
                    ctx.engine,
                    &format!("ui_class_weapon{}", count),
                    item.item().icon.unwrap_or(""),
                );
                trap::Cvar_Set(
                    ctx.engine,
                    &format!("ui_class_weapondesc{}", count),
                    item.item().description.unwrap_or(""),
                );
                count += 1;
                trap::Cvar_Set(ctx.engine, &format!("ui_class_weapondesc{}", count), " ");
                // Blank it out to start with
            }
        }

        i += 1;
    }

    // now for inventory items
    let mut i = 0;
    let mut count: c_int = 0;

    while i < HI_NUM_HOLDABLE {
        if invenItems & (1 << i) != 0 {
            let item = BG_FindItemForHoldable(i);
            trap::Cvar_Set(
                ctx.engine,
                &format!("ui_class_item{}", count),
                item.item().icon.unwrap_or(""),
            );
            trap::Cvar_Set(
                ctx.engine,
                &format!("ui_class_itemdesc{}", count),
                item.item().description.unwrap_or(""),
            );
            count += 1;
        } else {
            trap::Cvar_Set(ctx.engine, &format!("ui_class_itemdesc{}", count), " ");
        }
        i += 1;
    }

    // now for force powers
    let mut i = 0;
    let mut count: c_int = 0;

    while i < NUM_FORCE_POWERS {
        trap::Cvar_Set(ctx.engine, &format!("ui_class_powerlevel{}", i), "0"); // Zero this out to start.
        if i < 9 {
            trap::Cvar_Set(ctx.engine, &format!("ui_class_powerlevelslot{}", i), "0");
            // Zero this out to start.
        }

        if forcePowerLevels[i as usize] != 0 {
            trap::Cvar_Set(
                ctx.engine,
                &format!("ui_class_powerlevel{}", count),
                &format!("{}", forcePowerLevels[i as usize]),
            );
            trap::Cvar_Set(
                ctx.engine,
                &format!("ui_class_power{}", count),
                HOLOCRON_ICONS[i as usize],
            );
            count += 1;
        }

        i += 1;
    }

    // now health and armor
    trap::Cvar_Set(ctx.engine, "ui_class_health", &format!("{}", maxhealth));
    trap::Cvar_Set(ctx.engine, "ui_class_armor", &format!("{}", maxarmor));
    trap::Cvar_Set(ctx.engine, "ui_class_speed", &format!("{:.2}", speed));

    // now get the icon path based on the shader index
    let shader = if classShader != 0 {
        trap::R_ShaderNameFromIndex(ctx.engine, classShader, MAX_QPATH)
    } else {
        // no shader
        String::new()
    };
    trap::Cvar_Set(ctx.engine, "ui_class_icon", &shader);
}

/// Raven `UI_ParseGameInfo`.
///
/// Source: `oracle/codemp/ui/ui_main.c:10122-10169`
pub fn UI_ParseGameInfo(ctx: &mut UiContext, teamFile: &str) {
    let buff = GetMenuBuffer(ctx, teamFile);
    if buff.is_empty() {
        return;
    }

    let mut p: &str = &buff;

    loop {
        let (token, rest) = COM_Parse(p, true);
        p = rest;
        if token.is_empty() || token.starts_with('}') {
            break;
        }

        if Q_stricmp(&token, "}") == 0 {
            break;
        }

        if Q_stricmp(&token, "gametypes") == 0 {
            if GameType_Parse(ctx, &mut p, false) {
                continue;
            } else {
                break;
            }
        }

        if Q_stricmp(&token, "joingametypes") == 0 {
            if GameType_Parse(ctx, &mut p, true) {
                continue;
            } else {
                break;
            }
        }

        if Q_stricmp(&token, "maps") == 0 {
            // start a new menu
            MapList_Parse(ctx, &mut p);
        }
    }
}

/// Raven `UI_UpdateCvarsForClass`.
///
/// PORT-NOTE: `BG_GetClassOnBaseClass` returns a raw `*mut siegeClass_t` into
/// `world.bg_state.bgSiegeClasses` (DEC-36 addendum 11); the deref is confined
/// to computing the class index, which is what gets threaded onward — no
/// reference into `bg_state` stays live across a `&mut ctx` use.
///
/// Source: `oracle/codemp/ui/ui_main.c:9596-9639`
pub fn UI_UpdateCvarsForClass(
    ctx: &mut UiContext,
    dc: &mut dyn DisplayContext,
    team: c_int,
    baseClass: c_int,
    index: c_int,
) {
    // Is it a valid team
    if team == SIEGETEAM_TEAM1 || team == SIEGETEAM_TEAM2 {
        // Is it a valid base class?
        if baseClass >= SPC_INFANTRY as c_int && baseClass < SPC_MAX as c_int {
            // A valid index?
            if index >= 0
                && index < BG_SiegeCountBaseClass(team, baseClass as c_short, &ctx.world.bg_state)
            {
                if ctx.world.main.g_siegedFeederForcedSet == 0 {
                    let holdClass = BG_GetClassOnBaseClass(
                        team,
                        baseClass as c_short,
                        index as c_short,
                        &ctx.world.bg_state,
                    );
                    if !holdClass.is_null() {
                        // clicked a valid item
                        let classNum = {
                            let scl = unsafe { &*holdClass };
                            UI_SiegeClassNum(&ctx.world.bg_state, scl)
                        };
                        ctx.world.main.g_UIGloballySelectedSiegeClass = classNum;
                        // §19: Raven indexes the fixed `g_UIClassDescriptions` array
                        // unchecked; the `Vec` index can't be out of range because
                        // `UI_SiegeClassNum` returns 0 on a miss (panic unreachable).
                        let siegeClassNum = classNum as usize;
                        let desc = ctx.world.main.g_UIClassDescriptions[siegeClassNum].clone();
                        trap::Cvar_Set(ctx.engine, "ui_classDesc", &desc);
                        ctx.world.main.g_siegedFeederForcedSet = 1;
                        Menu_SetFeederSelection(
                            &mut ctx.world.menus,
                            dc,
                            None,
                            FEEDER_SIEGE_BASE_CLASS,
                            -1,
                            None,
                        );
                        UI_SiegeSetCvarsForClass(ctx, Some(siegeClassNum));

                        let holdBuf = BG_GetUIPortraitFile(
                            team,
                            baseClass as c_short,
                            index as c_short,
                            &ctx.world.bg_state,
                        );
                        if let Some(holdBuf) = holdBuf {
                            trap::Cvar_Set(ctx.engine, "ui_classPortrait", &holdBuf);
                        }
                    }
                }
                ctx.world.main.g_siegedFeederForcedSet = 0;
            } else {
                trap::Cvar_Set(ctx.engine, "ui_classDesc", " ");
            }
        }
    }
}

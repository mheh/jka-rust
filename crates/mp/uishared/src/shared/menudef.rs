//! Raven `menudef.h` — the `.menu`-script `#define` vocabulary (item/window
//! type & style tags, ownerDraw codes, feeder ids, voice-chat command
//! strings).
//!
//! Source: `oracle/ui/menudef.h`

use core::ffi::c_int;

/// Raven `#define ITEM_TYPE_TEXT 0` — simple text.
/// Source: `oracle/ui/menudef.h:9`
pub const ITEM_TYPE_TEXT: c_int = 0;
/// Raven `#define ITEM_TYPE_BUTTON 1` — button, basically text with a border.
/// Source: `oracle/ui/menudef.h:10`
pub const ITEM_TYPE_BUTTON: c_int = 1;
/// Raven `#define ITEM_TYPE_RADIOBUTTON 2` — toggle button, may be grouped.
/// Source: `oracle/ui/menudef.h:11`
pub const ITEM_TYPE_RADIOBUTTON: c_int = 2;
/// Raven `#define ITEM_TYPE_CHECKBOX 3` — check box.
/// Source: `oracle/ui/menudef.h:12`
pub const ITEM_TYPE_CHECKBOX: c_int = 3;
/// Raven `#define ITEM_TYPE_EDITFIELD 4` — editable text, associated with a cvar.
/// Source: `oracle/ui/menudef.h:13`
pub const ITEM_TYPE_EDITFIELD: c_int = 4;
/// Raven `#define ITEM_TYPE_COMBO 5` — drop down list.
/// Source: `oracle/ui/menudef.h:14`
pub const ITEM_TYPE_COMBO: c_int = 5;
/// Raven `#define ITEM_TYPE_LISTBOX 6` — scrollable list.
/// Source: `oracle/ui/menudef.h:15`
pub const ITEM_TYPE_LISTBOX: c_int = 6;
/// Raven `#define ITEM_TYPE_MODEL 7` — model.
/// Source: `oracle/ui/menudef.h:16`
pub const ITEM_TYPE_MODEL: c_int = 7;
/// Raven `#define ITEM_TYPE_OWNERDRAW 8` — owner draw, name specs what it is.
/// Source: `oracle/ui/menudef.h:17`
pub const ITEM_TYPE_OWNERDRAW: c_int = 8;
/// Raven `#define ITEM_TYPE_NUMERICFIELD 9` — editable text, associated with a cvar.
/// Source: `oracle/ui/menudef.h:18`
pub const ITEM_TYPE_NUMERICFIELD: c_int = 9;
/// Raven `#define ITEM_TYPE_SLIDER 10` — mouse speed, volume, etc.
/// Source: `oracle/ui/menudef.h:19`
pub const ITEM_TYPE_SLIDER: c_int = 10;
/// Raven `#define ITEM_TYPE_YESNO 11` — yes no cvar setting.
/// Source: `oracle/ui/menudef.h:20`
pub const ITEM_TYPE_YESNO: c_int = 11;
/// Raven `#define ITEM_TYPE_MULTI 12` — multiple list setting, enumerated.
/// Source: `oracle/ui/menudef.h:21`
pub const ITEM_TYPE_MULTI: c_int = 12;
/// Raven `#define ITEM_TYPE_BIND 13` — multiple list setting, enumerated.
/// Source: `oracle/ui/menudef.h:22`
pub const ITEM_TYPE_BIND: c_int = 13;
/// Raven `#define ITEM_TYPE_TEXTSCROLL 14` — scrolls text.
/// Source: `oracle/ui/menudef.h:23`
pub const ITEM_TYPE_TEXTSCROLL: c_int = 14;

/// Raven `#define ITEM_ALIGN_LEFT 0` — left alignment.
/// Source: `oracle/ui/menudef.h:25`
pub const ITEM_ALIGN_LEFT: c_int = 0;
/// Raven `#define ITEM_ALIGN_CENTER 1` — center alignment.
/// Source: `oracle/ui/menudef.h:26`
pub const ITEM_ALIGN_CENTER: c_int = 1;
/// Raven `#define ITEM_ALIGN_RIGHT 2` — right alignment.
/// Source: `oracle/ui/menudef.h:27`
pub const ITEM_ALIGN_RIGHT: c_int = 2;

/// Raven `#define ITEM_TEXTSTYLE_NORMAL 0` — normal text.
/// Source: `oracle/ui/menudef.h:29`
pub const ITEM_TEXTSTYLE_NORMAL: c_int = 0;
/// Raven `#define ITEM_TEXTSTYLE_BLINK 1` — fast blinking.
/// Source: `oracle/ui/menudef.h:30`
pub const ITEM_TEXTSTYLE_BLINK: c_int = 1;
/// Raven `#define ITEM_TEXTSTYLE_PULSE 2` — slow pulsing.
/// Source: `oracle/ui/menudef.h:31`
pub const ITEM_TEXTSTYLE_PULSE: c_int = 2;
/// Raven `#define ITEM_TEXTSTYLE_SHADOWED 3` — drop shadow (need a color for this).
/// Source: `oracle/ui/menudef.h:32`
pub const ITEM_TEXTSTYLE_SHADOWED: c_int = 3;
/// Raven `#define ITEM_TEXTSTYLE_OUTLINED 4` — drop shadow (need a color for this).
/// Source: `oracle/ui/menudef.h:33`
pub const ITEM_TEXTSTYLE_OUTLINED: c_int = 4;
/// Raven `#define ITEM_TEXTSTYLE_OUTLINESHADOWED 5` — drop shadow (need a color for this).
/// Source: `oracle/ui/menudef.h:34`
pub const ITEM_TEXTSTYLE_OUTLINESHADOWED: c_int = 5;
/// Raven `#define ITEM_TEXTSTYLE_SHADOWEDMORE 6` — drop shadow (need a color for this).
/// Source: `oracle/ui/menudef.h:35`
pub const ITEM_TEXTSTYLE_SHADOWEDMORE: c_int = 6;

/// Raven `#define WINDOW_BORDER_NONE 0` — no border.
/// Source: `oracle/ui/menudef.h:37`
pub const WINDOW_BORDER_NONE: c_int = 0;
/// Raven `#define WINDOW_BORDER_FULL 1` — full border based on border color (single pixel).
/// Source: `oracle/ui/menudef.h:38`
pub const WINDOW_BORDER_FULL: c_int = 1;
/// Raven `#define WINDOW_BORDER_HORZ 2` — horizontal borders only.
/// Source: `oracle/ui/menudef.h:39`
pub const WINDOW_BORDER_HORZ: c_int = 2;
/// Raven `#define WINDOW_BORDER_VERT 3` — vertical borders only.
/// Source: `oracle/ui/menudef.h:40`
pub const WINDOW_BORDER_VERT: c_int = 3;
/// Raven `#define WINDOW_BORDER_KCGRADIENT 4` — horizontal border using the gradient bars.
/// Source: `oracle/ui/menudef.h:41`
pub const WINDOW_BORDER_KCGRADIENT: c_int = 4;

/// Raven `#define WINDOW_STYLE_EMPTY 0` — no background.
/// Source: `oracle/ui/menudef.h:43`
pub const WINDOW_STYLE_EMPTY: c_int = 0;
/// Raven `#define WINDOW_STYLE_FILLED 1` — filled with background color.
/// Source: `oracle/ui/menudef.h:44`
pub const WINDOW_STYLE_FILLED: c_int = 1;
/// Raven `#define WINDOW_STYLE_GRADIENT 2` — gradient bar based on background color.
/// Source: `oracle/ui/menudef.h:45`
pub const WINDOW_STYLE_GRADIENT: c_int = 2;
/// Raven `#define WINDOW_STYLE_SHADER 3` — gradient bar based on background color.
/// Source: `oracle/ui/menudef.h:46`
pub const WINDOW_STYLE_SHADER: c_int = 3;
/// Raven `#define WINDOW_STYLE_TEAMCOLOR 4` — team color.
/// Source: `oracle/ui/menudef.h:47`
pub const WINDOW_STYLE_TEAMCOLOR: c_int = 4;
/// Raven `#define WINDOW_STYLE_CINEMATIC 5` — cinematic.
/// Source: `oracle/ui/menudef.h:48`
pub const WINDOW_STYLE_CINEMATIC: c_int = 5;

/// Raven `#define MENU_TRUE 1` — uh.. true.
/// Source: `oracle/ui/menudef.h:50`
pub const MENU_TRUE: c_int = 1;
/// Raven `#define MENU_FALSE 0` — and false.
/// Source: `oracle/ui/menudef.h:51`
pub const MENU_FALSE: c_int = 0;

/// Raven `#define HUD_VERTICAL 0x00`.
/// Source: `oracle/ui/menudef.h:53`
pub const HUD_VERTICAL: c_int = 0x00;
/// Raven `#define HUD_HORIZONTAL 0x01`.
/// Source: `oracle/ui/menudef.h:54`
pub const HUD_HORIZONTAL: c_int = 0x01;

/// Raven `#define LISTBOX_TEXT 0x00` — list box element types.
/// Source: `oracle/ui/menudef.h:57`
pub const LISTBOX_TEXT: c_int = 0x00;
/// Raven `#define LISTBOX_IMAGE 0x01`.
/// Source: `oracle/ui/menudef.h:58`
pub const LISTBOX_IMAGE: c_int = 0x01;

/// Raven list-feeder ids (`FEEDER_*`) — `.menu` script feeder selectors.
/// `FEEDER_HEADS` (0x00) and `FEEDER_CLANS` (0x03) are commented out in the
/// oracle header and are not ported.
///
/// Source: `oracle/ui/menudef.h:62-118`
pub const FEEDER_MAPS: c_int = 0x01;
pub const FEEDER_SERVERS: c_int = 0x02;
pub const FEEDER_ALLMAPS: c_int = 0x04;
pub const FEEDER_REDTEAM_LIST: c_int = 0x05;
pub const FEEDER_BLUETEAM_LIST: c_int = 0x06;
pub const FEEDER_PLAYER_LIST: c_int = 0x07;
pub const FEEDER_TEAM_LIST: c_int = 0x08;
pub const FEEDER_MODS: c_int = 0x09;
pub const FEEDER_DEMOS: c_int = 0x0a;
pub const FEEDER_SCOREBOARD: c_int = 0x0b;
pub const FEEDER_Q3HEADS: c_int = 0x0c;
pub const FEEDER_SERVERSTATUS: c_int = 0x0d;
pub const FEEDER_FINDPLAYER: c_int = 0x0e;
pub const FEEDER_CINEMATICS: c_int = 0x0f;
pub const FEEDER_FORCECFG: c_int = 0x10;
pub const FEEDER_SIEGE_TEAM1: c_int = 0x11;
pub const FEEDER_SIEGE_TEAM2: c_int = 0x12;
pub const FEEDER_PLAYER_SPECIES: c_int = 0x13;
pub const FEEDER_PLAYER_SKIN_HEAD: c_int = 0x14;
pub const FEEDER_PLAYER_SKIN_TORSO: c_int = 0x15;
pub const FEEDER_PLAYER_SKIN_LEGS: c_int = 0x16;
pub const FEEDER_COLORCHOICES: c_int = 0x17;
pub const FEEDER_TEAM1_INFANTRY: c_int = 0x18;
pub const FEEDER_TEAM1_VANGUARD: c_int = 0x19;
pub const FEEDER_TEAM1_SUPPORT: c_int = 0x1a;
pub const FEEDER_TEAM1_JEDI: c_int = 0x1b;
pub const FEEDER_TEAM1_DEMO: c_int = 0x1c;
pub const FEEDER_TEAM1_HEAVY: c_int = 0x1d;
pub const FEEDER_TEAM2_INFANTRY: c_int = 0x1e;
pub const FEEDER_TEAM2_VANGUARD: c_int = 0x1f;
pub const FEEDER_TEAM2_SUPPORT: c_int = 0x20;
pub const FEEDER_TEAM2_JEDI: c_int = 0x21;
pub const FEEDER_TEAM2_DEMO: c_int = 0x22;
pub const FEEDER_TEAM2_HEAVY: c_int = 0x23;
pub const FEEDER_SIEGE_BASE_CLASS: c_int = 0x24;
pub const FEEDER_SIEGE_CLASS_WEAPONS: c_int = 0x25;
pub const FEEDER_SIEGE_CLASS_INVENTORY: c_int = 0x26;
pub const FEEDER_SIEGE_CLASS_FORCE: c_int = 0x27;
pub const FEEDER_LANGUAGES: c_int = 0x28;
pub const FEEDER_MOVES: c_int = 0x29;
pub const FEEDER_MOVES_TITLES: c_int = 0x2a;
pub const FEEDER_SABER_SINGLE_INFO: c_int = 0x2b;
pub const FEEDER_SABER_STAFF_INFO: c_int = 0x2c;
/// Xbox specific, hope no one minds.
pub const FEEDER_XBL_ACCOUNTS: c_int = 0xA0;
pub const FEEDER_XBL_PLAYERS: c_int = 0xA1;
pub const FEEDER_XBL_FRIENDS: c_int = 0xA2;
pub const FEEDER_XBL_SERVERS: c_int = 0xA3;

/// Raven cgame HUD display flags (`CG_SHOW_*`).
/// Source: `oracle/ui/menudef.h:121-141`
pub const CG_SHOW_BLUE_TEAM_HAS_REDFLAG: c_int = 0x0000_0001;
pub const CG_SHOW_RED_TEAM_HAS_BLUEFLAG: c_int = 0x0000_0002;
pub const CG_SHOW_ANYTEAMGAME: c_int = 0x0000_0004;
pub const CG_SHOW_HARVESTER: c_int = 0x0000_0008;
pub const CG_SHOW_ONEFLAG: c_int = 0x0000_0010;
pub const CG_SHOW_CTF: c_int = 0x0000_0020;
pub const CG_SHOW_OBELISK: c_int = 0x0000_0040;
pub const CG_SHOW_HEALTHCRITICAL: c_int = 0x0000_0080;
pub const CG_SHOW_SINGLEPLAYER: c_int = 0x0000_0100;
pub const CG_SHOW_TOURNAMENT: c_int = 0x0000_0200;
pub const CG_SHOW_DURINGINCOMINGVOICE: c_int = 0x0000_0400;
pub const CG_SHOW_IF_PLAYER_HAS_FLAG: c_int = 0x0000_0800;
pub const CG_SHOW_LANPLAYONLY: c_int = 0x0000_1000;
pub const CG_SHOW_MINED: c_int = 0x0000_2000;
pub const CG_SHOW_HEALTHOK: c_int = 0x0000_4000;
pub const CG_SHOW_TEAMINFO: c_int = 0x0000_8000;
pub const CG_SHOW_NOTEAMINFO: c_int = 0x0001_0000;
pub const CG_SHOW_OTHERTEAMHASFLAG: c_int = 0x0002_0000;
pub const CG_SHOW_YOURTEAMHASENEMYFLAG: c_int = 0x0004_0000;
pub const CG_SHOW_ANYNONTEAMGAME: c_int = 0x0008_0000;
pub const CG_SHOW_2DONLY: c_int = 0x1000_0000;

/// Raven ui ownerDraw visibility flags (`UI_SHOW_*`, `UI_OwnerDrawVisible`'s
/// `flags` argument).
/// Source: `oracle/ui/menudef.h:144-156`
pub const UI_SHOW_LEADER: c_int = 0x0000_0001;
pub const UI_SHOW_NOTLEADER: c_int = 0x0000_0002;
pub const UI_SHOW_FAVORITESERVERS: c_int = 0x0000_0004;
pub const UI_SHOW_ANYNONTEAMGAME: c_int = 0x0000_0008;
pub const UI_SHOW_ANYTEAMGAME: c_int = 0x0000_0010;
pub const UI_SHOW_NEWHIGHSCORE: c_int = 0x0000_0020;
pub const UI_SHOW_DEMOAVAILABLE: c_int = 0x0000_0040;
pub const UI_SHOW_NEWBESTTIME: c_int = 0x0000_0080;
pub const UI_SHOW_FFA: c_int = 0x0000_0100;
pub const UI_SHOW_NOTFFA: c_int = 0x0000_0200;
pub const UI_SHOW_NETANYNONTEAMGAME: c_int = 0x0000_0400;
pub const UI_SHOW_NETANYTEAMGAME: c_int = 0x0000_0800;
pub const UI_SHOW_NOTFAVORITESERVERS: c_int = 0x0000_1000;

/// Raven cgame ownerDraw codes (`CG_OWNERDRAW_BASE` + `CG_*`). Kept outside
/// an enum ("ideally these should be done outside of this file but this
/// makes it much easier for the macro expansion to convert them for the
/// designers ( from the .menu files )" — Raven).
/// Source: `oracle/ui/menudef.h:165-241`
pub const CG_OWNERDRAW_BASE: c_int = 1;
pub const CG_PLAYER_ARMOR_ICON: c_int = 1;
pub const CG_PLAYER_ARMOR_VALUE: c_int = 2;
pub const CG_PLAYER_HEAD: c_int = 3;
pub const CG_PLAYER_HEALTH: c_int = 4;
pub const CG_PLAYER_AMMO_ICON: c_int = 5;
pub const CG_PLAYER_AMMO_VALUE: c_int = 6;
pub const CG_SELECTEDPLAYER_HEAD: c_int = 7;
pub const CG_SELECTEDPLAYER_NAME: c_int = 8;
pub const CG_SELECTEDPLAYER_LOCATION: c_int = 9;
pub const CG_SELECTEDPLAYER_STATUS: c_int = 10;
pub const CG_SELECTEDPLAYER_WEAPON: c_int = 11;
pub const CG_SELECTEDPLAYER_POWERUP: c_int = 12;
pub const CG_FLAGCARRIER_HEAD: c_int = 13;
pub const CG_FLAGCARRIER_NAME: c_int = 14;
pub const CG_FLAGCARRIER_LOCATION: c_int = 15;
pub const CG_FLAGCARRIER_STATUS: c_int = 16;
pub const CG_FLAGCARRIER_WEAPON: c_int = 17;
pub const CG_FLAGCARRIER_POWERUP: c_int = 18;
pub const CG_PLAYER_ITEM: c_int = 19;
pub const CG_PLAYER_SCORE: c_int = 20;
pub const CG_BLUE_FLAGHEAD: c_int = 21;
pub const CG_BLUE_FLAGSTATUS: c_int = 22;
pub const CG_BLUE_FLAGNAME: c_int = 23;
pub const CG_RED_FLAGHEAD: c_int = 24;
pub const CG_RED_FLAGSTATUS: c_int = 25;
pub const CG_RED_FLAGNAME: c_int = 26;
pub const CG_BLUE_SCORE: c_int = 27;
pub const CG_RED_SCORE: c_int = 28;
pub const CG_RED_NAME: c_int = 29;
pub const CG_BLUE_NAME: c_int = 30;
/// Only shows in harvester.
pub const CG_HARVESTER_SKULLS: c_int = 31;
/// Only shows in one flag.
pub const CG_ONEFLAG_STATUS: c_int = 32;
pub const CG_PLAYER_LOCATION: c_int = 33;
pub const CG_TEAM_COLOR: c_int = 34;
pub const CG_CTF_POWERUP: c_int = 35;
pub const CG_AREA_POWERUP: c_int = 36;
/// Painted with old system.
pub const CG_AREA_LAGOMETER: c_int = 37;
pub const CG_PLAYER_HASFLAG: c_int = 38;
/// Not done.
pub const CG_GAME_TYPE: c_int = 39;
pub const CG_SELECTEDPLAYER_ARMOR: c_int = 40;
pub const CG_SELECTEDPLAYER_HEALTH: c_int = 41;
pub const CG_PLAYER_STATUS: c_int = 42;
/// Painted with old system.
pub const CG_FRAGGED_MSG: c_int = 43;
/// Painted with old system.
pub const CG_PROXMINED_MSG: c_int = 44;
/// Painted with old system.
pub const CG_AREA_FPSINFO: c_int = 45;
/// Painted with old system.
pub const CG_AREA_SYSTEMCHAT: c_int = 46;
/// Painted with old system.
pub const CG_AREA_TEAMCHAT: c_int = 47;
/// Painted with old system.
pub const CG_AREA_CHAT: c_int = 48;
pub const CG_GAME_STATUS: c_int = 49;
pub const CG_KILLER: c_int = 50;
pub const CG_PLAYER_ARMOR_ICON2D: c_int = 51;
pub const CG_PLAYER_AMMO_ICON2D: c_int = 52;
pub const CG_ACCURACY: c_int = 53;
pub const CG_ASSISTS: c_int = 54;
pub const CG_DEFEND: c_int = 55;
pub const CG_EXCELLENT: c_int = 56;
pub const CG_IMPRESSIVE: c_int = 57;
pub const CG_PERFECT: c_int = 58;
pub const CG_GAUNTLET: c_int = 59;
pub const CG_SPECTATORS: c_int = 60;
pub const CG_TEAMINFO: c_int = 61;
pub const CG_VOICE_HEAD: c_int = 62;
pub const CG_VOICE_NAME: c_int = 63;
pub const CG_PLAYER_HASFLAG2D: c_int = 64;
/// Only shows in harvester.
pub const CG_HARVESTER_SKULLS2D: c_int = 65;
pub const CG_CAPFRAGLIMIT: c_int = 66;
pub const CG_1STPLACE: c_int = 67;
pub const CG_2NDPLACE: c_int = 68;
pub const CG_CAPTURES: c_int = 69;
pub const CG_PLAYER_FORCE_VALUE: c_int = 70;

/// Raven ui ownerDraw codes (`UI_OWNERDRAW_BASE` + `UI_*`).
/// Source: `oracle/ui/menudef.h:246-353`
pub const UI_OWNERDRAW_BASE: c_int = 200;
pub const UI_HANDICAP: c_int = 200;
pub const UI_EFFECTS: c_int = 201;
pub const UI_PLAYERMODEL: c_int = 202;
pub const UI_CLANNAME: c_int = 203;
pub const UI_CLANLOGO: c_int = 204;
pub const UI_GAMETYPE: c_int = 205;
pub const UI_MAPPREVIEW: c_int = 206;
pub const UI_SKILL: c_int = 207;
pub const UI_BLUETEAMNAME: c_int = 208;
pub const UI_REDTEAMNAME: c_int = 209;
pub const UI_BLUETEAM1: c_int = 210;
pub const UI_BLUETEAM2: c_int = 211;
pub const UI_BLUETEAM3: c_int = 212;
pub const UI_BLUETEAM4: c_int = 213;
pub const UI_BLUETEAM5: c_int = 214;
pub const UI_REDTEAM1: c_int = 215;
pub const UI_REDTEAM2: c_int = 216;
pub const UI_REDTEAM3: c_int = 217;
pub const UI_REDTEAM4: c_int = 218;
pub const UI_REDTEAM5: c_int = 219;
pub const UI_NETSOURCE: c_int = 220;
pub const UI_NETMAPPREVIEW: c_int = 221;
pub const UI_NETFILTER: c_int = 222;
pub const UI_TIER: c_int = 223;
pub const UI_OPPONENTMODEL: c_int = 224;
pub const UI_TIERMAP1: c_int = 225;
pub const UI_TIERMAP2: c_int = 226;
pub const UI_TIERMAP3: c_int = 227;
pub const UI_PLAYERLOGO: c_int = 228;
pub const UI_OPPONENTLOGO: c_int = 229;
pub const UI_PLAYERLOGO_METAL: c_int = 230;
pub const UI_OPPONENTLOGO_METAL: c_int = 231;
pub const UI_PLAYERLOGO_NAME: c_int = 232;
pub const UI_OPPONENTLOGO_NAME: c_int = 233;
pub const UI_TIER_MAPNAME: c_int = 234;
pub const UI_TIER_GAMETYPE: c_int = 235;
pub const UI_ALLMAPS_SELECTION: c_int = 236;
pub const UI_OPPONENT_NAME: c_int = 237;
pub const UI_VOTE_KICK: c_int = 238;
pub const UI_BOTNAME: c_int = 239;
pub const UI_BOTSKILL: c_int = 240;
pub const UI_REDBLUE: c_int = 241;
pub const UI_CROSSHAIR: c_int = 242;
pub const UI_SELECTEDPLAYER: c_int = 243;
pub const UI_MAPCINEMATIC: c_int = 244;
pub const UI_NETGAMETYPE: c_int = 245;
pub const UI_NETMAPCINEMATIC: c_int = 246;
pub const UI_SERVERREFRESHDATE: c_int = 247;
pub const UI_SERVERMOTD: c_int = 248;
pub const UI_GLINFO: c_int = 249;
pub const UI_KEYBINDSTATUS: c_int = 250;
pub const UI_CLANCINEMATIC: c_int = 251;
pub const UI_MAP_TIMETOBEAT: c_int = 252;
pub const UI_JOINGAMETYPE: c_int = 253;
pub const UI_PREVIEWCINEMATIC: c_int = 254;
pub const UI_STARTMAPCINEMATIC: c_int = 255;
pub const UI_MAPS_SELECTION: c_int = 256;
pub const UI_FORCE_SIDE: c_int = 257;
pub const UI_FORCE_RANK: c_int = 258;
pub const UI_FORCE_RANK_HEAL: c_int = 259;
pub const UI_FORCE_RANK_LEVITATION: c_int = 260;
pub const UI_FORCE_RANK_SPEED: c_int = 261;
pub const UI_FORCE_RANK_PUSH: c_int = 262;
pub const UI_FORCE_RANK_PULL: c_int = 263;
pub const UI_FORCE_RANK_TELEPATHY: c_int = 264;
pub const UI_FORCE_RANK_GRIP: c_int = 265;
pub const UI_FORCE_RANK_LIGHTNING: c_int = 266;
pub const UI_FORCE_RANK_RAGE: c_int = 267;
pub const UI_FORCE_RANK_PROTECT: c_int = 268;
pub const UI_FORCE_RANK_ABSORB: c_int = 269;
pub const UI_FORCE_RANK_TEAM_HEAL: c_int = 270;
pub const UI_FORCE_RANK_TEAM_FORCE: c_int = 271;
pub const UI_FORCE_RANK_DRAIN: c_int = 272;
pub const UI_FORCE_RANK_SEE: c_int = 273;
pub const UI_FORCE_RANK_SABERATTACK: c_int = 274;
pub const UI_FORCE_RANK_SABERDEFEND: c_int = 275;
pub const UI_FORCE_RANK_SABERTHROW: c_int = 276;
pub const UI_VERSION: c_int = 277;
pub const UI_TOTALFORCESTARS: c_int = 278;
pub const UI_AUTOSWITCHLIST: c_int = 279;
/// "How handy it would be if this were an enum." — Raven.
pub const UI_BLUETEAM6: c_int = 280;
pub const UI_BLUETEAM7: c_int = 281;
pub const UI_BLUETEAM8: c_int = 282;
pub const UI_REDTEAM6: c_int = 283;
pub const UI_REDTEAM7: c_int = 284;
pub const UI_REDTEAM8: c_int = 285;
/// "Yes it would be handy" — Raven.
pub const UI_FORCE_MASTERY_SET: c_int = 286;
pub const UI_SKIN_COLOR: c_int = 287;
pub const UI_FORCE_POINTS: c_int = 288;
/// Extra, for patch.
pub const UI_JEDI_NONJEDI: c_int = 289;
/// Xbox-only, for complicated passcode entry screen.
pub const UI_XBOX_PASSCODE: c_int = 290;
pub const UI_CHAT_MAIN: c_int = 291;
pub const UI_CHAT_ATTACK: c_int = 292;
pub const UI_CHAT_DEFEND: c_int = 293;
pub const UI_CHAT_REQUEST: c_int = 294;
pub const UI_CHAT_REPLY: c_int = 295;
pub const UI_CHAT_SPOT: c_int = 296;
pub const UI_CHAT_TACTICAL: c_int = 297;

/// Raven voice-chat command strings (`VOICECHAT_*`), sent over the network as
/// literal ASCII tokens (`cg_VoiceChats.c`/`bg_voiceChatList`).
/// Source: `oracle/ui/menudef.h:355-388`
pub const VOICECHAT_GETFLAG: &str = "getflag";
pub const VOICECHAT_OFFENSE: &str = "offense";
pub const VOICECHAT_DEFEND: &str = "defend";
pub const VOICECHAT_DEFENDFLAG: &str = "defendflag";
pub const VOICECHAT_PATROL: &str = "patrol";
pub const VOICECHAT_CAMP: &str = "camp";
pub const VOICECHAT_FOLLOWME: &str = "followme";
pub const VOICECHAT_RETURNFLAG: &str = "returnflag";
pub const VOICECHAT_FOLLOWFLAGCARRIER: &str = "followflagcarrier";
pub const VOICECHAT_YES: &str = "yes";
pub const VOICECHAT_NO: &str = "no";
pub const VOICECHAT_ONGETFLAG: &str = "ongetflag";
pub const VOICECHAT_ONOFFENSE: &str = "onoffense";
pub const VOICECHAT_ONDEFENSE: &str = "ondefense";
pub const VOICECHAT_ONPATROL: &str = "onpatrol";
pub const VOICECHAT_ONCAMPING: &str = "oncamp";
pub const VOICECHAT_ONFOLLOW: &str = "onfollow";
pub const VOICECHAT_ONFOLLOWCARRIER: &str = "onfollowcarrier";
pub const VOICECHAT_ONRETURNFLAG: &str = "onreturnflag";
pub const VOICECHAT_INPOSITION: &str = "inposition";
pub const VOICECHAT_IHAVEFLAG: &str = "ihaveflag";
pub const VOICECHAT_BASEATTACK: &str = "baseattack";
pub const VOICECHAT_ENEMYHASFLAG: &str = "enemyhasflag";
pub const VOICECHAT_STARTLEADER: &str = "startleader";
pub const VOICECHAT_STOPLEADER: &str = "stopleader";
pub const VOICECHAT_TRASH: &str = "trash";
pub const VOICECHAT_WHOISLEADER: &str = "whoisleader";
pub const VOICECHAT_WANTONDEFENSE: &str = "wantondefense";
pub const VOICECHAT_WANTONOFFENSE: &str = "wantonoffense";
pub const VOICECHAT_KILLINSULT: &str = "kill_insult";
pub const VOICECHAT_TAUNT: &str = "taunt";
pub const VOICECHAT_DEATHINSULT: &str = "death_insult";
pub const VOICECHAT_KILLGAUNTLET: &str = "kill_gauntlet";
pub const VOICECHAT_PRAISE: &str = "praise";

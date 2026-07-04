//! `GameGlobals` — the remaining game-tier mutable file-scope globals
//! and file-statics as one owned GameWorld sub-struct (fork ruling 1:
//! file-scope mutable globals become GameWorld fields, grouped by owning
//! `.c` file). Pass-2 porters read/write these through `ctx.world`; they
//! never add a field. Scalar decls carry their Rust type; non-scalar
//! decls (pointers/structs/arrays) are `()` placeholders with a
//! `//TODO: Port <type>` marker — the porter fills the real type when
//! porting that body (bg/qshared-owned globals and const tables are
//! intentionally excluded — not GameWorld state).
#![allow(non_snake_case, non_camel_case_types, unused)]

use crate::prelude::*;

/// Raven game-tier mutable file-scope globals (fork ruling 1).
#[derive(Default)]
pub struct GameGlobals {
    // --- `NPC.c` file-scope globals ---
    //TODO: Port gentity_t **
    // Source: oracle/oracle/codemp/game/NPC.c:34
    pub NPC: (),
    //TODO: Port gNPC_t **
    // Source: oracle/oracle/codemp/game/NPC.c:35
    pub NPCInfo: (),
    //TODO: Port gentity_t **
    // Source: oracle/oracle/codemp/game/NPC.c:625
    pub _saved_NPC: (),
    //TODO: Port gNPC_t **
    // Source: oracle/oracle/codemp/game/NPC.c:626
    pub _saved_NPCInfo: (),
    //TODO: Port gclient_t **
    // Source: oracle/oracle/codemp/game/NPC.c:627
    pub _saved_client: (),
    //TODO: Port gclient_t **
    // Source: oracle/oracle/codemp/game/NPC.c:36
    pub client: (),
    //TODO: Port visibility_t
    // Source: oracle/oracle/codemp/game/NPC.c:38
    pub enemyVisibility: (),
    //TODO: Port usercmd_t
    // Source: oracle/oracle/codemp/game/NPC.c:37
    pub ucmd: (),
    // --- `NPC_AI_GalakMech.c` file-scope globals ---
    /// `enemyCS4`. Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:34`
    pub enemyCS4: qboolean,
    /// `enemyDist4`. Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:39`
    pub enemyDist4: f32,
    /// `enemyLOS4`. Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:33`
    pub enemyLOS4: qboolean,
    /// `faceEnemy4`. Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:36`
    pub faceEnemy4: qboolean,
    /// `hitAlly4`. Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:35`
    pub hitAlly4: qboolean,
    /// `move4`. Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:37`
    pub move4: qboolean,
    /// `shoot4`. Source: `oracle/oracle/codemp/game/NPC_AI_GalakMech.c:38`
    pub shoot4: qboolean,
    // --- `NPC_AI_Grenadier.c` file-scope globals ---
    /// `enemyCS3`. Source: `oracle/oracle/codemp/game/NPC_AI_Grenadier.c:35`
    pub enemyCS3: qboolean,
    /// `enemyDist3`. Source: `oracle/oracle/codemp/game/NPC_AI_Grenadier.c:39`
    pub enemyDist3: f32,
    /// `enemyLOS3`. Source: `oracle/oracle/codemp/game/NPC_AI_Grenadier.c:34`
    pub enemyLOS3: qboolean,
    /// `faceEnemy3`. Source: `oracle/oracle/codemp/game/NPC_AI_Grenadier.c:36`
    pub faceEnemy3: qboolean,
    /// `move3`. Source: `oracle/oracle/codemp/game/NPC_AI_Grenadier.c:37`
    pub move3: qboolean,
    /// `shoot3`. Source: `oracle/oracle/codemp/game/NPC_AI_Grenadier.c:38`
    pub shoot3: qboolean,
    // --- `NPC_AI_Jedi.c` file-scope globals ---
    //TODO: Port int[TEAM_NUM_TEAMS]
    // Source: oracle/oracle/codemp/game/NPC_AI_Jedi.c:94
    pub jediSpeechDebounceTime: (),
    // --- `NPC_AI_Sniper.c` file-scope globals ---
    /// `enemyCS2`. Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:30`
    pub enemyCS2: qboolean,
    /// `enemyDist2`. Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:34`
    pub enemyDist2: f32,
    /// `enemyLOS2`. Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:29`
    pub enemyLOS2: qboolean,
    /// `faceEnemy2`. Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:31`
    pub faceEnemy2: qboolean,
    /// `move2`. Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:32`
    pub move2: qboolean,
    /// `shoot2`. Source: `oracle/oracle/codemp/game/NPC_AI_Sniper.c:33`
    pub shoot2: qboolean,
    // --- `NPC_AI_Stormtrooper.c` file-scope globals ---
    /// `enemyCS`. Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:41`
    pub enemyCS: qboolean,
    /// `enemyDist`. Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:47`
    pub enemyDist: f32,
    /// `enemyInFOV`. Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:42`
    pub enemyInFOV: qboolean,
    /// `enemyLOS`. Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:40`
    pub enemyLOS: qboolean,
    /// `faceEnemy`. Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:44`
    pub faceEnemy: qboolean,
    //TODO: Port int[TEAM_NUM_TEAMS]
    // Source: oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:50
    pub groupSpeechDebounceTime: (),
    /// `hitAlly`. Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:43`
    pub hitAlly: qboolean,
    /// `move`. Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:45`
    pub r#move: qboolean,
    /// `shoot`. Source: `oracle/oracle/codemp/game/NPC_AI_Stormtrooper.c:46`
    pub shoot: qboolean,
    // --- `NPC_move.c` file-scope globals ---
    //TODO: Port navInfo_t
    // Source: oracle/oracle/codemp/game/NPC_move.c:14
    pub frameNavInfo: (),
    // --- `NPC_spawn.c` file-scope globals ---
    //TODO: Port gNPC_t *[MAX_GENTITIES]*
    // Source: oracle/oracle/codemp/game/NPC_spawn.c:1276
    pub gNPCPtrs: (),
    /// `showBBoxes`. Source: `oracle/oracle/codemp/game/NPC_spawn.c:4182`
    pub showBBoxes: qboolean,
    // --- `NPC_stats.c` file-scope globals ---
    //TODO: Port char[MAX_NPC_DATA_SIZE]
    // Source: oracle/oracle/codemp/game/NPC_stats.c:3238
    pub npcParseBuffer: (),
    // --- `ai_main.c` file-scope globals ---
    //TODO: Port bot_state_t *[MAX_CLIENTS]*
    // Source: oracle/oracle/codemp/game/ai_main.c:46
    pub botstates: (),
    //TODO: Port gentity_t **
    // Source: oracle/oracle/codemp/game/ai_main.c:94
    pub droppedBlueFlag: (),
    //TODO: Port gentity_t **
    // Source: oracle/oracle/codemp/game/ai_main.c:92
    pub droppedRedFlag: (),
    //TODO: Port gentity_t **
    // Source: oracle/oracle/codemp/game/ai_main.c:93
    pub eFlagBlue: (),
    //TODO: Port gentity_t **
    // Source: oracle/oracle/codemp/game/ai_main.c:91
    pub eFlagRed: (),
    //TODO: Port wpobject_t **
    // Source: oracle/oracle/codemp/game/ai_main.c:88
    pub flagBlue: (),
    //TODO: Port wpobject_t **
    // Source: oracle/oracle/codemp/game/ai_main.c:86
    pub flagRed: (),
    //TODO: Port boteventtracker_t[MAX_CLIENTS]
    // Source: oracle/oracle/codemp/game/ai_main.c:59
    pub gBotEventTracker: (),
    /// `gUpdateVars`. Source: `oracle/oracle/codemp/game/ai_main.c:7485`
    pub gUpdateVars: c_int,
    /// `numbots`. Source: `oracle/oracle/codemp/game/ai_main.c:48`
    pub numbots: c_int,
    //TODO: Port wpobject_t **
    // Source: oracle/oracle/codemp/game/ai_main.c:89
    pub oFlagBlue: (),
    //TODO: Port wpobject_t **
    // Source: oracle/oracle/codemp/game/ai_main.c:87
    pub oFlagRed: (),
    /// `regularupdate_time`. Source: `oracle/oracle/codemp/game/ai_main.c:52`
    pub regularupdate_time: f32,
    // --- `ai_main.h` file-scope globals ---
    //TODO: Port wpobject_t *[MAX_WPARRAY_SIZE]*
    // Source: oracle/oracle/codemp/game/ai_main.h:398
    pub gWPArray: (),
    // --- `ai_util.c` file-scope globals ---
    //TODO: Port char[MAX_CLIENTS][MAX_CHAT_BUFFER_SIZE]
    // Source: oracle/oracle/codemp/game/ai_util.c:12
    pub gBotChatBuffer: (),
    // --- `ai_wpnav.c` file-scope globals ---
    /// `gBotEdit`. Source: `oracle/oracle/codemp/game/ai_wpnav.c:8`
    pub gBotEdit: f32,
    /// `gDeactivated`. Source: `oracle/oracle/codemp/game/ai_wpnav.c:7`
    pub gDeactivated: f32,
    /// `gLastPrintedIndex`. Source: `oracle/oracle/codemp/game/ai_wpnav.c:16`
    pub gLastPrintedIndex: c_int,
    /// `gLevelFlags`. Source: `oracle/oracle/codemp/game/ai_wpnav.c:23`
    pub gLevelFlags: c_int,
    /// `gSpawnPointNum`. Source: `oracle/oracle/codemp/game/ai_wpnav.c:2506`
    pub gSpawnPointNum: c_int,
    //TODO: Port gentity_t *[MAX_SPAWNPOINT_ARRAY]*
    // Source: oracle/oracle/codemp/game/ai_wpnav.c:2507
    pub gSpawnPoints: (),
    /// `gWPNum`. Source: `oracle/oracle/codemp/game/ai_wpnav.c:13`
    pub gWPNum: c_int,
    /// `gWPRenderTime`. Source: `oracle/oracle/codemp/game/ai_wpnav.c:6`
    pub gWPRenderTime: f32,
    /// `gWPRenderedFrame`. Source: `oracle/oracle/codemp/game/ai_wpnav.c:9`
    pub gWPRenderedFrame: c_int,
    /// `nodenum`. Source: `oracle/oracle/codemp/game/ai_wpnav.c:20`
    pub nodenum: c_int,
    //TODO: Port nodeobject_t[MAX_NODETABLE_SIZE]
    // Source: oracle/oracle/codemp/game/ai_wpnav.c:19
    pub nodetable: (),
    // --- `g_bot.c` file-scope globals ---
    //TODO: Port botSpawnQueue_t[BOT_SPAWN_QUEUE_DEPTH]
    // Source: oracle/oracle/codemp/game/g_bot.c:27
    pub botSpawnQueue: (),
    /// `g_numArenas`. Source: `oracle/oracle/codemp/game/g_bot.c:12`
    pub g_numArenas: c_int,
    /// `g_numBots`. Source: `oracle/oracle/codemp/game/g_bot.c:8`
    pub g_numBots: c_int,
    // --- `g_client.c` file-scope globals ---
    //TODO: Port void **
    // Source: oracle/oracle/codemp/game/g_client.c:1511
    pub g2SaberInstance: (),
    //TODO: Port gentity_t **
    // Source: oracle/oracle/codemp/game/g_client.c:471
    pub gJMSaberEnt: (),
    // --- `g_cmds.c` file-scope globals ---
    /// `g_dontPenalizeTeam`. Source: `oracle/oracle/codemp/game/g_cmds.c:750`
    pub g_dontPenalizeTeam: qboolean,
    /// `g_preventTeamBegin`. Source: `oracle/oracle/codemp/game/g_cmds.c:751`
    pub g_preventTeamBegin: qboolean,
    // --- `g_combat.c` file-scope globals ---
    /// `gGAvoidDismember`. Source: `oracle/oracle/codemp/game/g_combat.c:3753`
    pub gGAvoidDismember: c_int,
    /// `gPainHitLoc`. Source: `oracle/oracle/codemp/game/g_combat.c:4574`
    pub gPainHitLoc: c_int,
    /// `gPainMOD`. Source: `oracle/oracle/codemp/game/g_combat.c:4573`
    pub gPainMOD: c_int,
    // --- `g_items.c` file-scope globals ---
    //TODO: Port qboolean[MAX_ITEMS]
    // Source: oracle/oracle/codemp/game/g_items.c:2966
    pub itemRegistered: (),
    //TODO: Port qhandle_t
    // Source: oracle/oracle/codemp/game/g_items.c:103
    pub shieldActivateSound: (),
    //TODO: Port qhandle_t
    // Source: oracle/oracle/codemp/game/g_items.c:102
    pub shieldAttachSound: (),
    //TODO: Port qhandle_t
    // Source: oracle/oracle/codemp/game/g_items.c:105
    pub shieldDamageSound: (),
    //TODO: Port qhandle_t
    // Source: oracle/oracle/codemp/game/g_items.c:104
    pub shieldDeactivateSound: (),
    //TODO: Port qhandle_t
    // Source: oracle/oracle/codemp/game/g_items.c:101
    pub shieldLoopSound: (),
    // --- `g_log.c` file-scope globals ---
    //TODO: Port qboolean[MAX_CLIENTS]
    // Source: oracle/oracle/codemp/game/g_log.c:27
    pub G_WeaponLogClientTouch: (),
    //TODO: Port int[MAX_CLIENTS][MOD_MAX]
    // Source: oracle/oracle/codemp/game/g_log.c:21
    pub G_WeaponLogDamage: (),
    //TODO: Port int[MAX_CLIENTS][WP_NUM_WEAPONS]
    // Source: oracle/oracle/codemp/game/g_log.c:23
    pub G_WeaponLogDeaths: (),
    //TODO: Port int[MAX_CLIENTS][WP_NUM_WEAPONS]
    // Source: oracle/oracle/codemp/game/g_log.c:20
    pub G_WeaponLogFired: (),
    //TODO: Port int[MAX_CLIENTS][MAX_CLIENTS]
    // Source: oracle/oracle/codemp/game/g_log.c:24
    pub G_WeaponLogFrags: (),
    //TODO: Port int[MAX_CLIENTS][PW_NUM_POWERUPS]
    // Source: oracle/oracle/codemp/game/g_log.c:29
    pub G_WeaponLogItems: (),
    //TODO: Port int[MAX_CLIENTS][MOD_MAX]
    // Source: oracle/oracle/codemp/game/g_log.c:22
    pub G_WeaponLogKills: (),
    //TODO: Port int[MAX_CLIENTS]
    // Source: oracle/oracle/codemp/game/g_log.c:26
    pub G_WeaponLogLastTime: (),
    //TODO: Port int[MAX_CLIENTS][WP_NUM_WEAPONS]
    // Source: oracle/oracle/codemp/game/g_log.c:19
    pub G_WeaponLogPickups: (),
    //TODO: Port int[MAX_CLIENTS][HI_NUM_HOLDABLE]
    // Source: oracle/oracle/codemp/game/g_log.c:28
    pub G_WeaponLogPowerups: (),
    //TODO: Port int[MAX_CLIENTS][WP_NUM_WEAPONS]
    // Source: oracle/oracle/codemp/game/g_log.c:25
    pub G_WeaponLogTime: (),
    // --- `g_main.c` file-scope globals ---
    /// `eventClearTime`. Source: `oracle/oracle/codemp/game/g_main.c:11`
    pub eventClearTime: c_int,
    /// `gDidDuelStuff`. Source: `oracle/oracle/codemp/game/g_main.c:2305`
    pub gDidDuelStuff: qboolean,
    /// `gDoSlowMoDuel`. Source: `oracle/oracle/codemp/game/g_main.c:3517`
    pub gDoSlowMoDuel: qboolean,
    /// `gDuelExit`. Source: `oracle/oracle/codemp/game/g_main.c:30`
    pub gDuelExit: qboolean,
    /// `gQueueScoreMessage`. Source: `oracle/oracle/codemp/game/g_main.c:1691`
    pub gQueueScoreMessage: qboolean,
    /// `gQueueScoreMessageTime`. Source: `oracle/oracle/codemp/game/g_main.c:1692`
    pub gQueueScoreMessageTime: c_int,
    /// `gSlowMoDuelTime`. Source: `oracle/oracle/codemp/game/g_main.c:3518`
    pub gSlowMoDuelTime: c_int,
    /// `g_LastFrameTime`. Source: `oracle/oracle/codemp/game/g_main.c:3514`
    pub g_LastFrameTime: c_int,
    /// `g_TimeSinceLastFrame`. Source: `oracle/oracle/codemp/game/g_main.c:3515`
    pub g_TimeSinceLastFrame: c_int,
    /// `g_dontFrickinCheck`. Source: `oracle/oracle/codemp/game/g_main.c:1427`
    pub g_dontFrickinCheck: qboolean,
    /// `g_duelPrintTimer`. Source: `oracle/oracle/codemp/game/g_main.c:2947`
    pub g_duelPrintTimer: c_int,
    /// `g_endPDuel`. Source: `oracle/oracle/codemp/game/g_main.c:2587`
    pub g_endPDuel: qboolean,
    /// `g_noPDuelCheck`. Source: `oracle/oracle/codemp/game/g_main.c:1717`
    pub g_noPDuelCheck: qboolean,
    /// `g_siegeRespawnCheck`. Source: `oracle/oracle/codemp/game/g_main.c:3580`
    pub g_siegeRespawnCheck: c_int,
    /// `killPlayerTimer`. Source: `oracle/oracle/codemp/game/g_main.c:15`
    pub killPlayerTimer: c_int,
    /// `navCalcPathTime`. Source: `oracle/oracle/codemp/game/g_main.c:12`
    pub navCalcPathTime: c_int,
    // --- `g_misc.c` file-scope globals ---
    /// `gEscapeTime`. Source: `oracle/oracle/codemp/game/g_misc.c:2541`
    pub gEscapeTime: c_int,
    /// `gEscaping`. Source: `oracle/oracle/codemp/game/g_misc.c:2540`
    pub gEscaping: qboolean,
    /// `g_shooterClientInit`. Source: `oracle/oracle/codemp/game/g_misc.c:3352`
    pub g_shooterClientInit: qboolean,
    //TODO: Port shooterClient_t[MAX_SHOOTERS]
    // Source: oracle/oracle/codemp/game/g_misc.c:3351
    pub g_shooterClients: (),
    // --- `g_mover.c` file-scope globals ---
    //TODO: Port **
    // Source: oracle/oracle/codemp/game/g_mover.c:24
    pub pushed_p: (),
    // --- `g_nav.c` file-scope globals ---
    /// `NAVDEBUG_curGoal`. Source: `oracle/oracle/codemp/game/g_nav.c:1607`
    pub NAVDEBUG_curGoal: c_int,
    /// `NAVDEBUG_showCollision`. Source: `oracle/oracle/codemp/game/g_nav.c:1606`
    pub NAVDEBUG_showCollision: qboolean,
    /// `NAVDEBUG_showCombatPoints`. Source: `oracle/oracle/codemp/game/g_nav.c:1604`
    pub NAVDEBUG_showCombatPoints: qboolean,
    /// `NAVDEBUG_showEdges`. Source: `oracle/oracle/codemp/game/g_nav.c:1601`
    pub NAVDEBUG_showEdges: qboolean,
    /// `NAVDEBUG_showEnemyPath`. Source: `oracle/oracle/codemp/game/g_nav.c:1603`
    pub NAVDEBUG_showEnemyPath: qboolean,
    /// `NAVDEBUG_showNavGoals`. Source: `oracle/oracle/codemp/game/g_nav.c:1605`
    pub NAVDEBUG_showNavGoals: qboolean,
    /// `NAVDEBUG_showNodes`. Source: `oracle/oracle/codemp/game/g_nav.c:1599`
    pub NAVDEBUG_showNodes: qboolean,
    /// `NAVDEBUG_showRadius`. Source: `oracle/oracle/codemp/game/g_nav.c:1600`
    pub NAVDEBUG_showRadius: qboolean,
    /// `NAVDEBUG_showTestPath`. Source: `oracle/oracle/codemp/game/g_nav.c:1602`
    pub NAVDEBUG_showTestPath: qboolean,
    //TODO: Port char **
    // Source: oracle/oracle/codemp/game/g_nav.c:1616
    pub fatalErrorPointer: (),
    /// `fatalErrors`. Source: `oracle/oracle/codemp/game/g_nav.c:1615`
    pub fatalErrors: c_int,
    /// `navCalculatePaths`. Source: `oracle/oracle/codemp/game/g_nav.c:1597`
    pub navCalculatePaths: qboolean,
    /// `numStoredWaypoints`. Source: `oracle/oracle/codemp/game/g_nav.c:1658`
    pub numStoredWaypoints: c_int,
    //TODO: Port waypointData_t[MAX_STORED_WAYPOINTS]
    // Source: oracle/oracle/codemp/game/g_nav.c:1660
    pub tempWaypointList: (),
    // --- `g_saga.c` file-scope globals ---
    /// `gImperialCountdown`. Source: `oracle/oracle/codemp/game/g_saga.c:30`
    pub gImperialCountdown: c_int,
    /// `gRebelCountdown`. Source: `oracle/oracle/codemp/game/g_saga.c:31`
    pub gRebelCountdown: c_int,
    /// `gSiegeBeginTime`. Source: `oracle/oracle/codemp/game/g_saga.c:39`
    pub gSiegeBeginTime: c_int,
    /// `gSiegeRoundBegun`. Source: `oracle/oracle/codemp/game/g_saga.c:36`
    pub gSiegeRoundBegun: qboolean,
    /// `gSiegeRoundEnded`. Source: `oracle/oracle/codemp/game/g_saga.c:37`
    pub gSiegeRoundEnded: qboolean,
    /// `gSiegeRoundWinningTeam`. Source: `oracle/oracle/codemp/game/g_saga.c:38`
    pub gSiegeRoundWinningTeam: qboolean,
    /// `g_preroundState`. Source: `oracle/oracle/codemp/game/g_saga.c:41`
    pub g_preroundState: c_int,
    //TODO: Port siegePers_t
    // Source: oracle/oracle/codemp/game/g_saga.c:20
    pub g_siegePersistant: (),
    /// `imperial_attackers`. Source: `oracle/oracle/codemp/game/g_saga.c:34`
    pub imperial_attackers: c_int,
    /// `imperial_goals_completed`. Source: `oracle/oracle/codemp/game/g_saga.c:23`
    pub imperial_goals_completed: c_int,
    /// `imperial_goals_required`. Source: `oracle/oracle/codemp/game/g_saga.c:22`
    pub imperial_goals_required: c_int,
    /// `imperial_time_limit`. Source: `oracle/oracle/codemp/game/g_saga.c:27`
    pub imperial_time_limit: c_int,
    /// `rebel_attackers`. Source: `oracle/oracle/codemp/game/g_saga.c:33`
    pub rebel_attackers: c_int,
    /// `rebel_goals_completed`. Source: `oracle/oracle/codemp/game/g_saga.c:25`
    pub rebel_goals_completed: c_int,
    /// `rebel_goals_required`. Source: `oracle/oracle/codemp/game/g_saga.c:24`
    pub rebel_goals_required: c_int,
    /// `rebel_time_limit`. Source: `oracle/oracle/codemp/game/g_saga.c:28`
    pub rebel_time_limit: c_int,
    // --- `g_spawn.c` file-scope globals ---
    //TODO: Port void **
    // Source: oracle/oracle/codemp/game/g_spawn.c:1234
    pub precachedKyle: (),
    // --- `g_svcmds.c` file-scope globals ---
    //TODO: Port ipFilter_t[MAX_IPFILTERS]
    // Source: oracle/oracle/codemp/game/g_svcmds.c:54
    pub ipFilters: (),
    /// `numIPFilters`. Source: `oracle/oracle/codemp/game/g_svcmds.c:55`
    pub numIPFilters: c_int,
    // --- `g_target.c` file-scope globals ---
    /// `numNewICARUSEnts`. Source: `oracle/oracle/codemp/game/g_target.c:753`
    pub numNewICARUSEnts: c_int,
    // --- `g_team.c` file-scope globals ---
    //TODO: Port teamgame_t
    // Source: oracle/oracle/codemp/game/g_team.c:18
    pub teamgame: (),
    // --- `g_timer.c` file-scope globals ---
    //TODO: Port gtimer_t **
    // Source: oracle/oracle/codemp/game/g_timer.c:19
    pub g_timerFreeList: (),
    //TODO: Port gtimer_t[ MAX_GTIMERS ]
    // Source: oracle/oracle/codemp/game/g_timer.c:17
    pub g_timerPool: (),
    //TODO: Port gtimer_t *[ MAX_GENTITIES ]*
    // Source: oracle/oracle/codemp/game/g_timer.c:18
    pub g_timers: (),
    // --- `g_trigger.c` file-scope globals ---
    /// `gTrigFallSound`. Source: `oracle/oracle/codemp/game/g_trigger.c:6`
    pub gTrigFallSound: c_int,
    // --- `g_utils.c` file-scope globals ---
    //TODO: Port gclient_t *[MAX_GENTITIES]*
    // Source: oracle/oracle/codemp/game/g_utils.c:428
    pub gClPtrs: (),
    //TODO: Port int[MAX_G2_KILL_QUEUE]
    // Source: oracle/oracle/codemp/game/g_utils.c:877
    pub gG2KillIndex: (),
    /// `gG2KillNum`. Source: `oracle/oracle/codemp/game/g_utils.c:878`
    pub gG2KillNum: c_int,
    /// `g_vehiclePoolInit`. Source: `oracle/oracle/codemp/game/g_utils.c:387`
    pub g_vehiclePoolInit: qboolean,
    //TODO: Port qboolean[MAX_VEHICLES_AT_A_TIME]
    // Source: oracle/oracle/codemp/game/g_utils.c:386
    pub g_vehiclePoolOccupied: (),
    /// `remapCount`. Source: `oracle/oracle/codemp/game/g_utils.c:17`
    pub remapCount: c_int,
    //TODO: Port shaderRemap_t[MAX_SHADER_REMAPS]
    // Source: oracle/oracle/codemp/game/g_utils.c:18
    pub remappedShaders: (),
    // --- `g_weapon.c` file-scope globals ---
    /// `s_quadFactor`. Source: `oracle/oracle/codemp/game/g_weapon.c:12`
    pub s_quadFactor: f32,
    // --- `w_saber.c` file-scope globals ---
    //TODO: Port qboolean[MAX_SABER_VICTIMS]
    // Source: oracle/oracle/codemp/game/w_saber.c:3509
    pub dismemberDmg: (),
    /// `numVictims`. Source: `oracle/oracle/codemp/game/w_saber.c:3511`
    pub numVictims: c_int,
    /// `saberClashEventParm`. Source: `oracle/oracle/codemp/game/w_saber.c:3797`
    pub saberClashEventParm: c_int,
    /// `saberDoClashEffect`. Source: `oracle/oracle/codemp/game/w_saber.c:3794`
    pub saberDoClashEffect: qboolean,
    /// `saberHitFraction`. Source: `oracle/oracle/codemp/game/w_saber.c:3848`
    pub saberHitFraction: f32,
    /// `saberHitSaber`. Source: `oracle/oracle/codemp/game/w_saber.c:3847`
    pub saberHitSaber: qboolean,
    /// `saberHitWall`. Source: `oracle/oracle/codemp/game/w_saber.c:3846`
    pub saberHitWall: qboolean,
    //TODO: Port int[MAX_SABER_VICTIMS]
    // Source: oracle/oracle/codemp/game/w_saber.c:3510
    pub saberKnockbackFlags: (),
    /// `saberSpinSound`. Source: `oracle/oracle/codemp/game/w_saber.c:18`
    pub saberSpinSound: c_int,
    //TODO: Port float[MAX_SABER_VICTIMS]
    // Source: oracle/oracle/codemp/game/w_saber.c:3506
    pub totalDmg: (),
    //TODO: Port int[MAX_SABER_VICTIMS]
    // Source: oracle/oracle/codemp/game/w_saber.c:3504
    pub victimEntityNum: (),
    //TODO: Port qboolean[MAX_SABER_VICTIMS]
    // Source: oracle/oracle/codemp/game/w_saber.c:3505
    pub victimHitEffectDone: (),
}

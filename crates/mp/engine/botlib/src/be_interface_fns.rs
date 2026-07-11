#![allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_assignments,
    unused_parens,
    clippy::too_many_arguments
)]

//! MP botlib `be_interface.cpp` — the botlib export/import wiring: client/
//! entity validation, `BotLibSetup`/`Shutdown`/`StartFrame`/`LoadMap`/
//! `UpdateEntity` export shims, the `aas_export_t`/`ea_export_t`/`ai_export_t`
//! dispatch-table builders, and `GetBotLibAPI` (the botlib's DLL entry point).
//!
//! Source: `oracle/codemp/botlib/be_interface.cpp`
//!
//! PORT-NOTE(fns-collision): destination is `be_interface_fns.rs`, not
//! `be_interface.rs` — the oracle stem collides with the existing
//! `be_interface/` type directory (`botlib_globals_s.rs`); DESTINATION line
//! from the packet already reflects the `_fns` escape.

use core::ffi::{c_char, c_int, c_void};

use crate::be_interface::botlib_globals_s::botlib_globals_t;
use crate::BotLib;

use mp_qshared::common::mp::botlib::aas_export_s::aas_export_t;
use mp_qshared::common::mp::botlib::ai_export_s::ai_export_t;
use mp_qshared::common::mp::botlib::bot_entitystate_s::bot_entitystate_t;
use mp_qshared::common::mp::botlib::botlib_error::{
    BLERR_INVALIDENTITYNUMBER, BLERR_LIBRARYNOTSETUP, BLERR_NOERROR,
};
use mp_qshared::common::mp::botlib::botlib_export_s::botlib_export_t;
use mp_qshared::common::mp::botlib::botlib_import_s::botlib_import_t;
use mp_qshared::common::mp::botlib::botlib_misc::BOTLIB_API_VERSION;
use mp_qshared::common::mp::botlib::ea_export_s::ea_export_t;
use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_MESSAGE};
use mp_qshared::shared::{qboolean, qfalse, qtrue, vec3_t};
use libc::{CLOCKS_PER_SEC, clock};
use crate::be_aas_bspq3_fns::{AAS_FloatForBSPEpairKey, AAS_IntForBSPEpairKey, AAS_NextBSPEntity, AAS_PointContents, AAS_ValueForBSPEpairKey, AAS_VectorForBSPEpairKey};
use crate::be_aas_entity::{AAS_EntityInfo, AAS_UpdateEntity};
use crate::be_aas_main::{AAS_Initialized, AAS_LoadMap, AAS_Setup, AAS_Shutdown, AAS_StartFrame, AAS_Time};
use crate::be_aas_move::{AAS_PredictClientMovement, AAS_Swimming};
use crate::be_aas_reach_fns::{AAS_AreaReachability};
use crate::be_aas_route_fns::{AAS_AreaTravelTimeToGoalArea, AAS_EnableRoutingArea, AAS_PredictRoute};
use crate::be_aas_routealt_fns::{AAS_AlternativeRouteGoals};
use crate::be_aas_sample_fns::{AAS_AreaInfo, AAS_BBoxAreas, AAS_PointAreaNum, AAS_PointReachabilityAreaIndex, AAS_PresenceTypeBoundingBox, AAS_TraceAreas};
use crate::be_ai_char_fns::{BotFreeCharacter, BotLoadCharacter, BotShutdownCharacters, Characteristic_BFloat, Characteristic_BInteger, Characteristic_Float, Characteristic_Integer, Characteristic_String};
use crate::be_ai_chat_fns::{BotAllocChatState, BotChatLength, BotEnterChat, BotFindMatch, BotFreeChatState, BotGetChatMessage, BotInitialChat, BotLoadChatFile, BotMatchVariable, BotNextConsoleMessage, BotNumConsoleMessages, BotNumInitialChats, BotQueueConsoleMessage, BotRemoveConsoleMessage, BotReplaceSynonyms, BotReplyChat, BotSetChatGender, BotSetChatName, BotShutdownChatAI, StringContains, UnifyWhiteSpaces};
use crate::be_ai_gen::{GeneticParentsAndChildSelection};
use crate::be_ai_goal_fns::{BotAllocGoalState, BotAvoidGoalTime, BotChooseLTGItem, BotChooseNBGItem, BotDumpAvoidGoals, BotDumpGoalStack, BotEmptyGoalStack, BotFreeGoalState, BotFreeItemWeights, BotGetLevelItemGoal, BotGetMapLocationGoal, BotGetNextCampSpotGoal, BotGetSecondGoal, BotGetTopGoal, BotGoalName, BotInitLevelItems, BotInterbreedGoalFuzzyLogic, BotItemGoalInVisButNotVisible, BotLoadItemWeights, BotMutateGoalFuzzyLogic, BotPopGoal, BotPushGoal, BotRemoveFromAvoidGoals, BotResetAvoidGoals, BotResetGoalState, BotSaveGoalFuzzyLogic, BotSetAvoidGoalTime, BotShutdownGoalAI, BotTouchingGoal, BotUpdateEntityItems};
use crate::be_ai_move_fns::{BotAddAvoidSpot, BotAllocMoveState, BotFreeMoveState, BotInitMoveState, BotMoveInDirection, BotMoveToGoal, BotMovementViewTarget, BotPredictVisiblePosition, BotReachabilityArea, BotResetAvoidReach, BotResetLastAvoidReach, BotResetMoveState, BotSetBrushModelTypes, BotShutdownMoveAI};
use crate::be_ai_weight_fns::{BotShutdownWeights};
use crate::be_ea_fns::{EA_Action, EA_Alt_Attack, EA_Attack, EA_Command, EA_Crouch, EA_DelayedJump, EA_EndRegular, EA_ForcePower, EA_Gesture, EA_GetInput, EA_Jump, EA_Move, EA_MoveBack, EA_MoveDown, EA_MoveForward, EA_MoveLeft, EA_MoveRight, EA_MoveUp, EA_ResetInput, EA_Respawn, EA_Say, EA_SayTeam, EA_SelectWeapon, EA_Setup, EA_Shutdown, EA_Talk, EA_Use, EA_View};
use crate::l_libvar_fns::{LibVarDeAllocAll, LibVarGetString, LibVarGetValue, LibVarSet, LibVarValue};
use crate::l_log_fns::{Log_Open, Log_Shutdown};
use crate::l_precomp_fns::{PC_AddGlobalDefine, PC_CheckOpenSourceHandles, PC_FreeSourceHandle, PC_LoadGlobalDefines, PC_LoadSourceHandle, PC_ReadTokenHandle, PC_RemoveAllGlobalDefines, PC_SourceFileAndLine};

/// Raven `Sys_MilliSeconds`.
///
/// Source: `oracle/codemp/botlib/be_interface.cpp:60-63`
pub fn Sys_MilliSeconds() -> c_int {
    // PORT-NOTE(CLOCKS_PER_SEC): the packet flags `CLOCKS_PER_SEC` as an
    // unresolved const (no rosetta row — it is a libc macro, not a Raven
    // symbol); referenced verbatim rather than guessed, escalated below.
    unsafe { (libc::clock() as c_int) * 1000 / CLOCKS_PER_SEC }
}

/// Raven `ValidClientNumber`.
///
/// Source: `oracle/codemp/botlib/be_interface.cpp:70-80`
pub fn ValidClientNumber(bot: &mut BotLib, num: c_int, str: *mut c_char) -> qboolean {
    unsafe {
        if num < 0 || num > bot.botlibglobals.maxclients {
            // weird: the disabled stuff results in a crash
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"%s: invalid client number %d, [0, %d]\n".as_ptr() as *mut c_char,
                str,
                num,
                bot.botlibglobals.maxclients,
            );
            return qfalse;
        }
        qtrue
    }
}

/// Raven `ValidEntityNumber`.
///
/// Source: `oracle/codemp/botlib/be_interface.cpp:87-96`
pub fn ValidEntityNumber(bot: &mut BotLib, num: c_int, str: *mut c_char) -> qboolean {
    unsafe {
        if num < 0 || num > bot.botlibglobals.maxentities {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"%s: invalid entity number %d, [0, %d]\n".as_ptr() as *mut c_char,
                str,
                num,
                bot.botlibglobals.maxentities,
            );
            return qfalse;
        }
        qtrue
    }
}

/// Raven `BotLibSetup` — `be_interface.cpp`'s own "is setup" check (distinct
/// from the `Export_BotLibSetup` init entry point below).
///
/// Source: `oracle/codemp/botlib/be_interface.cpp:103-111`
pub fn BotLibSetup(bot: &mut BotLib, str: *mut c_char) -> qboolean {
    unsafe {
        if bot.botlibglobals.botlibsetup == 0 {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"%s: bot library used before being setup\n".as_ptr() as *mut c_char,
                str,
            );
            return qfalse;
        }
        qtrue
    }
}

/// Raven `BotExportTest` — debug-only visualization/reachability test hook.
///
/// PORT-NOTE(DEBUG): the entire body is `#ifdef DEBUG`; this workspace builds
/// the WinDed Release macro set (FINAL_BUILD undefined, `DEBUG` undefined —
/// see decisions.md/porting-rules appendix), so the compiled-out body reduces
/// to the trailing `return 0;` — transcribed faithfully for that
/// configuration rather than porting the dead DEBUG block.
///
/// Source: `oracle/codemp/botlib/be_interface.cpp:302-631`
pub fn BotExportTest(parm0: c_int, parm1: *mut c_char, parm2: vec3_t, parm3: vec3_t) -> c_int {
    0
}

/// Raven `Export_BotLibVarGet`.
///
/// Source: `oracle/codemp/botlib/be_interface.cpp:213-221`
pub fn Export_BotLibVarGet(
    bot: &mut BotLib,
    var_name: *mut c_char,
    value: *mut c_char,
    size: c_int,
) -> c_int {
    unsafe {
        let varvalue = LibVarGetString(bot, var_name);
        libc::strncpy(value, varvalue, (size - 1) as usize);
        *value.offset((size - 1) as isize) = 0;
        BLERR_NOERROR
    }
}

/// Raven `Init_EA_Export` — populates the elementary-action export table.
///
/// PORT-NOTE(dispatch-table): `ea_export_t`'s fields are ABI `unsafe extern
/// "C" fn` pointers (no receiver), while the ported `EA_*` fns thread a `bot:
/// &mut BotLib` receiver per the state-threading rule — flagged in
/// shape_mismatches; assignments transcribed by name, reconciliation is an
/// integration concern (fn-ID/wrapper shape TBD).
///
/// Source: `oracle/codemp/botlib/be_interface.cpp:694-724`
pub fn Init_EA_Export(ea: *mut ea_export_t) {
    unsafe {
        // ClientCommand elementary actions
        (*ea).EA_Command = Some(EA_Command);
        (*ea).EA_Say = Some(EA_Say);
        (*ea).EA_SayTeam = Some(EA_SayTeam);

        (*ea).EA_Action = Some(EA_Action);
        (*ea).EA_Gesture = Some(EA_Gesture);
        (*ea).EA_Talk = Some(EA_Talk);
        (*ea).EA_Attack = Some(EA_Attack);
        (*ea).EA_Alt_Attack = Some(EA_Alt_Attack);
        (*ea).EA_ForcePower = Some(EA_ForcePower);
        (*ea).EA_Use = Some(EA_Use);
        (*ea).EA_Respawn = Some(EA_Respawn);
        (*ea).EA_Crouch = Some(EA_Crouch);
        (*ea).EA_MoveUp = Some(EA_MoveUp);
        (*ea).EA_MoveDown = Some(EA_MoveDown);
        (*ea).EA_MoveForward = Some(EA_MoveForward);
        (*ea).EA_MoveBack = Some(EA_MoveBack);
        (*ea).EA_MoveLeft = Some(EA_MoveLeft);
        (*ea).EA_MoveRight = Some(EA_MoveRight);

        (*ea).EA_SelectWeapon = Some(EA_SelectWeapon);
        (*ea).EA_Jump = Some(EA_Jump);
        (*ea).EA_DelayedJump = Some(EA_DelayedJump);
        (*ea).EA_Move = Some(EA_Move);
        (*ea).EA_View = Some(EA_View);
        (*ea).EA_GetInput = Some(EA_GetInput);
        (*ea).EA_EndRegular = Some(EA_EndRegular);
        (*ea).EA_ResetInput = Some(EA_ResetInput);
    }
}

/// Raven `Export_BotLibVarSet`.
///
/// Source: `oracle/codemp/botlib/be_interface.cpp:202-206`
pub fn Export_BotLibVarSet(bot: &mut BotLib, var_name: *mut c_char, value: *mut c_char) -> c_int {
    LibVarSet(bot, var_name, value);
    BLERR_NOERROR
}

/// Raven `Export_BotLibUpdateEntity`.
///
/// Source: `oracle/codemp/botlib/be_interface.cpp:269-275`
pub fn Export_BotLibUpdateEntity(
    bot: &mut BotLib,
    ent: c_int,
    state: *mut bot_entitystate_t,
) -> c_int {
    if BotLibSetup(bot, c"BotUpdateEntity".as_ptr() as *mut c_char) == qfalse {
        return BLERR_LIBRARYNOTSETUP;
    }
    if ValidEntityNumber(bot, ent, c"BotUpdateEntity".as_ptr() as *mut c_char) == qfalse {
        return BLERR_INVALIDENTITYNUMBER;
    }
    AAS_UpdateEntity(bot, ent, state)
}

/// Raven `Export_BotLibSetup` — the botlib init entry point.
///
/// Source: `oracle/codemp/botlib/be_interface.cpp:119-152`
pub fn Export_BotLibSetup(bot: &mut BotLib) -> c_int {
    unsafe {
        bot.bot_developer = LibVarGetValue(bot, c"bot_developer".as_ptr() as *mut c_char) as c_int;
        libc::memset(
            &mut bot.botlibglobals as *mut botlib_globals_t as *mut c_void,
            0,
            core::mem::size_of::<botlib_globals_t>(),
        );
        // initialize byte swapping (litte endian etc.)
        // Swap_Init();
        Log_Open(bot, c"botlib.log".as_ptr() as *mut c_char);
        //
        // botimport.Print(PRT_MESSAGE, "------- BotLib Initialization -------\n");
        //
        bot.botlibglobals.maxclients = LibVarValue(
            bot,
            c"maxclients".as_ptr() as *mut c_char,
            c"128".as_ptr() as *mut c_char,
        ) as c_int;
        bot.botlibglobals.maxentities = LibVarValue(
            bot,
            c"maxentities".as_ptr() as *mut c_char,
            c"1024".as_ptr() as *mut c_char,
        ) as c_int;

        let mut errnum = AAS_Setup(bot); //be_aas_main.c
        if errnum != BLERR_NOERROR {
            return errnum;
        }
        errnum = EA_Setup(bot); //be_ea.c
        if errnum != BLERR_NOERROR {
            return errnum;
        }
        /*
        errnum = BotSetupWeaponAI();	//be_ai_weap.c
        if errnum != BLERR_NOERROR { return errnum; }
        errnum = BotSetupGoalAI();		//be_ai_goal.c
        if errnum != BLERR_NOERROR { return errnum; }
        errnum = BotSetupChatAI();		//be_ai_chat.c
        if errnum != BLERR_NOERROR { return errnum; }
        errnum = BotSetupMoveAI();		//be_ai_move.c
        if errnum != BLERR_NOERROR { return errnum; }
        */
        bot.botlibsetup = qtrue as c_int;
        bot.botlibglobals.botlibsetup = qtrue as i32;

        BLERR_NOERROR
    }
}

/// Raven `Export_BotLibShutdown`.
///
/// Source: `oracle/codemp/botlib/be_interface.cpp:159-195`
pub fn Export_BotLibShutdown(bot: &mut BotLib) -> c_int {
    if BotLibSetup(bot, c"BotLibShutdown".as_ptr() as *mut c_char) == qfalse {
        return BLERR_LIBRARYNOTSETUP;
    }
    // PORT-NOTE(DEMO): `#ifndef DEMO` guards only a commented-out
    // `DumpFileCRCs()` call — no live code to port either way.
    //
    BotShutdownChatAI(bot); // be_ai_chat.c
    BotShutdownMoveAI(bot); // be_ai_move.c
    BotShutdownGoalAI(bot); // be_ai_goal.c
    BotShutdownWeaponAI(bot); // be_ai_weap.c
    BotShutdownWeights(bot); // be_ai_weight.c
    BotShutdownCharacters(bot); // be_ai_char.c
                                // shut down aas
    AAS_Shutdown(bot);
    // shut down bot elementary actions
    EA_Shutdown(bot);
    // free all libvars
    LibVarDeAllocAll(bot);
    // remove all global defines from the pre compiler
    PC_RemoveAllGlobalDefines(bot);

    // dump all allocated memory
    // DumpMemory();
    // PORT-NOTE(DEBUG): `PrintMemoryLabels()` is `#ifdef DEBUG` — Release
    // build (DEBUG undefined) drops it, per BotExportTest's note above.
    //
    // shut down library log file
    Log_Shutdown(bot);
    //
    bot.botlibsetup = qfalse as c_int;
    bot.botlibglobals.botlibsetup = qfalse as i32;
    // print any files still open
    PC_CheckOpenSourceHandles(bot);
    //
    BLERR_NOERROR
}

/// Raven `Export_BotLibLoadMap`.
///
/// PORT-NOTE(DEBUG): the `Sys_MilliSeconds` timing pair is `#ifdef DEBUG`;
/// dropped for the Release configuration (see BotExportTest's note).
///
/// Source: `oracle/codemp/botlib/be_interface.cpp:239-262`
pub fn Export_BotLibLoadMap(bot: &mut BotLib, mapname: *const c_char) -> c_int {
    unsafe {
        if BotLibSetup(bot, c"BotLoadMap".as_ptr() as *mut c_char) == qfalse {
            return BLERR_LIBRARYNOTSETUP;
        }
        //
        bot.botimport.Print.unwrap()(
            PRT_MESSAGE,
            c"------------ Map Loading ------------\n".as_ptr() as *mut c_char,
        );
        // startup AAS for the current map, model and sound index
        let errnum = AAS_LoadMap(bot, mapname);
        if errnum != BLERR_NOERROR {
            return errnum;
        }
        // initialize the items in the level
        BotInitLevelItems(bot); // be_ai_goal.h
        BotSetBrushModelTypes(bot); // be_ai_move.h
                                    //
        bot.botimport.Print.unwrap()(
            PRT_MESSAGE,
            c"-------------------------------------\n".as_ptr() as *mut c_char,
        );
        //
        BLERR_NOERROR
    }
}

/// Raven `Init_AAS_Export` — populates the AAS export table.
///
/// PORT-NOTE(dispatch-table): see `Init_EA_Export`'s note — the receiver
/// mismatch applies here too.
///
/// Source: `oracle/codemp/botlib/be_interface.cpp:639-686`
pub fn Init_AAS_Export(aas: *mut aas_export_t) {
    unsafe {
        //--------------------------------------------
        // be_aas_entity.c
        //--------------------------------------------
        (*aas).AAS_EntityInfo = Some(AAS_EntityInfo);
        //--------------------------------------------
        // be_aas_main.c
        //--------------------------------------------
        (*aas).AAS_Initialized = Some(AAS_Initialized);
        (*aas).AAS_PresenceTypeBoundingBox = Some(AAS_PresenceTypeBoundingBox);
        (*aas).AAS_Time = Some(AAS_Time);
        //--------------------------------------------
        // be_aas_sample.c
        //--------------------------------------------
        (*aas).AAS_PointAreaNum = Some(AAS_PointAreaNum);
        (*aas).AAS_PointReachabilityAreaIndex = Some(AAS_PointReachabilityAreaIndex);
        (*aas).AAS_TraceAreas = Some(AAS_TraceAreas);
        (*aas).AAS_BBoxAreas = Some(AAS_BBoxAreas);
        (*aas).AAS_AreaInfo = Some(AAS_AreaInfo);
        //--------------------------------------------
        // be_aas_bspq3.c
        //--------------------------------------------
        (*aas).AAS_PointContents = Some(AAS_PointContents);
        (*aas).AAS_NextBSPEntity = Some(AAS_NextBSPEntity);
        (*aas).AAS_ValueForBSPEpairKey = Some(AAS_ValueForBSPEpairKey);
        (*aas).AAS_VectorForBSPEpairKey = Some(AAS_VectorForBSPEpairKey);
        (*aas).AAS_FloatForBSPEpairKey = Some(AAS_FloatForBSPEpairKey);
        (*aas).AAS_IntForBSPEpairKey = Some(AAS_IntForBSPEpairKey);
        //--------------------------------------------
        // be_aas_reach.c
        //--------------------------------------------
        (*aas).AAS_AreaReachability = Some(AAS_AreaReachability);
        //--------------------------------------------
        // be_aas_route.c
        //--------------------------------------------
        (*aas).AAS_AreaTravelTimeToGoalArea = Some(AAS_AreaTravelTimeToGoalArea);
        (*aas).AAS_EnableRoutingArea = Some(AAS_EnableRoutingArea);
        (*aas).AAS_PredictRoute = Some(AAS_PredictRoute);
        //--------------------------------------------
        // be_aas_altroute.c
        //--------------------------------------------
        (*aas).AAS_AlternativeRouteGoals = Some(AAS_AlternativeRouteGoals);
        //--------------------------------------------
        // be_aas_move.c
        //--------------------------------------------
        (*aas).AAS_Swimming = Some(AAS_Swimming);
        (*aas).AAS_PredictClientMovement = Some(AAS_PredictClientMovement);
    }
}

/// Raven `Export_BotLibStartFrame`.
///
/// Source: `oracle/codemp/botlib/be_interface.cpp:228-232`
pub fn Export_BotLibStartFrame(bot: &mut BotLib, time: f32) -> c_int {
    if BotLibSetup(bot, c"BotStartFrame".as_ptr() as *mut c_char) == qfalse {
        return BLERR_LIBRARYNOTSETUP;
    }
    AAS_StartFrame(bot, time)
}

/// Raven `Init_AI_Export` — populates the AI export table.
///
/// PORT-NOTE(dispatch-table): see `Init_EA_Export`'s note — the receiver
/// mismatch applies here too.
///
/// Source: `oracle/codemp/botlib/be_interface.cpp:732-826`
pub fn Init_AI_Export(ai: *mut ai_export_t) {
    unsafe {
        //-----------------------------------
        // be_ai_char.h
        //-----------------------------------
        (*ai).BotLoadCharacter = Some(BotLoadCharacter);
        (*ai).BotFreeCharacter = Some(BotFreeCharacter);
        (*ai).Characteristic_Float = Some(Characteristic_Float);
        (*ai).Characteristic_BFloat = Some(Characteristic_BFloat);
        (*ai).Characteristic_Integer = Some(Characteristic_Integer);
        (*ai).Characteristic_BInteger = Some(Characteristic_BInteger);
        (*ai).Characteristic_String = Some(Characteristic_String);
        //-----------------------------------
        // be_ai_chat.h
        //-----------------------------------
        (*ai).BotAllocChatState = Some(BotAllocChatState);
        (*ai).BotFreeChatState = Some(BotFreeChatState);
        (*ai).BotQueueConsoleMessage = Some(BotQueueConsoleMessage);
        (*ai).BotRemoveConsoleMessage = Some(BotRemoveConsoleMessage);
        (*ai).BotNextConsoleMessage = Some(BotNextConsoleMessage);
        (*ai).BotNumConsoleMessages = Some(BotNumConsoleMessages);
        (*ai).BotInitialChat = Some(BotInitialChat);
        (*ai).BotNumInitialChats = Some(BotNumInitialChats);
        (*ai).BotReplyChat = Some(BotReplyChat);
        (*ai).BotChatLength = Some(BotChatLength);
        (*ai).BotEnterChat = Some(BotEnterChat);
        (*ai).BotGetChatMessage = Some(BotGetChatMessage);
        (*ai).StringContains = Some(StringContains);
        (*ai).BotFindMatch = Some(BotFindMatch);
        (*ai).BotMatchVariable = Some(BotMatchVariable);
        (*ai).UnifyWhiteSpaces = Some(UnifyWhiteSpaces);
        (*ai).BotReplaceSynonyms = Some(BotReplaceSynonyms);
        (*ai).BotLoadChatFile = Some(BotLoadChatFile);
        (*ai).BotSetChatGender = Some(BotSetChatGender);
        (*ai).BotSetChatName = Some(BotSetChatName);
        //-----------------------------------
        // be_ai_goal.h
        //-----------------------------------
        (*ai).BotResetGoalState = Some(BotResetGoalState);
        (*ai).BotResetAvoidGoals = Some(BotResetAvoidGoals);
        (*ai).BotRemoveFromAvoidGoals = Some(BotRemoveFromAvoidGoals);
        (*ai).BotPushGoal = Some(BotPushGoal);
        (*ai).BotPopGoal = Some(BotPopGoal);
        (*ai).BotEmptyGoalStack = Some(BotEmptyGoalStack);
        (*ai).BotDumpAvoidGoals = Some(BotDumpAvoidGoals);
        (*ai).BotDumpGoalStack = Some(BotDumpGoalStack);
        (*ai).BotGoalName = Some(BotGoalName);
        (*ai).BotGetTopGoal = Some(BotGetTopGoal);
        (*ai).BotGetSecondGoal = Some(BotGetSecondGoal);
        (*ai).BotChooseLTGItem = Some(BotChooseLTGItem);
        (*ai).BotChooseNBGItem = Some(BotChooseNBGItem);
        (*ai).BotTouchingGoal = Some(BotTouchingGoal);
        (*ai).BotItemGoalInVisButNotVisible = Some(BotItemGoalInVisButNotVisible);
        (*ai).BotGetLevelItemGoal = Some(BotGetLevelItemGoal);
        (*ai).BotGetNextCampSpotGoal = Some(BotGetNextCampSpotGoal);
        (*ai).BotGetMapLocationGoal = Some(BotGetMapLocationGoal);
        (*ai).BotAvoidGoalTime = Some(BotAvoidGoalTime);
        (*ai).BotSetAvoidGoalTime = Some(BotSetAvoidGoalTime);
        (*ai).BotInitLevelItems = Some(BotInitLevelItems);
        (*ai).BotUpdateEntityItems = Some(BotUpdateEntityItems);
        (*ai).BotLoadItemWeights = Some(BotLoadItemWeights);
        (*ai).BotFreeItemWeights = Some(BotFreeItemWeights);
        (*ai).BotInterbreedGoalFuzzyLogic = Some(BotInterbreedGoalFuzzyLogic);
        (*ai).BotSaveGoalFuzzyLogic = Some(BotSaveGoalFuzzyLogic);
        (*ai).BotMutateGoalFuzzyLogic = Some(BotMutateGoalFuzzyLogic);
        (*ai).BotAllocGoalState = Some(BotAllocGoalState);
        (*ai).BotFreeGoalState = Some(BotFreeGoalState);
        //-----------------------------------
        // be_ai_move.h
        //-----------------------------------
        (*ai).BotResetMoveState = Some(BotResetMoveState);
        (*ai).BotMoveToGoal = Some(BotMoveToGoal);
        (*ai).BotMoveInDirection = Some(BotMoveInDirection);
        (*ai).BotResetAvoidReach = Some(BotResetAvoidReach);
        (*ai).BotResetLastAvoidReach = Some(BotResetLastAvoidReach);
        (*ai).BotReachabilityArea = Some(BotReachabilityArea);
        (*ai).BotMovementViewTarget = Some(BotMovementViewTarget);
        (*ai).BotPredictVisiblePosition = Some(BotPredictVisiblePosition);
        (*ai).BotAllocMoveState = Some(BotAllocMoveState);
        (*ai).BotFreeMoveState = Some(BotFreeMoveState);
        (*ai).BotInitMoveState = Some(BotInitMoveState);
        (*ai).BotAddAvoidSpot = Some(BotAddAvoidSpot);
        //-----------------------------------
        // be_ai_weap.h
        //-----------------------------------
        (*ai).BotChooseBestFightWeapon = Some(BotChooseBestFightWeapon);
        (*ai).BotGetWeaponInfo = Some(BotGetWeaponInfo);
        (*ai).BotLoadWeaponWeights = Some(BotLoadWeaponWeights);
        (*ai).BotAllocWeaponState = Some(BotAllocWeaponState);
        (*ai).BotFreeWeaponState = Some(BotFreeWeaponState);
        (*ai).BotResetWeaponState = Some(BotResetWeaponState);
        //-----------------------------------
        // be_ai_gen.h
        //-----------------------------------
        (*ai).GeneticParentsAndChildSelection = Some(GeneticParentsAndChildSelection);
    }
}

/// Raven `GetBotLibAPI` — the botlib DLL entry point.
///
/// Source: `oracle/codemp/botlib/be_interface.cpp:834-869`
pub fn GetBotLibAPI(
    bot: &mut BotLib,
    apiVersion: c_int,
    import: *mut botlib_import_t,
) -> *mut botlib_export_t {
    unsafe {
        assert!(!import.is_null()); // bk001129 - this wasn't set for base/
        bot.botimport = *import;
        assert!(bot.botimport.Print.is_some()); // bk001129 - pars pro toto

        libc::memset(
            &mut bot.be_botlib_export as *mut botlib_export_t as *mut c_void,
            0,
            core::mem::size_of::<botlib_export_t>(),
        );

        if apiVersion != BOTLIB_API_VERSION {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"Mismatched BOTLIB_API_VERSION: expected %i, got %i\n".as_ptr() as *mut c_char,
                BOTLIB_API_VERSION,
                apiVersion,
            );
            return core::ptr::null_mut();
        }

        Init_AAS_Export(&mut bot.be_botlib_export.aas);
        Init_EA_Export(&mut bot.be_botlib_export.ea);
        Init_AI_Export(&mut bot.be_botlib_export.ai);

        bot.be_botlib_export.BotLibSetup = Some(Export_BotLibSetup);
        bot.be_botlib_export.BotLibShutdown = Some(Export_BotLibShutdown);
        bot.be_botlib_export.BotLibVarSet = Some(Export_BotLibVarSet);
        bot.be_botlib_export.BotLibVarGet = Some(Export_BotLibVarGet);

        bot.be_botlib_export.PC_AddGlobalDefine = Some(PC_AddGlobalDefine);
        bot.be_botlib_export.PC_LoadSourceHandle = Some(PC_LoadSourceHandle);
        bot.be_botlib_export.PC_FreeSourceHandle = Some(PC_FreeSourceHandle);
        bot.be_botlib_export.PC_ReadTokenHandle = Some(PC_ReadTokenHandle);
        bot.be_botlib_export.PC_SourceFileAndLine = Some(PC_SourceFileAndLine);
        bot.be_botlib_export.PC_LoadGlobalDefines = Some(PC_LoadGlobalDefines);
        bot.be_botlib_export.PC_RemoveAllGlobalDefines = Some(PC_RemoveAllGlobalDefines);

        bot.be_botlib_export.BotLibStartFrame = Some(Export_BotLibStartFrame);
        bot.be_botlib_export.BotLibLoadMap = Some(Export_BotLibLoadMap);
        bot.be_botlib_export.BotLibUpdateEntity = Some(Export_BotLibUpdateEntity);
        bot.be_botlib_export.Test = Some(BotExportTest);

        &mut bot.be_botlib_export as *mut botlib_export_t
    }
}

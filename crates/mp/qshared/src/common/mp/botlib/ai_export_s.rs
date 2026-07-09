#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_float, c_int, c_ulong};

use crate::common::mp::qcommon::bot_goal_t;
use crate::shared::vec3_t;

use super::bot_consolemessage_s::bot_consolemessage_s;
use super::bot_initmove_s::bot_initmove_t;
use super::bot_match_s::bot_match_s;
use super::bot_moveresult_s::bot_moveresult_t;
use super::weaponinfo_s::weaponinfo_s;

/// Raven `ai_export_t` — botlib AI function table (character/chat/goal/move/weap/gen
/// exports) the engine hands to the game module.
///
/// Raven: (unnamed).
/// Type definition source: `oracle/codemp/game/botlib.h:289-385`
#[repr(C)]
pub struct ai_export_s {
    //-----------------------------------
    // be_ai_char.h
    //-----------------------------------
    pub BotLoadCharacter:
        Option<unsafe extern "C" fn(charfile: *mut c_char, skill: c_float) -> c_int>,
    pub BotFreeCharacter: Option<unsafe extern "C" fn(character: c_int)>,
    pub Characteristic_Float: Option<unsafe extern "C" fn(character: c_int, index: c_int) -> c_float>,
    pub Characteristic_BFloat: Option<
        unsafe extern "C" fn(character: c_int, index: c_int, min: c_float, max: c_float) -> c_float,
    >,
    pub Characteristic_Integer: Option<unsafe extern "C" fn(character: c_int, index: c_int) -> c_int>,
    pub Characteristic_BInteger: Option<
        unsafe extern "C" fn(character: c_int, index: c_int, min: c_int, max: c_int) -> c_int,
    >,
    pub Characteristic_String: Option<
        unsafe extern "C" fn(character: c_int, index: c_int, buf: *mut c_char, size: c_int),
    >,
    //-----------------------------------
    // be_ai_chat.h
    //-----------------------------------
    pub BotAllocChatState: Option<unsafe extern "C" fn() -> c_int>,
    pub BotFreeChatState: Option<unsafe extern "C" fn(handle: c_int)>,
    pub BotQueueConsoleMessage:
        Option<unsafe extern "C" fn(chatstate: c_int, r#type: c_int, message: *mut c_char)>,
    pub BotRemoveConsoleMessage: Option<unsafe extern "C" fn(chatstate: c_int, handle: c_int)>,
    pub BotNextConsoleMessage: Option<
        unsafe extern "C" fn(chatstate: c_int, cm: *mut bot_consolemessage_s) -> c_int,
    >,
    pub BotNumConsoleMessages: Option<unsafe extern "C" fn(chatstate: c_int) -> c_int>,
    pub BotInitialChat: Option<
        unsafe extern "C" fn(
            chatstate: c_int,
            r#type: *mut c_char,
            mcontext: c_int,
            var0: *mut c_char,
            var1: *mut c_char,
            var2: *mut c_char,
            var3: *mut c_char,
            var4: *mut c_char,
            var5: *mut c_char,
            var6: *mut c_char,
            var7: *mut c_char,
        ),
    >,
    pub BotNumInitialChats: Option<unsafe extern "C" fn(chatstate: c_int, r#type: *mut c_char) -> c_int>,
    pub BotReplyChat: Option<
        unsafe extern "C" fn(
            chatstate: c_int,
            message: *mut c_char,
            mcontext: c_int,
            vcontext: c_int,
            var0: *mut c_char,
            var1: *mut c_char,
            var2: *mut c_char,
            var3: *mut c_char,
            var4: *mut c_char,
            var5: *mut c_char,
            var6: *mut c_char,
            var7: *mut c_char,
        ) -> c_int,
    >,
    pub BotChatLength: Option<unsafe extern "C" fn(chatstate: c_int) -> c_int>,
    pub BotEnterChat: Option<unsafe extern "C" fn(chatstate: c_int, client: c_int, sendto: c_int)>,
    pub BotGetChatMessage: Option<unsafe extern "C" fn(chatstate: c_int, buf: *mut c_char, size: c_int)>,
    pub StringContains: Option<
        unsafe extern "C" fn(str1: *mut c_char, str2: *mut c_char, casesensitive: c_int) -> c_int,
    >,
    pub BotFindMatch: Option<
        unsafe extern "C" fn(str: *mut c_char, r#match: *mut bot_match_s, context: c_ulong) -> c_int,
    >,
    pub BotMatchVariable: Option<
        unsafe extern "C" fn(r#match: *mut bot_match_s, variable: c_int, buf: *mut c_char, size: c_int),
    >,
    pub UnifyWhiteSpaces: Option<unsafe extern "C" fn(string: *mut c_char)>,
    pub BotReplaceSynonyms: Option<unsafe extern "C" fn(string: *mut c_char, context: c_ulong)>,
    pub BotLoadChatFile: Option<
        unsafe extern "C" fn(chatstate: c_int, chatfile: *mut c_char, chatname: *mut c_char) -> c_int,
    >,
    pub BotSetChatGender: Option<unsafe extern "C" fn(chatstate: c_int, gender: c_int)>,
    pub BotSetChatName: Option<unsafe extern "C" fn(chatstate: c_int, name: *mut c_char, client: c_int)>,
    //-----------------------------------
    // be_ai_goal.h
    //-----------------------------------
    pub BotResetGoalState: Option<unsafe extern "C" fn(goalstate: c_int)>,
    pub BotResetAvoidGoals: Option<unsafe extern "C" fn(goalstate: c_int)>,
    pub BotRemoveFromAvoidGoals: Option<unsafe extern "C" fn(goalstate: c_int, number: c_int)>,
    pub BotPushGoal: Option<unsafe extern "C" fn(goalstate: c_int, goal: *mut bot_goal_t)>,
    pub BotPopGoal: Option<unsafe extern "C" fn(goalstate: c_int)>,
    pub BotEmptyGoalStack: Option<unsafe extern "C" fn(goalstate: c_int)>,
    pub BotDumpAvoidGoals: Option<unsafe extern "C" fn(goalstate: c_int)>,
    pub BotDumpGoalStack: Option<unsafe extern "C" fn(goalstate: c_int)>,
    pub BotGoalName: Option<unsafe extern "C" fn(number: c_int, name: *mut c_char, size: c_int)>,
    pub BotGetTopGoal: Option<unsafe extern "C" fn(goalstate: c_int, goal: *mut bot_goal_t) -> c_int>,
    pub BotGetSecondGoal: Option<unsafe extern "C" fn(goalstate: c_int, goal: *mut bot_goal_t) -> c_int>,
    pub BotChooseLTGItem: Option<
        unsafe extern "C" fn(
            goalstate: c_int,
            origin: *const vec3_t,
            inventory: *mut c_int,
            travelflags: c_int,
        ) -> c_int,
    >,
    pub BotChooseNBGItem: Option<
        unsafe extern "C" fn(
            goalstate: c_int,
            origin: *const vec3_t,
            inventory: *mut c_int,
            travelflags: c_int,
            ltg: *mut bot_goal_t,
            maxtime: c_float,
        ) -> c_int,
    >,
    pub BotTouchingGoal:
        Option<unsafe extern "C" fn(origin: *const vec3_t, goal: *mut bot_goal_t) -> c_int>,
    pub BotItemGoalInVisButNotVisible: Option<
        unsafe extern "C" fn(
            viewer: c_int,
            eye: *const vec3_t,
            viewangles: *const vec3_t,
            goal: *mut bot_goal_t,
        ) -> c_int,
    >,
    pub BotGetLevelItemGoal: Option<
        unsafe extern "C" fn(index: c_int, classname: *mut c_char, goal: *mut bot_goal_t) -> c_int,
    >,
    pub BotGetNextCampSpotGoal:
        Option<unsafe extern "C" fn(num: c_int, goal: *mut bot_goal_t) -> c_int>,
    pub BotGetMapLocationGoal:
        Option<unsafe extern "C" fn(name: *mut c_char, goal: *mut bot_goal_t) -> c_int>,
    pub BotAvoidGoalTime: Option<unsafe extern "C" fn(goalstate: c_int, number: c_int) -> c_float>,
    pub BotSetAvoidGoalTime:
        Option<unsafe extern "C" fn(goalstate: c_int, number: c_int, avoidtime: c_float)>,
    pub BotInitLevelItems: Option<unsafe extern "C" fn()>,
    pub BotUpdateEntityItems: Option<unsafe extern "C" fn()>,
    pub BotLoadItemWeights: Option<unsafe extern "C" fn(goalstate: c_int, filename: *mut c_char) -> c_int>,
    pub BotFreeItemWeights: Option<unsafe extern "C" fn(goalstate: c_int)>,
    pub BotInterbreedGoalFuzzyLogic:
        Option<unsafe extern "C" fn(parent1: c_int, parent2: c_int, child: c_int)>,
    pub BotSaveGoalFuzzyLogic: Option<unsafe extern "C" fn(goalstate: c_int, filename: *mut c_char)>,
    pub BotMutateGoalFuzzyLogic: Option<unsafe extern "C" fn(goalstate: c_int, range: c_float)>,
    pub BotAllocGoalState: Option<unsafe extern "C" fn(client: c_int) -> c_int>,
    pub BotFreeGoalState: Option<unsafe extern "C" fn(handle: c_int)>,
    //-----------------------------------
    // be_ai_move.h
    //-----------------------------------
    pub BotResetMoveState: Option<unsafe extern "C" fn(movestate: c_int)>,
    pub BotMoveToGoal: Option<
        unsafe extern "C" fn(
            result: *mut bot_moveresult_t,
            movestate: c_int,
            goal: *mut bot_goal_t,
            travelflags: c_int,
        ),
    >,
    pub BotMoveInDirection: Option<
        unsafe extern "C" fn(
            movestate: c_int,
            dir: *const vec3_t,
            speed: c_float,
            r#type: c_int,
        ) -> c_int,
    >,
    pub BotResetAvoidReach: Option<unsafe extern "C" fn(movestate: c_int)>,
    pub BotResetLastAvoidReach: Option<unsafe extern "C" fn(movestate: c_int)>,
    pub BotReachabilityArea: Option<unsafe extern "C" fn(origin: *const vec3_t, testground: c_int) -> c_int>,
    pub BotMovementViewTarget: Option<
        unsafe extern "C" fn(
            movestate: c_int,
            goal: *mut bot_goal_t,
            travelflags: c_int,
            lookahead: c_float,
            target: *mut vec3_t,
        ) -> c_int,
    >,
    pub BotPredictVisiblePosition: Option<
        unsafe extern "C" fn(
            origin: *const vec3_t,
            areanum: c_int,
            goal: *mut bot_goal_t,
            travelflags: c_int,
            target: *mut vec3_t,
        ) -> c_int,
    >,
    pub BotAllocMoveState: Option<unsafe extern "C" fn() -> c_int>,
    pub BotFreeMoveState: Option<unsafe extern "C" fn(handle: c_int)>,
    pub BotInitMoveState: Option<unsafe extern "C" fn(handle: c_int, initmove: *mut bot_initmove_t)>,
    pub BotAddAvoidSpot: Option<
        unsafe extern "C" fn(movestate: c_int, origin: *const vec3_t, radius: c_float, r#type: c_int),
    >,
    //-----------------------------------
    // be_ai_weap.h
    //-----------------------------------
    pub BotChooseBestFightWeapon:
        Option<unsafe extern "C" fn(weaponstate: c_int, inventory: *mut c_int) -> c_int>,
    pub BotGetWeaponInfo: Option<
        unsafe extern "C" fn(weaponstate: c_int, weapon: c_int, weaponinfo: *mut weaponinfo_s),
    >,
    pub BotLoadWeaponWeights:
        Option<unsafe extern "C" fn(weaponstate: c_int, filename: *mut c_char) -> c_int>,
    pub BotAllocWeaponState: Option<unsafe extern "C" fn() -> c_int>,
    pub BotFreeWeaponState: Option<unsafe extern "C" fn(weaponstate: c_int)>,
    pub BotResetWeaponState: Option<unsafe extern "C" fn(weaponstate: c_int)>,
    //-----------------------------------
    // be_ai_gen.h
    //-----------------------------------
    pub GeneticParentsAndChildSelection: Option<
        unsafe extern "C" fn(
            numranks: c_int,
            ranks: *mut c_float,
            parent1: *mut c_int,
            parent2: *mut c_int,
            child: *mut c_int,
        ),
    >,
}

/// Raven `ai_export_t` typedef alias.
pub type ai_export_t = ai_export_s;

const _: () = assert!(core::mem::size_of::<ai_export_t>() == 600);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotLoadCharacter) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotFreeCharacter) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, Characteristic_Float) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, Characteristic_BFloat) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, Characteristic_Integer) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, Characteristic_BInteger) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, Characteristic_String) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotAllocChatState) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotFreeChatState) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotQueueConsoleMessage) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotRemoveConsoleMessage) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotNextConsoleMessage) == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotNumConsoleMessages) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotInitialChat) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotNumInitialChats) == 112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotReplyChat) == 120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotChatLength) == 128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotEnterChat) == 136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotGetChatMessage) == 144);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, StringContains) == 152);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotFindMatch) == 160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotMatchVariable) == 168);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, UnifyWhiteSpaces) == 176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotReplaceSynonyms) == 184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotLoadChatFile) == 192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotSetChatGender) == 200);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotSetChatName) == 208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotResetGoalState) == 216);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotResetAvoidGoals) == 224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotRemoveFromAvoidGoals) == 232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotPushGoal) == 240);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotPopGoal) == 248);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotEmptyGoalStack) == 256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotDumpAvoidGoals) == 264);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotDumpGoalStack) == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotGoalName) == 280);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotGetTopGoal) == 288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotGetSecondGoal) == 296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotChooseLTGItem) == 304);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotChooseNBGItem) == 312);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotTouchingGoal) == 320);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotItemGoalInVisButNotVisible) == 328);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotGetLevelItemGoal) == 336);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotGetNextCampSpotGoal) == 344);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotGetMapLocationGoal) == 352);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotAvoidGoalTime) == 360);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotSetAvoidGoalTime) == 368);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotInitLevelItems) == 376);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotUpdateEntityItems) == 384);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotLoadItemWeights) == 392);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotFreeItemWeights) == 400);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotInterbreedGoalFuzzyLogic) == 408);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotSaveGoalFuzzyLogic) == 416);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotMutateGoalFuzzyLogic) == 424);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotAllocGoalState) == 432);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotFreeGoalState) == 440);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotResetMoveState) == 448);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotMoveToGoal) == 456);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotMoveInDirection) == 464);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotResetAvoidReach) == 472);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotResetLastAvoidReach) == 480);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotReachabilityArea) == 488);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotMovementViewTarget) == 496);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotPredictVisiblePosition) == 504);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotAllocMoveState) == 512);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotFreeMoveState) == 520);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotInitMoveState) == 528);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotAddAvoidSpot) == 536);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotChooseBestFightWeapon) == 544);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotGetWeaponInfo) == 552);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotLoadWeaponWeights) == 560);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotAllocWeaponState) == 568);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotFreeWeaponState) == 576);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, BotResetWeaponState) == 584);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(ai_export_t, GeneticParentsAndChildSelection) == 592);

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_ulong};

use crate::BotLib;
use mp_engine_qcommon::common::Common;
use mp_qshared::common::mp::botlib::bot_consolemessage_s::bot_consolemessage_t;
use mp_qshared::common::mp::botlib::bot_initmove_s::bot_initmove_t;
use mp_qshared::common::mp::botlib::bot_match_s::bot_match_t;
use mp_qshared::common::mp::botlib::bot_moveresult_s::bot_moveresult_t;
use mp_qshared::common::mp::botlib::weaponinfo_s::weaponinfo_t;
use mp_qshared::common::mp::qcommon::bot_goal::bot_goal_t;
use mp_qshared::shared::vec3_t;

/// Raven `ai_export_t` — botlib AI function table (character/chat/goal/move/weap/gen
/// exports) the engine hands to the game module.
///
/// Raven: (unnamed).
/// Type definition source: `oracle/codemp/game/botlib.h:289-385`
//
// Engine-internal per the 2026-07-11 ruling: statically linked in jampDed, no
// ABI crossing, layout free. Fn-pointer fields carry the ported `&mut BotLib`
// (and `&mut Common`, where the ported fn takes it) receiver — the stored fn's
// real signature is LAW.
pub struct ai_export_s {
    //-----------------------------------
    // be_ai_char.h
    //-----------------------------------
    pub BotLoadCharacter: Option<fn(bot: &mut BotLib, charfile: *mut c_char, skill: f32) -> c_int>,
    pub BotFreeCharacter: Option<fn(bot: &mut BotLib, handle: c_int)>,
    pub Characteristic_Float: Option<fn(bot: &mut BotLib, character: c_int, index: c_int) -> f32>,
    pub Characteristic_BFloat: Option<
        fn(bot: &mut BotLib, character: c_int, index: c_int, min: f32, max: f32) -> f32,
    >,
    pub Characteristic_Integer:
        Option<fn(bot: &mut BotLib, character: c_int, index: c_int) -> c_int>,
    pub Characteristic_BInteger: Option<
        fn(bot: &mut BotLib, character: c_int, index: c_int, min: c_int, max: c_int) -> c_int,
    >,
    pub Characteristic_String:
        Option<fn(bot: &mut BotLib, character: c_int, index: c_int, buf: *mut c_char, size: c_int)>,
    //-----------------------------------
    // be_ai_chat.h
    //-----------------------------------
    pub BotAllocChatState: Option<fn(bot: &mut BotLib) -> c_int>,
    pub BotFreeChatState: Option<fn(bot: &mut BotLib, handle: c_int)>,
    pub BotQueueConsoleMessage:
        Option<fn(bot: &mut BotLib, chatstate: c_int, r#type: c_int, message: *mut c_char)>,
    pub BotRemoveConsoleMessage: Option<fn(bot: &mut BotLib, chatstate: c_int, handle: c_int)>,
    pub BotNextConsoleMessage:
        Option<fn(bot: &mut BotLib, chatstate: c_int, cm: *mut bot_consolemessage_t) -> c_int>,
    pub BotNumConsoleMessages: Option<fn(bot: &mut BotLib, chatstate: c_int) -> c_int>,
    pub BotInitialChat: Option<
        fn(
            common: &mut Common,
            bot: &mut BotLib,
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
    pub BotNumInitialChats: Option<fn(bot: &mut BotLib, chatstate: c_int, r#type: *mut c_char) -> c_int>,
    pub BotReplyChat: Option<
        fn(
            common: &mut Common,
            bot: &mut BotLib,
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
    pub BotChatLength: Option<fn(bot: &mut BotLib, chatstate: c_int) -> c_int>,
    pub BotEnterChat: Option<fn(bot: &mut BotLib, chatstate: c_int, clientto: c_int, sendto: c_int)>,
    pub BotGetChatMessage:
        Option<fn(bot: &mut BotLib, chatstate: c_int, buf: *mut c_char, size: c_int)>,
    pub StringContains:
        Option<fn(str1: *mut c_char, str2: *mut c_char, casesensitive: c_int) -> c_int>,
    pub BotFindMatch: Option<
        fn(bot: &mut BotLib, str: *mut c_char, r#match: *mut bot_match_t, context: c_ulong) -> c_int,
    >,
    pub BotMatchVariable: Option<
        fn(
            bot: &mut BotLib,
            r#match: *mut bot_match_t,
            variable: c_int,
            buf: *mut c_char,
            size: c_int,
        ),
    >,
    pub UnifyWhiteSpaces: Option<fn(string: *mut c_char)>,
    pub BotReplaceSynonyms: Option<fn(bot: &mut BotLib, string: *mut c_char, context: c_ulong)>,
    pub BotLoadChatFile: Option<
        fn(
            common: &mut Common,
            bot: &mut BotLib,
            chatstate: c_int,
            chatfile: *mut c_char,
            chatname: *mut c_char,
        ) -> c_int,
    >,
    pub BotSetChatGender: Option<fn(bot: &mut BotLib, chatstate: c_int, gender: c_int)>,
    pub BotSetChatName: Option<fn(bot: &mut BotLib, chatstate: c_int, name: *mut c_char, client: c_int)>,
    //-----------------------------------
    // be_ai_goal.h
    //-----------------------------------
    pub BotResetGoalState: Option<fn(bot: &mut BotLib, goalstate: c_int)>,
    pub BotResetAvoidGoals: Option<fn(bot: &mut BotLib, goalstate: c_int)>,
    pub BotRemoveFromAvoidGoals: Option<fn(bot: &mut BotLib, goalstate: c_int, number: c_int)>,
    pub BotPushGoal: Option<fn(bot: &mut BotLib, goalstate: c_int, goal: *mut bot_goal_t)>,
    pub BotPopGoal: Option<fn(bot: &mut BotLib, goalstate: c_int)>,
    pub BotEmptyGoalStack: Option<fn(bot: &mut BotLib, goalstate: c_int)>,
    pub BotDumpAvoidGoals: Option<fn(bot: &mut BotLib, goalstate: c_int)>,
    pub BotDumpGoalStack: Option<fn(bot: &mut BotLib, goalstate: c_int)>,
    pub BotGoalName: Option<fn(bot: &mut BotLib, number: c_int, name: *mut c_char, size: c_int)>,
    pub BotGetTopGoal: Option<fn(bot: &mut BotLib, goalstate: c_int, goal: *mut bot_goal_t) -> c_int>,
    pub BotGetSecondGoal:
        Option<fn(bot: &mut BotLib, goalstate: c_int, goal: *mut bot_goal_t) -> c_int>,
    pub BotChooseLTGItem: Option<
        fn(
            common: &mut Common,
            bot: &mut BotLib,
            goalstate: c_int,
            origin: vec3_t,
            inventory: *mut c_int,
            travelflags: c_int,
        ) -> c_int,
    >,
    pub BotChooseNBGItem: Option<
        fn(
            common: &mut Common,
            bot: &mut BotLib,
            goalstate: c_int,
            origin: vec3_t,
            inventory: *mut c_int,
            travelflags: c_int,
            ltg: *mut bot_goal_t,
            maxtime: f32,
        ) -> c_int,
    >,
    pub BotTouchingGoal: Option<fn(bot: &mut BotLib, origin: vec3_t, goal: *mut bot_goal_t) -> c_int>,
    pub BotItemGoalInVisButNotVisible: Option<
        fn(
            bot: &mut BotLib,
            viewer: c_int,
            eye: vec3_t,
            viewangles: vec3_t,
            goal: *mut bot_goal_t,
        ) -> c_int,
    >,
    pub BotGetLevelItemGoal: Option<
        fn(bot: &mut BotLib, index: c_int, classname: *mut c_char, goal: *mut bot_goal_t) -> c_int,
    >,
    pub BotGetNextCampSpotGoal: Option<fn(bot: &mut BotLib, num: c_int, goal: *mut bot_goal_t) -> c_int>,
    pub BotGetMapLocationGoal:
        Option<fn(bot: &mut BotLib, name: *mut c_char, goal: *mut bot_goal_t) -> c_int>,
    pub BotAvoidGoalTime: Option<fn(bot: &mut BotLib, goalstate: c_int, number: c_int) -> f32>,
    pub BotSetAvoidGoalTime:
        Option<fn(bot: &mut BotLib, goalstate: c_int, number: c_int, avoidtime: f32)>,
    pub BotInitLevelItems: Option<fn(bot: &mut BotLib)>,
    pub BotUpdateEntityItems: Option<fn(bot: &mut BotLib)>,
    pub BotLoadItemWeights: Option<fn(bot: &mut BotLib, goalstate: c_int, filename: *mut c_char) -> c_int>,
    pub BotFreeItemWeights: Option<fn(bot: &mut BotLib, goalstate: c_int)>,
    pub BotInterbreedGoalFuzzyLogic:
        Option<fn(bot: &mut BotLib, parent1: c_int, parent2: c_int, child: c_int)>,
    pub BotSaveGoalFuzzyLogic: Option<fn(bot: &mut BotLib, goalstate: c_int, filename: *mut c_char)>,
    pub BotMutateGoalFuzzyLogic:
        Option<fn(common: &mut Common, bot: &mut BotLib, goalstate: c_int, range: f32)>,
    pub BotAllocGoalState: Option<fn(bot: &mut BotLib, client: c_int) -> c_int>,
    pub BotFreeGoalState: Option<fn(bot: &mut BotLib, handle: c_int)>,
    //-----------------------------------
    // be_ai_move.h
    //-----------------------------------
    pub BotResetMoveState: Option<fn(bot: &mut BotLib, movestate: c_int)>,
    pub BotMoveToGoal: Option<
        fn(
            common: &mut Common,
            bot: &mut BotLib,
            result: *mut bot_moveresult_t,
            movestate: c_int,
            goal: *mut bot_goal_t,
            travelflags: c_int,
        ),
    >,
    pub BotMoveInDirection: Option<
        fn(bot: &mut BotLib, movestate: c_int, dir: vec3_t, speed: f32, r#type: c_int) -> c_int,
    >,
    pub BotResetAvoidReach: Option<fn(bot: &mut BotLib, movestate: c_int)>,
    pub BotResetLastAvoidReach: Option<fn(bot: &mut BotLib, movestate: c_int)>,
    pub BotReachabilityArea: Option<fn(bot: &mut BotLib, origin: vec3_t, testground: c_int) -> c_int>,
    pub BotMovementViewTarget: Option<
        fn(
            bot: &mut BotLib,
            movestate: c_int,
            goal: *mut bot_goal_t,
            travelflags: c_int,
            lookahead: f32,
            target: vec3_t,
        ) -> c_int,
    >,
    pub BotPredictVisiblePosition: Option<
        fn(
            bot: &mut BotLib,
            origin: vec3_t,
            areanum: c_int,
            goal: *mut bot_goal_t,
            travelflags: c_int,
            target: vec3_t,
        ) -> c_int,
    >,
    pub BotAllocMoveState: Option<fn(bot: &mut BotLib) -> c_int>,
    pub BotFreeMoveState: Option<fn(bot: &mut BotLib, handle: c_int)>,
    pub BotInitMoveState: Option<fn(bot: &mut BotLib, handle: c_int, initmove: *mut bot_initmove_t)>,
    pub BotAddAvoidSpot: Option<
        fn(bot: &mut BotLib, movestate: c_int, origin: vec3_t, radius: f32, r#type: c_int),
    >,
    //-----------------------------------
    // be_ai_weap.h
    //-----------------------------------
    pub BotChooseBestFightWeapon:
        Option<fn(bot: &mut BotLib, weaponstate: c_int, inventory: *mut c_int) -> c_int>,
    pub BotGetWeaponInfo: Option<
        fn(bot: &mut BotLib, weaponstate: c_int, weapon: c_int, weaponinfo: *mut weaponinfo_t),
    >,
    pub BotLoadWeaponWeights:
        Option<fn(bot: &mut BotLib, weaponstate: c_int, filename: *mut c_char) -> c_int>,
    pub BotAllocWeaponState: Option<fn(bot: &mut BotLib) -> c_int>,
    pub BotFreeWeaponState: Option<fn(bot: &mut BotLib, weaponstate: c_int)>,
    pub BotResetWeaponState: Option<fn(bot: &mut BotLib, weaponstate: c_int)>,
    //-----------------------------------
    // be_ai_gen.h
    //-----------------------------------
    pub GeneticParentsAndChildSelection: Option<
        fn(
            common: &mut Common,
            bot: &mut BotLib,
            numranks: c_int,
            ranks: *mut f32,
            parent1: *mut c_int,
            parent2: *mut c_int,
            child: *mut c_int,
        ) -> c_int,
    >,
}

/// Raven `ai_export_t` typedef alias.
pub type ai_export_t = ai_export_s;

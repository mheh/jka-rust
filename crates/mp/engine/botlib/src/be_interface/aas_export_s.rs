#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use crate::BotLib;
use mp_qshared::common::mp::botlib::aas_altroutegoal_s::aas_altroutegoal_t;
use mp_qshared::common::mp::botlib::aas_clientmove_s::aas_clientmove_s;
use mp_qshared::common::mp::botlib::aas_entityinfo_s::aas_entityinfo_t;
use mp_qshared::common::mp::botlib::aas_predictroute_s::aas_predictroute_s;
use mp_qshared::common::mp::qcommon::aas_areainfo::aas_areainfo_t;
use mp_qshared::shared::vec3_t;

/// Raven `aas_export_t` — AAS (Area Awareness System) function table exported by
/// the botlib to the game/cgame.
///
/// Type definition source: `oracle/codemp/game/botlib.h:195-253`
//
// Engine-internal per the 2026-07-11 ruling: statically linked in jampDed, no
// ABI crossing, layout free. Fn-pointer fields carry the ported `&mut BotLib`
// receiver (the stored fn's real signature is LAW).
pub struct aas_export_s {
    //-----------------------------------
    // be_aas_entity.h
    //-----------------------------------
    pub AAS_EntityInfo: Option<fn(bot: &mut BotLib, entnum: c_int, info: *mut aas_entityinfo_t)>,
    //-----------------------------------
    // be_aas_main.h
    //-----------------------------------
    pub AAS_Initialized: Option<fn(bot: &mut BotLib) -> c_int>,
    pub AAS_PresenceTypeBoundingBox:
        Option<fn(bot: &mut BotLib, presencetype: c_int, mins: vec3_t, maxs: vec3_t)>,
    pub AAS_Time: Option<fn(bot: &mut BotLib) -> f32>,
    //--------------------------------------------
    // be_aas_sample.c
    //--------------------------------------------
    pub AAS_PointAreaNum: Option<fn(bot: &mut BotLib, point: vec3_t) -> c_int>,
    pub AAS_PointReachabilityAreaIndex: Option<fn(bot: &mut BotLib, origin: vec3_t) -> c_int>,
    pub AAS_TraceAreas: Option<
        fn(
            bot: &mut BotLib,
            start: vec3_t,
            end: vec3_t,
            areas: *mut c_int,
            points: *mut vec3_t,
            maxareas: c_int,
        ) -> c_int,
    >,
    pub AAS_BBoxAreas: Option<
        fn(
            bot: &mut BotLib,
            absmins: vec3_t,
            absmaxs: vec3_t,
            areas: *mut c_int,
            maxareas: c_int,
        ) -> c_int,
    >,
    pub AAS_AreaInfo:
        Option<fn(bot: &mut BotLib, areanum: c_int, info: *mut aas_areainfo_t) -> c_int>,
    //--------------------------------------------
    // be_aas_bspq3.c
    //--------------------------------------------
    pub AAS_PointContents: Option<fn(bot: &mut BotLib, point: vec3_t) -> c_int>,
    pub AAS_NextBSPEntity: Option<fn(bot: &mut BotLib, ent: c_int) -> c_int>,
    pub AAS_ValueForBSPEpairKey: Option<
        fn(
            bot: &mut BotLib,
            ent: c_int,
            key: *mut c_char,
            value: *mut c_char,
            size: c_int,
        ) -> c_int,
    >,
    pub AAS_VectorForBSPEpairKey:
        Option<fn(bot: &mut BotLib, ent: c_int, key: *mut c_char, v: vec3_t) -> c_int>,
    pub AAS_FloatForBSPEpairKey:
        Option<fn(bot: &mut BotLib, ent: c_int, key: *mut c_char, value: *mut f32) -> c_int>,
    pub AAS_IntForBSPEpairKey:
        Option<fn(bot: &mut BotLib, ent: c_int, key: *mut c_char, value: *mut c_int) -> c_int>,
    //--------------------------------------------
    // be_aas_reach.c
    //--------------------------------------------
    pub AAS_AreaReachability: Option<fn(bot: &mut BotLib, areanum: c_int) -> c_int>,
    //--------------------------------------------
    // be_aas_route.c
    //--------------------------------------------
    pub AAS_AreaTravelTimeToGoalArea: Option<
        fn(
            bot: &mut BotLib,
            areanum: c_int,
            origin: vec3_t,
            goalareanum: c_int,
            travelflags: c_int,
        ) -> c_int,
    >,
    pub AAS_EnableRoutingArea: Option<fn(bot: &mut BotLib, areanum: c_int, enable: c_int) -> c_int>,
    pub AAS_PredictRoute: Option<
        fn(
            bot: &mut BotLib,
            route: *mut aas_predictroute_s,
            areanum: c_int,
            origin: vec3_t,
            goalareanum: c_int,
            travelflags: c_int,
            maxareas: c_int,
            maxtime: c_int,
            stopevent: c_int,
            stopcontents: c_int,
            stoptfl: c_int,
            stopareanum: c_int,
        ) -> c_int,
    >,
    //--------------------------------------------
    // be_aas_altroute.c
    //--------------------------------------------
    pub AAS_AlternativeRouteGoals: Option<
        fn(
            bot: &mut BotLib,
            start: vec3_t,
            startareanum: c_int,
            goal: vec3_t,
            goalareanum: c_int,
            travelflags: c_int,
            altroutegoals: *mut aas_altroutegoal_t,
            maxaltroutegoals: c_int,
            r#type: c_int,
        ) -> c_int,
    >,
    //--------------------------------------------
    // be_aas_move.c
    //--------------------------------------------
    pub AAS_Swimming: Option<fn(bot: &mut BotLib, origin: vec3_t) -> c_int>,
    pub AAS_PredictClientMovement: Option<
        fn(
            bot: &mut BotLib,
            r#move: *mut aas_clientmove_s,
            entnum: c_int,
            origin: vec3_t,
            presencetype: c_int,
            onground: c_int,
            velocity: vec3_t,
            cmdmove: vec3_t,
            cmdframes: c_int,
            maxframes: c_int,
            frametime: f32,
            stopevent: c_int,
            stopareanum: c_int,
            visualize: c_int,
        ) -> c_int,
    >,
}

/// Raven `aas_export_t` typedef alias.
pub type aas_export_t = aas_export_s;

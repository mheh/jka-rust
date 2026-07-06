#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_float, c_int};

use crate::common::mp::qcommon::aas_areainfo_t;
use crate::shared::vec3_t;

use super::aas_altroutegoal_s::aas_altroutegoal_t;
use super::aas_clientmove_s::aas_clientmove_t;
use super::aas_entityinfo_s::aas_entityinfo_t;
use super::aas_predictroute_s::aas_predictroute_t;

/// Raven `aas_export_t` — AAS (Area Awareness System) function table exported by
/// the botlib to the game/cgame.
///
/// Type definition source: `oracle/oracle/codemp/game/botlib.h:195-253`
#[repr(C)]
pub struct aas_export_s {
    //-----------------------------------
    // be_aas_entity.h
    //-----------------------------------
    pub AAS_EntityInfo: Option<unsafe extern "C" fn(entnum: c_int, info: *mut aas_entityinfo_t)>,
    //-----------------------------------
    // be_aas_main.h
    //-----------------------------------
    pub AAS_Initialized: Option<unsafe extern "C" fn() -> c_int>,
    pub AAS_PresenceTypeBoundingBox: Option<
        unsafe extern "C" fn(presencetype: c_int, mins: *mut vec3_t, maxs: *mut vec3_t),
    >,
    pub AAS_Time: Option<unsafe extern "C" fn() -> c_float>,
    //--------------------------------------------
    // be_aas_sample.c
    //--------------------------------------------
    pub AAS_PointAreaNum: Option<unsafe extern "C" fn(point: *mut vec3_t) -> c_int>,
    pub AAS_PointReachabilityAreaIndex: Option<unsafe extern "C" fn(point: *mut vec3_t) -> c_int>,
    pub AAS_TraceAreas: Option<
        unsafe extern "C" fn(
            start: *mut vec3_t,
            end: *mut vec3_t,
            areas: *mut c_int,
            points: *mut vec3_t,
            maxareas: c_int,
        ) -> c_int,
    >,
    pub AAS_BBoxAreas: Option<
        unsafe extern "C" fn(
            absmins: *mut vec3_t,
            absmaxs: *mut vec3_t,
            areas: *mut c_int,
            maxareas: c_int,
        ) -> c_int,
    >,
    pub AAS_AreaInfo:
        Option<unsafe extern "C" fn(areanum: c_int, info: *mut aas_areainfo_t) -> c_int>,
    //--------------------------------------------
    // be_aas_bspq3.c
    //--------------------------------------------
    pub AAS_PointContents: Option<unsafe extern "C" fn(point: *mut vec3_t) -> c_int>,
    pub AAS_NextBSPEntity: Option<unsafe extern "C" fn(ent: c_int) -> c_int>,
    pub AAS_ValueForBSPEpairKey: Option<
        unsafe extern "C" fn(
            ent: c_int,
            key: *mut c_char,
            value: *mut c_char,
            size: c_int,
        ) -> c_int,
    >,
    pub AAS_VectorForBSPEpairKey:
        Option<unsafe extern "C" fn(ent: c_int, key: *mut c_char, v: *mut vec3_t) -> c_int>,
    pub AAS_FloatForBSPEpairKey:
        Option<unsafe extern "C" fn(ent: c_int, key: *mut c_char, value: *mut c_float) -> c_int>,
    pub AAS_IntForBSPEpairKey:
        Option<unsafe extern "C" fn(ent: c_int, key: *mut c_char, value: *mut c_int) -> c_int>,
    //--------------------------------------------
    // be_aas_reach.c
    //--------------------------------------------
    pub AAS_AreaReachability: Option<unsafe extern "C" fn(areanum: c_int) -> c_int>,
    //--------------------------------------------
    // be_aas_route.c
    //--------------------------------------------
    pub AAS_AreaTravelTimeToGoalArea: Option<
        unsafe extern "C" fn(
            areanum: c_int,
            origin: *mut vec3_t,
            goalareanum: c_int,
            travelflags: c_int,
        ) -> c_int,
    >,
    pub AAS_EnableRoutingArea:
        Option<unsafe extern "C" fn(areanum: c_int, enable: c_int) -> c_int>,
    pub AAS_PredictRoute: Option<
        unsafe extern "C" fn(
            route: *mut aas_predictroute_t,
            areanum: c_int,
            origin: *mut vec3_t,
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
        unsafe extern "C" fn(
            start: *mut vec3_t,
            startareanum: c_int,
            goal: *mut vec3_t,
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
    pub AAS_Swimming: Option<unsafe extern "C" fn(origin: *mut vec3_t) -> c_int>,
    pub AAS_PredictClientMovement: Option<
        unsafe extern "C" fn(
            r#move: *mut aas_clientmove_t,
            entnum: c_int,
            origin: *mut vec3_t,
            presencetype: c_int,
            onground: c_int,
            velocity: *mut vec3_t,
            cmdmove: *mut vec3_t,
            cmdframes: c_int,
            maxframes: c_int,
            frametime: c_float,
            stopevent: c_int,
            stopareanum: c_int,
            visualize: c_int,
        ) -> c_int,
    >,
}

/// Raven `aas_export_t` typedef alias.
pub type aas_export_t = aas_export_s;

const _: () = assert!(core::mem::size_of::<aas_export_t>() == 176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_EntityInfo) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_Initialized) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_PresenceTypeBoundingBox) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_Time) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_PointAreaNum) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_PointReachabilityAreaIndex) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_TraceAreas) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_BBoxAreas) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_AreaInfo) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_PointContents) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_NextBSPEntity) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_ValueForBSPEpairKey) == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_VectorForBSPEpairKey) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_FloatForBSPEpairKey) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_IntForBSPEpairKey) == 112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_AreaReachability) == 120);
#[cfg(target_pointer_width = "64")]
const _: () =
    assert!(core::mem::offset_of!(aas_export_t, AAS_AreaTravelTimeToGoalArea) == 128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_EnableRoutingArea) == 136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_PredictRoute) == 144);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_AlternativeRouteGoals) == 152);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_Swimming) == 160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(aas_export_t, AAS_PredictClientMovement) == 168);

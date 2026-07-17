#![allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_assignments
)]

//! Function bodies for Raven's `be_aas_entity.cpp` (AAS entity tracking:
//! per-entity info snapshots, area/BSP-leaf linking, nearest/next-entity
//! queries).
//!
//! Ported per the engine C-track packets (`botlib__0422`..`botlib__1466`).
//! Source: `oracle/codemp/botlib/be_aas_entity.cpp`.

use core::ffi::c_char;
use core::ffi::c_int;

use mp_qshared::common::mp::botlib::aas_entityinfo_s::aas_entityinfo_t;
use mp_qshared::common::mp::botlib::bot_entitystate_s::bot_entitystate_t;
use mp_qshared::common::mp::botlib::botlib_error::{BLERR_NOAASFILE, BLERR_NOERROR};
use mp_qshared::common::mp::botlib::print_type::{PRT_FATAL, PRT_MESSAGE};
use mp_qshared::common::mp::botlib::solid_t::solid_t;
use mp_qshared::shared::limits::ENTITYNUM_WORLD;
use mp_qshared::shared::vec3_t;

use crate::aasfile::presence_type::PRESENCE_NORMAL;
use crate::be_aas_def::bsp_entdata_s::bsp_entdata_t;
use crate::BotLib;

use mp_engine_qcommon::common_fns::{Com_Memcpy, Com_Memset};

use crate::be_aas_bspq3_fns::{
    AAS_BSPLinkEntity, AAS_BSPModelMinsMaxsOrigin, AAS_UnlinkFromBSPLeaves,
};
use crate::be_aas_main::AAS_Time;
use crate::be_aas_reach_fns::AAS_BestReachableLinkArea;
use crate::be_aas_sample_fns::{AAS_LinkEntityClientBBox, AAS_UnlinkFromAreas};

/// Raven `AAS_EntityOrigin`.
///
/// Source: `oracle/codemp/botlib/be_aas_entity.cpp:178-188`
pub fn AAS_EntityOrigin(bot: &mut BotLib, entnum: c_int, origin: *mut vec3_t) {
    unsafe {
        if entnum < 0 || entnum >= bot.aasworld.maxentities {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"AAS_EntityOrigin: entnum %d out of range\n".as_ptr() as *mut c_char,
                entnum,
            );
            (*origin)[0] = 0.0;
            (*origin)[1] = 0.0;
            (*origin)[2] = 0.0;
            return;
        } //end if

        let ent = &*bot.aasworld.entities.add(entnum as usize);
        (*origin)[0] = ent.i.origin[0];
        (*origin)[1] = ent.i.origin[1];
        (*origin)[2] = ent.i.origin[2];
    }
}

/// Raven `AAS_EntityModelindex`.
///
/// Source: `oracle/codemp/botlib/be_aas_entity.cpp:195-203`
pub fn AAS_EntityModelindex(bot: &mut BotLib, entnum: c_int) -> c_int {
    unsafe {
        if entnum < 0 || entnum >= bot.aasworld.maxentities {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"AAS_EntityModelindex: entnum %d out of range\n".as_ptr() as *mut c_char,
                entnum,
            );
            return 0;
        } //end if
        (*bot.aasworld.entities.add(entnum as usize)).i.modelindex
    }
}

/// Raven `AAS_EntityType`.
///
/// Source: `oracle/codemp/botlib/be_aas_entity.cpp:210-220`
pub fn AAS_EntityType(bot: &mut BotLib, entnum: c_int) -> c_int {
    unsafe {
        if bot.aasworld.initialized == 0 {
            return 0;
        }

        if entnum < 0 || entnum >= bot.aasworld.maxentities {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"AAS_EntityType: entnum %d out of range\n".as_ptr() as *mut c_char,
                entnum,
            );
            return 0;
        } //end if
        (*bot.aasworld.entities.add(entnum as usize)).i.r#type
    }
}

/// Raven `AAS_EntityModelNum`.
///
/// Source: `oracle/codemp/botlib/be_aas_entity.cpp:227-237`
pub fn AAS_EntityModelNum(bot: &mut BotLib, entnum: c_int) -> c_int {
    unsafe {
        if bot.aasworld.initialized == 0 {
            return 0;
        }

        if entnum < 0 || entnum >= bot.aasworld.maxentities {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"AAS_EntityModelNum: entnum %d out of range\n".as_ptr() as *mut c_char,
                entnum,
            );
            return 0;
        } //end if
        (*bot.aasworld.entities.add(entnum as usize)).i.modelindex
    }
}

/// Raven `AAS_OriginOfMoverWithModelNum`.
///
/// Source: `oracle/codemp/botlib/be_aas_entity.cpp:244-262`
pub fn AAS_OriginOfMoverWithModelNum(
    bot: &mut BotLib,
    modelnum: c_int,
    origin: *mut vec3_t,
) -> c_int {
    unsafe {
        for i in 0..bot.aasworld.maxentities {
            let ent = &*bot.aasworld.entities.add(i as usize);
            // Oracle compares against its file-local shadow enum (ET_MOVER = 4),
            // not entityType_t::ET_MOVER (6). Source: oracle/codemp/botlib/be_aas_entity.cpp:32-37,252
            if ent.i.r#type == 4 {
                if ent.i.modelindex == modelnum {
                    (*origin)[0] = ent.i.origin[0];
                    (*origin)[1] = ent.i.origin[1];
                    (*origin)[2] = ent.i.origin[2];
                    return native_types::qtrue as c_int;
                } //end if
            } //end if
        } //end for
        native_types::qfalse as c_int
    }
}

/// Raven `AAS_EntitySize`.
///
/// Source: `oracle/codemp/botlib/be_aas_entity.cpp:269-284`
pub fn AAS_EntitySize(bot: &mut BotLib, entnum: c_int, mins: *mut vec3_t, maxs: *mut vec3_t) {
    unsafe {
        if bot.aasworld.initialized == 0 {
            return;
        }

        if entnum < 0 || entnum >= bot.aasworld.maxentities {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"AAS_EntitySize: entnum %d out of range\n".as_ptr() as *mut c_char,
                entnum,
            );
            return;
        } //end if

        let ent = &*bot.aasworld.entities.add(entnum as usize);
        (*mins)[0] = ent.i.mins[0];
        (*mins)[1] = ent.i.mins[1];
        (*mins)[2] = ent.i.mins[2];
        (*maxs)[0] = ent.i.maxs[0];
        (*maxs)[1] = ent.i.maxs[1];
        (*maxs)[2] = ent.i.maxs[2];
    }
}

/// Raven `AAS_EntityBSPData`.
///
/// Source: `oracle/codemp/botlib/be_aas_entity.cpp:291-302`
pub fn AAS_EntityBSPData(bot: &mut BotLib, entnum: c_int, entdata: *mut bsp_entdata_t) {
    unsafe {
        let ent = &*bot.aasworld.entities.add(entnum as usize);
        (*entdata).origin = ent.i.origin;
        (*entdata).angles = ent.i.angles;
        (*entdata).absmins[0] = ent.i.origin[0] + ent.i.mins[0];
        (*entdata).absmins[1] = ent.i.origin[1] + ent.i.mins[1];
        (*entdata).absmins[2] = ent.i.origin[2] + ent.i.mins[2];
        (*entdata).absmaxs[0] = ent.i.origin[0] + ent.i.maxs[0];
        (*entdata).absmaxs[1] = ent.i.origin[1] + ent.i.maxs[1];
        (*entdata).absmaxs[2] = ent.i.origin[2] + ent.i.maxs[2];
        (*entdata).solid = ent.i.solid;
        (*entdata).modelnum = ent.i.modelindex - 1;
    }
}

/// Raven `AAS_ResetEntityLinks`.
///
/// Source: `oracle/codemp/botlib/be_aas_entity.cpp:309-317`
pub fn AAS_ResetEntityLinks(bot: &mut BotLib) {
    unsafe {
        for i in 0..bot.aasworld.maxentities {
            (*bot.aasworld.entities.add(i as usize)).areas = core::ptr::null_mut();
            (*bot.aasworld.entities.add(i as usize)).leaves = core::ptr::null_mut();
        } //end for
    }
}

/// Raven `AAS_InvalidateEntities`.
///
/// Source: `oracle/codemp/botlib/be_aas_entity.cpp:324-332`
pub fn AAS_InvalidateEntities(bot: &mut BotLib) {
    unsafe {
        for i in 0..bot.aasworld.maxentities {
            (*bot.aasworld.entities.add(i as usize)).i.valid = native_types::qfalse as c_int;
            (*bot.aasworld.entities.add(i as usize)).i.number = i;
        } //end for
    }
}

/// Raven `AAS_NearestEntity`.
///
/// Source: `oracle/codemp/botlib/be_aas_entity.cpp:362-390`
pub fn AAS_NearestEntity(bot: &mut BotLib, origin: vec3_t, modelindex: c_int) -> c_int {
    unsafe {
        let mut bestentnum: c_int = 0;
        let mut bestdist: f32 = 99999.0;
        for i in 0..bot.aasworld.maxentities {
            let ent = &*bot.aasworld.entities.add(i as usize);
            if ent.i.modelindex != modelindex {
                continue;
            }
            let dir: vec3_t = [
                ent.i.origin[0] - origin[0],
                ent.i.origin[1] - origin[1],
                ent.i.origin[2] - origin[2],
            ];
            if dir[0].abs() < 40.0 {
                if dir[1].abs() < 40.0 {
                    let dist = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
                    if dist < bestdist {
                        bestdist = dist;
                        bestentnum = i;
                    } //end if
                } //end if
            } //end if
        } //end for
        bestentnum
    }
}

/// Raven `AAS_NextEntity`.
///
/// Source: `oracle/codemp/botlib/be_aas_entity.cpp:410-420`
pub fn AAS_NextEntity(bot: &mut BotLib, entnum: c_int) -> c_int {
    unsafe {
        if bot.aasworld.loaded == 0 {
            return 0;
        }

        let mut entnum = entnum;
        if entnum < 0 {
            entnum = -1;
        }
        loop {
            entnum += 1;
            if entnum >= bot.aasworld.maxentities {
                break;
            }
            if (*bot.aasworld.entities.add(entnum as usize)).i.valid != 0 {
                return entnum;
            }
        } //end while
        0
    }
}

/// Raven `AAS_EntityInfo`.
///
/// Source: `oracle/codemp/botlib/be_aas_entity.cpp:154-171`
pub fn AAS_EntityInfo(bot: &mut BotLib, entnum: c_int, info: *mut aas_entityinfo_t) {
    unsafe {
        if bot.aasworld.initialized == 0 {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"AAS_EntityInfo: aasworld not initialized\n".as_ptr() as *mut c_char,
            );
            Com_Memset(info as *mut (), 0, core::mem::size_of::<aas_entityinfo_t>());
            return;
        } //end if

        if entnum < 0 || entnum >= bot.aasworld.maxentities {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"AAS_EntityInfo: entnum %d out of range\n".as_ptr() as *mut c_char,
                entnum,
            );
            Com_Memset(info as *mut (), 0, core::mem::size_of::<aas_entityinfo_t>());
            return;
        } //end if

        Com_Memcpy(
            info as *mut (),
            &(*bot.aasworld.entities.add(entnum as usize)).i as *const aas_entityinfo_t
                as *const (),
            core::mem::size_of::<aas_entityinfo_t>(),
        );
    }
}

/// Raven `AAS_UnlinkInvalidEntities`.
///
/// Source: `oracle/codemp/botlib/be_aas_entity.cpp:339-355`
pub fn AAS_UnlinkInvalidEntities(bot: &mut BotLib) {
    unsafe {
        for i in 0..bot.aasworld.maxentities {
            let ent = &mut *bot.aasworld.entities.add(i as usize);
            if ent.i.valid == 0 {
                AAS_UnlinkFromAreas(bot, ent.areas);
                let ent = &mut *bot.aasworld.entities.add(i as usize);
                ent.areas = core::ptr::null_mut();
                AAS_UnlinkFromBSPLeaves(ent.leaves);
                let ent = &mut *bot.aasworld.entities.add(i as usize);
                ent.leaves = core::ptr::null_mut();
            } //end for
        } //end for
    }
}

/// Raven `AAS_UpdateEntity`.
///
/// Source: `oracle/codemp/botlib/be_aas_entity.cpp:46-147`
pub fn AAS_UpdateEntity(bot: &mut BotLib, entnum: c_int, state: *mut bot_entitystate_t) -> c_int {
    unsafe {
        if bot.aasworld.loaded == 0 {
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"AAS_UpdateEntity: not loaded\n".as_ptr() as *mut c_char,
            );
            return BLERR_NOAASFILE;
        } //end if

        if state.is_null() {
            //unlink the entity
            let ent = &mut *bot.aasworld.entities.add(entnum as usize);
            AAS_UnlinkFromAreas(bot, ent.areas);
            //unlink the entity from the BSP leaves
            let ent = &mut *bot.aasworld.entities.add(entnum as usize);
            AAS_UnlinkFromBSPLeaves(ent.leaves);
            //
            let ent = &mut *bot.aasworld.entities.add(entnum as usize);
            ent.areas = core::ptr::null_mut();
            //
            ent.leaves = core::ptr::null_mut();
            return BLERR_NOERROR;
        }

        let mut relink;
        let time = AAS_Time(bot);
        let ent = &mut *bot.aasworld.entities.add(entnum as usize);

        ent.i.update_time = time - ent.i.ltime;
        ent.i.r#type = (*state).r#type;
        ent.i.flags = (*state).flags;
        ent.i.ltime = time;
        ent.i.lastvisorigin = ent.i.origin;
        ent.i.old_origin = (*state).old_origin;
        ent.i.solid = (*state).solid;
        ent.i.groundent = (*state).groundent;
        ent.i.modelindex = (*state).modelindex;
        ent.i.modelindex2 = (*state).modelindex2;
        ent.i.frame = (*state).frame;
        ent.i.event = (*state).event;
        ent.i.eventParm = (*state).eventParm;
        ent.i.powerups = (*state).powerups;
        ent.i.weapon = (*state).weapon;
        ent.i.legsAnim = (*state).legsAnim;
        ent.i.torsoAnim = (*state).torsoAnim;
        //number of the entity
        ent.i.number = entnum;
        //updated so set valid flag
        ent.i.valid = native_types::qtrue as c_int;
        //link everything the first frame
        if bot.aasworld.numframes == 1 {
            relink = native_types::qtrue as c_int;
        } else {
            relink = native_types::qfalse as c_int;
        }
        //
        if ent.i.solid == solid_t::SOLID_BSP as c_int {
            //if the angles of the model changed
            if (*state).angles != ent.i.angles {
                ent.i.angles = (*state).angles;
                relink = native_types::qtrue as c_int;
            } //end if
              //get the mins and maxs of the model
              //FIXME: rotate mins and maxs
            AAS_BSPModelMinsMaxsOrigin(
                bot,
                ent.i.modelindex,
                ent.i.angles,
                &mut ent.i.mins,
                &mut ent.i.maxs,
                &mut [0.0; 3],
            );
        //end if
        } else if ent.i.solid == solid_t::SOLID_BBOX as c_int {
            //if the bounding box size changed
            if (*state).mins != ent.i.mins || (*state).maxs != ent.i.maxs {
                ent.i.mins = (*state).mins;
                ent.i.maxs = (*state).maxs;
                relink = native_types::qtrue as c_int;
            } //end if
            ent.i.angles = (*state).angles;
        } //end if
          //if the origin changed
        if (*state).origin != ent.i.origin {
            ent.i.origin = (*state).origin;
            relink = native_types::qtrue as c_int;
        } //end if
          //if the entity should be relinked
        if relink != 0 {
            //don't link the world model
            if entnum != ENTITYNUM_WORLD {
                //absolute mins and maxs
                let absmins: vec3_t = [
                    ent.i.mins[0] + ent.i.origin[0],
                    ent.i.mins[1] + ent.i.origin[1],
                    ent.i.mins[2] + ent.i.origin[2],
                ];
                let absmaxs: vec3_t = [
                    ent.i.maxs[0] + ent.i.origin[0],
                    ent.i.maxs[1] + ent.i.origin[1],
                    ent.i.maxs[2] + ent.i.origin[2],
                ];
                //unlink the entity
                AAS_UnlinkFromAreas(bot, ent.areas);
                //relink the entity to the AAS areas (use the larges bbox)
                let ent = &mut *bot.aasworld.entities.add(entnum as usize);
                ent.areas = AAS_LinkEntityClientBBox(
                    bot,
                    absmins,
                    absmaxs,
                    entnum,
                    PRESENCE_NORMAL as c_int,
                );
                //unlink the entity from the BSP leaves
                let ent = &mut *bot.aasworld.entities.add(entnum as usize);
                AAS_UnlinkFromBSPLeaves(ent.leaves);
                //link the entity to the world BSP tree
                let ent = &mut *bot.aasworld.entities.add(entnum as usize);
                ent.leaves = AAS_BSPLinkEntity(absmins, absmaxs, entnum, 0) as *mut _;
            } //end if
        } //end if
        BLERR_NOERROR
    }
}

/// Raven `AAS_BestReachableEntityArea`.
///
/// Source: `oracle/codemp/botlib/be_aas_entity.cpp:397-403`
pub fn AAS_BestReachableEntityArea(bot: &mut BotLib, entnum: c_int) -> c_int {
    unsafe {
        let ent = &*bot.aasworld.entities.add(entnum as usize);
        AAS_BestReachableLinkArea(bot, ent.areas)
    }
}

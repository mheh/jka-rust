//! `sv_world.cpp` — the world-linkage bsp (`worldSector_t` chain), entity
//! link/unlink, area queries, and the `SV_Trace`/`SV_ClipMoveToEntities`
//! collision sweep.
//!
//! Source: `oracle/codemp/server/sv_world.cpp`

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_engine_ghoul2::api_collision::{g2api_collision_detect, g2api_collision_detect_cache};
use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_ghoul2::shared::cghoul2_info_v::CGhoul2Info_v;
use mp_engine_qcommon::cm_load::{CM_LeafArea, CM_LeafCluster};
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::common::Common;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::game::class_t::class_t;
use mp_qshared::common::mp::game::g_public::{
    G2TRFLAG_DOGHOULTRACE, G2TRFLAG_GETSURFINDEX, G2TRFLAG_HITCORPSES, G2TRFLAG_THICK, SVF_CAPSULE,
    SVF_OWNERNOTSHARED,
};
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::limits::{ENTITYNUM_NONE, ENTITYNUM_WORLD, MAX_CLIENTS, MAX_GENTITIES};
use mp_qshared::shared::q_math::RadiusFromBounds;
use mp_qshared::shared::surface_flags::{
    CONTENTS_BODY, CONTENTS_LIGHTSABER, CONTENTS_NOSHOT, CONTENTS_SOLID, MASK_SHOT, SOLID_BMODEL,
};
use native_math::vector::vec3_t;
use native_types::clipHandle_t;

use crate::server::area_parms_t::areaParms_t;
use crate::server::moveclip_t::moveclip_t;
use crate::server::server_state_t::serverState_t;
use crate::server::sv_entity_s::{svEntity_t, MAX_ENT_CLUSTERS};
use crate::server::world_sector_s::{worldSector_t, AREA_DEPTH, AREA_NODES, MAX_TOTAL_ENT_LEAFS};
use crate::sv_game::SV_SvEntityForGentity;
use crate::Server;

/// Raven `VectorDistance` (`sv_world.cpp`-local `static float`).
///
/// Source: `oracle/codemp/server/sv_world.cpp:513-519`
fn VectorDistance(p1: vec3_t, p2: vec3_t) -> f32 {
    let mut dir: vec3_t = [0.0; 3];
    VectorSubtract(p2, p1, &mut dir);
    VectorLength(dir)
}

// Local vector helpers (established per-file convention — see
// `cm_load.rs`/`be_aas_move.rs`): Raven's `VectorSubtract`/`VectorCopy`/
// `VectorAdd`/`VectorLength` macros, transcribed as free fns colocated with
// their callers in this file.
fn VectorSubtract(a: vec3_t, b: vec3_t, out: &mut vec3_t) {
    out[0] = a[0] - b[0];
    out[1] = a[1] - b[1];
    out[2] = a[2] - b[2];
}

fn VectorAdd(a: vec3_t, b: vec3_t, out: &mut vec3_t) {
    out[0] = a[0] + b[0];
    out[1] = a[1] + b[1];
    out[2] = a[2] + b[2];
}

fn VectorCopy(src: vec3_t, dst: &mut vec3_t) {
    dst[0] = src[0];
    dst[1] = src[1];
    dst[2] = src[2];
}

fn VectorLength(v: vec3_t) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Raven `SV_CreateworldSector`.
///
/// Source: `oracle/codemp/server/sv_world.cpp:90-123`
pub fn SV_CreateworldSector(
    sv: &mut Server,
    depth: c_int,
    mins: vec3_t,
    maxs: vec3_t,
) -> *mut worldSector_t {
    let idx = sv.world_sectors.sv_numworldSectors as usize;
    sv.world_sectors.sv_numworldSectors += 1;
    let anode: *mut worldSector_t = &mut sv.world_sectors.sv_worldSectors[idx];

    unsafe {
        if depth == AREA_DEPTH {
            (*anode).axis = -1;
            (*anode).children[0] = core::ptr::null_mut();
            (*anode).children[1] = core::ptr::null_mut();
            return anode;
        }

        let mut size: vec3_t = [0.0; 3];
        VectorSubtract(maxs, mins, &mut size);
        if size[0] > size[1] {
            (*anode).axis = 0;
        } else {
            (*anode).axis = 1;
        }

        let axis = (*anode).axis as usize;
        (*anode).dist = 0.5 * (maxs[axis] + mins[axis]);

        let mut mins1: vec3_t = [0.0; 3];
        let mut mins2: vec3_t = [0.0; 3];
        let mut maxs1: vec3_t = [0.0; 3];
        let mut maxs2: vec3_t = [0.0; 3];
        VectorCopy(mins, &mut mins1);
        VectorCopy(mins, &mut mins2);
        VectorCopy(maxs, &mut maxs1);
        VectorCopy(maxs, &mut maxs2);

        maxs1[axis] = (*anode).dist;
        mins2[axis] = (*anode).dist;

        (*anode).children[0] = SV_CreateworldSector(sv, depth + 1, mins2, maxs2);
        // Re-take the pointer: the recursive call above may have grown/moved
        // nothing (the sector table is a fixed array), but `sv` was
        // reborrowed — re-derive `anode` from the same index for the second
        // child write, matching Raven's single-pointer reuse.
        let anode: *mut worldSector_t = &mut sv.world_sectors.sv_worldSectors[idx];
        (*anode).children[1] = SV_CreateworldSector(sv, depth + 1, mins1, maxs1);

        anode
    }
}

/// Raven `SV_SectorList_f`.
///
/// Source: `oracle/codemp/server/sv_world.cpp:67-81`
pub fn SV_SectorList_f(common: &mut Common, sv: &mut Server) {
    for i in 0..AREA_NODES {
        let sec = &sv.world_sectors.sv_worldSectors[i];
        let mut c = 0;
        unsafe {
            let mut ent = sec.entities;
            while !ent.is_null() {
                c += 1;
                ent = (*ent).nextEntityInWorldSector;
            }
        }
        mp_engine_qcommon::common::common::com_printf(
            common,
            &format!("sector {}: {} entities\n", i, c),
        );
    }
}

/// Raven `SV_UnlinkEntity`.
///
/// Source: `oracle/codemp/server/sv_world.cpp:151-179`
pub fn SV_UnlinkEntity(common: &mut Common, sv: &mut Server, gEnt: *mut sharedEntity_t) {
    unsafe {
        let ent = SV_SvEntityForGentity(sv, gEnt);

        (*gEnt).r.linked = 0;

        let ws = (*ent).worldSector;
        if ws.is_null() {
            return; // not linked in anywhere
        }
        (*ent).worldSector = core::ptr::null_mut();

        if (*ws).entities == ent {
            (*ws).entities = (*ent).nextEntityInWorldSector;
            return;
        }

        let mut scan = (*ws).entities;
        while !scan.is_null() {
            if (*scan).nextEntityInWorldSector == ent {
                (*scan).nextEntityInWorldSector = (*ent).nextEntityInWorldSector;
                return;
            }
            scan = (*scan).nextEntityInWorldSector;
        }

        mp_engine_qcommon::common::common::com_printf(
            common,
            "WARNING: SV_UnlinkEntity: not found in worldSector\n",
        );
    }
}

/// Raven `SV_AreaEntities_r`.
///
/// Source: `oracle/codemp/server/sv_world.cpp:373-414`
pub fn SV_AreaEntities_r(
    common: &mut Common,
    sv: &mut Server,
    node: *mut worldSector_t,
    ap: *mut areaParms_t,
) {
    unsafe {
        let mut check = (*node).entities;
        while !check.is_null() {
            let next = (*check).nextEntityInWorldSector;

            let gcheck = crate::sv_game::SV_GEntityForSvEntity(sv, check);

            if (*gcheck).r.absmin[0] > (*ap).maxs.offset(0).read()
                || (*gcheck).r.absmin[1] > (*ap).maxs.offset(1).read()
                || (*gcheck).r.absmin[2] > (*ap).maxs.offset(2).read()
                || (*gcheck).r.absmax[0] < (*ap).mins.offset(0).read()
                || (*gcheck).r.absmax[1] < (*ap).mins.offset(1).read()
                || (*gcheck).r.absmax[2] < (*ap).mins.offset(2).read()
            {
                check = next;
                continue;
            }

            if (*ap).count == (*ap).maxcount {
                mp_engine_qcommon::common_fns::Com_DPrintf(common, "SV_AreaEntities: MAXCOUNT\n");
                return;
            }

            let idx = (check as isize - sv.sv.svEntities.as_mut_ptr() as isize)
                / core::mem::size_of::<svEntity_t>() as isize;
            *(*ap).list.offset((*ap).count as isize) = idx as c_int;
            (*ap).count += 1;

            check = next;
        }

        if (*node).axis == -1 {
            return; // terminal node
        }

        // recurse down both sides
        let axis = (*node).axis as usize;
        if (*ap).maxs.offset(axis as isize).read() > (*node).dist {
            SV_AreaEntities_r(common, sv, (*node).children[0], ap);
        }
        if (*ap).mins.offset(axis as isize).read() < (*node).dist {
            SV_AreaEntities_r(common, sv, (*node).children[1], ap);
        }
    }
}

/// Raven `SV_ClipHandleForEntity`.
///
/// Source: `oracle/codemp/server/sv_world.cpp:19-31`
pub fn SV_ClipHandleForEntity(cm: &mut CollisionWorld, ent: *const sharedEntity_t) -> clipHandle_t {
    unsafe {
        if (*ent).r.bmodel != 0 {
            // explicit hulls in the BSP model
            return mp_engine_qcommon::cm_load::CM_InlineModel(cm, (*ent).s.modelindex);
        }
        if (*ent).r.svFlags & SVF_CAPSULE != 0 {
            // create a temp capsule from bounding box sizes
            return mp_engine_qcommon::cm_load::CM_TempBoxModel(
                cm,
                (*ent).r.mins,
                (*ent).r.maxs,
                1,
            );
        }

        // create a temp tree from bounding box sizes
        mp_engine_qcommon::cm_load::CM_TempBoxModel(cm, (*ent).r.mins, (*ent).r.maxs, 0)
    }
}

/// Raven `SV_LinkEntity`.
///
/// Source: `oracle/codemp/server/sv_world.cpp:189-347`
#[allow(clippy::too_many_arguments)]
pub fn SV_LinkEntity(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    gEnt: *mut sharedEntity_t,
) {
    unsafe {
        let ent = SV_SvEntityForGentity(sv, gEnt);

        if !(*ent).worldSector.is_null() {
            SV_UnlinkEntity(common, sv, gEnt); // unlink from old position
        }

        // encode the size into the entityState_t for client prediction
        if (*gEnt).r.bmodel != 0 {
            (*gEnt).s.solid = SOLID_BMODEL; // a solid_box will never create this value
        } else if (*gEnt).r.contents & (CONTENTS_SOLID | CONTENTS_BODY) != 0 {
            // assume that x/y are equal and symetric
            let mut i = (*gEnt).r.maxs[0] as c_int;
            if i < 1 {
                i = 1;
            }
            if i > 255 {
                i = 255;
            }

            // z is not symetric
            let mut j = (-(*gEnt).r.mins[2]) as c_int;
            if j < 1 {
                j = 1;
            }
            if j > 255 {
                j = 255;
            }

            // and z maxs can be negative...
            let mut k = ((*gEnt).r.maxs[2] + 32.0) as c_int;
            if k < 1 {
                k = 1;
            }
            if k > 255 {
                k = 255;
            }

            (*gEnt).s.solid = (k << 16) | (j << 8) | i;

            if (*gEnt).s.solid == SOLID_BMODEL {
                //yikes, this would make everything explode violently.
                (*gEnt).s.solid = (k << 16) | (j << 8) | (i - 1);
            }
        } else {
            (*gEnt).s.solid = 0;
        }

        // get the position
        let origin = (*gEnt).r.currentOrigin;
        let angles = (*gEnt).r.currentAngles;

        // set the abs box
        if (*gEnt).r.bmodel != 0 && (angles[0] != 0.0 || angles[1] != 0.0 || angles[2] != 0.0) {
            // expand for rotation
            let max = RadiusFromBounds((*gEnt).r.mins, (*gEnt).r.maxs);
            for i in 0..3 {
                (*gEnt).r.absmin[i] = origin[i] - max;
                (*gEnt).r.absmax[i] = origin[i] + max;
            }
        } else {
            // normal
            let mut absmin: vec3_t = [0.0; 3];
            let mut absmax: vec3_t = [0.0; 3];
            VectorAdd(origin, (*gEnt).r.mins, &mut absmin);
            VectorAdd(origin, (*gEnt).r.maxs, &mut absmax);
            (*gEnt).r.absmin = absmin;
            (*gEnt).r.absmax = absmax;
        }

        // because movement is clipped an epsilon away from an actual edge,
        // we must fully check even when bounding boxes don't quite touch
        (*gEnt).r.absmin[0] -= 1.0;
        (*gEnt).r.absmin[1] -= 1.0;
        (*gEnt).r.absmin[2] -= 1.0;
        (*gEnt).r.absmax[0] += 1.0;
        (*gEnt).r.absmax[1] += 1.0;
        (*gEnt).r.absmax[2] += 1.0;

        // link to PVS leafs
        (*ent).numClusters = 0;
        (*ent).lastCluster = 0;
        (*ent).areanum = -1;
        (*ent).areanum2 = -1;

        //get all leafs, including solids
        let mut leafs: [c_int; MAX_TOTAL_ENT_LEAFS] = [0; MAX_TOTAL_ENT_LEAFS];
        let mut lastLeaf: c_int = 0;
        let num_leafs = mp_engine_qcommon::cm_test::CM_BoxLeafnums(
            cm,
            (*gEnt).r.absmin,
            (*gEnt).r.absmax,
            leafs.as_mut_ptr(),
            MAX_TOTAL_ENT_LEAFS as c_int,
            &mut lastLeaf,
        );

        // if none of the leafs were inside the map, the
        // entity is outside the world and can be considered unlinked
        if num_leafs == 0 {
            return;
        }

        // set areas, even from clusters that don't fit in the entity array
        let mut i: c_int = 0;
        while i < num_leafs {
            let area = CM_LeafArea(cm, leafs[i as usize]);
            if area != -1 {
                // doors may legally straggle two areas,
                // but nothing should evern need more than that
                if (*ent).areanum != -1 && (*ent).areanum != area {
                    if (*ent).areanum2 != -1
                        && (*ent).areanum2 != area
                        && sv.sv.state == serverState_t::SS_LOADING
                    {
                        mp_engine_qcommon::common_fns::Com_DPrintf(
                            common,
                            &format!(
                                "Object {} touching 3 areas at {} {} {}\n",
                                (*gEnt).s.number,
                                (*gEnt).r.absmin[0],
                                (*gEnt).r.absmin[1],
                                (*gEnt).r.absmin[2]
                            ),
                        );
                    }
                    (*ent).areanum2 = area;
                } else {
                    (*ent).areanum = area;
                }
            }
            i += 1;
        }

        // store as many explicit clusters as we can
        (*ent).numClusters = 0;
        let mut i: c_int = 0;
        while i < num_leafs {
            let cluster = CM_LeafCluster(cm, leafs[i as usize]);
            if cluster != -1 {
                (*ent).clusternums[(*ent).numClusters as usize] = cluster;
                (*ent).numClusters += 1;
                if (*ent).numClusters as usize == MAX_ENT_CLUSTERS {
                    break;
                }
            }
            i += 1;
        }

        // store off a last cluster if we need to
        if i != num_leafs {
            (*ent).lastCluster = CM_LeafCluster(cm, lastLeaf);
        }

        (*gEnt).r.linkcount += 1;

        // find the first world sector node that the ent's box crosses
        let mut node: *mut worldSector_t = sv.world_sectors.sv_worldSectors.as_mut_ptr();
        loop {
            if (*node).axis == -1 {
                break;
            }
            let axis = (*node).axis as usize;
            if (*gEnt).r.absmin[axis] > (*node).dist {
                node = (*node).children[0];
            } else if (*gEnt).r.absmax[axis] < (*node).dist {
                node = (*node).children[1];
            } else {
                break; // crosses the node
            }
        }

        // link it in
        (*ent).worldSector = node;
        (*ent).nextEntityInWorldSector = (*node).entities;
        (*node).entities = ent;

        (*gEnt).r.linked = 1;
    }
}

/// Raven `SV_AreaEntities`.
///
/// Source: `oracle/codemp/server/sv_world.cpp:421-433`
pub fn SV_AreaEntities(
    common: &mut Common,
    sv: &mut Server,
    mins: vec3_t,
    maxs: vec3_t,
    entityList: *mut c_int,
    maxcount: c_int,
) -> c_int {
    let mut ap = areaParms_t {
        mins: mins.as_ptr(),
        maxs: maxs.as_ptr(),
        list: entityList,
        count: 0,
        maxcount,
    };

    // Take the raw node pointer before passing `sv` — the raw pointer aliases
    // into `sv.world_sectors` (as in Raven) without holding a borrow.
    let node = sv.world_sectors.sv_worldSectors.as_mut_ptr();
    SV_AreaEntities_r(common, sv, node, &mut ap);

    ap.count
}

/// Raven `SV_ClearWorld`.
///
/// Source: `oracle/codemp/server/sv_world.cpp:131-142`
pub fn SV_ClearWorld(cm: &mut CollisionWorld, sv: &mut Server) {
    let size = core::mem::size_of_val(&sv.world_sectors.sv_worldSectors);
    mp_engine_qcommon::common_fns::Com_Memset(
        sv.world_sectors.sv_worldSectors.as_mut_ptr() as *mut (),
        0,
        size,
    );
    sv.world_sectors.sv_numworldSectors = 0;

    // get world map bounds
    let h = mp_engine_qcommon::cm_load::CM_InlineModel(cm, 0);
    let mins: vec3_t = [0.0; 3];
    let maxs: vec3_t = [0.0; 3];
    // CM_ModelBounds now takes mins/maxs by value (shape-mismatch out-param
    // documented at its definition, cm_load.rs); reconciled call, no write-back.
    mp_engine_qcommon::cm_load::CM_ModelBounds(cm, h, mins, maxs);
    SV_CreateworldSector(sv, 0, mins, maxs);
}

/// Raven `SV_PointContents`.
///
/// Source: `oracle/codemp/server/sv_world.cpp:871-903`
pub fn SV_PointContents(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    p: vec3_t,
    passEntityNum: c_int,
) -> c_int {
    // get base contents from world
    let mut contents = mp_engine_qcommon::cm_test::CM_PointContents(cm, p, 0);

    // or in contents from all the other entities
    let mut touch: [c_int; MAX_GENTITIES as usize] = [0; MAX_GENTITIES as usize];
    let num = SV_AreaEntities(common, sv, p, p, touch.as_mut_ptr(), MAX_GENTITIES as c_int);

    unsafe {
        for i in 0..num {
            if touch[i as usize] == passEntityNum {
                continue;
            }
            let hit = crate::sv_game::SV_GentityNum(sv, touch[i as usize]);
            // might intersect, so do an exact clip
            let clipHandle = SV_ClipHandleForEntity(cm, hit);
            let mut angles = (*hit).s.angles;
            if (*hit).r.bmodel == 0 {
                angles = VEC3_ORIGIN; // boxes don't rotate
            }

            let c2 = mp_engine_qcommon::cm_test::CM_TransformedPointContents(
                cm,
                p,
                clipHandle,
                (*hit).s.origin,
                (*hit).s.angles,
            );
            let _ = angles;

            contents |= c2;
        }
    }

    contents
}

// PORT-NOTE(vec3_origin): `vec3_origin` (`q_shared.h:1179`) has no reachable
// qshared/Common home yet (same gap `cm_trace.rs` already notes); stood in
// with a local const, escalated in missing_symbols.
const VEC3_ORIGIN: vec3_t = [0.0, 0.0, 0.0];

/// Raven `SV_ClipToEntity`.
///
/// Source: `oracle/codemp/server/sv_world.cpp:470-503`
#[allow(clippy::too_many_arguments)]
pub fn SV_ClipToEntity(
    view: &mut EngineHostView,
    sv: &mut Server,
    trace: *mut trace_t,
    start: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    end: vec3_t,
    entityNum: c_int,
    contentmask: c_int,
    capsule: c_int,
) {
    unsafe {
        let touch = crate::sv_game::SV_GentityNum(sv, entityNum);

        mp_engine_qcommon::common_fns::Com_Memset(
            trace as *mut (),
            0,
            core::mem::size_of::<trace_t>(),
        );

        // if it doesn't have any brushes of a type we
        // are looking for, ignore it
        if contentmask & (*touch).r.contents == 0 {
            (*trace).fraction = 1.0;
            return;
        }

        // might intersect, so do an exact clip
        let clipHandle = SV_ClipHandleForEntity(view.cm, touch);

        let origin = (*touch).r.currentOrigin;
        let mut angles = (*touch).r.currentAngles;

        if (*touch).r.bmodel == 0 {
            angles = VEC3_ORIGIN; // boxes don't rotate
        }

        mp_engine_qcommon::cm_trace::CM_TransformedBoxTrace(
            view,
            trace,
            start,
            end,
            mins,
            maxs,
            clipHandle,
            contentmask,
            origin,
            angles,
            capsule,
        );

        if (*trace).fraction < 1.0 {
            (*trace).entityNum = (*touch).s.number as core::ffi::c_short;
        }
    }
}

/// Raven `SV_ClipMoveToEntities` (`sv_world.cpp`-local `static void`).
///
/// Source: `oracle/codemp/server/sv_world.cpp:522-789`
///
/// PORT-NOTE(g2-if0): Raven's body has a dead `#if 0` Ghoul2-collision block
/// (lines 643-687 of the cite) superseded by the live `#else` arm below it
/// (line 688 on); only the live arm is transcribed, per C preprocessing.
#[allow(clippy::too_many_arguments)]
pub fn SV_ClipMoveToEntities(view: &mut EngineHostView, sv: &mut Server, clip: *mut moveclip_t) {
    unsafe {
        let mut touchlist: [c_int; MAX_GENTITIES as usize] = [0; MAX_GENTITIES as usize];
        let num = SV_AreaEntities(
            view.common,
            sv,
            (*clip).boxmins,
            (*clip).boxmaxs,
            touchlist.as_mut_ptr(),
            MAX_GENTITIES as c_int,
        );

        let passOwnerNum: c_int;
        if (*clip).passEntityNum != ENTITYNUM_NONE {
            let passEnt = crate::sv_game::SV_GentityNum(sv, (*clip).passEntityNum);
            passOwnerNum = if (*passEnt).r.ownerNum == ENTITYNUM_NONE {
                -1
            } else {
                (*passEnt).r.ownerNum
            };
        } else {
            passOwnerNum = -1;
        }

        let mut thisOwnerShared = true;
        let passEnt2 = crate::sv_game::SV_GentityNum(sv, (*clip).passEntityNum);
        if (*passEnt2).r.svFlags & SVF_OWNERNOTSHARED != 0 {
            thisOwnerShared = false;
        }

        for i in 0..num {
            if (*clip).trace.allsolid != 0 {
                return;
            }
            let touch = crate::sv_game::SV_GentityNum(sv, touchlist[i as usize]);

            // see if we should ignore this entity
            if (*clip).passEntityNum != ENTITYNUM_NONE {
                if touchlist[i as usize] == (*clip).passEntityNum {
                    continue; // don't clip against the pass entity
                }
                if (*touch).r.ownerNum == (*clip).passEntityNum {
                    if (*touch).r.svFlags & SVF_OWNERNOTSHARED != 0 {
                        if (*clip).contentmask != (MASK_SHOT | CONTENTS_LIGHTSABER)
                            && (*clip).contentmask != MASK_SHOT
                        {
                            //it's not a laser hitting the other "missile", don't care then
                            continue;
                        }
                    } else {
                        continue; // don't clip against own missiles
                    }
                }
                if (*touch).r.ownerNum == passOwnerNum
                    && ((*touch).r.svFlags & SVF_OWNERNOTSHARED) == 0
                    && thisOwnerShared
                {
                    continue; // don't clip against other missiles from our owner
                }

                if (*touch).s.eType == mp_bg::public::entity_type::entityType_t::ET_MISSILE as c_int
                    && ((*touch).r.svFlags & SVF_OWNERNOTSHARED) == 0
                    && (*touch).r.ownerNum == passOwnerNum
                {
                    //blah, hack
                    continue;
                }
            }

            // if it doesn't have any brushes of a type we
            // are looking for, ignore it
            if (*clip).contentmask & (*touch).r.contents == 0 {
                continue;
            }

            if ((*clip).contentmask == (MASK_SHOT | CONTENTS_LIGHTSABER)
                || (*clip).contentmask == MASK_SHOT)
                && ((*touch).r.contents > 0 && ((*touch).r.contents & CONTENTS_NOSHOT != 0))
            {
                continue;
            }

            // might intersect, so do an exact clip
            let clipHandle = SV_ClipHandleForEntity(view.cm, touch);

            let origin = (*touch).r.currentOrigin;
            let mut angles = (*touch).r.currentAngles;

            if (*touch).r.bmodel == 0 {
                angles = VEC3_ORIGIN; // boxes don't rotate
            }

            let mut trace = trace_t {
                allsolid: 0,
                startsolid: 0,
                entityNum: 0,
                fraction: 0.0,
                endpos: [0.0; 3],
                plane: core::mem::zeroed(),
                surfaceFlags: 0,
                contents: 0,
            };
            mp_engine_qcommon::cm_trace::CM_TransformedBoxTrace(
                view,
                &mut trace,
                (*clip).start,
                (*clip).end,
                (*clip).mins.cast::<[f32; 3]>().read(),
                (*clip).maxs.cast::<[f32; 3]>().read(),
                clipHandle,
                (*clip).contentmask,
                origin,
                angles,
                (*clip).capsule,
            );

            let mut oldTrace: trace_t = core::mem::zeroed();
            if (*clip).traceFlags & G2TRFLAG_DOGHOULTRACE != 0 {
                // keep these older variables around for a bit, incase we need to
                // replace them in the Ghoul2 Collision check
                oldTrace = (*clip).trace;
            }

            if trace.allsolid != 0 {
                (*clip).trace.allsolid = 1;
                trace.entityNum = (*touch).s.number as core::ffi::c_short;
            } else if trace.startsolid != 0 {
                (*clip).trace.startsolid = 1;
                trace.entityNum = (*touch).s.number as core::ffi::c_short;

                //rww - added this because we want to get the number of an ent even if our trace starts inside it.
                (*clip).trace.entityNum = (*touch).s.number as core::ffi::c_short;
            }

            if trace.fraction < (*clip).trace.fraction {
                // make sure we keep a startsolid from a previous trace
                let oldStart = (*clip).trace.startsolid;

                trace.entityNum = (*touch).s.number as core::ffi::c_short;
                (*clip).trace = trace;
                (*clip).trace.startsolid = (*clip).trace.startsolid | oldStart;
            }

            //rww - since this is multiplayer and we don't have the luxury of
            //violating networking rules in horrible ways, this must be done
            //somewhat differently.
            if ((*clip).traceFlags & G2TRFLAG_DOGHOULTRACE != 0)
                && trace.entityNum == (*touch).s.number as core::ffi::c_short
                && !(*touch).ghoul2.is_null()
                && (((*clip).traceFlags & G2TRFLAG_HITCORPSES != 0)
                    || ((*touch).s.eFlags & mp_bg::public::entity_flags::EF_DEAD) == 0)
            {
                //standard behavior will be to ignore g2 col on dead ents, but if
                //traceFlags is set to allow, then we'll try g2 col on EF_DEAD
                //people too.
                let mut angles2: vec3_t = [0.0; 3];
                let mut fRadius = 0.0f32;

                if (*clip).mins.cast::<[f32; 3]>().read()[0] != 0.0
                    || (*clip).maxs.cast::<[f32; 3]>().read()[0] != 0.0
                {
                    fRadius = ((*clip).maxs.cast::<[f32; 3]>().read()[0]
                        - (*clip).mins.cast::<[f32; 3]>().read()[0])
                        / 2.0;
                }

                if (*clip).traceFlags & G2TRFLAG_THICK != 0 {
                    //if using this flag, make sure it's at least 1.0f
                    if fRadius < 1.0 {
                        fRadius = 1.0;
                    }
                }

                if ((*touch).s.number) < MAX_CLIENTS as c_int {
                    VectorCopy((*touch).s.apos.trBase, &mut angles2);
                } else {
                    VectorCopy((*touch).r.currentAngles, &mut angles2);
                }
                angles2[mp_qshared::shared::q_math::ROLL as usize] = 0.0;
                angles2[mp_qshared::shared::q_math::PITCH as usize] = 0.0;

                // PORT-NOTE(FINAL_BUILD): Raven guards this Com_Printf debug
                // line with `#ifndef FINAL_BUILD`; FINAL_BUILD is undefined
                // for this build per plan appendix, so the guard is always
                // true and the print is unconditional here.
                if view.cvar_integer("sv_showghoultraces") != 0 {
                    mp_engine_qcommon::common::common::com_printf(
                        view.common,
                        &format!(
                            "Ghoul2 trace   lod={:1}   length={:6.0}   to {}\n",
                            (*clip).useLod,
                            VectorDistance((*clip).start, (*clip).end),
                            "" // PORT-NOTE(mFileName): CGhoul2Info_v indexing/mFileName
                               // string extraction is a Ghoul2System-owned accessor not
                               // yet reachable from this crate — escalated.
                        ),
                    );
                }

                // The ported `g2api_collision_detect[_cache]` fns return the
                // distance-sorted, populated collision records as an owned
                // `Vec<CollisionRecord_t>` (the out-param array + `CMiniHeap
                // *G2VertSpace` scratch arg both drop, per the ghoul2-server
                // design); `(*touch).ghoul2` is the opaque `*mut CGhoul2Info_v`.
                let ghoul2 = &mut *((*touch).ghoul2 as *mut CGhoul2Info_v);
                // SAFETY: view-constructor slot, single-threaded; this ghoul2
                // cast aliases `view.g2`, but the g2api callees take it directly
                // and never re-cast that slot for the borrow's duration (rule 7).
                let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
                // Real `&mut Server` is in scope — read `svs.time` directly
                // rather than through the sv-touching `sv_time()` view method.
                let sv_time = sv.svs.time;
                let g2trace = if view.cvar_integer("com_optvehtrace") != 0
                    && (*touch).s.eType == mp_bg::public::entity_type::entityType_t::ET_NPC as c_int
                    && (*touch).s.NPC_class == class_t::CLASS_VEHICLE as c_int
                    && !(*touch).m_pVehicle.is_null()
                {
                    //for vehicles cache the transform data.
                    g2api_collision_detect_cache(
                        g2,
                        view,
                        ghoul2,
                        angles2,
                        (*touch).r.currentOrigin,
                        sv_time,
                        (*touch).s.number,
                        (*clip).start,
                        (*clip).end,
                        (*touch).modelScale,
                        0,
                        (*clip).useLod,
                        fRadius,
                    )
                } else {
                    g2api_collision_detect(
                        g2,
                        view,
                        ghoul2,
                        angles2,
                        (*touch).r.currentOrigin,
                        sv_time,
                        (*touch).s.number,
                        (*clip).start,
                        (*clip).end,
                        (*touch).modelScale,
                        0,
                        (*clip).useLod,
                        fRadius,
                    )
                };

                // The returned records are the populated entries only
                // (distance-ordered), so the oracle's "stop at the first
                // `mEntityNum == -1`" scan reduces to the first record matching
                // this entity.
                let bestTr = g2trace
                    .iter()
                    .position(|r| r.mEntityNum == (*touch).s.number)
                    .map_or(-1i32, |i| i as i32);

                if bestTr == -1 {
                    // Well then, put the trace back to the old one.
                    (*clip).trace = oldTrace;
                } else {
                    // Otherwise, set the endpos/normal/etc. to the model
                    // location hit instead of leaving it out in space.
                    let bt = bestTr as usize;
                    VectorCopy(g2trace[bt].mCollisionPosition, &mut (*clip).trace.endpos);
                    VectorCopy(
                        g2trace[bt].mCollisionNormal,
                        &mut (*clip).trace.plane.normal,
                    );

                    if (*clip).traceFlags & G2TRFLAG_GETSURFINDEX != 0 {
                        //we have requested that surfaceFlags be stomped over
                        //with the g2 hit surface index.
                        if (*clip).trace.entityNum == g2trace[bt].mEntityNum as core::ffi::c_short {
                            (*clip).trace.surfaceFlags = g2trace[bt].mSurfaceIndex;
                        }
                    }
                }
            }
        }
    }
}

/// Raven `SV_Trace`.
///
/// Source: `oracle/codemp/server/sv_world.cpp:803-862`
#[allow(clippy::too_many_arguments)]
pub fn SV_Trace(
    view: &mut EngineHostView,
    results: *mut trace_t,
    start: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    end: vec3_t,
    passEntityNum: c_int,
    contentmask: c_int,
    capsule: c_int,
    traceFlags: c_int,
    useLod: c_int,
) {
    // PORT-NOTE(nullable-vec3): Raven's `!mins`/`!maxs` → vec3_origin guard
    // tests possibly-NULL `const vec3_t` pointers; the resolved signature takes
    // `mins`/`maxs` by value, so the substitution moved to the G_TRACE/
    // G_G2TRACE/G_TRACECAPSULE syscall arms (`sv_game.rs vma_vec3_or_origin`),
    // where the game module's NULL word still arrives (bot-AI trap_Trace).

    unsafe {
        let mut clip: moveclip_t = core::mem::zeroed();

        mp_engine_qcommon::common_fns::Com_Memset(
            &mut clip as *mut moveclip_t as *mut (),
            0,
            core::mem::size_of::<moveclip_t>(),
        );

        // clip to world
        mp_engine_qcommon::cm_trace::CM_BoxTrace(
            view,
            &mut clip.trace,
            start,
            end,
            mins,
            maxs,
            0,
            contentmask,
            capsule,
        );
        clip.trace.entityNum = if clip.trace.fraction != 1.0 {
            ENTITYNUM_WORLD as core::ffi::c_short
        } else {
            ENTITYNUM_NONE as core::ffi::c_short
        };
        if clip.trace.fraction == 0.0 {
            *results = clip.trace;
            return; // blocked immediately by the world
        }

        clip.contentmask = contentmask;

        VectorCopy(start, &mut clip.start);
        clip.traceFlags = traceFlags;
        clip.useLod = useLod;

        VectorCopy(end, &mut clip.end);
        clip.mins = mins.as_ptr();
        clip.maxs = maxs.as_ptr();
        clip.passEntityNum = passEntityNum;
        clip.capsule = capsule;

        // create the bounding box of the entire move
        // we can limit it to the part of the move not
        // already clipped off by the world, which can be
        // a significant savings for line of sight and shot traces
        for i in 0..3 {
            if end[i] > start[i] {
                clip.boxmins[i] = clip.start[i] + mins[i] - 1.0;
                clip.boxmaxs[i] = clip.end[i] + maxs[i] + 1.0;
            } else {
                clip.boxmins[i] = clip.end[i] + mins[i] - 1.0;
                clip.boxmaxs[i] = clip.start[i] + maxs[i] + 1.0;
            }
        }

        // clip to other solid entities
        // SAFETY: view-constructor slot, single-threaded, no other live cast of
        // this slot for the borrow's duration; `SV_ClipMoveToEntities` takes the
        // cast `sv` directly and never re-casts `view.sv` (rule 7).
        let sv = &mut *(view.sv.as_raw() as *mut Server);
        SV_ClipMoveToEntities(view, sv, &mut clip);

        *results = clip.trace;
    }
}

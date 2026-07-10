#![allow(non_snake_case, non_camel_case_types)]
//! `cm_test.cpp` — leaf/brush enumeration walks, point/area contents queries,
//! and area-portal flood-fill state.
//!
//! Source: `oracle/codemp/qcommon/cm_test.cpp`
//!
//! PORT-NOTE(vector-math): `DotProduct`/`VectorCopy`/`VectorSubtract` route
//! through `mp_qshared::shared::q_math`'s reachable `_DotProduct`/
//! `_VectorCopy`/`_VectorSubtract` (rosetta vec3/q_math mapping).
//! `AngleVectors`/`BoxOnPlaneSide` still have no reachable home in this
//! crate's dependency graph (their only Rust port lives in `mp_game`, a tier
//! above the engine) — forward-declared below; escalated as missing symbols
//! for the finisher to wire to a q_math home reachable from
//! `mp_engine_qcommon`.

use core::ffi::c_int;

use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::surface_flags::CONTENTS_TERRAIN;
use mp_qshared::shared::{clipHandle_t, qboolean, qfalse, qtrue, vec3_t};

use crate::cm::c_area_t::cArea_t;
use crate::cm::c_leaf_t::cLeaf_t;
use crate::cm::c_node_t::cNode_t;
use crate::cm::cbrush_s::cbrush_t;
use crate::cm::clip_map_t::clipMap_t;
use crate::cm::cm_local_consts::BOX_MODEL_HANDLE;
use crate::cm::cmodel_s::cmodel_t;
use crate::cm::leaf_list_s::leafList_t;
use crate::cm_load::{CCMLandScape, CM_ClipHandleToModel};
use crate::collision_world::CollisionWorld;
use crate::common::Common;
use crate::common_fns::Com_Memset;
use mp_qshared::shared::q_math::{_DotProduct, _VectorCopy, _VectorSubtract};

// PORT-NOTE(q_math-reach continued): `BoxOnPlaneSide`/`AngleVectors` have no
// home reachable from this crate (only ported in `mp_game`, a tier above);
// forward-declared here in the established `extern "Rust"` shape (vm_fns.rs/
// cm_load.rs precedent), narrowed to their `mp_game::q_math` signatures;
// escalated as missing symbols for the finisher.
extern "Rust" {
    fn BoxOnPlaneSide(
        emins: vec3_t,
        emaxs: vec3_t,
        p: *mut mp_qshared::shared::collision::cplane_t,
    ) -> c_int;
    fn AngleVectors(
        angles: vec3_t,
        forward: Option<&mut vec3_t>,
        right: Option<&mut vec3_t>,
        up: Option<&mut vec3_t>,
    );
}

/// Raven `byte`.
type byte = u8;

/// Raven `CM_PointLeafnum_r`.
///
/// Source: `oracle/codemp/qcommon/cm_test.cpp:16-44`
pub fn CM_PointLeafnum_r(
    cm: &mut CollisionWorld,
    p: vec3_t,
    mut num: c_int,
    local: *mut clipMap_t,
) -> c_int {
    unsafe {
        while num >= 0 {
            let node: *mut cNode_t = (*local).nodes.add(num as usize);
            let plane = (*node).plane;

            let d = if (*plane).r#type < 3 {
                p[(*plane).r#type as usize] - (*plane).dist
            } else {
                _DotProduct((*plane).normal, p) - (*plane).dist
            };
            if d < 0.0 {
                num = (*node).children[1];
            } else {
                num = (*node).children[0];
            }
        }

        cm.c_pointcontents += 1; // optimize counter

        -1 - num
    }
}

/// Raven `CM_StoreLeafs`.
///
/// Source: `oracle/codemp/qcommon/cm_test.cpp:63-78`
pub fn CM_StoreLeafs(cm: &mut CollisionWorld, ll: *mut leafList_t, nodenum: c_int) {
    unsafe {
        let leafNum = -1 - nodenum;

        // store the lastLeaf even if the list is overflowed
        if (*cm.cmg.leafs.add(leafNum as usize)).cluster != -1 {
            (*ll).lastLeaf = leafNum;
        }

        if (*ll).count >= (*ll).maxcount {
            (*ll).overflowed = qtrue;
            return;
        }
        let count = (*ll).count;
        (*ll).count += 1;
        *(*ll).list.add(count as usize) = leafNum;
    }
}

/// Raven `CM_StoreBrushes`.
///
/// Source: `oracle/codemp/qcommon/cm_test.cpp:80-121`
pub fn CM_StoreBrushes(cm: &mut CollisionWorld, ll: *mut leafList_t, nodenum: c_int) {
    unsafe {
        let leafnum = -1 - nodenum;

        let leaf: *mut cLeaf_t = cm.cmg.leafs.add(leafnum as usize);

        for k in 0..(*leaf).numLeafBrushes {
            let brushnum = *cm
                .cmg
                .leafbrushes
                .add(((*leaf).firstLeafBrush + k) as usize);
            let b: *mut cbrush_t = cm.cmg.brushes.add(brushnum as usize);
            if (*b).checkcount as c_int == cm.cmg.checkcount {
                continue; // already checked this brush in another leaf
            }
            (*b).checkcount = cm.cmg.checkcount as u16;
            let mut i = 0;
            while i < 3 {
                if (*b).bounds[0][i] >= (*ll).bounds[1][i]
                    || (*b).bounds[1][i] <= (*ll).bounds[0][i]
                {
                    break;
                }
                i += 1;
            }
            if i != 3 {
                continue;
            }
            if (*ll).count >= (*ll).maxcount {
                (*ll).overflowed = qtrue;
                return;
            }
            let count = (*ll).count;
            (*ll).count += 1;
            *((*ll).list as *mut *mut cbrush_t).add(count as usize) = b;
        }
        // #if 0 patch-storing block dropped: dead code under retail build (Raven's own #if 0).
    }
}

/// Raven `CM_BoxLeafnums_r`.
///
/// Source: `oracle/codemp/qcommon/cm_test.cpp:130-161`
pub fn CM_BoxLeafnums_r(cm: &mut CollisionWorld, ll: *mut leafList_t, mut nodenum: c_int) {
    unsafe {
        loop {
            if nodenum < 0 {
                if let Some(store) = (*ll).storeLeafs {
                    store(cm, ll, nodenum);
                }
                return;
            }

            let node: *mut cNode_t = cm.cmg.nodes.add(nodenum as usize);
            let plane = (*node).plane;

            let s = BoxOnPlaneSide((*ll).bounds[0], (*ll).bounds[1], plane);
            if s == 1 {
                nodenum = (*node).children[0];
            } else if s == 2 {
                nodenum = (*node).children[1];
            } else {
                // go down both
                CM_BoxLeafnums_r(cm, ll, (*node).children[0]);
                nodenum = (*node).children[1];
            }
        }
    }
}

/// Raven `CM_ClusterPVS`.
///
/// Source: `oracle/codemp/qcommon/cm_test.cpp:351-357`
pub fn CM_ClusterPVS(cm: &mut CollisionWorld, cluster: c_int) -> *mut byte {
    unsafe {
        if cluster < 0 || cluster >= cm.cmg.numClusters || cm.cmg.vised == qfalse {
            return cm.cmg.visibility;
        }

        cm.cmg
            .visibility
            .add((cluster * cm.cmg.clusterBytes) as usize)
    }
}

/// Raven `CM_PointLeafnum`.
///
/// Source: `oracle/codemp/qcommon/cm_test.cpp:46-51`
pub fn CM_PointLeafnum(cm: &mut CollisionWorld, p: vec3_t) -> c_int {
    if cm.cmg.numNodes == 0 {
        // map not loaded
        return 0;
    }
    let cmg = core::ptr::addr_of_mut!(cm.cmg);
    CM_PointLeafnum_r(cm, p, 0, cmg)
}

/// Raven `CM_BoxLeafnums`.
///
/// Source: `oracle/codemp/qcommon/cm_test.cpp:168-187`
pub fn CM_BoxLeafnums(
    cm: &mut CollisionWorld,
    mins: vec3_t,
    maxs: vec3_t,
    boxList: *mut c_int,
    listsize: c_int,
    lastLeaf: *mut c_int,
) -> c_int {
    // rwwRMG - changed to boxList to not conflict with list type
    let mut ll: leafList_t = unsafe { core::mem::zeroed() };

    cm.cmg.checkcount += 1;

    _VectorCopy(mins, &mut ll.bounds[0]);
    _VectorCopy(maxs, &mut ll.bounds[1]);
    ll.count = 0;
    ll.maxcount = listsize;
    ll.list = boxList;
    ll.storeLeafs = Some(CM_StoreLeafs);
    ll.lastLeaf = 0;
    ll.overflowed = qfalse;

    CM_BoxLeafnums_r(cm, &mut ll, 0);

    unsafe {
        *lastLeaf = ll.lastLeaf;
    }
    ll.count
}

/// Raven `CM_BoxBrushes`.
///
/// Source: `oracle/codemp/qcommon/cm_test.cpp:194-212`
pub fn CM_BoxBrushes(
    cm: &mut CollisionWorld,
    mins: vec3_t,
    maxs: vec3_t,
    boxList: *mut *mut cbrush_t,
    listsize: c_int,
) -> c_int {
    // rwwRMG - changed to boxList to not conflict with list type
    let mut ll: leafList_t = unsafe { core::mem::zeroed() };

    cm.cmg.checkcount += 1;

    _VectorCopy(mins, &mut ll.bounds[0]);
    _VectorCopy(maxs, &mut ll.bounds[1]);
    ll.count = 0;
    ll.maxcount = listsize;
    ll.list = boxList as *mut c_int;
    ll.storeLeafs = Some(CM_StoreBrushes);
    ll.lastLeaf = 0;
    ll.overflowed = qfalse;

    CM_BoxLeafnums_r(cm, &mut ll, 0);

    ll.count
}

/// Raven `CM_WriteAreaBits`.
///
/// Source: `oracle/codemp/qcommon/cm_test.cpp:545-572`
pub fn CM_WriteAreaBits(cm: &mut CollisionWorld, buffer: *mut byte, area: c_int) -> c_int {
    let bytes = (cm.cmg.numAreas + 7) >> 3;

    // PORT-NOTE(bspc): `#ifndef BSPC` retail arm kept; `BSPC` has no rosetta row
    // (bsp-compiler-only build, not part of the engine port surface).
    if unsafe { (*cm.cm_noAreas).integer } != 0 || area == -1 {
        // for debugging, send everything
        Com_Memset(buffer as *mut (), 255, bytes as usize);
    } else {
        let floodnum = unsafe { (*cm.cmg.areas.add(area as usize)).floodnum };
        for i in 0..cm.cmg.numAreas {
            if unsafe { (*cm.cmg.areas.add(i as usize)).floodnum } == floodnum || area == -1 {
                unsafe {
                    *buffer.add((i >> 3) as usize) |= 1 << (i & 7);
                }
            }
        }
    }

    bytes
}

/// Raven `CM_FloodArea_r`.
///
/// Source: `oracle/codemp/qcommon/cm_test.cpp:395-416`
pub fn CM_FloodArea_r(areaNum: c_int, floodnum: c_int, cm: &mut clipMap_t) {
    let area: *mut cArea_t = unsafe { cm.areas.add(areaNum as usize) };

    unsafe {
        if (*area).floodvalid == cm.floodvalid {
            if (*area).floodnum == floodnum {
                return;
            }
            crate::common::error::com_error(
                errorParm_t::ERR_DROP,
                "FloodArea_r: reflooded".to_string(),
            );
        }

        (*area).floodnum = floodnum;
        (*area).floodvalid = cm.floodvalid;
        let con = cm.areaPortals.add((areaNum * cm.numAreas) as usize);
        for i in 0..cm.numAreas {
            if *con.add(i as usize) > 0 {
                CM_FloodArea_r(i, floodnum, cm);
            }
        }
    }
}

/// Raven `CM_PointContents`.
///
/// Source: `oracle/codemp/qcommon/cm_test.cpp:224-296`
pub fn CM_PointContents(cm: &mut CollisionWorld, p: vec3_t, model: clipHandle_t) -> c_int {
    unsafe {
        if cm.cmg.numNodes == 0 {
            // map not loaded
            return 0;
        }

        let leaf: *mut cLeaf_t;
        let local: *mut clipMap_t;
        if model != 0 {
            let mut local_out: *mut clipMap_t = core::ptr::null_mut();
            let clipm: *mut cmodel_t = CM_ClipHandleToModel(cm, model, &mut local_out);
            local = local_out;
            if (*clipm).firstNode != -1 {
                let leafnum = CM_PointLeafnum_r(cm, p, 0, local);
                leaf = (*local).leafs.add(leafnum as usize);
            } else {
                leaf = &mut (*clipm).leaf;
            }
        } else {
            local = core::ptr::addr_of_mut!(cm.cmg);
            let leafnum = CM_PointLeafnum_r(cm, p, 0, local);
            leaf = (*local).leafs.add(leafnum as usize);
        }

        let mut contents: c_int = 0;
        for k in 0..(*leaf).numLeafBrushes {
            let brushnum = *(*local)
                .leafbrushes
                .add(((*leaf).firstLeafBrush + k) as usize);
            let b: *mut cbrush_t = (*local).brushes.add(brushnum as usize);

            // see if the point is in the brush
            let mut i: u16 = 0;
            while i < (*b).numsides {
                let side = (*b).sides.add(i as usize);
                let d = _DotProduct(p, (*(*side).plane).normal);
                // FIXME test for Cash
                // if ( d >= b->sides[i].plane->dist ) {
                if d > (*(*side).plane).dist {
                    break;
                }
                i += 1;
            }

            if i == (*b).numsides {
                contents |= (*b).contents;
                if !cm.cmg.landScape.is_null() && (contents & CONTENTS_TERRAIN) != 0 {
                    if p[2] < cm.terrain_water_height() {
                        contents |= cm.terrain_water_contents();
                    }
                }
            }
        }

        contents
    }
}

/// Raven `CM_FloodAreaConnections`.
///
/// Source: `oracle/codemp/qcommon/cm_test.cpp:448-466`
pub fn CM_FloodAreaConnections(cm: &mut clipMap_t) {
    // all current floods are now invalid
    cm.floodvalid += 1;
    let mut floodnum = 0;

    for i in 0..cm.numAreas {
        let area: *mut cArea_t = unsafe { cm.areas.add(i as usize) };
        if unsafe { (*area).floodvalid } == cm.floodvalid {
            continue; // already flooded into
        }
        floodnum += 1;
        CM_FloodArea_r(i, floodnum, cm);
    }
}

/// Raven `CM_TransformedPointContents`.
///
/// Source: `oracle/codemp/qcommon/cm_test.cpp:306-327`
pub fn CM_TransformedPointContents(
    cm: &mut CollisionWorld,
    p: vec3_t,
    model: clipHandle_t,
    origin: vec3_t,
    angles: vec3_t,
) -> c_int {
    let mut p_l: vec3_t = [0.0; 3];
    let mut temp: vec3_t;

    // subtract origin offset
    _VectorSubtract(p, origin, &mut p_l);

    // rotate start and end into the models frame of reference
    if model != BOX_MODEL_HANDLE as clipHandle_t
        && (angles[0] != 0.0 || angles[1] != 0.0 || angles[2] != 0.0)
    {
        let mut forward: vec3_t = [0.0; 3];
        let mut right: vec3_t = [0.0; 3];
        let mut up: vec3_t = [0.0; 3];
        unsafe {
            AngleVectors(angles, Some(&mut forward), Some(&mut right), Some(&mut up));
        }

        temp = p_l;
        p_l[0] = _DotProduct(temp, forward);
        p_l[1] = -_DotProduct(temp, right);
        p_l[2] = _DotProduct(temp, up);
    }

    CM_PointContents(cm, p_l, model)
}

/// Raven `CM_AdjustAreaPortalState`.
///
/// Source: `oracle/codemp/qcommon/cm_test.cpp:476-501`
pub fn CM_AdjustAreaPortalState(
    cm: &mut CollisionWorld,
    area1: c_int,
    area2: c_int,
    open: qboolean,
) {
    if area1 < 0 || area2 < 0 {
        return;
    }

    if area1 >= cm.cmg.numAreas || area2 >= cm.cmg.numAreas {
        crate::common::error::com_error(
            errorParm_t::ERR_DROP,
            "CM_ChangeAreaPortalState: bad area number".to_string(),
        );
    }

    unsafe {
        if open != 0 {
            *cm.cmg
                .areaPortals
                .add((area1 * cm.cmg.numAreas + area2) as usize) += 1;
            *cm.cmg
                .areaPortals
                .add((area2 * cm.cmg.numAreas + area1) as usize) += 1;
        } else {
            *cm.cmg
                .areaPortals
                .add((area1 * cm.cmg.numAreas + area2) as usize) -= 1;
            *cm.cmg
                .areaPortals
                .add((area2 * cm.cmg.numAreas + area1) as usize) -= 1;
            if *cm
                .cmg
                .areaPortals
                .add((area2 * cm.cmg.numAreas + area1) as usize)
                < 0
            {
                crate::common::error::com_error(
                    errorParm_t::ERR_DROP,
                    "CM_AdjustAreaPortalState: negative reference count".to_string(),
                );
            }
        }
    }

    CM_FloodAreaConnections(&mut cm.cmg);
}

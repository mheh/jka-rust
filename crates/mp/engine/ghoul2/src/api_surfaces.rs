//! `G2API` surfaces — the per-surface on/off override list mutators/readers,
//! the root-surface (LOD-swap parent) setter, generated-surface add/remove,
//! and the name/index/parent lookups + debug lister.
//!
//! Per `docs/subsystems/ghoul2-server.md` roster (`api_surfaces.rs`, class
//! "G2API surfaces"): `SetSurfaceOnOff`/`GetSurfaceOnOff`/`SetRootSurface`/
//! `AddSurface`/`RemoveSurface`/`GetParentSurface`/`GetSurfaceIndex`/
//! `GetSurfaceName`/`GetSurfaceRenderStatus`/`ListSurfaces`. The internal
//! `G2_Surfaces` helpers these dispatch into (`G2_SetSurfaceOnOff`,
//! `G2_IsSurfaceOff`, `G2_SetRootSurface`, `G2_AddSurface`, `G2_RemoveSurface`,
//! `G2_GetParentSurface`, `G2_GetSurfaceIndex`, `G2_IsSurfaceRendered`, …) are
//! a separate roster row (`surfaces.rs`, class "G2_Surfaces internal").
//!
//! Every entry threads `g2: &mut Ghoul2System` (ruling 4/11, state threaded
//! not reached). Per-instance overloads keep the oracle's exact receiver
//! shape (`G2SV-D6`, 1:1 signature): functions declared on `CGhoul2Info_v
//! &ghoul2` take `ghoul2: &mut CGhoul2Info_v` (`SetSurfaceOnOff` collapses it
//! to `ghoul2[0]` internally, matching `api_bolts.rs`'s `g2api_set_new_origin`;
//! `SetRootSurface` additionally takes the oracle's own `model_index`), while
//! functions declared directly on `CGhoul2Info *ghlInfo` take
//! `ghl_info: &mut CGhoul2Info` (matching `api_bolts.rs`'s `g2api_remove_bolt`/
//! `g2api_add_bolt_surf_num`).
//!
//! Per the doc's "Slice hooks" per-file host-service map, `api_surfaces.rs` is
//! a **thin-wrapper** file: its own §C7 marshalling is host-free, but every
//! entry opens with `G2_SetupModelPointers` (a loader model-memory read,
//! `model_mdxm`/`model_mdxa`, ruling 36) and forwards into an internal
//! (`surfaces.rs`, itself listed Host-consuming — its live model reads resolve
//! `currentModel`/`animModel` through `EngineHost` rather than caching a raw
//! `mp_renderer` pointer, `G2SV-D5`), so **every** entry here threads
//! `host: &mut impl EngineHost` alongside `g2`, even `GetSurfaceOnOff` (no
//! explicit `G2_SetupModelPointers` call in its own body, but its
//! `G2_IsSurfaceOff` forward reads model memory the same way). `GetSurfaceName`
//! and `ListSurfaces` additionally call `Com_Error`/`Com_Printf` directly —
//! also routed through `host`.

use mp_host_interface::EngineHost;
use mp_qshared::shared::error_parm::errorParm_t;

use crate::ghoul2_system::Ghoul2System;
use crate::shared::cghoul2_info::CGhoul2Info;
use crate::shared::cghoul2_info_v::CGhoul2Info_v;

/// Raven `qboolean G2API_SetSurfaceOnOff(CGhoul2Info_v &ghoul2, const char
/// *surfaceName, const int flags)` — collapses to `ghoul2[0]` (`qfalse` when
/// `ghoul2` is empty), then `qfalse` on `G2_SetupModelPointers` failure, else
/// flushes the mesh-frame cache and forwards to `G2_SetSurfaceOnOff`.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:706-722`
pub fn g2api_set_surface_on_off(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    surface_name: &str,
    flags: i32,
) -> bool {
    // Raven: `(int)&ghoul2 && ghoul2.size()>0` — the address-of-reference
    // half is never zero, so this collapses to a plain non-empty check.
    if ghoul2.size(g2) <= 0 {
        return false;
    }
    let ghl_info = ghoul2.get_mut(g2, 0);
    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return false;
    }
    // ensure we flush the cache
    ghl_info.mesh_frame_num = 0;
    crate::surfaces::g2_set_surface_on_off(host, ghl_info, surface_name, flags)
}

/// Raven `int G2API_GetSurfaceOnOff(CGhoul2Info *ghlInfo, const char
/// *surfaceName)` — `-1` on a null instance (no `G2_SetupModelPointers` gate
/// here, unlike its siblings), else `G2_IsSurfaceOff`'s off-flags bitmask
/// (which itself reads `ghlInfo`'s already-resolved model memory).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:724-731`
pub fn g2api_get_surface_on_off(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    surface_name: &str,
) -> i32 {
    // Raven's `if (ghlInfo)` null-guard has no analog: `ghl_info` is a live
    // `&mut CGhoul2Info` reference, never null, so this always forwards.
    let _ = g2;
    let slist = &ghl_info.slist;
    crate::surfaces::g2_is_surface_off(host, ghl_info, slist, surface_name)
}

/// Raven `qboolean G2API_SetRootSurface(CGhoul2Info_v &ghoul2, const int
/// modelIndex, const char *surfaceName)` — `qfalse` on `G2_SetupModelPointers`
/// failure, else `G2_SetRootSurface`'s result (sets the LOD-swap root surface
/// on `ghoul2[modelIndex]`).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:733-741`
pub fn g2api_set_root_surface(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    model_index: i32,
    surface_name: &str,
) -> bool {
    if !crate::misc::g2_setup_model_pointers_v(g2, host, ghoul2) {
        return false;
    }
    crate::surfaces::g2_set_root_surface(g2, host, ghoul2, model_index, surface_name)
}

/// Raven `int G2API_AddSurface(CGhoul2Info *ghlInfo, int surfaceNumber, int
/// polyNumber, float BarycentricI, float BarycentricJ, int lod)` — `-1` on
/// `G2_SetupModelPointers` failure, else flushes the mesh-frame cache and
/// returns `G2_AddSurface`'s new generated-surface index.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:743-752`
#[allow(clippy::too_many_arguments)]
pub fn g2api_add_surface(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    surface_number: i32,
    poly_number: i32,
    barycentric_i: f32,
    barycentric_j: f32,
    lod: i32,
) -> i32 {
    let _ = g2;
    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return -1;
    }
    // ensure we flush the cache
    ghl_info.mesh_frame_num = 0;
    crate::surfaces::g2_add_surface(
        host,
        ghl_info,
        surface_number,
        poly_number,
        barycentric_i,
        barycentric_j,
        lod,
    )
}

/// Raven `qboolean G2API_RemoveSurface(CGhoul2Info *ghlInfo, const int
/// index)` — `qfalse` on `G2_SetupModelPointers` failure, else flushes the
/// mesh-frame cache and returns `G2_RemoveSurface`'s result.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:754-763`
pub fn g2api_remove_surface(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    index: i32,
) -> bool {
    let _ = g2;
    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return false;
    }
    // ensure we flush the cache
    ghl_info.mesh_frame_num = 0;
    crate::surfaces::g2_remove_surface(&mut ghl_info.slist, index)
}

/// Raven `int G2API_GetParentSurface(CGhoul2Info *ghlInfo, const int index)`
/// — `-1` on `G2_SetupModelPointers` failure, else `G2_GetParentSurface`'s
/// result.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:765-772`
pub fn g2api_get_parent_surface(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    index: i32,
) -> i32 {
    let _ = g2;
    if crate::misc::g2_setup_model_pointers(host, ghl_info) {
        crate::surfaces::g2_get_parent_surface(host, ghl_info, index)
    } else {
        -1
    }
}

/// Raven `int G2API_GetSurfaceIndex(CGhoul2Info *ghlInfo, const char
/// *surfaceName)` — `-1` on `G2_SetupModelPointers` failure, else
/// `G2_GetSurfaceIndex`'s result.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2403-2410`
pub fn g2api_get_surface_index(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    surface_name: &str,
) -> i32 {
    let _ = g2;
    if crate::misc::g2_setup_model_pointers(host, ghl_info) {
        crate::surfaces::g2_get_surface_index(host, ghl_info, surface_name)
    } else {
        -1
    }
}

/// Raven `char *G2API_GetSurfaceName(CGhoul2Info *ghlInfo, int surfNumber)` —
/// always returns a valid name (the static empty-string fallback `noSurface`
/// on `G2_SetupModelPointers` failure, an out-of-range `surfNumber`, or an
/// unfound surface; never `NULL`), so the Rust return is an owned `String`,
/// not `Option`. Reads model memory (`mod->mdxm`) via the internal
/// index-based `G2_FindSurface` (`misc.rs`) and calls `Com_Error`/`Com_Printf`
/// directly on the bad-model (`#ifndef FINAL_BUILD`) and invalid-`surfNumber`
/// paths — both routed through `host`.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2360-2400`
pub fn g2api_get_surface_name(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    surf_number: i32,
) -> String {
    let _ = g2;
    // Raven `static char noSurface[1] = "";` — the shared empty-string fallback.
    const NO_SURFACE: &str = "";

    // `mdxmHeader_t` layout (oracle/codemp/renderer/mdx_format.h:151-172): int
    // ident, int version, char name[64], char animName[64], int animIndex, int
    // numBones, int numLODs, int ofsLODs, int numSurfaces, int
    // ofsSurfHierarchy, int ofsEnd — every field 4-byte-aligned with no
    // padding, so `numSurfaces` sits at byte offset 152 and
    // `sizeof(mdxmHeader_t) == 164` (the `mdxmHierarchyOffsets_t` follows
    // immediately, `:2394`). `mdxmSurfHierarchy_t::name` (`:189`) is its first
    // field, a NUL-terminated `MAX_QPATH` (64-byte) buffer. This crate never
    // names the `mdxm*` types (`G2SV-D5`); the offsets below are the same raw
    // byte arithmetic Raven itself does off the loader-owned block.
    const NUM_SURFACES_OFFSET: usize = 152;
    const HEADER_SIZE: usize = 164;
    const SURF_NAME_LEN: usize = 64;

    if !crate::misc::g2_setup_model_pointers(host, ghl_info) {
        return NO_SURFACE.to_string();
    }

    let mdxm = host.model_mdxm(ghl_info.model);
    if mdxm.is_null() {
        host.error(
            errorParm_t::ERR_DROP,
            &format!(
                "G2API_GetSurfaceName: Bad model on instance {}.",
                ghl_info.file_name
            ),
        );
    }

    let num_surfaces = unsafe { *(mdxm.byte_add(NUM_SURFACES_OFFSET) as *const i32) };

    // ok, I guess it's semi-valid for the user to be passing in surface > numSurfs
    // because they don't know how many surfs a model may have.. but how did they
    // get that surf index to begin with? Oh well.
    if surf_number < 0 || surf_number >= num_surfaces {
        host.print(&format!(
            "G2API_GetSurfaceName: You passed in an invalid surface number ({}) for model {}.\n",
            surf_number, ghl_info.file_name
        ));
        return NO_SURFACE.to_string();
    }

    // The index-based `G2_FindSurface` (`misc.rs`) unconditionally returns a
    // computed index — no found/not-found signal (misc.rs's shape-choice
    // note) — so the oracle's `if (surf)` guard is always taken here.
    let this_surface_index = crate::misc::g2_find_surface(host, ghl_info.model, surf_number, 0);

    if this_surface_index < 0 || this_surface_index >= num_surfaces {
        host.error(
            errorParm_t::ERR_DROP,
            &format!(
                "G2API_GetSurfaceName: Bad surf num ({}) on surf for instance {}.",
                this_surface_index, ghl_info.file_name
            ),
        );
    }

    let surf_indexes = unsafe { mdxm.byte_add(HEADER_SIZE) };
    let offset = unsafe { *(surf_indexes.byte_add(this_surface_index as usize * 4) as *const i32) };
    let surf_info = unsafe { surf_indexes.byte_offset(offset as isize) };
    let name_bytes = unsafe { core::slice::from_raw_parts(surf_info as *const u8, SURF_NAME_LEN) };
    let end = name_bytes
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(SURF_NAME_LEN);
    String::from_utf8_lossy(&name_bytes[..end]).into_owned()
}

/// Raven `int G2API_GetSurfaceRenderStatus(CGhoul2Info *ghlInfo, const char
/// *surfaceName)` — `-1` on `G2_SetupModelPointers` failure, else
/// `G2_IsSurfaceRendered`'s result.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:774-781`
pub fn g2api_get_surface_render_status(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    surface_name: &str,
) -> i32 {
    let _ = g2;
    if crate::misc::g2_setup_model_pointers(host, ghl_info) {
        let slist = &ghl_info.slist;
        crate::surfaces::g2_is_surface_rendered(host, ghl_info, surface_name, slist)
    } else {
        -1
    }
}

/// Raven `void G2API_ListSurfaces(CGhoul2Info *ghlInfo)` — a no-op on
/// `G2_SetupModelPointers` failure, else forwards `ghlInfo->mFileName` to the
/// internal `G2_List_Model_Surfaces` (`misc.rs`), which walks `mod->mdxm`'s
/// surface hierarchy and prints every surface (+ descendants when
/// `r_verbose` is set) via `Com_Printf` — routed through `host`.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:1903-1909`
pub fn g2api_list_surfaces(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
) {
    let _ = g2;
    if crate::misc::g2_setup_model_pointers(host, ghl_info) {
        crate::misc::g2_list_model_surfaces(host, &ghl_info.file_name);
    }
}

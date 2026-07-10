//! `G2_Surfaces` internal — the per-surface on/off override list search/mutate
//! helpers, the root-surface (LOD-swap parent) setter and its recursive active-
//! surface walk, and the name/index/parent/render-status lookups the `G2API`
//! surface wrappers (`api_surfaces.rs`) forward into.
//!
//! Per `docs/subsystems/ghoul2-server.md` roster (`surfaces.rs`, class
//! "G2_Surfaces internal"): `G2_SetSurfaceOnOff`/`IsSurfaceOff`/`SetRootSurface`/
//! `AddSurface`/`RemoveSurface`/`FindOverrideSurface`/`IsSurfaceLegal`/
//! `GetParentSurface`/`GetSurfaceIndex`/`IsSurfaceRendered` — the doc's
//! roster-row one-liner. Enumerating the oracle TU (`oracle/codemp/ghoul2/
//! G2_surfaces.cpp`) directly per porting-rules §F17/CLAUDE.md ("private
//! helpers included") turns up four more functions physically defined in this
//! TU that the one-liner does not name but that belong here all the same:
//! - `G2_SetSurfaceOnOffFromSkin` (`:201-226`) — not header-declared in
//!   `G2_local.h`, but forward-declared ad hoc and called from `G2_API.cpp:680,688`
//!   (`G2API_SetSkin`, `api_models.rs`'s `g2api_set_skin`), so it is reached
//!   cross-file and must stay `pub`.
//! - `G2_FindRecursiveSurface` (`:266-304`) and
//!   `G2_RemoveRedundantGeneratedSurfaces` (`:306-335`) — genuine file-private
//!   helpers (no `G2_local.h` declaration, no caller outside this TU) used only
//!   by `G2_SetRootSurface`'s active-surface walk.
//! - The **name-based** `G2_FindSurface` overload (`:98-145`,
//!   `G2_FindSurface(CGhoul2Info*, surfaceInfo_v&, const char*, int*)`) — a
//!   private helper of `SetSurfaceOnOff`/`IsSurfaceOff`/`IsSurfaceRendered`
//!   below. **Distinct** from the index-based overload `void
//!   *G2_FindSurface(void *mod, int index, int lod)` (`G2_local.h:82`, defined
//!   `G2_misc.cpp:1689`) that the doc's roster assigns to `misc.rs` — C++
//!   overload resolution on argument types picks between the two; only the
//!   `misc.rs` one is header-declared/cross-file.
//!
//! **Every function here that reaches `mod->mdxm` (`CGhoul2Info::model`, the
//! host-resolved `qhandle_t`) is Host-consuming** (`docs/subsystems/
//! ghoul2-server.md:1363`, threading `host: &mut impl EngineHost` per the
//! `## Seam definition`); the two pure list-search/mutate helpers that never
//! touch model memory (`G2_FindOverrideSurface`, `G2_RemoveSurface`) and the
//! scratch-array helper `G2_RemoveRedundantGeneratedSurfaces` are host-free.
//!
//! **Shape choice for the redundant `slist` parameter (§A1 internal latitude —
//! not part of the frozen `G2API` 1:1 surface, `G2SV-D6` only pins that outer
//! layer).** Every oracle call site of `G2_SetSurfaceOnOff` and
//! `G2_SetSurfaceOnOffFromSkin` passes `slist` as exactly `ghlInfo->mSlist`
//! (`G2_API.cpp:719`, `G2_surfaces.cpp:214,222`; grepped, no exception) and both
//! mutate it, so the two-parameter C++ shape would force two overlapping
//! borrows of the same `CGhoul2Info` at the call site for no behavioral gain;
//! they collapse to a single `ghl_info: &mut CGhoul2Info`. `G2_IsSurfaceOff` and
//! `G2_IsSurfaceRendered` only **read** `slist` (also always `ghlInfo->mSlist`,
//! `G2_API.cpp:728,778`) — a shared borrow doesn't conflict with `ghl_info`'s
//! own shared borrow, so they keep the oracle's faithful two-parameter shape.
//! `G2_RemoveSurface`/`G2_FindOverrideSurface`/`G2_FindRecursiveSurface`/
//! `G2_RemoveRedundantGeneratedSurfaces` take a bare `slist`/`root_list` with no
//! owning `CGhoul2Info` in the oracle signature at all (`G2_local.h:20-21`,
//! `G2_surfaces.cpp:266,306`) and keep that shape unchanged.
//!
//! **Shape choice for the private name-based `G2_FindSurface` (§A1).** Its
//! oracle return is `mdxmSurface_t *` (never nameable in this crate, `G2SV-D5`)
//! plus a write-through `int *surfIndex` out-param (written on both the found
//! and not-found paths — `:131-134,140-143`, `G2SV-D1`-shaped write-through).
//! All three in-crate callers below (`SetSurfaceOnOff`/`IsSurfaceOff`/
//! `IsSurfaceRendered`) test only `if (surf)` then read `slist[surfIndex]` —
//! **never** the pointer's own fields — so the pointer return carries no
//! information these callers use beyond found/not-found, which `surfIndex`
//! already encodes (`-1` iff not found). It collapses to `Option<i32>`
//! (`Some(index)` = found, `None` = `surfIndex == -1`), dropping the unused
//! opaque pointer rather than inventing a name for a type this crate must never
//! name (non-goals, type-location reconciliation).
//!
//! **Doc/oracle gaps found while enumerating this class (reported to the
//! caller, not fixed here):**
//! 1. The doc's per-file host-service map cites `surfaces.rs`'s host need as
//!    "`R_GetModelByHandle(RE_RegisterModel(...))`, `G2_surfaces.cpp:426`"
//!    (`docs/subsystems/ghoul2-server.md:1363-1364`). Line 426 sits inside a
//!    **`/* ... */`-commented-out dead code block** (`G2_surfaces.cpp:422-505`,
//!    an old `entstate->ghoul2` variant of `G2_SetRootSurface` closed by
//!    `assert(0);*/` at `:505`) — it never compiles. The conclusion (surfaces.rs
//!    is Host-consuming) still holds, just from the **live** body instead:
//!    `G2_SetRootSurface` (`:337-421`) itself dereferences
//!    `ghoul2[modelIndex].currentModel->mdxm`/`animModel->mdxa` directly
//!    (`:349,357,371-374`), and `G2_IsSurfaceLegal`/`G2_IsSurfaceOff`/
//!    `G2_IsSurfaceRendered`/the private `G2_FindSurface` all walk
//!    `mod->mdxm`'s surface hierarchy live. The citation's line number is wrong;
//!    the classification is right.
//! 2. `G2_SetSurfaceOnOffFromSkin` (`:201-226`) calls `R_GetSkinByHandle`
//!    (`:204`) to resolve `renderSkin`'s surface/shader-name table — the frozen
//!    15-method `EngineHost` (`## Seam definition`) has no skin-lookup accessor,
//!    same gap class `api_models.rs`'s module doc-comment already reports for
//!    `g2api_set_skin` (which calls this fn). Not fixed here (§A1: the stub
//!    still takes `host` per the nearest frozen parameter shape; only the
//!    skin-lookup line itself has no host method to call yet) — diverges via
//!    the frozen, real `host.error(...)` service (never invented), matching
//!    `api_models.rs`'s `register_server_model`/`register_model` treatment of
//!    the same missing-service class.
//! 3. `G2_SetRootSurface`'s live body (`:379-385`) builds a `CConstructBoneList`
//!    and calls `extern void G2_ConstructUsedBoneList(CConstructBoneList&)`
//!    (`:41`, defined `oracle/codemp/renderer/tr_ghoul2.cpp:2796`, unguarded by
//!    any `#ifndef DEDICATED`/build macro — genuinely server-live). Neither the
//!    doc's welded-renderer-bone-subset enumeration (Scope & non-goals) nor its
//!    Method transcription table names this function or assigns it a roster
//!    file; it appears to be a gap in the `tr_ghoul2.cpp` fn-extent accounting,
//!    not something this file owns (it is not a `G2_Surfaces internal` member —
//!    `CConstructBoneList` and `G2_ConstructUsedBoneList` live in the renderer
//!    TU) — flagged, not stubbed here. [`g2_set_root_surface`] below leaves its
//!    `active_bones` scratch array all-zero (its `Z_Malloc` + `memset(0)`
//!    initial state) rather than inventing the walk that would populate it.

use core::ffi::c_void;

use mp_host_interface::EngineHost;
use mp_qshared::shared::{errorParm_t, qhandle_t, MAX_QPATH};

use crate::ghoul2_system::Ghoul2System;
use crate::shared::cghoul2_info::CGhoul2Info;
use crate::shared::cghoul2_info_v::CGhoul2Info_v;
use crate::shared::surface_info_t::surfaceInfo_t;

// ---------------------------------------------------------------------------
// `G2SURFACEFLAG_*` (`oracle/codemp/renderer/mdx_format.h:41,49,50`) — only
// the three bits this file reads/writes. Duplicated locally rather than a new
// shared constants module, matching this crate's own per-file convention
// (`api_models.rs`'s `GHOUL2_NEWORIGIN`, `api_bolts.rs`'s `MODEL_WIDTH`/…).
// ---------------------------------------------------------------------------

const G2SURFACEFLAG_OFF: i32 = 0x00000002;
const G2SURFACEFLAG_NODESCENDANTS: i32 = 0x00000100;
const G2SURFACEFLAG_GENERATED: i32 = 0x00000200;

// ---------------------------------------------------------------------------
// `mModelBoltLink` bit-packing (`oracle/codemp/ghoul2/G2.h:30-40`) — only the
// `MODEL`/`BOLT` halves [`g2_set_root_surface`]'s bolt-removal loop decodes.
// Duplicated locally, matching `api_bolts.rs`'s identical table.
// ---------------------------------------------------------------------------

const MODEL_WIDTH: i32 = 10;
const BOLT_WIDTH: i32 = 10;
const MODEL_AND: i32 = (1 << MODEL_WIDTH) - 1;
const BOLT_AND: i32 = (1 << BOLT_WIDTH) - 1;
const BOLT_SHIFT: i32 = 0;
const MODEL_SHIFT: i32 = BOLT_SHIFT + BOLT_WIDTH;

// ---------------------------------------------------------------------------
// Raw mdxm/mdxa header + `mdxmSurfHierarchy_t` byte-offset table (`G2SV-D5`:
// the header types are never named in this crate — only byte arithmetic off
// the raw `EngineHost::model_mdxm`/`model_mdxa` pointer, exactly as the oracle
// body does off `model_t::mdxm`/`mdxa`). Every offset is derived from the
// field order in `oracle/codemp/renderer/mdx_format.h`; duplicated locally
// per this crate's own convention (`api_models.rs`'s identical table for the
// same header types; `api_bones.rs`'s smaller `mdxaHeader_t` table).
// ---------------------------------------------------------------------------

/// `mdxmHeader_t::animIndex` (`mdx_format.h:161`) — `ident`(4) + `version`(4)
/// + `name[64]` + `animName[64]` precede it.
const MDXM_OFS_ANIM_INDEX: usize = 136;
/// `mdxmHeader_t::numLODs` (`mdx_format.h:165`).
const MDXM_OFS_NUM_LODS: usize = 144;
/// `mdxmHeader_t::numSurfaces` (`mdx_format.h:168`).
const MDXM_OFS_NUM_SURFACES: usize = 152;
/// `mdxmHeader_t::ofsSurfHierarchy` (`mdx_format.h:169`).
const MDXM_OFS_OFS_SURF_HIERARCHY: usize = 156;
/// `sizeof(mdxmHeader_t)` — where the `mdxmHierarchyOffsets_t` offset table
/// starts (`(mdxmHierarchyOffsets_t*)((byte*)mod->mdxm + sizeof(mdxmHeader_t))`,
/// `G2_surfaces.cpp:118`); `ident`..`ofsEnd` are eleven 4-byte fields.
const MDXM_HEADER_SIZE: usize = 164;

/// `mdxaHeader_t::numBones` (`mdx_format.h:365`) — `ident`(4) + `version`(4) +
/// `name[64]` + `fScale`(4) + `numFrames`(4) + `ofsFrames`(4) precede it.
const MDXA_OFS_NUM_BONES: usize = 84;

/// `mdxmSurfHierarchy_t::flags` (`mdx_format.h:189`) — `name[64]` precedes it.
const SURF_HIER_OFS_FLAGS: usize = 64;
/// `mdxmSurfHierarchy_t::parentIndex` (`mdx_format.h:192`) — `name[64]`(0) +
/// `flags`(64) + `shader[64]`(68) + `shaderIndex`(132) precede it.
const SURF_HIER_OFS_PARENT_INDEX: usize = 136;
/// `mdxmSurfHierarchy_t::numChildren` (`mdx_format.h:193`).
const SURF_HIER_OFS_NUM_CHILDREN: usize = 140;
/// `mdxmSurfHierarchy_t::childIndexes` base offset (`mdx_format.h:194`); the
/// next surface entry starts `SURF_HIER_OFS_CHILD_INDEXES + 4*numChildren`
/// bytes later (`childIndexes[numChildren]`, size comment at `:195`).
const SURF_HIER_OFS_CHILD_INDEXES: usize = 144;

/// Read an `i32` at `offset` bytes into the block `base` points at (the block
/// is `EngineHost::model_mdxm`/`model_mdxa`'s raw pointer — `G2SV-D5`, the
/// header types are never named). `read_unaligned` because nothing here
/// proves 4-byte alignment on every host; this is the same-process native
/// byte order the engine already parsed the block into (no cross-endian
/// concern at this layer). Mirrors `api_models.rs`'s identical helper.
///
/// # Safety
/// `base` must be non-null and `offset..offset+4` must lie inside the block
/// the host returned.
unsafe fn read_i32_at(base: *const c_void, offset: usize) -> i32 {
    unsafe {
        (base as *const u8)
            .add(offset)
            .cast::<i32>()
            .read_unaligned()
    }
}

/// Case-insensitive compare of a raw `mdxmSurfHierarchy_t::name` (`char[64]`,
/// offset 0) against `name` — Raven's `stricmp`. Mirrors `api_bones.rs`'s
/// `mdxa_skel_name_matches`.
///
/// # Safety
/// `surf` must point at a valid `mdxmSurfHierarchy_t` entry.
unsafe fn surf_hier_name_matches(surf: *const c_void, name: &str) -> bool {
    let bytes = unsafe { core::slice::from_raw_parts(surf as *const u8, MAX_QPATH) };
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(MAX_QPATH);
    core::str::from_utf8(&bytes[..len])
        .map(|s| s.eq_ignore_ascii_case(name))
        .unwrap_or(false)
}

/// Read a raw `mdxmSurfHierarchy_t::name` (`char[64]`, offset 0) as an owned,
/// lossily-decoded `String` — needed where the name itself (not just a
/// match test) must be forwarded on, e.g. [`g2_is_surface_rendered`]'s
/// ancestor-name re-lookup.
///
/// # Safety
/// `surf` must point at a valid `mdxmSurfHierarchy_t` entry.
unsafe fn surf_hier_name(surf: *const c_void) -> String {
    let bytes = unsafe { core::slice::from_raw_parts(surf as *const u8, MAX_QPATH) };
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(MAX_QPATH);
    String::from_utf8_lossy(&bytes[..len]).into_owned()
}

/// Resolve `surfIndexes->offsets[this_surface_index]` into an
/// `mdxmSurfHierarchy_t*` — `(mdxmHierarchyOffsets_t*)((byte*)mdxm +
/// sizeof(mdxmHeader_t))`, then `(byte*)surfIndexes +
/// surfIndexes->offsets[this_surface_index]` (`G2_surfaces.cpp:118-119`, and
/// every other direct-index hierarchy lookup below repeats this exact
/// two-step computation).
///
/// # Safety
/// `mdxm` must be a valid, non-null `EngineHost::model_mdxm` block and
/// `this_surface_index` must be `< numSurfaces`.
unsafe fn surf_hierarchy_entry(mdxm: *const c_void, this_surface_index: i32) -> *const c_void {
    unsafe {
        let surf_indexes = (mdxm as *const u8).add(MDXM_HEADER_SIZE) as *const c_void;
        let offset = read_i32_at(surf_indexes, 4 * this_surface_index as usize);
        (surf_indexes as *const u8).add(offset as usize) as *const c_void
    }
}

/// Raven `surfaceInfo_t *G2_FindOverrideSurface(int surfaceNum, surfaceInfo_v
/// &surfaceList)` — linear search of the override list for `surfaceNum`;
/// `None` when not found. Pure read, host-free.
///
/// Source: `oracle/codemp/ghoul2/G2_surfaces.cpp:48-62`
pub fn g2_find_override_surface(
    surface_num: i32,
    surface_list: &[surfaceInfo_t],
) -> Option<&surfaceInfo_t> {
    surface_list.iter().find(|s| s.surface == surface_num)
}

/// Raven `int G2_IsSurfaceLegal(void *mod, const char *surfaceName, int
/// *flags)` — walks `mod->mdxm`'s surface hierarchy table for a
/// case-insensitive name match, returning the surface index; `*flags` is
/// written **only** on the match (`:76`), left untouched on the "walked off
/// the end, no match" `-1` return (`:82`) — a write-on-success-only out-param
/// (the `## Seam definition`'s "Out-param contract" discriminator,
/// `G2SV-D1` generalized), so it returns `Option<(i32, i32)>` = `Some((surface_index,
/// flags))` rather than a write-through `&mut` out-param.
///
/// Source: `oracle/codemp/ghoul2/G2_surfaces.cpp:65-83`
pub fn g2_is_surface_legal(
    host: &mut impl EngineHost,
    model: qhandle_t,
    surface_name: &str,
) -> Option<(i32, i32)> {
    let mdxm = host.model_mdxm(model);
    // SAFETY: every call site below has already established `model`'s
    // `mdxm` is non-null (matching the oracle's own unchecked
    // `mod_m->mdxm` dereference — this function itself has no null-check).
    unsafe {
        let num_surfaces = read_i32_at(mdxm, MDXM_OFS_NUM_SURFACES);
        let ofs_surf_hierarchy = read_i32_at(mdxm, MDXM_OFS_OFS_SURF_HIERARCHY);
        let mut surf = (mdxm as *const u8).add(ofs_surf_hierarchy as usize) as *const c_void;
        for i in 0..num_surfaces {
            if surf_hier_name_matches(surf, surface_name) {
                let flags = read_i32_at(surf, SURF_HIER_OFS_FLAGS);
                return Some((i, flags));
            }
            let num_children = read_i32_at(surf, SURF_HIER_OFS_NUM_CHILDREN);
            let stride = SURF_HIER_OFS_CHILD_INDEXES + 4 * num_children as usize;
            surf = (surf as *const u8).add(stride) as *const c_void;
        }
    }
    None
}

/// Raven `mdxmSurface_t *G2_FindSurface(CGhoul2Info *ghlInfo, surfaceInfo_v
/// &slist, const char *surfaceName, int *surfIndex)` — the **name-based**
/// overload (private helper of this file; distinct from the index-based
/// `G2_FindSurface(void*, int, int)` in `misc.rs`, see the module doc comment).
/// Searches `slist` back-to-front for an already-overridden surface matching
/// `surfaceName` (cross-referencing each entry's `mdxm` hierarchy name via the
/// index-based overload), falling back to none. `surfIndex` is write-through
/// on both paths (`-1` on miss, `:140-143`; the matched index on hit,
/// `:131-134`) — collapsed to a single `Option<i32>` return per the module
/// doc-comment's shape-choice note (the `mdxmSurface_t*` return itself is
/// never dereferenced by any in-crate caller).
///
/// Source: `oracle/codemp/ghoul2/G2_surfaces.cpp:98-145`
fn g2_find_surface_by_name(
    host: &mut impl EngineHost,
    ghl_info: &CGhoul2Info,
    slist: &[surfaceInfo_t],
    surface_name: &str,
) -> Option<i32> {
    let mdxm = host.model_mdxm(ghl_info.model);
    if mdxm.is_null() {
        // Raven: `assert(0); if (surfIndex) *surfIndex=-1; return 0;`
        // (`NDEBUG` no-ops the assert; the write-through/return are live).
        return None;
    }
    for i in (0..slist.len()).rev() {
        let entry = &slist[i];
        if entry.surface != 10000 && entry.surface != -1 {
            let this_surface_index =
                crate::misc::g2_find_surface(host, ghl_info.model, entry.surface, 0);
            let mdxm = host.model_mdxm(ghl_info.model);
            // SAFETY: `mdxm` non-null (checked above); `this_surface_index`
            // is the resolved hierarchy index for `entry.surface`.
            let matches = unsafe {
                surf_hier_name_matches(surf_hierarchy_entry(mdxm, this_surface_index), surface_name)
            };
            if matches {
                return Some(i as i32);
            }
        }
    }
    None
}

/// Raven `qboolean G2_SetSurfaceOnOff(CGhoul2Info *ghlInfo, surfaceInfo_v
/// &slist, const char *surfaceName, const int offFlags)` — `qfalse` when
/// `ghlInfo`'s model has no `mdxm`; else either updates an already-overridden
/// surface's `offFlags` (masking to just the `OFF`/`NODESCENDANTS` bits,
/// `:172-173`) via [`g2_find_surface_by_name`], or — if not yet overridden —
/// validates the name via [`g2_is_surface_legal`] and `push_back`s a fresh
/// override entry only when the incoming flags actually change anything
/// (`:188-194`); `qfalse` if the name isn't a legal surface at all. `slist`
/// collapses into `ghl_info` (module doc-comment shape-choice note): every
/// call site passes `ghlInfo->mSlist`, and this function mutates it.
///
/// Source: `oracle/codemp/ghoul2/G2_surfaces.cpp:148-199`
pub fn g2_set_surface_on_off(
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    surface_name: &str,
    off_flags: i32,
) -> bool {
    if host.model_mdxm(ghl_info.model).is_null() {
        // Raven: `assert(0); return qfalse;`
        return false;
    }

    let surf_index = g2_find_surface_by_name(host, ghl_info, &ghl_info.slist, surface_name);
    if let Some(idx) = surf_index {
        let entry = &mut ghl_info.slist[idx as usize];
        // "seems to me that we shouldn't overwrite the other flags. the only
        // bit we really care about in the incoming flags is the off bit"
        entry.offFlags &= !(G2SURFACEFLAG_OFF | G2SURFACEFLAG_NODESCENDANTS);
        entry.offFlags |= off_flags & (G2SURFACEFLAG_OFF | G2SURFACEFLAG_NODESCENDANTS);
        return true;
    }

    // not in the list already - verify this surface exists in the model mesh.
    if let Some((surface_num, flags)) = g2_is_surface_legal(host, ghl_info.model, surface_name) {
        let mut new_flags = flags;
        new_flags &= !(G2SURFACEFLAG_OFF | G2SURFACEFLAG_NODESCENDANTS);
        new_flags |= off_flags & (G2SURFACEFLAG_OFF | G2SURFACEFLAG_NODESCENDANTS);

        if new_flags != flags {
            // insert here then because it changed, no need to add an override otherwise
            ghl_info.slist.push(surfaceInfo_t {
                offFlags: new_flags,
                surface: surface_num,
                genBarycentricJ: 0.0,
                genBarycentricI: 0.0,
                genPolySurfaceIndex: 0,
                genLod: 0,
            });
        }
        return true;
    }
    false
}

/// `R_GetSkinByHandle` has no `EngineHost` equivalent (module doc-comment gap
/// #2 — the same missing-service class `api_models.rs`'s
/// `register_server_model`/`register_model` already report for
/// `RE_RegisterServerModel`/`RE_RegisterModel`). Diverges via the frozen,
/// real `host.error` service rather than inventing a skin surface table.
fn get_skin_by_handle(host: &mut impl EngineHost, render_skin: qhandle_t) -> ! {
    host.error(
        errorParm_t::ERR_DROP,
        &format!(
            "G2_Surfaces: EngineHost has no R_GetSkinByHandle({render_skin}) equivalent yet \
             (docs/subsystems/ghoul2-server.md gap note #2, G2_surfaces.cpp:204)"
        ),
    )
}

/// Raven `void G2_SetSurfaceOnOffFromSkin(CGhoul2Info *ghlInfo, qhandle_t
/// renderSkin)` — clears `ghlInfo->mSlist` and `mMeshFrameNum`, then for every
/// surface in the resolved skin (`R_GetSkinByHandle`, module doc-comment gap
/// #2) forwards to [`g2_set_surface_on_off`]: surfaces shaded `"*off"` are
/// turned off, all others turned on (skipping ones already `_off` by name,
/// `:219-220`). Not header-declared (`G2_local.h`) but forward-declared and
/// called ad hoc from `G2API_SetSkin` (`G2_API.cpp:680,688`,
/// `api_models.rs`'s `g2api_set_skin`) — kept `pub` for that cross-file edge.
///
/// Source: `oracle/codemp/ghoul2/G2_surfaces.cpp:201-226`
pub fn g2_set_surface_on_off_from_skin(
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    render_skin: qhandle_t,
) {
    ghl_info.slist.clear(); // remove any overrides we had before.
    ghl_info.mesh_frame_num = 0;

    // The skin's per-surface shader-name table has no `EngineHost` accessor
    // yet (module doc-comment gap #2) — diverges loudly rather than guessing.
    get_skin_by_handle(host, render_skin);
}

/// Raven `int G2_IsSurfaceOff (CGhoul2Info *ghlInfo, surfaceInfo_v &slist,
/// const char *surfaceName)` — `0` when `ghlInfo`'s model has no `mdxm`; else
/// an already-overridden surface's `offFlags` via [`g2_find_surface_by_name`],
/// falling back to the original model surface's default flags from `mdxm`'s
/// hierarchy table (linear name-match walk, `:250-260`) — an unreachable-in-
/// practice `assert(0); return 0;` closes the "name not found anywhere" case
/// (debug-only, dropped per house convention). `slist` kept as a faithful
/// second parameter (read-only, module doc-comment shape-choice note).
///
/// Source: `oracle/codemp/ghoul2/G2_surfaces.cpp:229-264`
pub fn g2_is_surface_off(
    host: &mut impl EngineHost,
    ghl_info: &CGhoul2Info,
    slist: &[surfaceInfo_t],
    surface_name: &str,
) -> i32 {
    let mdxm = host.model_mdxm(ghl_info.model);
    if mdxm.is_null() {
        return 0;
    }

    if let Some(idx) = g2_find_surface_by_name(host, ghl_info, slist, surface_name) {
        return slist[idx as usize].offFlags;
    }

    // ok, we didn't find it in the surface list. Lets look at the original
    // surface then.
    let mdxm = host.model_mdxm(ghl_info.model);
    // SAFETY: `mdxm` non-null (checked above).
    unsafe {
        let num_surfaces = read_i32_at(mdxm, MDXM_OFS_NUM_SURFACES);
        let ofs_surf_hierarchy = read_i32_at(mdxm, MDXM_OFS_OFS_SURF_HIERARCHY);
        let mut surf = (mdxm as *const u8).add(ofs_surf_hierarchy as usize) as *const c_void;
        for _ in 0..num_surfaces {
            if surf_hier_name_matches(surf, surface_name) {
                return read_i32_at(surf, SURF_HIER_OFS_FLAGS);
            }
            let num_children = read_i32_at(surf, SURF_HIER_OFS_NUM_CHILDREN);
            let stride = SURF_HIER_OFS_CHILD_INDEXES + 4 * num_children as usize;
            surf = (surf as *const u8).add(stride) as *const c_void;
        }
    }

    // Raven: `assert(0); return 0;` — unreachable in practice (NDEBUG no-op).
    0
}

/// Raven `void G2_FindRecursiveSurface(model_t *currentModel, int surfaceNum,
/// surfaceInfo_v &rootList, int *activeSurfaces)` — file-private helper of
/// [`g2_set_root_surface`]: resolves `surfaceNum`'s hierarchy entry (index-based
/// `G2_FindSurface`, `misc.rs`), applies any override from `rootList`
/// ([`g2_find_override_surface`]), marks `activeSurfaces[surfaceNum] = 1` unless
/// the resolved flags are `OFF`, stops recursing when `NODESCENDANTS` is also
/// set, and otherwise recurses into every child surface index.
///
/// Source: `oracle/codemp/ghoul2/G2_surfaces.cpp:266-304`
fn g2_find_recursive_surface(
    host: &mut impl EngineHost,
    current_model: qhandle_t,
    surface_num: i32,
    root_list: &[surfaceInfo_t],
    active_surfaces: &mut [i32],
) {
    let this_surface_index = crate::misc::g2_find_surface(host, current_model, surface_num, 0);
    let mdxm = host.model_mdxm(current_model);
    // SAFETY: `current_model` has already been validated non-null by
    // `g2_set_root_surface`'s own caller-side check.
    let surf_info = unsafe { surf_hierarchy_entry(mdxm, this_surface_index) };

    // see if we have an override surface in the surface list
    let surf_override = g2_find_override_surface(surface_num, root_list);

    // really, we should use the default flags for this surface unless it's
    // been overriden
    let off_flags = match surf_override {
        Some(o) => o.offFlags,
        None => unsafe { read_i32_at(surf_info, SURF_HIER_OFS_FLAGS) },
    };

    // if this surface is not off, indicate as such in the active surface list
    if off_flags & G2SURFACEFLAG_OFF == 0 {
        active_surfaces[surface_num as usize] = 1;
    } else if off_flags & G2SURFACEFLAG_NODESCENDANTS != 0 {
        // if we are turning off all descendants, then stop this recursion now
        return;
    }

    // now recursively call for the children
    // SAFETY: `surf_info` is the same validated block as above.
    let num_children = unsafe { read_i32_at(surf_info, SURF_HIER_OFS_NUM_CHILDREN) };
    for i in 0..num_children {
        // SAFETY: `i < numChildren`, inside `surf_info`'s `childIndexes` array.
        let child_surface_num =
            unsafe { read_i32_at(surf_info, SURF_HIER_OFS_CHILD_INDEXES + 4 * i as usize) };
        g2_find_recursive_surface(
            host,
            current_model,
            child_surface_num,
            root_list,
            active_surfaces,
        );
    }
}

/// Raven `void G2_RemoveRedundantGeneratedSurfaces(surfaceInfo_v &slist, int
/// *activeSurfaces)` — file-private helper of [`g2_set_root_surface`]: walks
/// `slist`, and for every still-live entry (`surface != -1`) whose referenced
/// surface (a generated poly's `genPolySurfaceIndex` low 16 bits, or the plain
/// `surface` index) is not marked active, removes it via [`g2_remove_surface`].
/// Host-free — only touches the already-computed `activeSurfaces` scratch
/// array and the override list.
///
/// Source: `oracle/codemp/ghoul2/G2_surfaces.cpp:306-335`
fn g2_remove_redundant_generated_surfaces(slist: &mut Vec<surfaceInfo_t>, active_surfaces: &[i32]) {
    // Raven: `for (i=0;i<slist.size();i++)` — re-reads `slist.size()` live
    // every iteration, so a mid-loop `resize()` (inside `g2_remove_surface`)
    // shrinks the bound the same way here (`slist.len()` re-read too).
    let mut i = 0;
    while i < slist.len() {
        if slist[i].surface != -1 {
            let is_generated = slist[i].offFlags & G2SURFACEFLAG_GENERATED != 0;
            let active_index = if is_generated {
                (slist[i].genPolySurfaceIndex & 0xffff) as usize
            } else {
                slist[i].surface as usize
            };
            if active_surfaces[active_index] == 0 {
                g2_remove_surface(slist, i as i32);
            }
        }
        i += 1;
    }
}

/// Raven `qboolean G2_SetRootSurface(CGhoul2Info_v &ghoul2, const int
/// modelIndex, const char *surfaceName)` — `qfalse` when `ghoul2[modelIndex]`'s
/// model has no `mdxm`; `qtrue` immediately if `surfaceName` is already the
/// current root (`:359-362`). Otherwise sets `mSurfaceRoot`, builds a
/// `numSurfaces`/`numBones`-sized active-surface/active-bone scratch pair,
/// walks the surface tree from the new root down via
/// [`g2_find_recursive_surface`], constructs the used-bone list (`extern
/// G2_ConstructUsedBoneList`, module doc-comment gap #3 — **not** owned by this
/// file), prunes now-inactive generated/override surfaces
/// ([`g2_remove_redundant_generated_surfaces`]), bones (`G2_RemoveRedundantBoneOverrides`,
/// `bones.rs`), and bolts (`G2_RemoveRedundantBolts`, `bolts.rs`), then removes
/// every model instance whose bolt-link now points at a removed bolt
/// (`G2API_RemoveGhoul2Model`, `crate::api_models::g2api_remove_ghoul2_model`).
/// The huge `entstate->ghoul2`-based duplicate body (`:422-505`) is
/// `/* ... */`-commented-out dead code (module doc-comment gap #1) — not
/// ported. Takes `g2: &mut Ghoul2System` (the `ghoul2.size()` iteration +
/// model-removal loop needs the arena/system, not just the one instance) per
/// ruling 4/11.
///
/// `active_bones` (module doc-comment gap #3) never gets populated by the
/// renderer's `G2_ConstructUsedBoneList` walk — genuinely absent from this
/// crate's roster — so it stays all-zero (its oracle `Z_Malloc` +
/// `memset(0)` initial state) rather than inventing that walk here.
///
/// Source: `oracle/codemp/ghoul2/G2_surfaces.cpp:337-421` (live body only)
pub fn g2_set_root_surface(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    model_index: i32,
    surface_name: &str,
) -> bool {
    let model = ghoul2.get(g2, model_index).model;
    if host.model_mdxm(model).is_null() {
        return false;
    }

    let Some((surf, _flags)) = g2_is_surface_legal(host, model, surface_name) else {
        return false;
    };

    // first see if this ghoul2 model already has this as a root surface
    if ghoul2.get(g2, model_index).surface_root == surf {
        return true;
    }

    // set the root surface
    ghoul2.get_mut(g2, model_index).surface_root = surf;

    // ok, now the tricky bits. firstly, generate a list of active / on
    // surfaces below the root point.
    let mdxm = host.model_mdxm(model);
    // SAFETY: `mdxm` non-null (checked above).
    let num_surfaces = unsafe { read_i32_at(mdxm, MDXM_OFS_NUM_SURFACES) };
    let mut active_surfaces = vec![0i32; num_surfaces.max(0) as usize];

    // SAFETY: `mdxm` non-null (checked above).
    let anim_index = unsafe { read_i32_at(mdxm, MDXM_OFS_ANIM_INDEX) };
    let mdxa = host.model_mdxa(anim_index);
    // SAFETY: mirrors the oracle's own unchecked `mod_a->mdxa->numBones`
    // dereference (`assert(...animModel)` above it is a dropped NDEBUG no-op).
    let num_bones = unsafe { read_i32_at(mdxa, MDXA_OFS_NUM_BONES) };
    // Never mutated: `G2_ConstructUsedBoneList` (module doc-comment gap #3)
    // would populate this, but is not this file's to invent — it stays at
    // its `Z_Malloc` + `memset(0)` initial state.
    let active_bones = vec![0i32; num_bones.max(0) as usize];

    {
        let root_list = &ghoul2.get(g2, model_index).slist;
        g2_find_recursive_surface(host, model, surf, root_list, &mut active_surfaces);
    }

    // now generate the used bone list — `G2_ConstructUsedBoneList` (module
    // doc-comment gap #3) is not this file's to invent; `active_bones` stays
    // all-zero.

    // now remove all procedural or override surfaces that refer to surfaces
    // that arent on this list.
    g2_remove_redundant_generated_surfaces(
        &mut ghoul2.get_mut(g2, model_index).slist,
        &active_surfaces,
    );

    // now remove all bones that are pointing at bones that aren't active.
    crate::bones::g2_remove_redundant_bone_overrides(
        &mut ghoul2.get_mut(g2, model_index).blist,
        &active_bones,
    );

    // then remove all bolts that point at surfaces or bones that *arent* active.
    {
        let info = ghoul2.get_mut(g2, model_index);
        crate::bolts::g2_remove_redundant_bolts(
            &mut info.bltlist,
            &info.slist,
            &active_surfaces,
            &active_bones,
        );
    }

    // then remove all models on this ghoul2 instance that use those bolts
    // that are being removed.
    let mut i = 0;
    while i < ghoul2.size(g2) {
        let bolt_link = ghoul2.get(g2, i).model_bolt_link;
        if bolt_link != -1 {
            let bolt_mod = (bolt_link >> MODEL_SHIFT) & MODEL_AND;
            let bolt_num = (bolt_link >> BOLT_SHIFT) & BOLT_AND;
            // if either the bolt list is too small, or the bolt we are
            // pointing at references nothing, remove this model.
            let target_size = ghoul2.get(g2, bolt_mod).bltlist.len() as i32;
            let remove = target_size <= bolt_num || {
                let target = ghoul2.get(g2, bolt_mod);
                let bolt = &target.bltlist[bolt_num as usize];
                bolt.boneNumber == -1 && bolt.surfaceNumber == -1
            };
            if remove {
                crate::api_models::g2api_remove_ghoul2_model(g2, ghoul2, i);
            }
        }
        i += 1;
    }
    // No support for this, for now.

    true
}

/// Raven `int G2_AddSurface(CGhoul2Info *ghoul2, int surfaceNumber, int
/// polyNumber, float BarycentricI, float BarycentricJ, int lod)` — clamps
/// `lod` via `G2_DecideTraceLod` (`misc.rs`, reads `mdxm->numLODs` -> `host`),
/// reuses the first free (`surface == -1`) generated-surface slot if one
/// exists, else `push_back`s a fresh one; returns the slot's index.
///
/// Source: `oracle/codemp/ghoul2/G2_surfaces.cpp:511-547`
#[allow(clippy::too_many_arguments)]
pub fn g2_add_surface(
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    surface_number: i32,
    poly_number: i32,
    barycentric_i: f32,
    barycentric_j: f32,
    lod: i32,
) -> i32 {
    // decide if LOD is legal.
    let lod = decide_trace_lod(host, ghl_info, lod);

    // first up, see if we have a free one already set up - look only from
    // the end of the constant surfaces onwards.
    for i in 0..ghl_info.slist.len() {
        if ghl_info.slist[i].surface == -1 {
            let entry = &mut ghl_info.slist[i];
            entry.offFlags = G2SURFACEFLAG_GENERATED;
            entry.surface = 10000; // no model will ever have 10000 surfaces
            entry.genBarycentricI = barycentric_i;
            entry.genBarycentricJ = barycentric_j;
            entry.genPolySurfaceIndex = ((poly_number & 0xffff) << 16) | (surface_number & 0xffff);
            entry.genLod = lod;
            return i as i32;
        }
    }

    // ok, didn't find one. Better create one.
    ghl_info.slist.push(surfaceInfo_t {
        offFlags: G2SURFACEFLAG_GENERATED,
        surface: 10000,
        genBarycentricJ: barycentric_j,
        genBarycentricI: barycentric_i,
        genPolySurfaceIndex: ((poly_number & 0xffff) << 16) | (surface_number & 0xffff),
        genLod: lod,
    });

    (ghl_info.slist.len() - 1) as i32
}

/// Inlines `G2_DecideTraceLod`'s clamp (`misc.rs`'s `g2_decide_trace_lod` is
/// private to that file — module-doc note: this file reads model memory on
/// its own `host`-threaded path rather than calling into that private fn):
/// clamp `use_lod` up to `ghl_info.lod_bias`, then down to
/// `mdxm->numLODs - 1`.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:376-398`
fn decide_trace_lod(host: &mut impl EngineHost, ghl_info: &CGhoul2Info, use_lod: i32) -> i32 {
    let mut return_lod = use_lod;

    // if we are overriding the LOD at top level, then we can afford to only
    // check this level of model.
    if ghl_info.lod_bias > return_lod {
        return_lod = ghl_info.lod_bias;
    }

    let mdxm = host.model_mdxm(ghl_info.model);
    // SAFETY: mirrors the oracle's own unchecked `ghoul2.currentModel->mdxm`
    // dereference (asserted non-null there; `NDEBUG` no-ops the assert).
    let num_lods = unsafe { read_i32_at(mdxm, MDXM_OFS_NUM_LODS) };

    // now ensure that we haven't selected a lod that doesn't exist for this model.
    if return_lod >= num_lods {
        return_lod = num_lods - 1;
    }

    return_lod
}

/// Raven `qboolean G2_RemoveSurface(surfaceInfo_v &slist, const int index)` —
/// `qfalse` (+ debug-only `assert(0)`) on `index == -1`; else marks
/// `slist[index].surface = -1` and trims any trailing run of now-dead (`-1`)
/// entries off the back of the vector. Host-free, standalone `slist` (never
/// tied to one specific `CGhoul2Info` in the oracle signature — module
/// doc-comment shape-choice note).
///
/// Source: `oracle/codemp/ghoul2/G2_surfaces.cpp:549-585`
pub fn g2_remove_surface(slist: &mut Vec<surfaceInfo_t>, index: i32) -> bool {
    // did we find it?
    if index == -1 {
        // Raven: `assert(0); return qfalse;` (NDEBUG no-ops the assert).
        return false;
    }

    // set us to be the 'not active' state.
    slist[index as usize].surface = -1;

    // now look through the list from the back and see if there is a block of
    // -1's we can resize off the end of the list.
    let mut new_size = slist.len();
    for i in (0..slist.len()).rev() {
        if slist[i].surface == -1 {
            new_size = i;
        } else {
            // once we hit one that isn't a -1, we are done.
            break;
        }
    }
    if new_size != slist.len() {
        slist.truncate(new_size);
    }

    true
}

/// Raven `int G2_GetParentSurface(CGhoul2Info *ghlInfo, const int index)` —
/// resolves `index`'s hierarchy entry (index-based `G2_FindSurface`, `misc.rs`)
/// and returns its `parentIndex`.
///
/// Source: `oracle/codemp/ghoul2/G2_surfaces.cpp:588-601`
pub fn g2_get_parent_surface(
    host: &mut impl EngineHost,
    ghl_info: &CGhoul2Info,
    index: i32,
) -> i32 {
    let this_surface_index = crate::misc::g2_find_surface(host, ghl_info.model, index, 0);
    let mdxm = host.model_mdxm(ghl_info.model);
    // SAFETY: contract per `EngineHost::model_mdxm`; `this_surface_index` is
    // the resolved hierarchy index.
    unsafe {
        read_i32_at(
            surf_hierarchy_entry(mdxm, this_surface_index),
            SURF_HIER_OFS_PARENT_INDEX,
        )
    }
}

/// Raven `int G2_GetSurfaceIndex(CGhoul2Info *ghlInfo, const char
/// *surfaceName)` — [`g2_is_surface_legal`]'s surface index, discarding the
/// flags half of its result (`-1` on `None`).
///
/// Source: `oracle/codemp/ghoul2/G2_surfaces.cpp:603-609`
pub fn g2_get_surface_index(
    host: &mut impl EngineHost,
    ghl_info: &CGhoul2Info,
    surface_name: &str,
) -> i32 {
    g2_is_surface_legal(host, ghl_info.model, surface_name)
        .map(|(index, _flags)| index)
        .unwrap_or(-1)
}

/// Raven `int G2_IsSurfaceRendered(CGhoul2Info *ghlInfo, const char
/// *surfaceName, surfaceInfo_v &slist)` — `-1` when `ghlInfo`'s model has no
/// `mdxm` or `surfaceName` isn't legal ([`g2_is_surface_legal`]); else walks
/// **up** the hierarchy from the surface's parent looking for any ancestor
/// whose (possibly slist-overridden) flags set `NODESCENDANTS`, OR-ing
/// `G2SURFACEFLAG_OFF` into the result if one is found; if no ancestor
/// overrides, falls back to the surface's own slist override
/// ([`g2_find_surface_by_name`]) or `0`. `slist` kept as a faithful third
/// parameter (read-only, module doc-comment shape-choice note).
///
/// The ancestor walk (`:635-660`) indexes the hierarchy table **directly** by
/// surface number (`surfIndexes->offsets[surfNum]`) rather than through the
/// mesh-lookup detour [`g2_get_parent_surface`]/[`g2_find_surface_by_name`]
/// take — that is how the oracle body itself is written (it never calls the
/// index-based `G2_FindSurface` here), not a shortcut taken by this port.
///
/// Source: `oracle/codemp/ghoul2/G2_surfaces.cpp:611-677`
pub fn g2_is_surface_rendered(
    host: &mut impl EngineHost,
    ghl_info: &CGhoul2Info,
    surface_name: &str,
    slist: &[surfaceInfo_t],
) -> i32 {
    let mdxm = host.model_mdxm(ghl_info.model);
    if mdxm.is_null() {
        return -1;
    }

    // now travel up the skeleton to see if any of it's ancestors have a 'no
    // descendants' turned on. find the original surface in the surface list.
    let Some((surf_num, mut flags)) = g2_is_surface_legal(host, ghl_info.model, surface_name)
    else {
        return -1;
    };

    // SAFETY: `mdxm` non-null (checked above); `surf_num` is a hierarchy
    // index, already resolved by `g2_is_surface_legal`'s own linear walk.
    let surf_info = unsafe { surf_hierarchy_entry(mdxm, surf_num) };
    let mut surf_num = unsafe { read_i32_at(surf_info, SURF_HIER_OFS_PARENT_INDEX) };

    // walk the surface hierarchy up until we hit the root.
    while surf_num != -1 {
        let mdxm = host.model_mdxm(ghl_info.model);
        // SAFETY: `mdxm` non-null (checked above); `surf_num` is a hierarchy index.
        let parent_surf_info = unsafe { surf_hierarchy_entry(mdxm, surf_num) };
        let parent_name = unsafe { surf_hier_name(parent_surf_info) };

        // find the original surface in the surface list. G2 was bug, above
        // comment was accurate, but we don't want the original flags, we
        // want the parent flags.
        let (_, mut parent_flags) =
            g2_is_surface_legal(host, ghl_info.model, &parent_name).unwrap_or((0, 0));

        // now see if we already have overriden this surface in the slist.
        if let Some(idx) = g2_find_surface_by_name(host, ghl_info, slist, &parent_name) {
            parent_flags = slist[idx as usize].offFlags;
        }

        // now we have the parent flags, lets see if any have the 'no
        // descendants' flag set.
        if parent_flags & G2SURFACEFLAG_NODESCENDANTS != 0 {
            flags |= G2SURFACEFLAG_OFF;
            break;
        }

        // set up scan of next parent.
        surf_num = unsafe { read_i32_at(parent_surf_info, SURF_HIER_OFS_PARENT_INDEX) };
    }

    if flags == 0 {
        // it's not being overridden by a parent - now see if we already have
        // overriden this surface in the slist.
        if let Some(idx) = g2_find_surface_by_name(host, ghl_info, slist, surface_name) {
            flags = slist[idx as usize].offFlags;
        }
    }

    flags
}

#[cfg(test)]
mod tests {
    use super::*;
    use mp_host_interface::mock::MockHost;

    /// Build one synthetic `mdxmSurfHierarchy_t` entry (`name[64]`, `flags`(4),
    /// `shader[64]` zeroed, `shaderIndex`(4) zeroed, `parentIndex`(4),
    /// `numChildren`(4), then `numChildren` `i32` child indexes). Mirrors
    /// `api_models.rs`'s `push_surf_entry` test helper, extended with the
    /// name/`parentIndex`/child-index fields this file's tests need.
    fn push_surf_hier_entry(
        buf: &mut Vec<u8>,
        name: &str,
        flags: i32,
        parent_index: i32,
        child_indexes: &[i32],
    ) {
        let mut name_bytes = [0u8; MAX_QPATH];
        name_bytes[..name.len()].copy_from_slice(name.as_bytes());
        buf.extend_from_slice(&name_bytes);
        buf.extend(flags.to_ne_bytes());
        buf.extend([0u8; MAX_QPATH]); // shader
        buf.extend(0i32.to_ne_bytes()); // shaderIndex
        buf.extend(parent_index.to_ne_bytes());
        buf.extend((child_indexes.len() as i32).to_ne_bytes());
        for &c in child_indexes {
            buf.extend(c.to_ne_bytes());
        }
    }

    /// Build a `mdxmHeader_t` prefix (`MDXM_HEADER_SIZE` bytes) with the four
    /// fields this file's tests read.
    fn build_mdxm_header(
        num_lods: i32,
        num_surfaces: i32,
        ofs_surf_hierarchy: i32,
        anim_index: i32,
    ) -> Vec<u8> {
        let mut buf = vec![0u8; MDXM_HEADER_SIZE];
        buf[MDXM_OFS_ANIM_INDEX..MDXM_OFS_ANIM_INDEX + 4]
            .copy_from_slice(&anim_index.to_ne_bytes());
        buf[MDXM_OFS_NUM_LODS..MDXM_OFS_NUM_LODS + 4].copy_from_slice(&num_lods.to_ne_bytes());
        buf[MDXM_OFS_NUM_SURFACES..MDXM_OFS_NUM_SURFACES + 4]
            .copy_from_slice(&num_surfaces.to_ne_bytes());
        buf[MDXM_OFS_OFS_SURF_HIERARCHY..MDXM_OFS_OFS_SURF_HIERARCHY + 4]
            .copy_from_slice(&ofs_surf_hierarchy.to_ne_bytes());
        buf
    }

    #[test]
    fn g2_find_override_surface_finds_and_misses() {
        let list = vec![surfaceInfo_t {
            offFlags: 5,
            surface: 3,
            genBarycentricJ: 0.0,
            genBarycentricI: 0.0,
            genPolySurfaceIndex: 0,
            genLod: 0,
        }];
        assert_eq!(g2_find_override_surface(3, &list).unwrap().offFlags, 5);
        assert!(g2_find_override_surface(4, &list).is_none());
    }

    #[test]
    fn g2_is_surface_legal_finds_named_surface_and_flags() {
        let mut buf = build_mdxm_header(1, 2, MDXM_HEADER_SIZE as i32, 0);
        push_surf_hier_entry(&mut buf, "root", 0, -1, &[]);
        push_surf_hier_entry(&mut buf, "child", G2SURFACEFLAG_NODESCENDANTS, 0, &[]);

        let mut host = MockHost::new();
        host.mdxm_blocks.insert(1, buf);

        assert_eq!(
            g2_is_surface_legal(&mut host, 1, "child"),
            Some((1, G2SURFACEFLAG_NODESCENDANTS))
        );
        assert_eq!(
            g2_is_surface_legal(&mut host, 1, "CHILD"),
            Some((1, G2SURFACEFLAG_NODESCENDANTS))
        );
        assert_eq!(g2_is_surface_legal(&mut host, 1, "nope"), None);
    }

    #[test]
    fn g2_get_surface_index_drops_the_flags_half() {
        let mut buf = build_mdxm_header(1, 1, MDXM_HEADER_SIZE as i32, 0);
        push_surf_hier_entry(&mut buf, "root", 0, -1, &[]);

        let mut host = MockHost::new();
        host.mdxm_blocks.insert(1, buf);

        let mut ghl_info = CGhoul2Info::default();
        ghl_info.model = 1;

        assert_eq!(g2_get_surface_index(&mut host, &ghl_info, "root"), 0);
        assert_eq!(g2_get_surface_index(&mut host, &ghl_info, "nope"), -1);
    }

    #[test]
    fn g2_remove_surface_marks_dead_and_trims_trailing_run() {
        let mut slist = vec![
            surfaceInfo_t {
                offFlags: 0,
                surface: 1,
                genBarycentricJ: 0.0,
                genBarycentricI: 0.0,
                genPolySurfaceIndex: 0,
                genLod: 0,
            },
            surfaceInfo_t {
                offFlags: 0,
                surface: 2,
                genBarycentricJ: 0.0,
                genBarycentricI: 0.0,
                genPolySurfaceIndex: 0,
                genLod: 0,
            },
            surfaceInfo_t {
                offFlags: 0,
                surface: -1,
                genBarycentricJ: 0.0,
                genBarycentricI: 0.0,
                genPolySurfaceIndex: 0,
                genLod: 0,
            },
        ];
        // removing index 1 leaves a trailing run of two `-1`s (index 1 and
        // the already-dead index 2) that gets trimmed off the back.
        assert!(g2_remove_surface(&mut slist, 1));
        assert_eq!(slist.len(), 1);
        assert_eq!(slist[0].surface, 1);
    }

    #[test]
    fn g2_remove_surface_rejects_index_negative_one() {
        let mut slist: Vec<surfaceInfo_t> = Vec::new();
        assert!(!g2_remove_surface(&mut slist, -1));
    }

    #[test]
    fn g2_remove_redundant_generated_surfaces_drops_inactive_entries() {
        let mut slist = vec![
            // plain surface reference, still active -> kept.
            surfaceInfo_t {
                offFlags: 0,
                surface: 0,
                genBarycentricJ: 0.0,
                genBarycentricI: 0.0,
                genPolySurfaceIndex: 0,
                genLod: 0,
            },
            // plain surface reference, inactive -> removed.
            surfaceInfo_t {
                offFlags: 0,
                surface: 1,
                genBarycentricJ: 0.0,
                genBarycentricI: 0.0,
                genPolySurfaceIndex: 0,
                genLod: 0,
            },
            // generated surface pointing at inactive surface 1 -> removed.
            surfaceInfo_t {
                offFlags: G2SURFACEFLAG_GENERATED,
                surface: 10000,
                genBarycentricJ: 0.0,
                genBarycentricI: 0.0,
                genPolySurfaceIndex: 1,
                genLod: 0,
            },
        ];
        let active_surfaces = [1, 0];
        g2_remove_redundant_generated_surfaces(&mut slist, &active_surfaces);
        assert_eq!(slist.len(), 1);
        assert_eq!(slist[0].surface, 0);
    }

    #[test]
    fn g2_add_surface_reuses_free_slot_then_appends() {
        let mut buf = build_mdxm_header(2, 1, MDXM_HEADER_SIZE as i32, 0);
        push_surf_hier_entry(&mut buf, "root", 0, -1, &[]);

        let mut host = MockHost::new();
        host.mdxm_blocks.insert(1, buf);

        let mut ghl_info = CGhoul2Info::default();
        ghl_info.model = 1;
        ghl_info.slist.push(surfaceInfo_t {
            offFlags: 0,
            surface: -1,
            genBarycentricJ: 0.0,
            genBarycentricI: 0.0,
            genPolySurfaceIndex: 0,
            genLod: 0,
        });

        let idx = g2_add_surface(&mut host, &mut ghl_info, 4, 9, 0.25, 0.5, 5);
        assert_eq!(idx, 0);
        assert_eq!(ghl_info.slist.len(), 1);
        assert_eq!(ghl_info.slist[0].surface, 10000);
        assert_eq!(ghl_info.slist[0].offFlags, G2SURFACEFLAG_GENERATED);
        assert_eq!(ghl_info.slist[0].genBarycentricI, 0.25);
        assert_eq!(ghl_info.slist[0].genBarycentricJ, 0.5);
        assert_eq!(ghl_info.slist[0].genPolySurfaceIndex, (9 << 16) | 4);
        // lod 5 clamped down to numLODs(2) - 1 = 1.
        assert_eq!(ghl_info.slist[0].genLod, 1);

        let idx2 = g2_add_surface(&mut host, &mut ghl_info, 1, 2, 0.0, 0.0, 0);
        assert_eq!(idx2, 1);
        assert_eq!(ghl_info.slist.len(), 2);
    }

    #[test]
    fn g2_set_surface_on_off_adds_a_new_override_when_flags_change() {
        let mut buf = build_mdxm_header(1, 1, MDXM_HEADER_SIZE as i32, 0);
        push_surf_hier_entry(&mut buf, "root", 0, -1, &[]);

        let mut host = MockHost::new();
        host.mdxm_blocks.insert(1, buf);

        let mut ghl_info = CGhoul2Info::default();
        ghl_info.model = 1;

        assert!(g2_set_surface_on_off(
            &mut host,
            &mut ghl_info,
            "root",
            G2SURFACEFLAG_OFF
        ));
        assert_eq!(ghl_info.slist.len(), 1);
        assert_eq!(ghl_info.slist[0].surface, 0);
        assert_eq!(ghl_info.slist[0].offFlags, G2SURFACEFLAG_OFF);
    }

    #[test]
    fn g2_set_surface_on_off_rejects_an_illegal_name() {
        let mut buf = build_mdxm_header(1, 1, MDXM_HEADER_SIZE as i32, 0);
        push_surf_hier_entry(&mut buf, "root", 0, -1, &[]);

        let mut host = MockHost::new();
        host.mdxm_blocks.insert(1, buf);

        let mut ghl_info = CGhoul2Info::default();
        ghl_info.model = 1;

        assert!(!g2_set_surface_on_off(
            &mut host,
            &mut ghl_info,
            "nope",
            G2SURFACEFLAG_OFF
        ));
        assert!(ghl_info.slist.is_empty());
    }

    #[test]
    fn g2_set_surface_on_off_from_skin_diverges_via_host_error() {
        let mut host = MockHost::new();
        let mut ghl_info = CGhoul2Info::default();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            g2_set_surface_on_off_from_skin(&mut host, &mut ghl_info, 1)
        }));
        assert!(result.is_err());
        assert_eq!(host.errors.len(), 1);
        assert_eq!(host.errors[0].0, errorParm_t::ERR_DROP);
    }

    #[test]
    fn g2_is_surface_off_falls_back_to_model_default_flags() {
        let mut buf = build_mdxm_header(1, 1, MDXM_HEADER_SIZE as i32, 0);
        push_surf_hier_entry(&mut buf, "root", G2SURFACEFLAG_OFF, -1, &[]);

        let mut host = MockHost::new();
        host.mdxm_blocks.insert(1, buf);

        let mut ghl_info = CGhoul2Info::default();
        ghl_info.model = 1;

        assert_eq!(
            g2_is_surface_off(&mut host, &ghl_info, &[], "root"),
            G2SURFACEFLAG_OFF
        );
    }

    #[test]
    fn g2_is_surface_rendered_returns_own_flags_when_root_has_no_ancestor() {
        let mut buf = build_mdxm_header(1, 2, MDXM_HEADER_SIZE as i32 + 8, 0);
        // offsets table (relative to MDXM_HEADER_SIZE): entry0 at +8, entry1
        // right after entry0's 144-byte (0-child) body, at +152.
        buf.extend(8i32.to_ne_bytes());
        buf.extend(152i32.to_ne_bytes());
        push_surf_hier_entry(&mut buf, "root", G2SURFACEFLAG_NODESCENDANTS, -1, &[]);
        push_surf_hier_entry(&mut buf, "child", 0, 0, &[]);

        let mut host = MockHost::new();
        host.mdxm_blocks.insert(1, buf);

        let mut ghl_info = CGhoul2Info::default();
        ghl_info.model = 1;

        // "root" has no ancestors (parentIndex -1): the ancestor loop never
        // runs, so this returns exactly its own legal flags.
        assert_eq!(
            g2_is_surface_rendered(&mut host, &ghl_info, "root", &[]),
            G2SURFACEFLAG_NODESCENDANTS
        );
    }

    #[test]
    fn g2_is_surface_rendered_ors_off_when_an_ancestor_has_nodescendants() {
        let mut buf = build_mdxm_header(1, 2, MDXM_HEADER_SIZE as i32 + 8, 0);
        buf.extend(8i32.to_ne_bytes());
        buf.extend(152i32.to_ne_bytes());
        push_surf_hier_entry(&mut buf, "root", G2SURFACEFLAG_NODESCENDANTS, -1, &[]);
        push_surf_hier_entry(&mut buf, "child", 0, 0, &[]);

        let mut host = MockHost::new();
        host.mdxm_blocks.insert(1, buf);

        let mut ghl_info = CGhoul2Info::default();
        ghl_info.model = 1;

        // "child"'s parent ("root") has NODESCENDANTS set -> OFF is OR'd in.
        assert_eq!(
            g2_is_surface_rendered(&mut host, &ghl_info, "child", &[]),
            G2SURFACEFLAG_OFF
        );
    }
}

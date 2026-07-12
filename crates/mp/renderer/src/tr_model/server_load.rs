//! `ServerLoad` — the sole live dedicated-server model entry point (§F
//! idiomatic reimplementation).
//!
//! Design: `docs/subsystems/tr-model.md` (FROZEN). Per the doc's `files:`
//! roster this is a sharded-by-concern file (§F21), not a distinct Raven
//! class or a distinct Rust type: all three methods below transcribe onto the
//! shared `RenderModels` owner struct (pinned in `render_models.rs`), split
//! into this file because they are the one live registration path (`RE_
//! RegisterServerModel` + its two format loaders). `register_server_model` is
//! the doc's pinned pub `## Seam definition` signature; `server_load_mdxa`/
//! `server_load_mdxm` are private (`§D12` porter latitude — the doc's `##
//! Method transcription table` lists them without a pinned signature).
//! `server_load_mdxa`'s skeleton signature omitted `host: &mut impl
//! EngineHost`, but it must call `re_register_server_models_malloc` (which
//! needs `host` for the PAK-checksum stamp, `TRM-D5`/ruling 59a) — added here
//! under the same §D12 latitude that already gave `server_load_mdxm` a `host`
//! parameter.
//!
//! Both format loaders take the target pool slot by `qhandle_t` index rather
//! than a `&mut ModelData` sibling parameter: `RenderModels` methods borrow
//! `&mut self` (for `self.cached`/`self.models`), and a separate live `&mut
//! ModelData` borrowed out of `self.models` would alias that receiver, so the
//! handle is re-resolved into `self.models` inside the method body instead
//! (ordinary split-field-borrow, not a signature Raven exposes — Raven passes
//! the raw `model_t*` it already holds from `R_AllocModel`).
//!
//! Source: `oracle/codemp/renderer/tr_model.cpp:683-792,799-993,1003-1154`

use core::ffi::c_char;

use mp_host_interface::EngineHost;
use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::{qhandle_t, MAX_QPATH};

use crate::mdx_format::mdxa_header_t::mdxaHeader_t;
use crate::mdx_format::mdxm_header_t::mdxmHeader_t;
use crate::mdx_format::mdxm_lod_t::mdxmLOD_t;
use crate::mdx_format::mdxm_lodsurf_offset_t::mdxmLODSurfOffset_t;
use crate::mdx_format::mdxm_surf_hierarchy_t::mdxmSurfHierarchy_t;
use crate::mdx_format::mdxm_surface_t::mdxmSurface_t;
use crate::tr_local::modtype_t::modtype_t;
use crate::tr_local::shader_commands_s::SHADER_MAX_INDEXES;
use crate::tr_local::stage_vars::SHADER_MAX_VERTEXES;
use crate::tr_local::surface_type_t::surfaceType_t;

use super::render_models::RenderModels;

/// Raven `MDXA_IDENT` (`"2LGA"` on an LE host, per the file magic comment) —
/// compared against the raw first 4 bytes of a registered file.
///
/// Source: `oracle/codemp/renderer/mdx_format.h:21`
const MDXA_IDENT: i32 =
    (('A' as i32) << 24) + (('G' as i32) << 16) + (('L' as i32) << 8) + ('2' as i32);

/// Raven `MDXM_IDENT` (`"2LGM"` on an LE host).
///
/// Source: `oracle/codemp/renderer/mdx_format.h:20`
const MDXM_IDENT: i32 =
    (('M' as i32) << 24) + (('G' as i32) << 16) + (('L' as i32) << 8) + ('2' as i32);

/// Raven `MDXA_VERSION`.
///
/// Source: `oracle/codemp/renderer/mdx_format.h:29`
const MDXA_VERSION: i32 = 6;

/// Raven `MDXM_VERSION`.
///
/// Source: `oracle/codemp/renderer/mdx_format.h:28`
const MDXM_VERSION: i32 = 6;

/// Raven's `LL(x)` macro — `x = LittleLong(x)` (`tr_model.cpp:20`). Identity
/// on the LE hosts this port targets (`TRM-D3`/ruling 54); kept as a named
/// call so every swap site below transcribes visibly against the oracle's
/// `LL(x)` call sites.
///
/// Source: `oracle/codemp/renderer/tr_model.cpp:20`
#[inline]
fn ll(x: i32) -> i32 {
    x.to_le()
}

/// Raven `Q_strncpyz` — truncate-and-NUL-terminate `src` into a fixed
/// `MAX_QPATH` `c_char` buffer (`mod->name = name` at `:1049`).
///
/// Source: `oracle/codemp/qcommon/q_shared.c` (`Q_strncpyz`)
fn write_qpath(dest: &mut [c_char; MAX_QPATH], src: &str) {
    for slot in dest.iter_mut() {
        *slot = 0;
    }
    let bytes = src.as_bytes();
    let n = bytes.len().min(MAX_QPATH - 1);
    for (i, &b) in bytes[..n].iter().enumerate() {
        dest[i] = b as c_char;
    }
}

/// Reads a NUL-terminated `c_char` field back out as an owned `String`
/// (`mdxm->animName`, `:867`). Model names are always ASCII paths, so the
/// `c_char -> u8 -> char` widening is lossless here.
fn read_qpath(bytes: &[c_char]) -> String {
    bytes
        .iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as u8 as char)
        .collect()
}

impl RenderModels {
    /// Raven `RE_RegisterServerModel` — the sole live dedicated model entry.
    ///
    /// Lazily registers `r_noserverghoul2` via `host.cvar_register` (the
    /// `Cvar_Get` at `:1020-1023`) — Raven guards this with a cached
    /// `cvar_t*` so it registers once; the port has no stored cvar pointer
    /// (`TRM-D2`) so it registers every call, which `cvar_register` makes
    /// idempotent. Hash lookup short-circuits an already-registered name;
    /// otherwise allocates a pool slot (`r_alloc_model`, `render_models.rs`),
    /// runs the LOD loop (`.md3` names bias `iLODStart` to `MD3_MAX_LODS-1`),
    /// fetching each LOD's disk bytes via the cache
    /// (`RE_RegisterModels_GetDiskFile`, `cached_model_binary.rs`) and
    /// dispatching on the file's ident to `server_load_mdxa`/`server_load_mdxm`
    /// (`MDXA_IDENT`/`MDXM_IDENT`) — any other ident is a hard fail. A loaded
    /// LOD frees its just-read buffer via `host.fs_free_file` unless it was
    /// already cached; a failed LOD other than LOD 0 just stops the loop
    /// (partial LODs get duplicated up to fill the higher slots), a failed
    /// LOD 0 fails the whole registration. The unknown-ident `goto fail` skips
    /// the `FS_FreeFile` call entirely (`:1109-1115`) — faithfully kept, not
    /// fixed (§A2).
    ///
    /// **Returns `mod.index` on success (`:1142`); the `fail:` label returns a
    /// literal `0`, NOT `mod.index` (`:1153`)** — a bad-ident entry stays
    /// hashed under its nonzero index (`re_insert_model_into_hash` still
    /// runs) while callers see `0` (the `G2_API.cpp` zero-check for failure).
    /// Do not `return` the handle unconditionally.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:1003-1154`
    pub fn register_server_model(&mut self, host: &mut impl EngineHost, name: &str) -> qhandle_t {
        // `if (!r_noServerGhoul2) { r_noServerGhoul2 = Cvar_Get(...); }` — no
        // stored cvar pointer here (TRM-D2), so this establishes the default
        // every call; `cvar_register` is idempotent.
        host.cvar_register("r_noserverghoul2", "0", 0);

        if name.is_empty() {
            return 0;
        }
        if name.len() >= MAX_QPATH {
            return 0;
        }

        // Hash lookup — replaces `mhHashTable`/`generateHashValue` with a
        // case-insensitive name -> handle map (`TRM-D3`/ruling 53).
        let lname = name.to_ascii_lowercase();
        if let Some(&handle) = self.hash.get(&lname) {
            return handle;
        }

        let Some(handle) = self.r_alloc_model() else {
            return 0;
        };
        let idx = handle as usize;

        // "only set the name after the model has been successfully loaded"
        // (Raven's comment) — Raven actually sets it here, before the LOD
        // loop; kept faithful to the code, not the comment.
        write_qpath(&mut self.models[idx].name, name);

        let lod_count = self.models[idx].md3.len(); // MD3_MAX_LODS
        let mut lod: i32 = if name.contains(".md3") {
            (lod_count - 1) as i32
        } else {
            0
        };
        self.models[idx].numLods = 0;

        let mut num_loaded = 0i32;

        while lod >= 0 {
            let mut filename = name.to_string();
            if lod != 0 {
                if let Some(dot) = filename.rfind('.') {
                    filename.truncate(dot);
                }
                filename.push_str(&format!("_{}.md3", lod));
            }

            if let Some((buf, mut already_cached)) =
                self.re_register_models_get_disk_file(host, &filename)
            {
                // `ident = *(unsigned *)buf; if (!bAlreadyCached) ident =
                // LittleLong(ident);` — reading the first 4 bytes as LE gives
                // the same result in both arms on this LE-only port.
                let ident = i32::from_le_bytes(buf[0..4].try_into().unwrap());

                let loaded = match ident {
                    MDXA_IDENT => {
                        self.server_load_mdxa(host, handle, &buf, &filename, &mut already_cached)
                    }
                    MDXM_IDENT => {
                        self.server_load_mdxm(host, handle, &buf, &filename, &mut already_cached)
                    }
                    _ => {
                        // `default: goto fail;` — jumps past the
                        // `FS_FreeFile` call below entirely; `buf` is simply
                        // dropped here (Raven leaks it, kept faithful, §A2).
                        self.models[idx].r#type = modtype_t::MOD_BAD;
                        self.re_insert_model_into_hash(name, handle);
                        return 0;
                    }
                };

                if !already_cached {
                    host.fs_free_file(buf);
                }

                if !loaded {
                    if lod == 0 {
                        self.models[idx].r#type = modtype_t::MOD_BAD;
                        self.re_insert_model_into_hash(name, handle);
                        return 0;
                    }
                    break;
                }

                self.models[idx].numLods += 1;
                num_loaded += 1;
            }
            // `continue` (GetDiskFile failure) falls straight through to the
            // decrement below, matching the C `for` loop's increment step.

            lod -= 1;
        }

        if num_loaded != 0 {
            // duplicate into higher LOD spots that weren't loaded, in case
            // the user changes r_lodbias on the fly (`for (lod--; lod>=0;
            // lod--)`).
            let mut l = lod - 1;
            while l >= 0 {
                self.models[idx].numLods += 1;
                self.models[idx].md3[l as usize] = self.models[idx].md3[(l + 1) as usize];
                l -= 1;
            }

            self.re_insert_model_into_hash(name, handle);
            return handle;
        }

        // fail: — still keep the model_t around (hashed as MOD_BAD) so the
        // name isn't rescanned; return the literal 0, not mod.index.
        self.models[idx].r#type = modtype_t::MOD_BAD;
        self.re_insert_model_into_hash(name, handle);
        0
    }

    /// Raven `ServerLoadMDXA` — loads a Ghoul 2 skeleton/animation (`.gla`)
    /// file into `model.mdxa`.
    ///
    /// Peeks `version`/`ofsEnd` out of `buffer` (LittleLong'd only when
    /// `already_cached` is false — an already-cached block was swapped on its
    /// first load); rejects a version mismatch. Sets `model.type = MOD_MDXA`
    /// and bumps `model.dataSize`, then hands `buffer` to
    /// `RE_RegisterServerModels_Malloc` (`cached_model_binary.rs`, `TAG_MODEL_GLA`)
    /// which returns the owning (possibly freshly-morphed) block as
    /// `model.mdxa`; that call's own `bAlreadyFound` out-param must equal the
    /// incoming `already_cached` (Raven's own `assert`). A first-time load
    /// flips `already_cached` to `true` (the caller must NOT `FS_FreeFile` a
    /// block this function has hijacked) and runs the header `LL()` swaps on
    /// `model.mdxa` (`:734-739`) — identity on LE (`TRM-D3`/ruling 54). Rejects
    /// `numFrames < 1`; an already-found block returns `qtrue` immediately,
    /// before any further swap (`:746-749`). The `#ifndef _M_IX86` skeletal/
    /// frame swaps (`:751-790`) are §20-dropped (dead arm on the `_M_IX86`
    /// WinDed target, `TRM-D3`/ruling 54).
    ///
    /// The `*mut mdxaHeader_t` cast onto `model.mdxa` operates on the
    /// 16-byte-aligned `AlignedBytes` base the cache entry owns (`TRM-D4`/
    /// ruling 58); keep it `unsafe`-confined at this seam (§D11) with a debug
    /// alignment assert at the cast site — do not invent an alignment
    /// strategy.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:683-792`
    fn server_load_mdxa(
        &mut self,
        host: &mut impl EngineHost,
        model: qhandle_t,
        buffer: &[u8],
        mod_name: &str,
        already_cached: &mut bool,
    ) -> bool {
        let mut version = i32::from_le_bytes(buffer[4..8].try_into().unwrap());
        let mut size = i32::from_le_bytes(buffer[96..100].try_into().unwrap()); // mdxaHeader_t::ofsEnd

        if !*already_cached {
            version = ll(version);
            size = ll(size);
        }

        if version != MDXA_VERSION {
            return false;
        }

        let idx = model as usize;
        self.models[idx].r#type = modtype_t::MOD_MDXA;
        self.models[idx].dataSize += size;

        let (ptr, already_found) = self.re_register_server_models_malloc(
            host,
            size,
            Some(buffer),
            mod_name,
            memtag_t::TAG_MODEL_GLA,
        );
        debug_assert_eq!(
            *already_cached, already_found,
            "bAlreadyCached == bAlreadyFound"
        );
        debug_assert_eq!(
            ptr as usize % 16,
            0,
            "AlignedBytes base must be 16-byte aligned"
        );

        let mdxa = ptr as *mut mdxaHeader_t;
        self.models[idx].mdxa = mdxa;

        if !already_found {
            // "we've just done a tag-morph" (Raven) — here, the one-time
            // ingest copy (`TRM-D4`(a)); `assert(mdxa == buffer)` doesn't
            // hold under that divergence and is dropped, not ported (§19).
            *already_cached = true;

            // SAFETY: `mdxa` is the just-copy-constructed 16-byte-aligned
            // `AlignedBytes` base (`TRM-D4`/ruling 58); the debug alignment
            // assert above covers the cast, per §D11.
            unsafe {
                (*mdxa).ident = ll((*mdxa).ident);
                (*mdxa).version = ll((*mdxa).version);
                (*mdxa).numFrames = ll((*mdxa).numFrames);
                (*mdxa).numBones = ll((*mdxa).numBones);
                (*mdxa).ofsFrames = ll((*mdxa).ofsFrames);
                (*mdxa).ofsEnd = ll((*mdxa).ofsEnd);
            }
        }

        // SAFETY: see above — `mdxa` is the aligned, live block either way.
        let num_frames = unsafe { (*mdxa).numFrames };
        if num_frames < 1 {
            return false;
        }

        if already_found {
            // "All done, stop here, do not LittleLong() etc." (Raven)
            return true;
        }

        // `#ifndef _M_IX86` skeletal/frame swaps (`:751-790`) — §20-dropped:
        // dead arm on the `_M_IX86` WinDed target (`TRM-D3`/ruling 54).
        true
    }

    /// Raven `ServerLoadMDXM` — loads a Ghoul 2 mesh (`.glm`) file into
    /// `model.mdxm`.
    ///
    /// Same version-peek/reject and `RE_RegisterServerModels_Malloc`
    /// (`TAG_MODEL_GLM`)/header-`LL()`/`already_cached` handshake as
    /// `server_load_mdxa` (`:823-864`). Recurses into `register_server_model`
    /// for the paired `.gla` (`mdxm->animName` + `".gla"`, `:867`) to fill
    /// `model.mdxm.animIndex`; a zero `animIndex` fails the load. Sets
    /// `model.numLods = mdxm.numLODs - 1` (`:873`) — this runs even on the
    /// already-cached path, before its own early `qtrue` return (`:875-878`).
    /// The surface-hierarchy walk (surf child-index `LL()`s, forced
    /// `shaderIndex = 0` — servers never register shaders — plus
    /// `RE_RegisterModels_StoreShaderRequest`, `:880-902`) and the LOD/surface
    /// field swaps + `SHADER_MAX_VERTEXES`/`SHADER_MAX_INDEXES` bounds checks
    /// + forced `surf.ident = SF_MDX` (`:904-937`) run on intel too — only the
    /// `#ifndef _M_IX86` bone-ref/triangle/vertex swaps nested inside that
    /// same loop (`:938-983`) are §20-dropped (`TRM-D3`/ruling 54).
    ///
    /// The `*mut mdxmHeader_t` cast onto `model.mdxm`, and every in-place
    /// surface/LOD field read+swap, operate on the 16-byte-aligned
    /// `AlignedBytes` base (`TRM-D4`/ruling 58); `unsafe`-confined at this seam
    /// (§D11) with a debug alignment assert at each cast site.
    ///
    /// Source: `oracle/codemp/renderer/tr_model.cpp:799-993`
    fn server_load_mdxm(
        &mut self,
        host: &mut impl EngineHost,
        model: qhandle_t,
        buffer: &[u8],
        mod_name: &str,
        already_cached: &mut bool,
    ) -> bool {
        let mut version = i32::from_le_bytes(buffer[4..8].try_into().unwrap());
        let mut size = i32::from_le_bytes(buffer[160..164].try_into().unwrap()); // mdxmHeader_t::ofsEnd

        if !*already_cached {
            version = ll(version);
            size = ll(size);
        }

        if version != MDXM_VERSION {
            return false;
        }

        let idx = model as usize;
        self.models[idx].r#type = modtype_t::MOD_MDXM;
        self.models[idx].dataSize += size;

        let (ptr, already_found) = self.re_register_server_models_malloc(
            host,
            size,
            Some(buffer),
            mod_name,
            memtag_t::TAG_MODEL_GLM,
        );
        debug_assert_eq!(
            *already_cached, already_found,
            "bAlreadyCached == bAlreadyFound"
        );
        debug_assert_eq!(
            ptr as usize % 16,
            0,
            "AlignedBytes base must be 16-byte aligned"
        );

        let mdxm = ptr as *mut mdxmHeader_t;
        self.models[idx].mdxm = mdxm;

        if !already_found {
            // "we've just done a tag-morph" — the one-time ingest copy
            // (`TRM-D4`(a)); `assert(mdxm == buffer)` doesn't hold under that
            // divergence and is dropped, not ported (§19).
            *already_cached = true;

            // SAFETY: `mdxm` is the just-copy-constructed 16-byte-aligned
            // `AlignedBytes` base (`TRM-D4`/ruling 58); the debug alignment
            // assert above covers the cast, per §D11.
            unsafe {
                (*mdxm).ident = ll((*mdxm).ident);
                (*mdxm).version = ll((*mdxm).version);
                (*mdxm).numLODs = ll((*mdxm).numLODs);
                (*mdxm).ofsLODs = ll((*mdxm).ofsLODs);
                (*mdxm).numSurfaces = ll((*mdxm).numSurfaces);
                (*mdxm).ofsSurfHierarchy = ll((*mdxm).ofsSurfHierarchy);
                (*mdxm).ofsEnd = ll((*mdxm).ofsEnd);
            }
        }

        // "go load in the animation file we need that has the skeletal
        // animation info for this model" — runs on both the fresh and
        // already-cached paths (`:866-871`).
        // SAFETY: `mdxm` is the aligned, live block either way; `animName`
        // is never itself byte-swapped (it's a char array).
        let anim_name = unsafe { read_qpath(&(*mdxm).animName) };
        let anim_filename = format!("{}.gla", anim_name);
        let anim_index = self.register_server_model(host, &anim_filename);
        // SAFETY: as above.
        unsafe {
            (*mdxm).animIndex = anim_index;
        }
        if anim_index == 0 {
            return false;
        }

        // "copy this up to the model for ease of use - it wil get inced
        // after this" — overwrites, does not accumulate; the caller's LOD
        // loop increments `numLods` by 1 right after this call returns.
        // SAFETY: as above.
        let num_lods = unsafe { (*mdxm).numLODs };
        self.models[idx].numLods = num_lods - 1;

        if already_found {
            // "All done. Stop, go no further, do not LittleLong(), do not
            // pass Go..."
            return true;
        }

        // "we need to do the middle part of this even for intel, because of
        // shader reg and err-check" (`:904`) — runs unconditionally below;
        // only the nested `#ifndef _M_IX86` blocks are §20-dropped.
        //
        // SAFETY: every pointer walk below stays inside the `AlignedBytes`
        // block the cache entry owns (its size is the file's `ofsEnd`), off
        // the 16-byte-aligned base asserted above (§D11).
        unsafe {
            let base = ptr;
            let num_surfaces = (*mdxm).numSurfaces;

            let mut surf_info =
                base.add((*mdxm).ofsSurfHierarchy as usize) as *mut mdxmSurfHierarchy_t;
            for _ in 0..num_surfaces {
                (*surf_info).numChildren = ll((*surf_info).numChildren);
                (*surf_info).parentIndex = ll((*surf_info).parentIndex);

                let num_children = (*surf_info).numChildren;
                let child_indexes = core::ptr::addr_of_mut!((*surf_info).childIndexes) as *mut i32;
                for j in 0..num_children as usize {
                    let child = child_indexes.add(j);
                    *child = ll(*child);
                }

                // "We will not be using shaders on the server."
                (*surf_info).shaderIndex = 0;

                let name_offset =
                    (core::ptr::addr_of!((*surf_info).shader) as usize - base as usize) as i32;
                let poke_offset =
                    (core::ptr::addr_of!((*surf_info).shaderIndex) as usize - base as usize) as i32;
                self.re_register_models_store_shader_request(mod_name, name_offset, poke_offset);

                let surf_info_size = core::mem::offset_of!(mdxmSurfHierarchy_t, childIndexes)
                    + (num_children as usize) * core::mem::size_of::<i32>();
                surf_info = (surf_info as *mut u8).add(surf_info_size) as *mut mdxmSurfHierarchy_t;
            }

            // Re-read post-swap header fields for the LOD walk.
            let mdxm = ptr as *mut mdxmHeader_t;
            let mut lod = base.add((*mdxm).ofsLODs as usize) as *mut mdxmLOD_t;
            for _ in 0..(*mdxm).numLODs {
                (*lod).ofsEnd = ll((*lod).ofsEnd);

                let mut surf = (lod as *mut u8).add(
                    core::mem::size_of::<mdxmLOD_t>()
                        + (num_surfaces as usize) * core::mem::size_of::<mdxmLODSurfOffset_t>(),
                ) as *mut mdxmSurface_t;
                for _ in 0..num_surfaces {
                    (*surf).numTriangles = ll((*surf).numTriangles);
                    (*surf).ofsTriangles = ll((*surf).ofsTriangles);
                    (*surf).numVerts = ll((*surf).numVerts);
                    (*surf).ofsVerts = ll((*surf).ofsVerts);
                    (*surf).ofsEnd = ll((*surf).ofsEnd);
                    (*surf).ofsHeader = ll((*surf).ofsHeader);
                    (*surf).numBoneReferences = ll((*surf).numBoneReferences);
                    (*surf).ofsBoneReferences = ll((*surf).ofsBoneReferences);

                    if (*surf).numVerts > SHADER_MAX_VERTEXES as i32 {
                        return false;
                    }
                    if (*surf).numTriangles * 3 > SHADER_MAX_INDEXES as i32 {
                        return false;
                    }

                    (*surf).ident = surfaceType_t::SF_MDX as i32;

                    // `#ifndef _M_IX86` bone-ref/triangle/vertex swaps
                    // (`:938-983`) — §20-dropped (`TRM-D3`/ruling 54).

                    surf = (surf as *mut u8).add((*surf).ofsEnd as usize) as *mut mdxmSurface_t;
                }

                lod = (lod as *mut u8).add((*lod).ofsEnd as usize) as *mut mdxmLOD_t;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The FourCC macros are written as native-int char shifts and compared
    /// directly against the file's first 4 bytes; on an LE host that integer
    /// unpacks to the ASCII magic the format-header comments name ("2LGA"/
    /// "2LGM", `mdx_format.h:20-21`). A sign or byte-order slip here would
    /// make every registration dispatch to the `default: goto fail` arm.
    #[test]
    fn mdxa_ident_matches_le_file_magic() {
        assert_eq!(MDXA_IDENT.to_le_bytes(), *b"2LGA");
    }

    #[test]
    fn mdxm_ident_matches_le_file_magic() {
        assert_eq!(MDXM_IDENT.to_le_bytes(), *b"2LGM");
    }

    /// `LL()` is identity on the LE hosts this port targets (`TRM-D3`/ruling
    /// 54) — pinned so a future accidental byte-swap implementation trips
    /// this test rather than silently corrupting every header field.
    #[test]
    fn ll_is_identity_on_le() {
        assert_eq!(ll(0x0102_0304), 0x0102_0304);
        assert_eq!(ll(-1), -1);
        assert_eq!(ll(0), 0);
    }

    /// `Q_strncpyz` semantics (`:1049`): copies up to `MAX_QPATH - 1` bytes
    /// and NUL-pads the remainder — never a short buffer overrun, never a
    /// missing terminator.
    #[test]
    fn write_qpath_pads_short_names_with_nul() {
        let mut dest = [1 as c_char; MAX_QPATH]; // poison with non-zero to prove padding
        write_qpath(&mut dest, "models/foo.md3");
        let expected = b"models/foo.md3";
        for (i, &b) in expected.iter().enumerate() {
            assert_eq!(dest[i], b as c_char);
        }
        for slot in &dest[expected.len()..] {
            assert_eq!(*slot, 0);
        }
    }

    /// A name at-or-over `MAX_QPATH` is truncated to `MAX_QPATH - 1` bytes
    /// plus a NUL terminator — `strncpy`'s "no terminator added on exact
    /// fill" case does not apply here since the dest is pre-zeroed and only
    /// `MAX_QPATH - 1` bytes are ever written.
    #[test]
    fn write_qpath_truncates_long_names() {
        let long_name = "a".repeat(MAX_QPATH + 10);
        let mut dest = [0 as c_char; MAX_QPATH];
        write_qpath(&mut dest, &long_name);
        assert_eq!(dest[MAX_QPATH - 1], 0);
        assert_eq!(dest[MAX_QPATH - 2], b'a' as c_char);
    }

    /// `read_qpath` must stop at the first NUL, not read past it into
    /// trailing garbage/poison bytes — this is what makes
    /// `mdxm->animName + ".gla"` safe to reconstruct as a Rust `String`.
    #[test]
    fn read_qpath_stops_at_first_nul() {
        let mut bytes = [b'X' as c_char; MAX_QPATH];
        for (i, &b) in b"skeleton".iter().enumerate() {
            bytes[i] = b as c_char;
        }
        bytes[8] = 0;
        assert_eq!(read_qpath(&bytes), "skeleton");
    }

    #[test]
    fn read_qpath_empty_on_leading_nul() {
        let bytes = [0 as c_char; MAX_QPATH];
        assert_eq!(read_qpath(&bytes), "");
    }
}

#![allow(non_camel_case_types, non_snake_case)]

//! `G2_Bolts` internal — the bolt-list mutators consumed by the `G2API_*` bolt
//! wrappers (`api_bolts.rs`).
//!
//! Per `docs/subsystems/ghoul2-server.md` roster (`bolts.rs`, class "G2_Bolts
//! internal"): `G2_Add_Bolt`/`G2_Add_Bolt_Surf_Num`/`G2_Remove_Bolt`/
//! `G2_Init_Bolt_List`/`G2_Find_Bolt_Bone_Num`/`G2_Find_Bolt_Surface_Num`/
//! `G2_RemoveRedundantBolts` — the entire content of `G2_bolts.cpp`, no private
//! helpers beyond these seven (the file calls out to `G2_IsSurfaceLegal`/
//! `G2_FindOverrideSurface`, both owned by `surfaces.rs`).
//!
//! Classified **host-free** by the doc's `## Slice hooks` ("its one apparent
//! host line `G2_bolts.cpp:194` is a commented-out `Com_Printf`") — no
//! `EngineHost` parameter on any signature below. See the `g2_add_bolt` doc
//! comment for a doc/oracle mismatch this classification misses (reported
//! separately, not improvised around here per porting-rules §F17).
//!
//! **Raven `assert(...)` calls quoted below are not ported as Rust
//! `assert!`/`panic!`**: this build defines `-DNDEBUG` (doc's Raven ground
//! truth, top of `docs/subsystems/ghoul2-server.md`'s "Build config"), so
//! every plain `assert(...)` in this file's oracle bodies is a no-op in the
//! shipped binary (same house convention as `api_models.rs`'s module doc
//! comment) — dropped, with a one-line citation at each site.

use mp_host_interface::EngineHost;

use crate::mdx::mdxa::MdxaView;
use crate::shared::bolt_info_t::boltInfo_t;
use crate::shared::cghoul2_info::CGhoul2Info;
use crate::shared::surface_info_t::surfaceInfo_t;
use mp_qshared::shared::mdxaBone_t;

/// Raven `G2SURFACEFLAG_GENERATED` — marks a bolt/surface override slot as
/// referencing a *generated* surface rather than an original model surface.
///
/// Source: `oracle/codemp/renderer/mdx_format.h:50`
const G2SURFACEFLAG_GENERATED: i32 = 0x0000_0200;

/// Raven `G2_Find_Bolt_Bone_Num` — given a bone number, see if that bone is
/// already in our bone list (`-1` entries are unused slots, skipped).
///
/// Source: `oracle/codemp/ghoul2/G2_bolts.cpp:21-42`
pub fn g2_find_bolt_bone_num(bltlist: &[boltInfo_t], bone_num: i32) -> i32 {
    for (i, bolt) in bltlist.iter().enumerate() {
        // if this bone entry has no info in it, bounce over it
        if bolt.boneNumber == -1 {
            continue;
        }

        if bolt.boneNumber == bone_num {
            return i as i32;
        }
    }

    // didn't find it
    -1
}

/// Raven `G2_Find_Bolt_Surface_Num` — given a surface number, see if that
/// surface is already in our surface list, gated by `(surfaceType & flags) ==
/// flags`.
///
/// Source: `oracle/codemp/ghoul2/G2_bolts.cpp:45-66`
pub fn g2_find_bolt_surface_num(bltlist: &[boltInfo_t], surface_num: i32, flags: i32) -> i32 {
    for (i, bolt) in bltlist.iter().enumerate() {
        // if this bone entry has no info in it, bounce over it
        if bolt.surfaceNumber == -1 {
            continue;
        }

        if bolt.surfaceNumber == surface_num && (bolt.surfaceType & flags) == flags {
            return i as i32;
        }
    }

    // didn't find it
    -1
}

/// Raven `G2_Add_Bolt_Surf_Num` — add a bolt on a known surface index: bumps
/// `boltUsed` if already bolted, else fills an empty (`-1`/`-1`) slot, else
/// pushes a new entry; `-1` when `surfNum >= slist.size()`.
///
/// `ghlInfo` is read only for its `assert(ghlInfo && ghlInfo->mValid)` — no
/// model-memory read, so this fn is genuinely host-free (unlike
/// `g2_add_bolt` below).
///
/// Source: `oracle/codemp/ghoul2/G2_bolts.cpp:70-117`
pub fn g2_add_bolt_surf_num(
    // Raven `assert(ghlInfo && ghlInfo->mValid)` (`:72`) is a no-op under
    // `-DNDEBUG` — dropped (see module doc comment); the parameter is kept
    // for signature fidelity but unused.
    _ghl_info: &CGhoul2Info,
    bltlist: &mut Vec<boltInfo_t>,
    slist: &[surfaceInfo_t],
    surf_num: i32,
) -> i32 {
    // first up, make sure have a surface first
    if surf_num >= slist.len() as i32 {
        return -1;
    }

    // look through entire list - see if it's already there first
    for (i, bolt) in bltlist.iter_mut().enumerate() {
        // already there??
        if bolt.surfaceNumber == surf_num {
            // increment the usage count
            bolt.boltUsed += 1;
            return i as i32;
        }
    }

    // we have a surface
    // look through entire list - see if it's already there first
    for (i, bolt) in bltlist.iter_mut().enumerate() {
        // if this surface entry has info in it, bounce over it
        if bolt.boneNumber == -1 && bolt.surfaceNumber == -1 {
            // if we found an entry that had a -1 for the bone / surface number,
            // then we hit a surface / bone slot that was empty
            bolt.surfaceNumber = surf_num;
            bolt.surfaceType = G2SURFACEFLAG_GENERATED;
            bolt.boltUsed = 1;
            return i as i32;
        }
    }

    // ok, we didn't find an existing surface of that name, or an empty slot.
    // Lets add an entry
    bltlist.push(boltInfo_t {
        boneNumber: -1,
        surfaceNumber: surf_num,
        surfaceType: G2SURFACEFLAG_GENERATED,
        boltUsed: 1,
        // Raven's stack-local `tempBolt` never initializes `position`
        // (`ghoul2_shared.h:170-182`'s `mdxaBone_t position` member) before
        // this `push_back` — genuinely uninitialized C++ memory. Zeroed here
        // (a defined value the transform chain always overwrites before any
        // read) rather than reproducing the indeterminate bytes (§F19).
        position: mdxaBone_t {
            matrix: [[0.0; 4]; 3],
        },
    });
    bltlist.len() as i32 - 1
}

/// Raven `G2_Add_Bolt` — add a bolt on `boneName`: first tries it as a surface
/// name via `G2_IsSurfaceLegal(mod_m, boneName, &flags)` (bumping/reusing/
/// pushing a bolt slot exactly like `g2_add_bolt_surf_num`, `surfaceType = 0`
/// on this path), then falls back to a `mod_a->mdxa` bone-name walk
/// (`mdxaSkelOffsets_t` + `mdxaSkel_t->name` `stricmp`, `:175-186`) if no
/// surface matched; `-1` if neither matches.
///
/// `host` added 2026-07-14, closing the doc/oracle mismatch this doc-comment
/// previously reported (§F17): the doc's "host-free" classification of this
/// file was wrong — Raven's body calls `G2_IsSurfaceLegal` (a `mod_m->mdxm`
/// read) and walks `mod_a->mdxa` bone names, so the frozen host-less
/// signature forced a permanent `-1` stub, which silently killed every
/// server-side bolt (saber muzzles collapsed to the entity origin; sabers
/// whiffed). The bone walk reads `ghl_info.a_header` — the cached block
/// `G2_SetupModelPointers` populates, exactly Raven's `mod_a` usage — so the
/// caller must run setup first (`g2api_add_bolt` does, as of the same fix).
///
/// Source: `oracle/codemp/ghoul2/G2_bolts.cpp:119-233`
pub fn g2_add_bolt(
    host: &mut impl EngineHost,
    ghl_info: &CGhoul2Info,
    bltlist: &mut Vec<boltInfo_t>,
    slist: &[surfaceInfo_t],
    bone_name: &str,
) -> i32 {
    // Raven's `slist` parameter is unread by this function's body — kept for
    // 1:1 arity fidelity only.
    let _ = slist;

    // first up, we'll search for that which this bolt names in all the surfaces
    let surf = crate::surfaces::g2_is_surface_legal(host, ghl_info.model, bone_name);

    // did we find it as a surface?
    if let Some((surf_num, _flags)) = surf {
        // look through entire list - see if it's already there first
        for (i, bolt) in bltlist.iter_mut().enumerate() {
            // already there??
            if bolt.surfaceNumber == surf_num {
                // increment the usage count
                bolt.boltUsed += 1;
                return i as i32;
            }
        }

        // look through entire list - see if we can re-use one
        for (i, bolt) in bltlist.iter_mut().enumerate() {
            // if this surface entry has info in it, bounce over it
            if bolt.boneNumber == -1 && bolt.surfaceNumber == -1 {
                // if we found an entry that had a -1 for the bone / surface
                // number, then we hit a surface / bone slot that was empty
                bolt.surfaceNumber = surf_num;
                bolt.boltUsed = 1;
                bolt.surfaceType = 0;
                return i as i32;
            }
        }

        // ok, we didn't find an existing surface of that name, or an empty
        // slot. Lets add an entry
        bltlist.push(boltInfo_t {
            boneNumber: -1,
            surfaceNumber: surf_num,
            surfaceType: 0,
            boltUsed: 1,
            // Raven's stack-local `tempBolt.position` is uninitialized C++
            // memory; zeroed here (§F19, same note as `g2_add_bolt_surf_num`).
            position: mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            },
        });
        return bltlist.len() as i32 - 1;
    }

    // no, check to see if it's a bone then
    // SAFETY: `a_header` is the `EngineHost::model_mdxa` block
    // `G2_SetupModelPointers` cached (the oracle's `mod_a->mdxa`, dereferenced
    // unchecked there too); callers run setup first (`g2api_add_bolt`).
    let mdxa = unsafe { MdxaView::from_block(ghl_info.a_header) };
    let num_bones = mdxa.num_bones();

    // walk the entire list of bones in the gla file for this model and see if
    // any match the name of the bone we want to find (`stricmp`, case-insensitive)
    let mut x = 0;
    while x < num_bones {
        if mdxa.skel(x).name_matches(bone_name) {
            break;
        }
        x += 1;
    }

    // check to see we did actually make a match with a bone in the model
    if x == num_bones {
        // didn't find it? Error
        return -1;
    }

    // look through entire list - see if it's already there first
    for (i, bolt) in bltlist.iter_mut().enumerate() {
        // already there??
        if bolt.boneNumber == x {
            // increment the usage count
            bolt.boltUsed += 1;
            return i as i32;
        }
    }

    // look through entire list - see if we can re-use it
    for (i, bolt) in bltlist.iter_mut().enumerate() {
        // if this bone entry has info in it, bounce over it
        if bolt.boneNumber == -1 && bolt.surfaceNumber == -1 {
            // if we found an entry that had a -1 for the bonenumber, then we
            // hit a bone slot that was empty
            bolt.boneNumber = x;
            bolt.boltUsed = 1;
            bolt.surfaceType = 0;
            return i as i32;
        }
    }

    // ok, we didn't find an existing bone of that name, or an empty slot.
    // Lets add an entry
    bltlist.push(boltInfo_t {
        boneNumber: x,
        surfaceNumber: -1,
        surfaceType: 0,
        boltUsed: 1,
        // Same §F19 zero-init note as the surface arm above.
        position: mdxaBone_t {
            matrix: [[0.0; 4]; 3],
        },
    });
    bltlist.len() as i32 - 1
}

/// Raven `G2_Remove_Bolt` — decrement `boltUsed`; on hitting zero, mark the
/// slot unused (`boneNumber`/`surfaceNumber = -1`) and shrink the list off the
/// back over any trailing run of unused slots. `qfalse` (+ `assert(0)`) on
/// `index == -1`.
///
/// Source: `oracle/codemp/ghoul2/G2_bolts.cpp:236-277`
pub fn g2_remove_bolt(bltlist: &mut Vec<boltInfo_t>, index: i32) -> bool {
    // did we find it?
    if index != -1 {
        let idx = index as usize;
        bltlist[idx].boltUsed -= 1;
        if bltlist[idx].boltUsed == 0 {
            // set this bone to not used
            bltlist[idx].boneNumber = -1;
            bltlist[idx].surfaceNumber = -1;

            let mut new_size = bltlist.len();
            // now look through the list from the back and see if there is a
            // block of -1's we can resize off the end of the list
            for i in (0..bltlist.len()).rev() {
                if bltlist[i].surfaceNumber == -1 && bltlist[i].boneNumber == -1 {
                    new_size = i;
                } else {
                    // once we hit one that isn't a -1, we are done.
                    break;
                }
            }
            // do we need to resize?
            if new_size != bltlist.len() {
                // yes, so lets do it
                bltlist.truncate(new_size);
            }
        }
        return true;
    }

    // Raven `assert(0)` (`:273`) is a no-op under `-DNDEBUG` — dropped.
    false
}

/// Raven `G2_Init_Bolt_List` — set the bolt list to all unused so the bone
/// transformation routine ignores it (`bltlist.clear()`).
///
/// Source: `oracle/codemp/ghoul2/G2_bolts.cpp:280-283`
pub fn g2_init_bolt_list(bltlist: &mut Vec<boltInfo_t>) {
    bltlist.clear();
}

/// Raven `G2_RemoveRedundantBolts` — remove any bolts that reference original
/// surfaces, generated surfaces, or bones that aren't active anymore. Calls
/// `G2_FindOverrideSurface` (`surfaces.rs`) — a pure `surfaceList` walk, no
/// model memory — and `g2_remove_bolt` above; genuinely host-free (unlike
/// `g2_add_bolt`).
///
/// Source: `oracle/codemp/ghoul2/G2_bolts.cpp:286-331`
pub fn g2_remove_redundant_bolts(
    bltlist: &mut Vec<boltInfo_t>,
    slist: &[surfaceInfo_t],
    active_surfaces: &[i32],
    active_bones: &[i32],
) {
    // walk the bolt list
    let mut i = 0;
    while i < bltlist.len() {
        // are we using this bolt?
        if bltlist[i].surfaceNumber != -1 || bltlist[i].boneNumber != -1 {
            // is this referenceing a surface?
            if bltlist[i].surfaceNumber != -1 {
                // is this bolt looking at a generated surface?
                if bltlist[i].surfaceType != 0
                    && crate::surfaces::g2_find_override_surface(bltlist[i].surfaceNumber, slist)
                        .is_none()
                {
                    // no - we want to remove this bolt, regardless of how many
                    // people are using it
                    bltlist[i].boltUsed = 1;
                    g2_remove_bolt(bltlist, i as i32);
                }

                // Raven's second check (`:308-316`) is a bare `{ }` compound
                // statement with **no** `else` keyword — despite the "no, it's
                // an original" comment reading as though it should be one, it
                // runs unconditionally after the `surfaceType` branch above
                // (ported faithfully, §A2). The bolt removed just above may
                // have shrunk `bltlist` (dropping index `i` entirely) or freed
                // this exact slot's `surfaceNumber` to `-1`; Raven's own
                // `activeSurfaces[bltlist[i].surfaceNumber]` read in either
                // case is an out-of-bounds/negative-index read the oracle
                // itself takes as UB. The one divergence here (§F19): an
                // out-of-range index is treated as "still active" (no further
                // removal) instead of reproducing the crash.
                if i < bltlist.len() {
                    let surf = bltlist[i].surfaceNumber;
                    let still_active = usize::try_from(surf)
                        .ok()
                        .and_then(|s| active_surfaces.get(s))
                        .map(|&flag| flag != 0)
                        .unwrap_or(true);
                    if !still_active {
                        // no - we want to remove this bolt, regardless of how
                        // many people are using it
                        bltlist[i].boltUsed = 1;
                        g2_remove_bolt(bltlist, i as i32);
                    }
                }
            }
            // no, must be looking at a bone then
            else if active_bones[bltlist[i].boneNumber as usize] == 0 {
                // is that bone active then?
                // no - we want to remove this bolt, regardless of how many
                // people are using it
                bltlist[i].boltUsed = 1;
                g2_remove_bolt(bltlist, i as i32);
            }
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::surface_info_t::surfaceInfo_t;
    use mp_qshared::shared::mdxaBone_t;

    fn bolt(
        bone_number: i32,
        surface_number: i32,
        surface_type: i32,
        bolt_used: i32,
    ) -> boltInfo_t {
        boltInfo_t {
            boneNumber: bone_number,
            surfaceNumber: surface_number,
            surfaceType: surface_type,
            boltUsed: bolt_used,
            position: mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            },
        }
    }

    fn surface(surface: i32) -> surfaceInfo_t {
        surfaceInfo_t {
            offFlags: 0,
            surface,
            genBarycentricJ: 0.0,
            genBarycentricI: 0.0,
            genPolySurfaceIndex: 0,
            genLod: 0,
        }
    }

    #[test]
    fn find_bolt_bone_num_skips_unused_slots() {
        let bltlist = vec![bolt(-1, -1, 0, 0), bolt(-1, 3, 0, 0), bolt(5, -1, 0, 1)];
        assert_eq!(g2_find_bolt_bone_num(&bltlist, 5), 2);
        assert_eq!(g2_find_bolt_bone_num(&bltlist, 3), -1); // 3 is a surface, not a bone
        assert_eq!(g2_find_bolt_bone_num(&bltlist, 9), -1);
    }

    #[test]
    fn find_bolt_surface_num_gates_on_flags() {
        let bltlist = vec![bolt(-1, 7, G2SURFACEFLAG_GENERATED, 1)];
        assert_eq!(
            g2_find_bolt_surface_num(&bltlist, 7, G2SURFACEFLAG_GENERATED),
            0
        );
        // flags must be a subset of surfaceType: (surfaceType & flags) == flags
        assert_eq!(g2_find_bolt_surface_num(&bltlist, 7, 0x1), -1);
        assert_eq!(g2_find_bolt_surface_num(&bltlist, 3, 0), -1);
    }

    #[test]
    fn add_bolt_surf_num_reuses_slot_then_pushes() {
        let ghl = CGhoul2Info::default();
        let slist = vec![surface(0), surface(1), surface(2)];
        let mut bltlist: Vec<boltInfo_t> = vec![bolt(-1, -1, 0, 0)];

        // surf_num out of range -> -1
        assert_eq!(g2_add_bolt_surf_num(&ghl, &mut bltlist, &slist, 5), -1);

        // reuses the one empty slot
        let idx = g2_add_bolt_surf_num(&ghl, &mut bltlist, &slist, 1);
        assert_eq!(idx, 0);
        assert_eq!(bltlist[0].surfaceNumber, 1);
        assert_eq!(bltlist[0].surfaceType, G2SURFACEFLAG_GENERATED);
        assert_eq!(bltlist[0].boltUsed, 1);

        // already-bolted surface bumps the usage count
        let idx_again = g2_add_bolt_surf_num(&ghl, &mut bltlist, &slist, 1);
        assert_eq!(idx_again, 0);
        assert_eq!(bltlist[0].boltUsed, 2);

        // no empty slot left -> pushes a new entry
        let idx_push = g2_add_bolt_surf_num(&ghl, &mut bltlist, &slist, 2);
        assert_eq!(idx_push, 1);
        assert_eq!(bltlist.len(), 2);
        assert_eq!(bltlist[1].surfaceNumber, 2);
        assert_eq!(bltlist[1].boneNumber, -1);
    }

    #[test]
    fn add_bolt_resolves_bone_names_and_reuses_slots() {
        use mp_host_interface::mock::MockHost;

        // Minimal `.glm` header: numSurfaces=0, so the surface-name search
        // misses and the bone arm runs (layout per `surfaces.rs` MDXM_*).
        let mut mdxm = vec![0u8; 164];
        mdxm[152..156].copy_from_slice(&0i32.to_ne_bytes());
        mdxm[156..160].copy_from_slice(&164i32.to_ne_bytes());

        // Minimal `.gla`: one bone named "testbone" (layout per `skeleton.rs`
        // MDXA_*: numBones at 84, offsets table after the 100-byte header,
        // `mdxaSkel_t.name` first in the entry).
        let mut mdxa = vec![0u8; 100];
        mdxa[84..88].copy_from_slice(&1i32.to_ne_bytes());
        mdxa.extend_from_slice(&4i32.to_ne_bytes()); // offsets[0]
        let mut skel = [0u8; 64 + 8];
        skel[..8].copy_from_slice(b"testbone");
        mdxa.extend_from_slice(&skel);
        // ofsEnd @96: the block's total self-describing size (MdxaView::from_block).
        let ofs_end = mdxa.len() as i32;
        mdxa[96..100].copy_from_slice(&ofs_end.to_le_bytes());

        let mut host = MockHost::new();
        host.mdxm_blocks.insert(1, mdxm);
        let mut ghl = CGhoul2Info::default();
        ghl.model = 1;
        ghl.a_header = mdxa.as_ptr() as *const core::ffi::c_void;

        let mut bltlist: Vec<boltInfo_t> = Vec::new();
        let slist: Vec<surfaceInfo_t> = Vec::new();

        // bone-name hit (case-insensitive, Raven `stricmp`)
        assert_eq!(
            g2_add_bolt(&mut host, &ghl, &mut bltlist, &slist, "TESTBONE"),
            0
        );
        assert_eq!(bltlist[0].boneNumber, 0);
        assert_eq!(bltlist[0].surfaceNumber, -1);
        assert_eq!(bltlist[0].boltUsed, 1);

        // re-add bumps the usage count on the same slot
        assert_eq!(
            g2_add_bolt(&mut host, &ghl, &mut bltlist, &slist, "testbone"),
            0
        );
        assert_eq!(bltlist[0].boltUsed, 2);

        // unknown name misses
        assert_eq!(
            g2_add_bolt(&mut host, &ghl, &mut bltlist, &slist, "nope"),
            -1
        );
    }

    #[test]
    fn remove_bolt_decrements_then_frees_and_shrinks_trailing_run() {
        let mut bltlist = vec![bolt(1, -1, 0, 1), bolt(2, -1, 0, 2), bolt(-1, 4, 0, 1)];

        // decrement only, still used
        assert!(g2_remove_bolt(&mut bltlist, 1));
        assert_eq!(bltlist[1].boltUsed, 1);
        assert_eq!(bltlist[1].boneNumber, 2);
        assert_eq!(bltlist.len(), 3);

        // hits zero: frees slot 2 (the trailing entry) -> shrinks the vector
        assert!(g2_remove_bolt(&mut bltlist, 2));
        assert_eq!(bltlist.len(), 2);

        // hits zero on slot 1 (now the trailing entry) -> also unused -> both
        // trailing unused slots get truncated off the back
        assert!(g2_remove_bolt(&mut bltlist, 1));
        assert_eq!(bltlist.len(), 1);
        assert_eq!(bltlist[0].boneNumber, 1);

        // index == -1 -> qfalse, dropped `assert(0)`
        assert!(!g2_remove_bolt(&mut bltlist, -1));
    }

    #[test]
    fn init_bolt_list_clears() {
        let mut bltlist = vec![bolt(1, -1, 0, 1)];
        g2_init_bolt_list(&mut bltlist);
        assert!(bltlist.is_empty());
    }

    // NOTE: `g2_remove_redundant_bolts`'s `surfaceType != 0` (generated-surface)
    // branch calls the sibling `crate::surfaces::g2_find_override_surface`,
    // whose body is still `todo!()` in this gated skeleton (per task rules,
    // tests here must not exercise a sibling's not-yet-ported body). Every
    // bolt below therefore keeps `surfaceType == 0` so only the always-run
    // second check and the bone branch are exercised.

    #[test]
    fn remove_redundant_bolts_drops_inactive_bone_and_keeps_active_surface() {
        let slist: Vec<surfaceInfo_t> = Vec::new();
        let active_surfaces = [1, 0]; // surface 0 active, surface 1 inactive
        let active_bones = [0, 1]; // bone 0 inactive, bone 1 active

        let mut bltlist = vec![
            // original surface bolt on an active surface -> the unconditional
            // second check (module doc's divergence note) leaves it alone
            bolt(-1, 0, 0, 1),
            // original surface bolt on an inactive surface -> removed
            bolt(-1, 1, 0, 1),
            // bone bolt on an inactive bone -> removed
            bolt(0, -1, 0, 1),
            // bone bolt on an active bone -> kept
            bolt(1, -1, 0, 1),
        ];

        g2_remove_redundant_bolts(&mut bltlist, &slist, &active_surfaces, &active_bones);

        assert_eq!(bltlist[0].surfaceNumber, 0); // kept (active surface)
        assert_eq!(bltlist[1].surfaceNumber, -1); // removed (inactive surface)
        assert_eq!(bltlist[1].boneNumber, -1);
        assert_eq!(bltlist[2].boneNumber, -1); // removed (inactive bone)
        assert_eq!(bltlist[2].surfaceNumber, -1);
        assert_eq!(bltlist[3].boneNumber, 1); // kept (active bone)
    }

    #[test]
    fn remove_redundant_bolts_out_of_range_surface_number_is_treated_as_active() {
        // Divergence coverage (§F19): Raven's unconditional second check reads
        // `activeSurfaces[bltlist[i].surfaceNumber]` with no bounds guard; an
        // out-of-range index must not panic here. The defined result is
        // "still active" (no removal), rather than reproducing the oracle's
        // undefined out-of-bounds read.
        let slist: Vec<surfaceInfo_t> = Vec::new();
        let active_surfaces: [i32; 1] = [1]; // only index 0 is a valid slot
        let active_bones: [i32; 0] = [];

        let mut bltlist = vec![bolt(-1, 5, 0, 1)]; // surfaceNumber 5 is out of range
        g2_remove_redundant_bolts(&mut bltlist, &slist, &active_surfaces, &active_bones);
        assert_eq!(bltlist[0].surfaceNumber, 5); // kept — out-of-range treated as active
    }
}

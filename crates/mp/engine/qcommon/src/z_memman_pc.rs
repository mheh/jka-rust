#![allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    clippy::too_many_arguments
)]

//! `z_memman_pc.cpp` — the PC zone allocator (Z_Malloc-family, doubly-linked
//! block list, per-tag stats) and the Hunk two-mark allocator layered on it.
//!
//! Source: `oracle/codemp/qcommon/z_memman_pc.cpp`

use core::ffi::c_int;

use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::ha_pref;
use native_types::qboolean;

/// Raven `ZONE_MAGIC` — zone-block header/tail guard value.
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:28`
const ZONE_MAGIC: i32 = 0x21436587;

use crate::collision_world::CollisionWorld;
use crate::common::Common;
use crate::vm_fns::VM_Clear;
use crate::z_memman::zone_header_s::zoneHeader_t;
use crate::z_memman::zone_tail_s::zoneTail_t;

// PORT-NOTE(rm-types): `RenderModels`/`RmManager`/`Ghoul2System`/`Server` are
// the state-receiver types pinned by the engine-fork-discovery preamble's
// receiver order (rmg-terrain.md / ghoul2-server.md / server crate own their
// real shape); none has landed importable here yet. Referenced by their
// exact resolved-signature names per the no-stub rule (common_fns.rs/
// vm_fns.rs precedent); reported as missing symbols for the finisher.
#[allow(dead_code)]
struct RenderModels;
#[allow(dead_code)]
struct Ghoul2System;
#[allow(dead_code)]
struct Server;

// PORT-NOTE(unlanded-callees): `Com_Printf` (com_common.cpp), `Cvar_Get`
// (cvar.cpp), `Cmd_AddCommand`/`Cmd_RemoveCommand` (cmd_pc.cpp),
// `SV_ShutdownGameProgs` (sv_game.cpp), `R_HunkClearCrap` (tr_init.cpp) have
// no ported body reachable from this file — same unlanded-callee gap as
// `vm_fns.rs`'s own local forward-declares for the same symbols. Narrowed to
// this file's own call-site shapes; escalated as missing symbols for the
// finisher.
extern "Rust" {
    fn Com_Printf(common: &mut Common, msg: &str);
    fn Cvar_Get(
        common: &mut Common,
        cm: &mut CollisionWorld,
        rm: &mut RenderModels,
        host: &mut dyn EngineHost,
        var_name: &str,
        var_value: &str,
        flags: c_int,
    ) -> *mut mp_qshared::shared::cvar::cvar_t;
    fn Cmd_AddCommand(
        common: &mut Common,
        cm: &mut CollisionWorld,
        rm: &mut RenderModels,
        host: &mut dyn EngineHost,
        cmd_name: &str,
        function: *const (),
    );
    fn Cmd_RemoveCommand(common: &mut Common, cmd_name: &str);
    fn SV_ShutdownGameProgs(common: &mut Common, sv: &mut Server);
    fn R_HunkClearCrap(rm: &mut RenderModels, host: &mut dyn EngineHost);
}

// PORT-NOTE(zmemman-body-gap): `Z_Malloc`/`Z_Free`/`Z_Validate` are
// `z_memman_pc.cpp`-native functions (this file's own subject) with no
// transcribed body anywhere in the tree yet — only forward-declared as
// unlanded callees by other files (`vm_fns.rs`/`vm_x86.rs`/`cm_load.rs`
// precedent for `Z_Malloc`/`Z_Free`). Declared here in the same shape;
// escalated as missing symbols (genuine logic port, not mechanical) for the
// finisher.
extern "Rust" {
    fn Z_Malloc(
        common: &mut Common,
        cm: &mut CollisionWorld,
        rm: &mut RenderModels,
        host: &mut dyn EngineHost,
        iSize: c_int,
        eTag: memtag_t,
        bZeroit: qboolean,
        iUnusedAlign: c_int,
    ) -> *mut ();
    fn Z_Free(common: &mut Common, pvAddress: *mut ());
    fn Z_Validate(common: &mut Common);
}

/// Raven `ZoneTailFromHeader`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:45-48`
pub fn ZoneTailFromHeader(pHeader: *mut zoneHeader_t) -> *mut zoneTail_t {
    unsafe {
        ((pHeader as *mut u8).add(core::mem::size_of::<zoneHeader_t>() + (*pHeader).iSize as usize))
            as *mut zoneTail_t
    }
}

/// Raven `Zone_FreeBlock`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:342-379`
pub fn Zone_FreeBlock(common: &mut Common, pMemory: *mut zoneHeader_t) {
    unsafe {
        if (*pMemory).eTag != memtag_t::TAG_STATIC {
            // Update stats...
            //
            common.TheZone.Stats.iCount -= 1;
            common.TheZone.Stats.iCurrent -= (*pMemory).iSize;
            common.TheZone.Stats.iSizesPerTag[(*pMemory).eTag as usize] -= (*pMemory).iSize;
            common.TheZone.Stats.iCountsPerTag[(*pMemory).eTag as usize] -= 1;

            // Sanity checks...
            //
            assert!((*(*pMemory).pPrev).pNext == pMemory);
            assert!((*pMemory).pNext.is_null() || (*(*pMemory).pNext).pPrev == pMemory);

            // Unlink and free...
            //
            (*(*pMemory).pPrev).pNext = (*pMemory).pNext;
            if !(*pMemory).pNext.is_null() {
                (*(*pMemory).pNext).pPrev = (*pMemory).pPrev;
            }
            libc::free(pMemory as *mut libc::c_void);

            // DETAILED_ZONE_DEBUG_CODE is not defined in this build; the
            // debug-only double-free counter block is dropped per the
            // #ifdef's own condition.
        }
    }
}

/// Raven `Z_MemSize`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:446-449`
pub fn Z_MemSize(common: &mut Common, eTag: memtag_t) -> c_int {
    common.TheZone.Stats.iSizesPerTag[eTag as usize]
}

/// Raven `Com_TheHunkMarkHasBeenMade`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:664-671`
pub fn Com_TheHunkMarkHasBeenMade(common: &mut Common) -> qboolean {
    if common.hunk_tag == memtag_t::TAG_HUNK_MARK2 {
        return native_types::qtrue;
    }
    native_types::qfalse
}

/// Raven `Hunk_SetMark`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:706-708`
pub fn Hunk_SetMark(common: &mut Common) {
    common.hunk_tag = memtag_t::TAG_HUNK_MARK2;
}

/// Raven `Hunk_CheckMark`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:727-734`
pub fn Hunk_CheckMark(common: &mut Common) -> qboolean {
    //if( hunk_low.mark || hunk_high.mark ) {
    if common.hunk_tag != memtag_t::TAG_HUNK_MARK1 {
        return native_types::qtrue;
    }
    native_types::qfalse
}

/// Raven `Z_TagFree`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:453-477`
pub fn Z_TagFree(common: &mut Common, eTag: memtag_t) {
    let mut pMemory: *mut zoneHeader_t = common.TheZone.Header.pNext;
    unsafe {
        while !pMemory.is_null() {
            let pNext = (*pMemory).pNext;
            if eTag == memtag_t::TAG_ALL || (*pMemory).eTag == eTag {
                Zone_FreeBlock(common, pMemory);
            }
            pMemory = pNext;
        }
    }
}

/// Raven `Hunk_MemoryRemaining`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:695-697`
pub fn Hunk_MemoryRemaining(common: &mut Common) -> c_int {
    // Yeah. Whatever. We've got no size now.
    (64 * 1024 * 1024)
        - (Z_MemSize(common, memtag_t::TAG_HUNK_MARK1)
            + Z_MemSize(common, memtag_t::TAG_HUNK_MARK2))
}

/// Raven `Com_ShutdownHunkMemory`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:683-688`
pub fn Com_ShutdownHunkMemory(common: &mut Common) {
    // Er, ok. Clear it then I guess.
    Z_TagFree(common, memtag_t::TAG_HUNK_MARK1);
    Z_TagFree(common, memtag_t::TAG_HUNK_MARK2);
}

/// Raven `Hunk_ClearToMark`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:717-720`
pub fn Hunk_ClearToMark(common: &mut Common) {
    // if this is not true then no mark has been made
    assert!(common.hunk_tag == memtag_t::TAG_HUNK_MARK2);
    Z_TagFree(common, memtag_t::TAG_HUNK_MARK2);
}

/// Raven `Hunk_ClearTempMemory`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:830-832`
pub fn Hunk_ClearTempMemory(common: &mut Common) {
    Z_TagFree(common, memtag_t::TAG_TEMP_HUNKALLOC);
}

/// Raven `Z_MorphMallocTag`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:313-340`
pub fn Z_MorphMallocTag(common: &mut Common, pvAddress: *mut (), eDesiredTag: memtag_t) {
    unsafe {
        let pMemory: *mut zoneHeader_t = (pvAddress as *mut zoneHeader_t).wrapping_sub(1);

        if (*pMemory).iMagic != ZONE_MAGIC {
            crate::common::error::com_error(
                errorParm_t::ERR_FATAL,
                "Z_MorphMallocTag(): Not a valid zone header!".to_string(),
            );
            return; // won't get here
        }

        // DEC existing tag stats...
        //
        //	TheZone.Stats.iCurrent	- unchanged
        //	TheZone.Stats.iCount	- unchanged
        common.TheZone.Stats.iSizesPerTag[(*pMemory).eTag as usize] -= (*pMemory).iSize;
        common.TheZone.Stats.iCountsPerTag[(*pMemory).eTag as usize] -= 1;

        // morph...
        //
        (*pMemory).eTag = eDesiredTag;

        // INC new tag stats...
        //
        //	TheZone.Stats.iCurrent	- unchanged
        //	TheZone.Stats.iCount	- unchanged
        common.TheZone.Stats.iSizesPerTag[(*pMemory).eTag as usize] += (*pMemory).iSize;
        common.TheZone.Stats.iCountsPerTag[(*pMemory).eTag as usize] += 1;
    }
}

/// Raven `Z_Size`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:383-399`
pub fn Z_Size(pvAddress: *mut ()) -> c_int {
    unsafe {
        let pMemory: *mut zoneHeader_t = (pvAddress as *mut zoneHeader_t).wrapping_sub(1);

        if (*pMemory).eTag == memtag_t::TAG_STATIC {
            return 0; // kind of
        }

        if (*pMemory).iMagic != ZONE_MAGIC {
            crate::common::error::com_error(
                errorParm_t::ERR_FATAL,
                "Z_Size(): Not a valid zone header!".to_string(),
            );
            return 0; // won't get here
        }

        (*pMemory).iSize
    }
}

/// Raven `Com_ShutdownZoneMemory`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:558-573`
pub fn Com_ShutdownZoneMemory(common: &mut Common) {
    //	Com_Printf("Shutting down zone memory .....\n");

    unsafe {
        Cmd_RemoveCommand(common, "zone_stats");
        Cmd_RemoveCommand(common, "zone_details");
    }

    if common.TheZone.Stats.iCount != 0 {
        // §E0382: `common` moves into the call as the first arg, so the
        // fields it needs must be read into locals before the call.
        let count = common.TheZone.Stats.iCount;
        let current = common.TheZone.Stats.iCurrent;
        unsafe {
            Com_Printf(
                common,
                &format!("Automatically freeing {count} blocks making up {current} bytes\n"),
            );
        }
        Z_TagFree(common, memtag_t::TAG_ALL);

        assert!(common.TheZone.Stats.iCount == 0);
        assert!(common.TheZone.Stats.iCurrent == 0);
    }
}

/// Raven `Com_InitZoneMemory`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:577-594`
pub fn Com_InitZoneMemory(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    // §19: Raven's `memset(&TheZone, 0, sizeof(TheZone))` zero-inits the
    // whole struct before setting the header magic; the Rust field is
    // Default-zeroed by the Engine aggregate (STATE-D13), so the memset
    // itself is a no-op here — only the magic write is transcribed.
    common.TheZone = Default::default();
    common.TheZone.Header.iMagic = ZONE_MAGIC;

    //#ifdef _DEBUG
    //	com_validateZone = Cvar_Get("com_validateZone", "1", 0);
    //#else
    unsafe {
        common.com_validateZone = Cvar_Get(common, cm, rm, host, "com_validateZone", "0", 0);
    }
    //#endif

    // PORT-NOTE(z-stats-details-body-gap): `Z_Stats_f`/`Z_Details_f`
    // (z_memman_pc.cpp:510-524) have no transcribed body anywhere in the
    // tree — genuine logic port, not a mechanical call-site fix; escalated
    // as missing symbols for the finisher.
    // Source: `oracle/codemp/qcommon/z_memman_pc.cpp:588-589`
    //TODO: Port Z_Stats_f
    //TODO: Port Z_Details_f
    unsafe {
        Cmd_AddCommand(common, cm, rm, host, "zone_stats", Z_Stats_f as *const ());
        Cmd_AddCommand(
            common,
            cm,
            rm,
            host,
            "zone_details",
            Z_Details_f as *const (),
        );
    }

    // #ifdef _DEBUG: zone_memrecovertest is a debug-only command; this is a
    // release build, so the block is dropped per its own guard.
}

/// Raven `Com_TouchMemory`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:636-660`
pub fn Com_TouchMemory(common: &mut Common) {
    //	int		start, end;
    let mut sum: i32;

    //	start = Sys_Milliseconds();
    unsafe {
        Z_Validate(common);
    }

    sum = 0;

    let mut pMemory: *mut zoneHeader_t = common.TheZone.Header.pNext;
    unsafe {
        while !pMemory.is_null() {
            let pMem = (pMemory.add(1)) as *const u8;
            let j = (*pMemory).iSize >> 2;
            let mut i = 0;
            while i < j {
                sum += *((pMem as *const i32).add(i as usize));
                i += 64;
            }

            pMemory = (*pMemory).pNext;
        }
    }
    let _ = sum;

    //	end = Sys_Milliseconds();
    //	Com_Printf( "Com_TouchMemory: %i msec\n", end - start );
}

/// Raven `Hunk_Alloc`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:791-793`
pub fn Hunk_Alloc(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    size: c_int,
    preference: ha_pref,
) -> *mut () {
    let _ = preference;
    // §E0382: `common` moves into the call as the first arg, so its
    // `hunk_tag` field must be read into a local before the call, not inline.
    let hunk_tag = common.hunk_tag;
    unsafe { Z_Malloc(common, cm, rm, host, size, hunk_tag, native_types::qtrue, 4) }
}

/// Raven `Hunk_FreeTempMemory`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:815-818`
pub fn Hunk_FreeTempMemory(common: &mut Common, buf: *mut ()) {
    unsafe {
        Z_Free(common, buf);
    }
}

/// Raven `Hunk_Clear`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:752-782`
pub fn Hunk_Clear(
    common: &mut Common,
    sv: &mut Server,
    rm: &mut RenderModels,
    g2: &mut Ghoul2System,
    host: &mut dyn EngineHost,
) {
    // DEDICATED: this is the dedicated-server build (§20/§C10 precedent —
    // the engine-fork-discovery rulings treat DEDICATED as the live
    // configuration), so the `#ifndef DEDICATED` client blocks
    // (CL_ShutdownCGame/CL_ShutdownUI/CIN_CloseAllVideos) are dropped.
    unsafe {
        SV_ShutdownGameProgs(common, sv);
    }

    common.hunk_tag = memtag_t::TAG_HUNK_MARK1;
    Z_TagFree(common, memtag_t::TAG_HUNK_MARK1);
    Z_TagFree(common, memtag_t::TAG_HUNK_MARK2);

    unsafe {
        R_HunkClearCrap(rm, host);
    }

    //	Com_Printf( "Hunk_Clear: reset the hunk ok\n" );
    VM_Clear(common);

    // See if any ghoul2 stuff was leaked, at this point it should be all
    // cleaned up.
    // _FULL_G2_LEAK_CHECKING is not defined in this build; the leak-check
    // assert/report block is dropped per its own guard.
    let _ = g2;
}

/// Raven `Com_InitHunkMemory`.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:678-681`
pub fn Com_InitHunkMemory(
    common: &mut Common,
    sv: &mut Server,
    rm: &mut RenderModels,
    g2: &mut Ghoul2System,
    host: &mut dyn EngineHost,
) {
    common.hunk_tag = memtag_t::TAG_HUNK_MARK1;
    Hunk_Clear(common, sv, rm, g2, host);
}

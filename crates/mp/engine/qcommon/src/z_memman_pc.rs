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

use core::ffi::{c_char, c_int};

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
use crate::cm_load::RenderModels;
#[allow(dead_code)]
pub(crate) struct Ghoul2System;
#[allow(dead_code)]
use crate::cmd_pc::Server;

// Real `Com_Printf` imported (sweep: extern forward-declares eliminated).
use crate::common::com_printf as Com_Printf;
// Genuinely-unported callees referenced at their canonical future homes.
// `Z_Malloc`/`Z_Free`/`Z_Validate` are this file's own subject (z_memman_pc.cpp)
// with no transcribed body yet — left bare at their home; reported.
// `SV_ShutdownGameProgs` (sv_game) and `R_HunkClearCrap` (tr_init) live across
// the server/renderer cycle seam — left bare; reported.
use crate::cmd::{Cmd_AddCommand, Cmd_RemoveCommand};
use crate::cvar_fns::Cvar_Get;

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

// static mem blocks to reduce a lot of small zone overhead
//
// Raven declares these `#pragma pack(1)` (`z_memman_pc.cpp:122-137`). The packed
// layout is behaviorally inert here: static blocks are never linked into
// `TheZone` (`pNext`/`pPrev` stay null) so `Z_Validate` never walks them, and
// `Z_Free`/`Z_Size` early-out on `TAG_STATIC` before reading the tail — so only
// `Header` and (for `CopyString`) `mem` are ever read, both at
// `sizeof(zoneHeader_t)`-aligned offsets. Natural `#[repr(C)]` is used to avoid
// Rust misaligned-access UB on the static's own address.

/// Raven `StaticZeroMem_t`.
/// Type definition source: `oracle/codemp/qcommon/z_memman_pc.cpp:124-129`
#[repr(C)]
struct StaticZeroMem_t {
    Header: zoneHeader_t,
    Tail: zoneTail_t,
}

/// Raven `StaticMem_t`.
/// Type definition source: `oracle/codemp/qcommon/z_memman_pc.cpp:131-136`
#[repr(C)]
struct StaticMem_t {
    Header: zoneHeader_t,
    mem: [u8; 2],
    Tail: zoneTail_t,
}

/// Sync wrapper for the read-only static zone blocks (raw-pointer list links
/// keep the payload `!Sync`; the blocks are never mutated in this build).
struct ZoneStatic<T>(T);
unsafe impl<T> Sync for ZoneStatic<T> {}

/// Const constructor for a `StaticMem_t` block (`TAG_STATIC`, size 2).
const fn new_static_mem(mem: [u8; 2]) -> StaticMem_t {
    StaticMem_t {
        Header: zoneHeader_t {
            iMagic: ZONE_MAGIC,
            eTag: memtag_t::TAG_STATIC,
            iSize: 2,
            pNext: core::ptr::null_mut(),
            pPrev: core::ptr::null_mut(),
        },
        mem,
        Tail: zoneTail_t { iMagic: ZONE_MAGIC },
    }
}

/// Raven `gZeroMalloc` — the block handed back for a zero-byte `Z_Malloc`.
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:139-140`
static gZeroMalloc: ZoneStatic<StaticZeroMem_t> = ZoneStatic(StaticZeroMem_t {
    Header: zoneHeader_t {
        iMagic: ZONE_MAGIC,
        eTag: memtag_t::TAG_STATIC,
        iSize: 0,
        pNext: core::ptr::null_mut(),
        pPrev: core::ptr::null_mut(),
    },
    Tail: zoneTail_t { iMagic: ZONE_MAGIC },
});

/// Raven `gEmptyString` — `CopyString("")`'s static empty-string block.
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:141-142`
static gEmptyString: ZoneStatic<StaticMem_t> = ZoneStatic(new_static_mem([b'\0', b'\0']));

/// Raven `gNumberString[]` — `CopyString`'s static single-digit blocks.
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:143-154`
static gNumberString: ZoneStatic<[StaticMem_t; 10]> = ZoneStatic([
    new_static_mem([b'0', b'\0']),
    new_static_mem([b'1', b'\0']),
    new_static_mem([b'2', b'\0']),
    new_static_mem([b'3', b'\0']),
    new_static_mem([b'4', b'\0']),
    new_static_mem([b'5', b'\0']),
    new_static_mem([b'6', b'\0']),
    new_static_mem([b'7', b'\0']),
    new_static_mem([b'8', b'\0']),
    new_static_mem([b'9', b'\0']),
]);

/// Raven `Z_Validate` — walks the zone list checking every header/tail magic.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:82-116`
pub fn Z_Validate(common: &Common) {
    if common.com_validateZone.is_null() || unsafe { (*common.com_validateZone).integer } == 0 {
        return;
    }

    let mut pMemory: *mut zoneHeader_t = common.TheZone.Header.pNext;
    unsafe {
        while !pMemory.is_null() {
            if (*pMemory).iMagic != ZONE_MAGIC {
                crate::common::error::com_error(
                    errorParm_t::ERR_FATAL,
                    "Z_Validate(): Corrupt zone header!".to_string(),
                );
            }

            if (*ZoneTailFromHeader(pMemory)).iMagic != ZONE_MAGIC {
                crate::common::error::com_error(
                    errorParm_t::ERR_FATAL,
                    "Z_Validate(): Corrupt zone tail!".to_string(),
                );
            }

            pMemory = (*pMemory).pNext;
        }
    }
}

/// Raven `Z_Malloc` — the zone allocator: wraps `malloc`/`calloc` with a magic
/// header/tail and per-tag stats, and on failure dumps non-vital caches and
/// retries before giving up.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:157-308`
pub fn Z_Malloc(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    iSize: c_int,
    eTag: memtag_t,
    bZeroit: qboolean,
    iUnusedAlign: c_int,
) -> *mut () {
    let _ = iUnusedAlign;
    // The file-scope `gbMemFreeupOccured` flag only gated the `#ifdef _WIN32`
    // `Sleep` hint and the `#ifdef _DEBUG` recover-test, both dropped in this
    // build, so it has no reader here and is omitted.

    if iSize == 0 {
        let pMemory = &gZeroMalloc.0 as *const StaticZeroMem_t as *mut zoneHeader_t;
        return unsafe { pMemory.add(1) as *mut () };
    }

    // Add in tracking info
    //
    let iRealSize: c_int = iSize
        + core::mem::size_of::<zoneHeader_t>() as c_int
        + core::mem::size_of::<zoneTail_t>() as c_int;

    // Allocate a chunk...
    //
    let mut pMemory: *mut zoneHeader_t = core::ptr::null_mut();
    while pMemory.is_null() {
        // #ifdef _WIN32: the `Sleep(1000)` de-fragmentation hint is dropped for
        // this build config.

        pMemory = if bZeroit != 0 {
            unsafe { libc::calloc(iRealSize as usize, 1) as *mut zoneHeader_t }
        } else {
            unsafe { libc::malloc(iRealSize as usize) as *mut zoneHeader_t }
        };
        if pMemory.is_null() {
            // new bit, if we fail to malloc memory, try dumping some of the
            // cached stuff that's non-vital and try again...

            // ditch the BSP cache...
            //
            if crate::cm_load::CM_DeleteCachedMap(common, cm, native_types::qfalse) != 0 {
                continue; // we've just ditched a whole load of memory, so try again with the malloc
            }

            // ditch any sounds not used on this level...
            //
            if SND_RegisterAudio_LevelLoadEnd(host, native_types::qtrue) != 0 {
                continue; // we've dropped at least one sound, so try again with the malloc
            }

            // #ifndef DEDICATED: the `RE_RegisterImages_LevelLoadEnd` image-cache
            // dump is dropped for the dedicated-server build config.

            // ditch the model-binaries cache...  (must be getting desperate here!)
            //
            if RE_RegisterModels_LevelLoadEnd(rm, host, native_types::qtrue) != 0 {
                continue;
            }

            // as a last panic measure, dump all the audio memory, but not if
            // we're in the audio loader (which is annoying, but I'm not sure how
            // to ensure we're not dumping any memory needed by the sound
            // currently being loaded if that was the case)...
            //
            if gbInsideLoadSound == 0 {
                let mut iBytesFreed = SND_FreeOldestSound(host);
                if iBytesFreed != 0 {
                    loop {
                        let iTheseBytesFreed = SND_FreeOldestSound(host);
                        if iTheseBytesFreed == 0 {
                            break;
                        }
                        iBytesFreed += iTheseBytesFreed;
                        if iBytesFreed >= iRealSize {
                            break; // early opt-out since we've managed to recover enough
                        }
                    }
                    continue;
                }
            }

            // sigh, dunno what else to try, I guess we'll have to give up and
            // report this as an out-of-mem error...
            //
            // findlabel:  "recovermem"
            Com_Printf(
                common,
                &format!(
                    "^1Z_Malloc(): Failed to alloc {} bytes (TAG_{}) !!!!!\n",
                    iSize, psTagStrings[eTag as usize]
                ),
            );
            Z_Details_f(common);
            crate::common::error::com_error(
                errorParm_t::ERR_FATAL,
                format!(
                    "(Repeat): Z_Malloc(): Failed to alloc {} bytes (TAG_{}) !!!!!\n",
                    iSize, psTagStrings[eTag as usize]
                ),
            );
        }
    }

    unsafe {
        // Link in
        (*pMemory).iMagic = ZONE_MAGIC;
        (*pMemory).eTag = eTag;
        (*pMemory).iSize = iSize;
        (*pMemory).pNext = common.TheZone.Header.pNext;
        common.TheZone.Header.pNext = pMemory;
        if !(*pMemory).pNext.is_null() {
            (*(*pMemory).pNext).pPrev = pMemory;
        }
        (*pMemory).pPrev = &mut common.TheZone.Header as *mut zoneHeader_t;
        //
        // add tail...
        //
        (*ZoneTailFromHeader(pMemory)).iMagic = ZONE_MAGIC;

        // Update stats...
        //
        common.TheZone.Stats.iCurrent += iSize;
        common.TheZone.Stats.iCount += 1;
        common.TheZone.Stats.iSizesPerTag[eTag as usize] += iSize;
        common.TheZone.Stats.iCountsPerTag[eTag as usize] += 1;

        if common.TheZone.Stats.iCurrent > common.TheZone.Stats.iPeak {
            common.TheZone.Stats.iPeak = common.TheZone.Stats.iCurrent;
        }
    }

    Z_Validate(common); // check for corruption

    unsafe { pMemory.add(1) as *mut () }
}

/// Raven `Z_Free` — validates and frees a zone block (no-op on null / static).
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:404-443`
pub fn Z_Free(common: &mut Common, pvAddress: *mut ()) {
    // I've put this in as a safety measure because of some bits of #ifdef BSPC
    // stuff -Ste.
    if pvAddress.is_null() {
        return;
    }

    unsafe {
        let pMemory: *mut zoneHeader_t = (pvAddress as *mut zoneHeader_t).wrapping_sub(1);

        if (*pMemory).eTag == memtag_t::TAG_STATIC {
            return;
        }

        // DETAILED_ZONE_DEBUG_CODE is not defined in this build; the debug-only
        // already-freed check is dropped per the #ifdef.

        if (*pMemory).iMagic != ZONE_MAGIC {
            crate::common::error::com_error(
                errorParm_t::ERR_FATAL,
                "Z_Free(): Corrupt zone header!".to_string(),
            );
        }
        if (*ZoneTailFromHeader(pMemory)).iMagic != ZONE_MAGIC {
            crate::common::error::com_error(
                errorParm_t::ERR_FATAL,
                "Z_Free(): Corrupt zone tail!".to_string(),
            );
        }

        Zone_FreeBlock(common, pMemory);
    }
}

/// Raven `S_Malloc` — `TAG_SMALL` `Z_Malloc` shorthand.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:480-482`
pub fn S_Malloc(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    iSize: c_int,
) -> *mut () {
    Z_Malloc(
        common,
        cm,
        rm,
        host,
        iSize,
        memtag_t::TAG_SMALL,
        native_types::qfalse,
        4,
    )
}

/// Raven `CopyString` — duplicates a string into the zone, returning shared
/// static blocks for `""` and single digits.
///
/// Raven NOTE: never write over the memory `CopyString` returns because memory
/// from a memstatic_t might be returned.
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:607-622`
pub fn CopyString(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    in_: *const c_char,
) -> *mut c_char {
    unsafe {
        if *in_ == 0 {
            return (&gEmptyString.0 as *const StaticMem_t as *const u8)
                .add(core::mem::size_of::<zoneHeader_t>()) as *mut c_char;
        } else if *in_.add(1) == 0 && *in_ >= b'0' as c_char && *in_ <= b'9' as c_char {
            let idx = (*in_ - b'0' as c_char) as usize;
            return (&gNumberString.0[idx] as *const StaticMem_t as *const u8)
                .add(core::mem::size_of::<zoneHeader_t>()) as *mut c_char;
        }

        let out = S_Malloc(common, cm, rm, host, (libc::strlen(in_) + 1) as c_int) as *mut c_char;
        libc::strcpy(out, in_);
        out
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
        Cmd_RemoveCommand(common, c"zone_stats".as_ptr());
        Cmd_RemoveCommand(common, c"zone_details".as_ptr());
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
    // whole struct before setting the header magic. `zone_t` has no `Default`
    // impl, so the memset is transcribed directly via `zeroed()` rather than
    // relying on aggregate zero-init.
    common.TheZone = unsafe { core::mem::zeroed() };
    common.TheZone.Header.iMagic = ZONE_MAGIC;

    //#ifdef _DEBUG
    //	com_validateZone = Cvar_Get("com_validateZone", "1", 0);
    //#else
    unsafe {
        common.com_validateZone = Cvar_Get(common, cm, rm, host, c"com_validateZone".as_ptr(), c"0".as_ptr(), 0);
    }
    //#endif

    unsafe {
        Cmd_AddCommand(common, cm, rm, host, c"zone_stats".as_ptr(), Z_Stats_f as *const ());
        Cmd_AddCommand(
            common,
            cm,
            rm,
            host,
            c"zone_details".as_ptr(),
            Z_Details_f as *const (),
        );
    }

    // #ifdef _DEBUG: zone_memrecovertest is a debug-only command; this is a
    // release build, so the block is dropped per its own guard.
}

/// Raven `psTagStrings` — memory-tag display names (`TAG_` prefix stripped),
/// indexed by `memtag_t`; the trailing entry is `TAG_COUNT` becoming a string.
/// Source: `oracle/codemp/qcommon/tags.h` (via `z_memman_pc.cpp:14-17`)
const psTagStrings: [&str; memtag_t::TAG_COUNT as usize + 1] = [
    "ALL",
    "BOTLIB",
    "CLIENTS",
    "BOTGAME",
    "DOWNLOAD",
    "GENERAL",
    "CLIPBOARD",
    "SND_MP3STREAMHDR",
    "SND_DYNAMICMUSIC",
    "BSP_DISKIMAGE",
    "VM",
    "SPECIAL_MEM_TEST",
    "HUNK_MARK1",
    "HUNK_MARK2",
    "EVENT",
    "FILESYS",
    "GHOUL2",
    "GHOUL2_GORE",
    "LISTFILES",
    "AMBIENTSET",
    "STATIC",
    "SMALL",
    "MODEL_MD3",
    "MODEL_GLM",
    "MODEL_GLA",
    "ICARUS",
    "ICARUS2",
    "ICARUS3",
    "ICARUS4",
    "ICARUS5",
    "SHADERTEXT",
    "SND_RAWDATA",
    "TEMP_WORKSPACE",
    "TEMP_PNG",
    "TEXTPOOL",
    "IMAGE_T",
    "INFLATE",
    "DEFLATE",
    "BSP",
    "GRIDMESH",
    "POINTCACHE",
    "TERRAIN",
    "R_TERRAIN",
    "RESAMPLE",
    "CM_TERRAIN",
    "CM_TERRAIN_TEMP",
    "TEMP_IMAGE",
    "VM_ALLOCATED",
    "TEMP_HUNKALLOC",
    "COUNT",
];

/// Raven `Z_Stats_f` — the `zone_stats` console command; prints the zone's
/// current and peak usage.
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:510-524`
fn Z_Stats_f(common: &mut Common) {
    let iCurrent = common.TheZone.Stats.iCurrent;
    let iCount = common.TheZone.Stats.iCount;
    let iPeak = common.TheZone.Stats.iPeak;

    crate::common::com_printf(
        common,
        &format!(
            "\nThe zone is using {} bytes ({:.2}MB) in {} memory blocks\n",
            iCurrent,
            iCurrent as f32 / 1024.0 / 1024.0,
            iCount
        ),
    );

    crate::common::com_printf(
        common,
        &format!(
            "The zone peaked at {} bytes ({:.2}MB)\n",
            iPeak,
            iPeak as f32 / 1024.0 / 1024.0
        ),
    );
}

/// Raven `Z_Details_f` — the `zone_details` console command; prints per-tag
/// byte/block totals, then the summary (`Z_Stats_f`).
///
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:526-553`
fn Z_Details_f(common: &mut Common) {
    crate::common::com_printf(
        common,
        "---------------------------------------------------------------------------\n",
    );
    crate::common::com_printf(common, &format!("{:>20} {:>9}\n", "Zone Tag", "Bytes"));
    crate::common::com_printf(common, &format!("{:>20} {:>9}\n", "--------", "-----"));
    for i in 0..memtag_t::TAG_COUNT as usize {
        let iThisCount = common.TheZone.Stats.iCountsPerTag[i];
        let iThisSize = common.TheZone.Stats.iSizesPerTag[i];

        if iThisCount != 0 {
            let fSize = iThisSize as f32 / 1024.0 / 1024.0;
            let iSize = fSize as c_int;
            let iRemainder = (100.0 * (fSize - fSize.floor())) as c_int;
            crate::common::com_printf(
                common,
                &format!(
                    "{:>20} {:>9} ({:>2}.{:02}MB) in {:>6} blocks ({:>9} average)\n",
                    psTagStrings[i],
                    iThisSize,
                    iSize,
                    iRemainder,
                    iThisCount,
                    iThisSize / iThisCount
                ),
            );
        }
    }
    crate::common::com_printf(
        common,
        "---------------------------------------------------------------------------\n",
    );

    Z_Stats_f(common);
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

//! `cl_cin.cpp` — the RoQ cinematic decoder and playback pipeline.
//!
//! Raven's sibling `cl_cin_console.cpp` is dropped, not ported (porting-rules
//! §20). It is the Xbox console replacement for this file: it plays Bink video
//! through a `BinkVideo` object out of `d:\base\video\`, and only
//! `oracle/codemp/x_exe/x_exe.vcproj` compiles it. The PC MP link set
//! (`oracle/codemp/jk2mp.vcproj:627`) takes `cl_cin.cpp` alone.
//!
//! Source: `oracle/codemp/client/cl_cin.cpp`

use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort};
use std::sync::Arc;

use mp_qshared::shared::cbuf_exec::cbufExec_t;
use mp_qshared::shared::cin_flags::{CIN_HOLD, CIN_LOOP, CIN_SHADER, CIN_SILENT, CIN_SYSTEM};
use mp_qshared::shared::cinematic_status::{e_status, FMV_EOF, FMV_IDLE, FMV_LOOPED, FMV_PLAY};
use mp_qshared::shared::connstate::connstate_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::fs_origin::fsOrigin_t;
use mp_qshared::shared::limits::MAX_OSPATH;
use mp_qshared::shared::q_string::COM_DefaultExtension;
use mp_qshared::shared::swap::LittleLong;
use native_types::{byte, qboolean, qfalse, qtrue};

use mp_engine_qcommon::cmd_common::{Cbuf_ExecuteText, Cmd_Argv};
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::common_fns::{Com_DPrintf, Com_Memcpy, Com_Memset};
use mp_engine_qcommon::cvar_fns::{Cvar_Set, Cvar_VariableString};
use mp_engine_qcommon::files_common::{FS_FCloseFile, FS_FOpenFileRead, FS_Read};
use mp_engine_qcommon::files_pc::FS_Seek;
use mp_engine_qcommon::sys_engine::Sys_StreamedRead;
use mp_engine_qcommon::timing::sys_milliseconds;
use mp_engine_qcommon::vm_fns::VM_Call;
use mp_engine_qcommon::z_memman_pc::{Hunk_AllocateTempMemory, Hunk_FreeTempMemory};
use native_platform::{Sys_BeginStreamedFile, Sys_EndStreamedFile};
use mp_renderer::hook_install::re_from_view;
use mp_renderer::tr_backend::{RE_StretchRaw, RE_UploadCinematic};
use native_string::latin1_to_string;
use native_string::q_string::Q_stricmp;

use mp_abi::ui::exports::MpUiExport;
use mp_abi::ui::public::ui_menu_command_t::UIMENU_NONE;
use mp_engine_icarus::q3_interface::S_COLOR_RED;
use mp_qshared::shared::screen::{SCREEN_HEIGHT, SCREEN_WIDTH};

use crate::cin::cin_consts::{
    DEFAULT_CIN_HEIGHT, DEFAULT_CIN_WIDTH, MAXSIZE, MAX_VIDEO_HANDLES, MINSIZE, ROQ_CODEBOOK,
    ROQ_PACKET, ROQ_QUAD_HANG, ROQ_QUAD_INFO, ROQ_QUAD_JPEG, ROQ_QUAD_VQ, ZA_SOUND_MONO,
    ZA_SOUND_STEREO,
};
use crate::cin::vq_blitter::VqBlitter;
use crate::cl_console::Con_Close;
use crate::client_host::Client;
use crate::client_host::snd_from_view;
use crate::snd_dma::{S_RawSamples, S_StopAllSounds, S_Update};

// PORT-NOTE(deps): `mp_qshared::shared::cbuf_exec::cbufExec_t`, `native_string`,
// and `mp_engine_icarus` (for `S_COLOR_RED`) are referenced per the packet
// rosetta but are not yet crate dependencies of `mp_engine_client`; escalate
// rather than silently adding them.

// PORT-NOTE(deps): `byte` is imported from `native_types` (already a crate
// dependency, same alias for `c_uchar`). The packet rosetta named
// `crates/mp/game/src/prelude.rs`, but `mp_game` is not a dependency of
// `mp_engine_client`; escalate the rosetta row rather than adding the
// dependency here.

/// Raven `RoQInterrupt`'s local `short sbuf[32768]` audio scratch.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:925`
const SBUF_LEN: usize = 32768;

/// Raven's `VQ2TO4(a,b,c,d)` macro — expands one 2x2 codebook pair into the 4x4
/// and 8x8 books. The four pointer arguments are read-and-advance in the macro,
/// so each one crosses as `&mut *mut c_uint` here.
///
/// # Safety
/// `a` and `b` must each address at least two entries, `c` four, and `d`
/// sixteen, exactly as the oracle's `vq2`/`vq4`/`vq8` cursors do.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:562-583`
#[allow(non_snake_case)]
unsafe fn VQ2TO4(
    a: &mut *mut c_uint,
    b: &mut *mut c_uint,
    c: &mut *mut c_uint,
    d: &mut *mut c_uint,
) {
    let a0 = *(*a);
    let a1 = *(*a).add(1);
    let b0 = *(*b);
    let b1 = *(*b).add(1);

    // The 4x4 book takes the four source texels once.
    for v in [a0, a1, b0, b1] {
        **c = v;
        *c = (*c).add(1);
    }
    // The 8x8 book takes the oracle's exact 16-texel order.
    for v in [
        a0, a0, a1, a1, b0, b0, b1, b1, a0, a0, a1, a1, b0, b0, b1, b1,
    ] {
        **d = v;
        *d = (*d).add(1);
    }

    *a = (*a).add(2);
    *b = (*b).add(2);
}

/// Reads a `cin_cache::fileName` buffer as a Latin-1 `String`, stopping at the
/// first NUL. The seam keeps the buffer as `[c_char; MAX_OSPATH]`, so the two
/// print and compare sites need this narrowing.
fn file_name_to_string(buf: &[c_char]) -> String {
    // SAFETY: the reinterpretation is byte-for-byte; `c_char` and `u8` share a layout.
    let bytes: &[u8] = unsafe { core::slice::from_raw_parts(buf.as_ptr() as *const u8, buf.len()) };
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    latin1_to_string(&bytes[..len])
}

/// Raven `CIN_HandleForVideo`.
///
/// This returns the first free slot in `cinTable`, or panics through
/// `com_error` when the table is full.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:137-147`
pub fn CIN_HandleForVideo(cl: &mut Client) -> c_int {
    for i in 0..MAX_VIDEO_HANDLES {
        if cl.cinTable[i as usize].fileName[0] == 0 {
            return i as c_int;
        }
    }
    com_error(
        errorParm_t::ERR_DROP,
        "CIN_HandleForVideo: none free".to_string(),
    );
    #[allow(unreachable_code)]
    -1
}

/// Raven `RllSetupTable`.
///
/// This fills the RoQ audio delta square table used by the RLL decoders.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:159-167`
pub fn RllSetupTable(cl: &mut Client) {
    for z in 0..128i32 {
        cl.cin.sqrTable[z as usize] = (z * z) as i16;
        cl.cin.sqrTable[(z + 128) as usize] = -cl.cin.sqrTable[z as usize];
    }
}

/// Raven `RllDecodeMonoToMono`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:184-198`
pub fn RllDecodeMonoToMono(
    cl: &mut Client,
    from: *mut c_uchar,
    to: *mut c_short,
    size: c_uint,
    signedOutput: c_char,
    flag: c_ushort,
) -> c_long {
    let mut prev: c_int = if signedOutput != 0 {
        flag as c_int - 0x8000
    } else {
        flag as c_int
    };

    for z in 0..size {
        unsafe {
            let sample = *from.add(z as usize) as usize;
            let val = (prev + cl.cin.sqrTable[sample] as c_int) as c_short;
            *to.add(z as usize) = val;
            prev = val as c_int;
        }
    }
    size as c_long //*sizeof(short));
}

/// Raven `RllDecodeMonoToStereo`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:215-231`
pub fn RllDecodeMonoToStereo(
    cl: &mut Client,
    from: *mut c_uchar,
    to: *mut c_short,
    size: c_uint,
    signedOutput: c_char,
    flag: c_ushort,
) -> c_long {
    let mut prev: c_int = if signedOutput != 0 {
        flag as c_int - 0x8000
    } else {
        flag as c_int
    };

    let mut z = 0;
    while z < size {
        unsafe {
            let sample = *from.add(z as usize) as usize;
            prev = (prev + cl.cin.sqrTable[sample] as c_int) as c_short as c_int;
            *to.add((z * 2) as usize) = prev as c_short;
            *to.add((z * 2 + 1) as usize) = prev as c_short;
        }
        z += 1;
    }

    size as c_long // * 2 * sizeof(short));
}

/// Raven `RllDecodeStereoToStereo`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:247-269`
pub fn RllDecodeStereoToStereo(
    cl: &mut Client,
    from: *mut c_uchar,
    to: *mut c_short,
    size: c_uint,
    signedOutput: c_char,
    flag: c_ushort,
) -> c_long {
    let mut zz = from;
    let (mut prevL, mut prevR): (c_int, c_int) = if signedOutput != 0 {
        (
            (flag as c_int & 0xff00) - 0x8000,
            ((flag as c_int & 0x00ff) << 8) - 0x8000,
        )
    } else {
        (flag as c_int & 0xff00, (flag as c_int & 0x00ff) << 8)
    };

    let mut z = 0;
    while z < size {
        unsafe {
            let sl = *zz as usize;
            zz = zz.add(1);
            prevL = (prevL + cl.cin.sqrTable[sl] as c_int) as c_short as c_int;
            let sr = *zz as usize;
            zz = zz.add(1);
            prevR = (prevR + cl.cin.sqrTable[sr] as c_int) as c_short as c_int;
            *to.add(z as usize) = prevL as c_short;
            *to.add((z + 1) as usize) = prevR as c_short;
        }
        z += 2;
    }

    (size >> 1) as c_long //*sizeof(short));
}

/// Raven `RllDecodeStereoToMono`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:285-305`
pub fn RllDecodeStereoToMono(
    cl: &mut Client,
    from: *mut c_uchar,
    to: *mut c_short,
    size: c_uint,
    signedOutput: c_char,
    flag: c_ushort,
) -> c_long {
    let (mut prevL, mut prevR): (c_int, c_int) = if signedOutput != 0 {
        (
            (flag as c_int & 0xff00) - 0x8000,
            ((flag as c_int & 0x00ff) << 8) - 0x8000,
        )
    } else {
        (flag as c_int & 0xff00, (flag as c_int & 0x00ff) << 8)
    };

    for z in 0..size {
        unsafe {
            let l = *from.add((z * 2) as usize) as usize;
            let r = *from.add((z * 2 + 1) as usize) as usize;
            prevL += cl.cin.sqrTable[l] as c_int;
            prevR += cl.cin.sqrTable[r] as c_int;
            *to.add(z as usize) = ((prevL + prevR) / 2) as c_short;
        }
    }

    size as c_long
}

/// Copies one `f64` lane, the way Raven's `ddst[i] = dsrc[i]` does.
///
/// Every blitter below reaches an `f64` lane that is only 4-byte aligned. The
/// source is a `c_ushort` codebook, and `RoQPrepMcomp` builds motion deltas as
/// `(x + xoff - 8) * 4`, so an odd horizontal vector lands the row on a 4-byte
/// boundary. x86 tolerates that and Raven relies on it; in Rust an aligned
/// dereference there is undefined behavior and aborts a debug build.
#[inline]
unsafe fn copy_lane(dst: *mut f64, src: *mut f64, i: usize) {
    dst.add(i).write_unaligned(src.add(i).read_unaligned());
}

/// Raven `move8_32`.
///
/// This copies eight 4-byte pixels per scanline across `spl` scanlines, eight
/// doubles at a time, treating the pixel row as `f64` lanes.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:315-339`
pub fn move8_32(src: *mut byte, dst: *mut byte, spl: c_int) {
    unsafe {
        let mut dsrc = src as *mut f64;
        let mut ddst = dst as *mut f64;
        let dspl = (spl >> 3) as isize;

        for _ in 0..7 {
            for i in 0..4 {
                copy_lane(ddst, dsrc, i);
            }
            dsrc = dsrc.offset(dspl);
            ddst = ddst.offset(dspl);
        }
        for i in 0..4 {
            copy_lane(ddst, dsrc, i);
        }
    }
}

/// Raven `move4_32`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:349-365`
pub fn move4_32(src: *mut byte, dst: *mut byte, spl: c_int) {
    unsafe {
        let mut dsrc = src as *mut f64;
        let mut ddst = dst as *mut f64;
        let dspl = (spl >> 3) as isize;

        for _ in 0..3 {
            for i in 0..2 {
                copy_lane(ddst, dsrc, i);
            }
            dsrc = dsrc.offset(dspl);
            ddst = ddst.offset(dspl);
        }
        for i in 0..2 {
            copy_lane(ddst, dsrc, i);
        }
    }
}

/// Raven `blit8_32`.
///
/// Unlike `move8_32`, the source stride is fixed at 4 doubles (`dsrc += 4`)
/// while the destination stride is `spl`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:375-399`
pub fn blit8_32(src: *mut byte, dst: *mut byte, spl: c_int) {
    unsafe {
        let mut dsrc = src as *mut f64;
        let mut ddst = dst as *mut f64;
        let dspl = (spl >> 3) as isize;

        for _ in 0..7 {
            for i in 0..4 {
                copy_lane(ddst, dsrc, i);
            }
            dsrc = dsrc.offset(4);
            ddst = ddst.offset(dspl);
        }
        for i in 0..4 {
            copy_lane(ddst, dsrc, i);
        }
    }
}

/// Raven `blit4_32`.
///
/// Raven's `movs` is `#define movs double`, so the pointer casts are `f64`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:409-425`
pub fn blit4_32(src: *mut byte, dst: *mut byte, spl: c_int) {
    unsafe {
        let mut dsrc = src as *mut f64;
        let mut ddst = dst as *mut f64;
        let dspl = (spl >> 3) as isize;

        for _ in 0..3 {
            for i in 0..2 {
                copy_lane(ddst, dsrc, i);
            }
            dsrc = dsrc.offset(2);
            ddst = ddst.offset(dspl);
        }
        for i in 0..2 {
            copy_lane(ddst, dsrc, i);
        }
    }
}

/// Raven `blit2_32`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:435-446`
pub fn blit2_32(src: *mut byte, dst: *mut byte, spl: c_int) {
    unsafe {
        let dsrc = src as *mut f64;
        let ddst = dst as *mut f64;
        let dspl = (spl >> 3) as isize;

        copy_lane(ddst, dsrc, 0);
        ddst.offset(dspl).write_unaligned(dsrc.add(1).read_unaligned());
    }
}

/// Raven `ROQ_GenYUVTables`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:542-560`
pub fn ROQ_GenYUVTables(cl: &mut Client) {
    let t_ub = (1.772_00f32 / 2.0f32) * (1i32 << 6) as f32 + 0.5f32;
    let t_vr = (1.402_00f32 / 2.0f32) * (1i32 << 6) as f32 + 0.5f32;
    let t_ug = (0.344_14f32 / 2.0f32) * (1i32 << 6) as f32 + 0.5f32;
    let t_vg = (0.714_14f32 / 2.0f32) * (1i32 << 6) as f32 + 0.5f32;

    for i in 0..256i64 {
        let x = (2 * i - 255) as f32;

        cl.ROQ_UB_tab[i as usize] = ((t_ub * x) + (1i32 << 5) as f32) as c_long;
        cl.ROQ_VR_tab[i as usize] = ((t_vr * x) + (1i32 << 5) as f32) as c_long;
        cl.ROQ_UG_tab[i as usize] = (-t_ug * x) as c_long;
        cl.ROQ_VG_tab[i as usize] = ((-t_vg * x) + (1i32 << 5) as f32) as c_long;
        cl.ROQ_YY_tab[i as usize] = ((i << 6) | (i >> 2)) as c_long;
    }
}

/// Raven `yuv_to_rgb24`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:625-637`
pub fn yuv_to_rgb24(cl: &mut Client, y: c_long, u: c_long, v: c_long) -> c_uint {
    let yy = cl.ROQ_YY_tab[y as usize];

    let mut r = (yy + cl.ROQ_VR_tab[v as usize]) >> 6;
    let mut g = (yy + cl.ROQ_UG_tab[u as usize] + cl.ROQ_VG_tab[v as usize]) >> 6;
    let mut b = (yy + cl.ROQ_UB_tab[u as usize]) >> 6;

    if r < 0 {
        r = 0;
    }
    if g < 0 {
        g = 0;
    }
    if b < 0 {
        b = 0;
    }
    if r > 255 {
        r = 255;
    }
    if g > 255 {
        g = 255;
    }
    if b > 255 {
        b = 255;
    }

    LittleLong(r as c_int | (g as c_int) << 8 | (b as c_int) << 16 | 255 << 24) as c_uint
}

/// Raven `recurseQuad`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:703-733`
pub fn recurseQuad(
    cl: &mut Client,
    startX: c_long,
    startY: c_long,
    quadSize: c_long,
    xOff: c_long,
    yOff: c_long,
) {
    let handle = cl.currentHandle as usize;
    let offset = cl.cinTable[handle].screenDelta;

    let (lowx, lowy): (c_long, c_long) = (0, 0);
    let mut bigx = cl.cinTable[handle].xsize as c_long;
    let mut bigy = cl.cinTable[handle].ysize as c_long;

    if bigx > cl.cinTable[handle].CIN_WIDTH as c_long {
        bigx = cl.cinTable[handle].CIN_WIDTH as c_long;
    }
    if bigy > cl.cinTable[handle].CIN_HEIGHT as c_long {
        bigy = cl.cinTable[handle].CIN_HEIGHT as c_long;
    }

    if startX >= lowx
        && (startX + quadSize) <= bigx
        && (startY + quadSize) <= bigy
        && startY >= lowy
        && quadSize <= MAXSIZE
    {
        let useY = startY;
        let byte_off = (useY + ((cl.cinTable[handle].CIN_HEIGHT as c_long - bigy) >> 1) + yOff)
            * cl.cinTable[handle].samplesPerLine
            + (startX + xOff) * 4;

        // §19: Raven bounds the quad against the stream's own `xsize`/`ysize`,
        // never against `linbuf` or `qStatus`, so a header claiming more than
        // 512x512 walks off both. A quad that does not fit is skipped.
        let linbuf_len = cl.cin.linbuf.len() as c_long;
        let in_bounds = byte_off >= 0
            && byte_off < linbuf_len
            && byte_off + offset >= 0
            && byte_off + offset < linbuf_len
            && (cl.cinTable[handle].onQuad as usize) < cl.cin.qStatus[0].len();

        if in_bounds {
            unsafe {
                let scroff = cl.cin.linbuf.as_mut_ptr().offset(byte_off as isize);
                let onquad = cl.cinTable[handle].onQuad as usize;
                cl.cin.qStatus[0][onquad] = scroff;
                cl.cin.qStatus[1][onquad] = scroff.offset(offset as isize);
                cl.cinTable[handle].onQuad += 1;
            }
        }
    }

    if quadSize != MINSIZE {
        let quadSize = quadSize >> 1;
        recurseQuad(cl, startX, startY, quadSize, xOff, yOff);
        recurseQuad(cl, startX + quadSize, startY, quadSize, xOff, yOff);
        recurseQuad(cl, startX, startY + quadSize, quadSize, xOff, yOff);
        recurseQuad(
            cl,
            startX + quadSize,
            startY + quadSize,
            quadSize,
            xOff,
            yOff,
        );
    }
}

/// Raven `readQuadInfo`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:788-827`
pub fn readQuadInfo(common: &mut Common, cl: &mut Client, qData: *mut byte) {
    if cl.currentHandle < 0 {
        return;
    }
    let handle = cl.currentHandle as usize;

    unsafe {
        cl.cinTable[handle].xsize = (*qData.add(0) as c_uint) + (*qData.add(1) as c_uint) * 256;
        cl.cinTable[handle].ysize = (*qData.add(2) as c_uint) + (*qData.add(3) as c_uint) * 256;
        cl.cinTable[handle].maxsize = (*qData.add(4) as c_uint) + (*qData.add(5) as c_uint) * 256;
        cl.cinTable[handle].minsize = (*qData.add(6) as c_uint) + (*qData.add(7) as c_uint) * 256;
    }

    cl.cinTable[handle].CIN_HEIGHT = cl.cinTable[handle].ysize as c_int;
    cl.cinTable[handle].CIN_WIDTH = cl.cinTable[handle].xsize as c_int;

    cl.cinTable[handle].samplesPerLine = cl.cinTable[handle].CIN_WIDTH as c_long * 4;
    cl.cinTable[handle].screenDelta =
        cl.cinTable[handle].CIN_HEIGHT as c_long * cl.cinTable[handle].samplesPerLine;

    cl.cinTable[handle].VQ0 = cl.cinTable[handle].VQNormal;
    cl.cinTable[handle].VQ1 = cl.cinTable[handle].VQBuffer;

    let linbuf = cl.cin.linbuf.as_mut_ptr() as usize;
    cl.cinTable[handle].t[0] =
        (0 - linbuf as c_long) + linbuf as c_long + cl.cinTable[handle].screenDelta;
    cl.cinTable[handle].t[1] =
        (0 - (linbuf as c_long + cl.cinTable[handle].screenDelta)) + linbuf as c_long;

    cl.cinTable[handle].drawX = cl.cinTable[handle].CIN_WIDTH as c_long;
    cl.cinTable[handle].drawY = cl.cinTable[handle].CIN_HEIGHT as c_long;
    // jic the card sucks
    if cl.cls.glconfig.maxTextureSize <= 256 {
        if cl.cinTable[handle].drawX > 256 {
            cl.cinTable[handle].drawX = 256;
        }
        if cl.cinTable[handle].drawY > 256 {
            cl.cinTable[handle].drawY = 256;
        }
        if cl.cinTable[handle].CIN_WIDTH != 256 || cl.cinTable[handle].CIN_HEIGHT != 256 {
            com_printf(
                common,
                "HACK: approxmimating cinematic for Rage Pro or Voodoo\n",
            );
        }
    }
    // §19: the MACOS_X arm reassigned `drawX` twice and never touched `drawY` -
    // a Raven typo. Mac is not a target platform here, so this arm never runs.
}

/// Raven `RoQPrepMcomp`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:837-856`
pub fn RoQPrepMcomp(cl: &mut Client, xoff: c_long, yoff: c_long) {
    let handle = cl.currentHandle as usize;
    let mut i = cl.cinTable[handle].samplesPerLine;
    let mut j: c_long = 4;
    if cl.cinTable[handle].xsize == cl.cinTable[handle].ysize * 4 {
        j += j;
        i += i;
    }

    // Raven declares `long i, j, x, y, temp, temp2` (`cl_cin.cpp:839`), so the
    // two counters are `c_long`. An `i64` literal here matches `c_long` on a
    // 64-bit target only, and breaks the ILP32 build.
    for y in 0..16 as c_long {
        let temp2 = (y + yoff - 8) * i;
        for x in 0..16 as c_long {
            let temp = (x + xoff - 8) * j;
            cl.cin.mcomp[((x * 16) + y) as usize] =
                (cl.cinTable[handle].normalBuffer0 - (temp2 + temp)) as c_uint;
        }
    }
}

/// Raven `RoQ_init`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1062-1083`
pub fn RoQ_init(common: &mut Common, cl: &mut Client) {
    let handle = cl.currentHandle as usize;
    let start =
        (sys_milliseconds(common) as f32 * common.cvar(common.com_timescale).value) as c_uint;
    cl.cinTable[handle].startTime = start;
    cl.cinTable[handle].lastTime = start;

    cl.cinTable[handle].RoQPlayed = 24;

    // get frame rate
    cl.cinTable[handle].roqFPS = cl.cin.file[6] as c_long + cl.cin.file[7] as c_long * 256;

    if cl.cinTable[handle].roqFPS == 0 {
        cl.cinTable[handle].roqFPS = 30;
    }

    cl.cinTable[handle].numQuads = -1;

    cl.cinTable[handle].roq_id = cl.cin.file[8] as c_uint + cl.cin.file[9] as c_uint * 256;
    cl.cinTable[handle].RoQFrameSize = cl.cin.file[10] as c_uint
        + cl.cin.file[11] as c_uint * 256
        + cl.cin.file[12] as c_uint * 65536;
    cl.cinTable[handle].roq_flags = cl.cin.file[14] as c_long + cl.cin.file[15] as c_long * 256;

    if cl.cinTable[handle].RoQFrameSize > 65536 || cl.cinTable[handle].RoQFrameSize == 0 {
        return;
    }
}

/// Raven `RoQShutdown`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1093-1127`
pub fn RoQShutdown(view: &mut EngineHostView, cl: &mut Client) {
    let handle = cl.currentHandle as usize;

    if cl.cinTable[handle].buf.is_null() {
        return;
    }

    if cl.cinTable[handle].status == FMV_IDLE {
        return;
    }
    Com_DPrintf(view.common, "finished cinematic\n");
    cl.cinTable[handle].status = FMV_IDLE;

    if cl.cinTable[handle].iFile != 0 {
        Sys_EndStreamedFile(cl.cinTable[handle].iFile);
        FS_FCloseFile(view.common, cl.cinTable[handle].iFile);
        cl.cinTable[handle].iFile = 0;
    }

    if cl.cinTable[handle].alterGameState != qfalse {
        cl.cls.state = connstate_t::CA_DISCONNECTED;
        // we can't just do a vstr nextmap, because
        // if we are aborting the intro cinematic with
        // a devmap command, nextmap would be valid by
        // the time it was referenced
        let s = Cvar_VariableString(view.common, "nextmap").to_string();
        if !s.is_empty() {
            Cbuf_ExecuteText(view, cbufExec_t::EXEC_APPEND as c_int, &format!("{}\n", s));
            Cvar_Set(view, "nextmap", "");
        }
        cl.CL_handle = -1;
    }
    cl.cinTable[handle].fileName[0] = 0;
    cl.currentHandle = -1;
}

/// Raven `CIN_SetExtents`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1324-1331`
pub fn CIN_SetExtents(cl: &mut Client, handle: c_int, x: c_int, y: c_int, w: c_int, h: c_int) {
    if handle < 0
        || handle >= MAX_VIDEO_HANDLES as c_int
        || cl.cinTable[handle as usize].status == FMV_EOF
    {
        return;
    }
    cl.cinTable[handle as usize].xpos = x;
    cl.cinTable[handle as usize].ypos = y;
    cl.cinTable[handle as usize].width = w;
    cl.cinTable[handle as usize].height = h;
    cl.cinTable[handle as usize].dirty = qtrue;
}

/// Raven `CIN_SetLooping`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1333-1336`
pub fn CIN_SetLooping(cl: &mut Client, handle: c_int, r#loop: qboolean) {
    if handle < 0
        || handle >= MAX_VIDEO_HANDLES as c_int
        || cl.cinTable[handle as usize].status == FMV_EOF
    {
        return;
    }
    cl.cinTable[handle as usize].looping = r#loop;
}

/// Raven `CIN_DrawCinematic`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1344-1415`
pub fn CIN_DrawCinematic(view: &mut EngineHostView, cl: &mut Client, handle: c_int) {
    if handle < 0
        || handle >= MAX_VIDEO_HANDLES as c_int
        || cl.cinTable[handle as usize].status == FMV_EOF
    {
        return;
    }

    if cl.cinTable[handle as usize].buf.is_null() {
        return;
    }

    let x = cl.cinTable[handle as usize].xpos as f32;
    let y = cl.cinTable[handle as usize].ypos as f32;
    let w = cl.cinTable[handle as usize].width as f32;
    let h = cl.cinTable[handle as usize].height as f32;

    if cl.cinTable[handle as usize].dirty != qfalse
        && (cl.cinTable[handle as usize].CIN_WIDTH as c_long
            != cl.cinTable[handle as usize].drawX
            || cl.cinTable[handle as usize].CIN_HEIGHT as c_long
                != cl.cinTable[handle as usize].drawY)
    {
        let xm = cl.cinTable[handle as usize].CIN_WIDTH / 256;
        let ym = cl.cinTable[handle as usize].CIN_HEIGHT / 256;
        let mut ll = 8;
        if cl.cinTable[handle as usize].CIN_WIDTH == 512 {
            ll = 9;
        }

        unsafe {
            let buf3 = cl.cinTable[handle as usize].buf as *mut c_int;
            let buf2 = Hunk_AllocateTempMemory(view, 256 * 256 * 4) as *mut c_int;

            if xm == 2 && ym == 2 {
                let bc3 = buf3 as *mut byte;
                let mut bc2 = buf2 as *mut byte;
                for iy in 0..256i32 {
                    let iiy = iy << 12;
                    let mut ix = 0;
                    while ix < 2048 {
                        for ic in ix..(ix + 4) {
                            *bc2 = ((*bc3.offset((iiy + ic) as isize) as c_int
                                + *bc3.offset((iiy + 4 + ic) as isize) as c_int
                                + *bc3.offset((iiy + 2048 + ic) as isize) as c_int
                                + *bc3.offset((iiy + 2048 + 4 + ic) as isize) as c_int)
                                >> 2) as byte;
                            bc2 = bc2.add(1);
                        }
                        ix += 8;
                    }
                }
            } else if xm == 2 && ym == 1 {
                let bc3 = buf3 as *mut byte;
                let mut bc2 = buf2 as *mut byte;
                for iy in 0..256i32 {
                    let iiy = iy << 11;
                    let mut ix = 0;
                    while ix < 2048 {
                        for ic in ix..(ix + 4) {
                            *bc2 = ((*bc3.offset((iiy + ic) as isize) as c_int
                                + *bc3.offset((iiy + 4 + ic) as isize) as c_int)
                                >> 1) as byte;
                            bc2 = bc2.add(1);
                        }
                        ix += 8;
                    }
                }
            } else {
                for iy in 0..256i32 {
                    for ix in 0..256i32 {
                        *buf2.offset(((iy << 8) + ix) as isize) =
                            *buf3.offset((((iy * ym) << ll) + (ix * xm)) as isize);
                    }
                }
            }
            // SAFETY: `buf2` is the temp-memory image this block just filled.
            let data = core::slice::from_raw_parts(buf2 as *const u8, 256 * 256 * 4);
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let re = re_from_view(view);
            RE_StretchRaw(
                &mut re.frame,
                &mut re.frame_data,
                Arc::make_mut(&mut re.sim.published),
                &mut re.img_state,
                &re.cvars,
                view.common,
                x as i32,
                y as i32,
                w as i32,
                h as i32,
                256,
                256,
                data,
                handle,
                true,
            );
            cl.cinTable[handle as usize].dirty = qfalse;
            Hunk_FreeTempMemory(view.common, buf2 as *mut ());
            return;
        }
    }

    let cols = cl.cinTable[handle as usize].drawX as i32;
    let rows = cl.cinTable[handle as usize].drawY as i32;
    let dirty = cl.cinTable[handle as usize].dirty != qfalse;
    // SAFETY: `buf` is the decoder's own surface, `cols * rows` RGBA texels.
    let data = unsafe {
        core::slice::from_raw_parts(
            cl.cinTable[handle as usize].buf as *const u8,
            (cols * rows * 4) as usize,
        )
    };
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let re = unsafe { re_from_view(view) };
    RE_StretchRaw(
        &mut re.frame,
        &mut re.frame_data,
        Arc::make_mut(&mut re.sim.published),
        &mut re.img_state,
        &re.cvars,
        view.common,
        x as i32,
        y as i32,
        w as i32,
        h as i32,
        cols,
        rows,
        data,
        handle,
        dirty,
    );
    cl.cinTable[handle as usize].dirty = qfalse;
}

/// Raven `CIN_UploadCinematic`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1472-1493`
pub fn CIN_UploadCinematic(view: &mut EngineHostView, cl: &mut Client, handle: c_int) {
    if handle >= 0 && handle < MAX_VIDEO_HANDLES as c_int {
        if cl.cinTable[handle as usize].buf.is_null() {
            return;
        }
        if cl.cinTable[handle as usize].playonwalls <= 0
            && cl.cinTable[handle as usize].dirty != qfalse
        {
            if cl.cinTable[handle as usize].playonwalls == 0 {
                cl.cinTable[handle as usize].playonwalls = -1;
            } else if cl.cinTable[handle as usize].playonwalls == -1 {
                cl.cinTable[handle as usize].playonwalls = -2;
            } else {
                cl.cinTable[handle as usize].dirty = qfalse;
            }
        }
        let cols = cl.cinTable[handle as usize].drawX as i32;
        let rows = cl.cinTable[handle as usize].drawY as i32;
        let dirty = cl.cinTable[handle as usize].dirty != qfalse;
        // SAFETY: `buf` is the decoder's own surface, `cols * rows` RGBA texels.
        let data = unsafe {
            core::slice::from_raw_parts(
                cl.cinTable[handle as usize].buf as *const u8,
                (cols * rows * 4) as usize,
            )
        };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_UploadCinematic(
            Arc::make_mut(&mut re.sim.published),
            &mut re.img_state,
            cols,
            rows,
            data,
            handle,
            dirty,
        );

        if view.common.cvar(cl.cl_inGameVideo).integer == 0
            && cl.cinTable[handle as usize].playonwalls == 1
        {
            cl.cinTable[handle as usize].playonwalls -= 1;
        }
    }
}

/// Raven `blitVQQuad32fs`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:456-532`
pub fn blitVQQuad32fs(cl: &mut Client, status: *mut *mut byte, data: *mut c_uchar) {
    let handle = cl.currentHandle as usize;
    let mut newd: c_ushort = 0;
    let mut celdata: c_ushort = 0;
    let mut index: usize = 0;
    let mut data = data;

    let spl = cl.cinTable[handle].samplesPerLine as c_int;

    unsafe {
        loop {
            if newd == 0 {
                newd = 7;
                celdata = *data.add(0) as c_ushort + *data.add(1) as c_ushort * 256;
                data = data.add(2);
            } else {
                newd -= 1;
            }

            let code = celdata & 0xc000;
            celdata <<= 2;

            match code {
                0x8000 => {
                    // vq code
                    let cell = *data as usize;
                    blit8_32(
                        (&mut cl.vq8[cell * 128] as *mut c_ushort) as *mut byte,
                        *status.add(index),
                        spl,
                    );
                    data = data.add(1);
                    index += 5;
                }
                0xc000 => {
                    // drop
                    index += 1; // skip 8x8
                    for _ in 0..4 {
                        if newd == 0 {
                            newd = 7;
                            celdata = *data.add(0) as c_ushort + *data.add(1) as c_ushort * 256;
                            data = data.add(2);
                        } else {
                            newd -= 1;
                        }

                        let code2 = celdata & 0xc000;
                        celdata <<= 2;

                        match code2 {
                            0x8000 => {
                                // 4x4 vq code
                                let cell = *data as usize;
                                blit4_32(
                                    (&mut cl.vq4[cell * 32] as *mut c_ushort) as *mut byte,
                                    *status.add(index),
                                    spl,
                                );
                                data = data.add(1);
                            }
                            0xc000 => {
                                // 2x2 vq code
                                let mut cell = *data as usize;
                                blit2_32(
                                    (&mut cl.vq2[cell * 8] as *mut c_ushort) as *mut byte,
                                    *status.add(index),
                                    spl,
                                );
                                data = data.add(1);
                                cell = *data as usize;
                                blit2_32(
                                    (&mut cl.vq2[cell * 8] as *mut c_ushort) as *mut byte,
                                    status.add(index).read().add(8),
                                    spl,
                                );
                                data = data.add(1);
                                cell = *data as usize;
                                blit2_32(
                                    (&mut cl.vq2[cell * 8] as *mut c_ushort) as *mut byte,
                                    status.add(index).read().add((spl * 2) as usize),
                                    spl,
                                );
                                data = data.add(1);
                                cell = *data as usize;
                                blit2_32(
                                    (&mut cl.vq2[cell * 8] as *mut c_ushort) as *mut byte,
                                    status.add(index).read().add((spl * 2 + 8) as usize),
                                    spl,
                                );
                                data = data.add(1);
                            }
                            0x4000 => {
                                // motion compensation
                                let mc = *data as usize;
                                let src = status
                                    .add(index)
                                    .read()
                                    .offset(cl.cin.mcomp[mc] as i32 as isize);
                                move4_32(src, *status.add(index), spl);
                                data = data.add(1);
                            }
                            _ => {}
                        }
                        index += 1;
                    }
                }
                0x4000 => {
                    // motion compensation
                    let mc = *data as usize;
                    // `mcomp` holds a signed byte delta that Raven stores in an
                    // `unsigned int` and adds to a 32-bit `byte *`; the `i32`
                    // step restores that wrap on a 64-bit pointer.
                    let mc_delta = cl.cin.mcomp[mc] as i32 as isize;
                    let src = status.add(index).read().offset(mc_delta);
                    move8_32(src, *status.add(index), spl);
                    data = data.add(1);
                    index += 5;
                }
                0x0000 => {
                    index += 5;
                }
                _ => {}
            }

            if status.add(index).read().is_null() {
                break;
            }
        }
    }
}

/// Raven `decodeCodeBook`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:648-693`
pub fn decodeCodeBook(cl: &mut Client, input: *mut byte, roq_flags: c_ushort) {
    let (mut two, mut four): (c_long, c_long);
    if roq_flags == 0 {
        two = 256;
        four = 256;
    } else {
        two = (roq_flags >> 8) as c_long;
        if two == 0 {
            two = 256;
        }
        four = (roq_flags & 0xff) as c_long;
    }

    four *= 2;

    let mut input = input;

    unsafe {
        let mut ibptr = cl.vq2.as_mut_ptr() as *mut c_uint;
        for _ in 0..two {
            let y0 = *input as c_long;
            input = input.add(1);
            let y1 = *input as c_long;
            input = input.add(1);
            let y2 = *input as c_long;
            input = input.add(1);
            let y3 = *input as c_long;
            input = input.add(1);
            let cr = *input as c_long;
            input = input.add(1);
            let cb = *input as c_long;
            input = input.add(1);
            *ibptr = yuv_to_rgb24(cl, y0, cr, cb) as c_uint;
            ibptr = ibptr.add(1);
            *ibptr = yuv_to_rgb24(cl, y1, cr, cb) as c_uint;
            ibptr = ibptr.add(1);
            *ibptr = yuv_to_rgb24(cl, y2, cr, cb) as c_uint;
            ibptr = ibptr.add(1);
            *ibptr = yuv_to_rgb24(cl, y3, cr, cb) as c_uint;
            ibptr = ibptr.add(1);
        }

        let mut icptr = cl.vq4.as_mut_ptr() as *mut c_uint;
        let mut idptr = cl.vq8.as_mut_ptr() as *mut c_uint;

        for _ in 0..four {
            let a_off = *input as usize;
            input = input.add(1);
            let b_off = *input as usize;
            input = input.add(1);
            let mut iaptr = (cl.vq2.as_mut_ptr() as *mut c_uint).add(a_off * 4);
            let mut ibptr2 = (cl.vq2.as_mut_ptr() as *mut c_uint).add(b_off * 4);
            for _ in 0..2 {
                VQ2TO4(&mut iaptr, &mut ibptr2, &mut icptr, &mut idptr);
            }
        }
    }
}

/// Raven `setupQuad`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:744-778`
pub fn setupQuad(cl: &mut Client, xOff: c_long, yOff: c_long) {
    let handle = cl.currentHandle as usize;
    if xOff == cl.cin.oldXOff
        && yOff == cl.cin.oldYOff
        && cl.cinTable[handle].ysize == cl.cin.oldysize
        && cl.cinTable[handle].xsize == cl.cin.oldxsize
    {
        return;
    }

    cl.cin.oldXOff = xOff;
    cl.cin.oldYOff = yOff;
    cl.cin.oldysize = cl.cinTable[handle].ysize;
    cl.cin.oldxsize = cl.cinTable[handle].xsize;

    // Raven computes `numQuadCels` twice. The first pass (over `CIN_WIDTH` /
    // `CIN_HEIGHT`) is overwritten before any read, so only the `xsize`/`ysize`
    // pass below survives.
    let mut numQuadCels =
        (cl.cinTable[handle].xsize as c_long * cl.cinTable[handle].ysize as c_long) / 16;
    numQuadCels += numQuadCels / 4;
    numQuadCels += 64; // for overflow

    cl.cinTable[handle].onQuad = 0;

    let mut y: c_long = 0;
    while y < cl.cinTable[handle].ysize as c_long {
        let mut x: c_long = 0;
        while x < cl.cinTable[handle].xsize as c_long {
            recurseQuad(cl, x, y, 16, xOff, yOff);
            x += 16;
        }
        y += 16;
    }

    let temp: *mut byte = std::ptr::null_mut();

    // §19: the terminator block takes the same `qStatus` bound `recurseQuad`
    // now applies, because `numQuadCels` scales with the stream's own header.
    let last = numQuadCels.min(cl.cin.qStatus[0].len() as c_long);
    for i in (numQuadCels - 64).max(0)..last {
        cl.cin.qStatus[0][i as usize] = temp; // eoq
        cl.cin.qStatus[1][i as usize] = temp; // eoq
    }
}

/// Raven `RoQReset`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:899-912`
pub fn RoQReset(view: &mut EngineHostView, cl: &mut Client) {
    if cl.currentHandle < 0 {
        return;
    }
    let handle = cl.currentHandle as usize;

    if cl.cinTable[handle].iFile != 0 {
        Sys_EndStreamedFile(cl.cinTable[handle].iFile);
        FS_Seek(
            view,
            cl.cinTable[handle].iFile,
            0,
            fsOrigin_t::FS_SEEK_SET as c_int,
        );
        FS_Read(
            view.common,
            cl.cin.file.as_mut_ptr() as *mut (),
            16,
            cl.cinTable[handle].iFile,
        );
        RoQ_init(view.common, cl);
        // let the background thread start reading ahead
        Sys_BeginStreamedFile(cl.cinTable[handle].iFile, 0x10000);
        cl.cinTable[handle].status = FMV_LOOPED;
    }
}

/// Raven `CIN_StopCinematic`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1134-1154`
pub fn CIN_StopCinematic(view: &mut EngineHostView, cl: &mut Client, handle: c_int) -> e_status {
    if handle < 0
        || handle >= MAX_VIDEO_HANDLES as c_int
        || cl.cinTable[handle as usize].status == FMV_EOF
    {
        return FMV_EOF;
    }
    cl.currentHandle = handle;

    Com_DPrintf(
        view.common,
        &format!(
            "trFMV::stop(), closing {}\n",
            file_name_to_string(&cl.cinTable[cl.currentHandle as usize].fileName)
        ),
    );

    if cl.cinTable[cl.currentHandle as usize].buf.is_null() {
        return FMV_EOF;
    }

    if cl.cinTable[cl.currentHandle as usize].alterGameState != qfalse {
        if cl.cls.state != connstate_t::CA_CINEMATIC {
            return cl.cinTable[cl.currentHandle as usize].status;
        }
    }
    cl.cinTable[cl.currentHandle as usize].status = FMV_EOF;
    RoQShutdown(view, cl);

    FMV_EOF
}

/// Raven `SCR_DrawCinematic`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1451-1455`
pub fn SCR_DrawCinematic(view: &mut EngineHostView, cl: &mut Client) {
    if cl.CL_handle >= 0 && cl.CL_handle < MAX_VIDEO_HANDLES as c_int {
        CIN_DrawCinematic(view, cl, cl.CL_handle);
    }
}

/// Raven `CIN_CloseAllVideos`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:126-134`
pub fn CIN_CloseAllVideos(view: &mut EngineHostView, cl: &mut Client) {
    for i in 0..MAX_VIDEO_HANDLES {
        if cl.cinTable[i as usize].fileName[0] != 0 {
            CIN_StopCinematic(view, cl, i as c_int);
        }
    }
}

/// Raven `initRoQ`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:866-874`
pub fn initRoQ(cl: &mut Client) {
    if cl.currentHandle < 0 {
        return;
    }
    let handle = cl.currentHandle as usize;

    cl.cinTable[handle].VQNormal = VqBlitter::BlitVQQuad32fs;
    cl.cinTable[handle].VQBuffer = VqBlitter::BlitVQQuad32fs;
    ROQ_GenYUVTables(cl);
    RllSetupTable(cl);
}

/// Raven `CIN_PlayCinematic`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1237-1322`
pub fn CIN_PlayCinematic(
    view: &mut EngineHostView,
    cl: &mut Client,
    arg: *const c_char,
    x: c_int,
    y: c_int,
    w: c_int,
    h: c_int,
    systemBits: c_int,
) -> c_int {
    let arg_str = unsafe { std::ffi::CStr::from_ptr(arg).to_string_lossy().into_owned() };

    // Raven's `char name[MAX_OSPATH]`. The buffer is real, not cosmetic: both
    // `Com_sprintf` and `COM_DefaultExtension` truncate against its size, and
    // `COM_DefaultExtension` only appends when the last path component holds no
    // `.` at all.
    let mut name_buf = [0 as c_char; MAX_OSPATH];
    let composed = if arg_str.contains('/') || arg_str.contains('\\') {
        arg_str.clone()
    } else {
        format!("video/{}", arg_str)
    };
    let composed = composed.as_bytes();
    let copied = composed.len().min(name_buf.len() - 1);
    for (slot, &b) in name_buf.iter_mut().zip(&composed[..copied]) {
        *slot = b as c_char;
    }
    COM_DefaultExtension(
        name_buf.as_mut_ptr(),
        name_buf.len() as c_int,
        c".roq".as_ptr(),
    );
    let name = file_name_to_string(&name_buf);

    if systemBits & CIN_SYSTEM == 0 {
        for i in 0..MAX_VIDEO_HANDLES {
            if file_name_to_string(&cl.cinTable[i as usize].fileName) == name
            {
                return i as c_int;
            }
        }
    }

    Com_DPrintf(view.common, &format!("SCR_PlayCinematic( {} )\n", arg_str));

    // `cl.cin` is a `Box`, so both the pointer and the length must go through
    // the deref. Without it this clears the 8-byte box pointer, not the 2.6 MB
    // decode surface Raven clears.
    let cin_size = core::mem::size_of_val(&*cl.cin);
    Com_Memset(&mut *cl.cin as *mut _ as *mut (), 0, cin_size);
    cl.currentHandle = CIN_HandleForVideo(cl);
    let handle = cl.currentHandle as usize;

    let name_bytes = name.as_bytes();
    let copy_len = name_bytes.len().min(cl.cinTable[handle].fileName.len() - 1);
    let name_bytes_i8: Vec<c_char> = name_bytes[..copy_len]
        .iter()
        .map(|&b| b as c_char)
        .collect();
    cl.cinTable[handle].fileName[..copy_len].copy_from_slice(&name_bytes_i8);
    cl.cinTable[handle].fileName[copy_len] = 0;

    cl.cinTable[handle].ROQSize = 0;
    cl.cinTable[handle].ROQSize =
        FS_FOpenFileRead(view, &name, &mut cl.cinTable[handle].iFile, true) as c_long;

    if cl.cinTable[handle].ROQSize <= 0 {
        Com_DPrintf(
            view.common,
            &format!("cinematic failed to open {}\n", arg_str),
        );
        cl.cinTable[handle].fileName[0] = 0;
        return -1;
    }

    CIN_SetExtents(cl, cl.currentHandle, x, y, w, h);
    CIN_SetLooping(
        cl,
        cl.currentHandle,
        if systemBits & CIN_LOOP != 0 {
            qtrue
        } else {
            qfalse
        },
    );

    cl.cinTable[handle].CIN_HEIGHT = DEFAULT_CIN_HEIGHT as c_int;
    cl.cinTable[handle].CIN_WIDTH = DEFAULT_CIN_WIDTH as c_int;
    cl.cinTable[handle].holdAtEnd = if systemBits & CIN_HOLD != 0 {
        qtrue
    } else {
        qfalse
    };
    cl.cinTable[handle].alterGameState = if systemBits & CIN_SYSTEM != 0 {
        qtrue
    } else {
        qfalse
    };
    cl.cinTable[handle].playonwalls = 1;
    cl.cinTable[handle].silent = if systemBits & CIN_SILENT != 0 {
        qtrue
    } else {
        qfalse
    };
    cl.cinTable[handle].shader = if systemBits & CIN_SHADER != 0 {
        qtrue
    } else {
        qfalse
    };

    if cl.cinTable[handle].alterGameState != qfalse {
        // close the menu
        if !cl.uivm.is_null() {
            VM_Call(
                view.common,
                cl.uivm,
                MpUiExport::UI_SET_ACTIVE_MENU as c_int,
                &[UIMENU_NONE as isize],
            );
        }
    } else {
        cl.cinTable[handle].playonwalls = view.common.cvar(cl.cl_inGameVideo).integer;
    }

    initRoQ(cl);

    FS_Read(
        view.common,
        cl.cin.file.as_mut_ptr() as *mut (),
        16,
        cl.cinTable[handle].iFile,
    );

    let roq_id = cl.cin.file[0] as c_ushort + cl.cin.file[1] as c_ushort * 256;
    if roq_id == 0x1084 {
        RoQ_init(view.common, cl);
        // let the background thread start reading ahead
        Sys_BeginStreamedFile(cl.cinTable[handle].iFile, 0x10000);

        cl.cinTable[handle].status = FMV_PLAY;
        Com_DPrintf(
            view.common,
            &format!("trFMV::play(), playing {}\n", arg_str),
        );

        if cl.cinTable[handle].alterGameState != qfalse {
            cl.cls.state = connstate_t::CA_CINEMATIC;
        }

        Con_Close(view.common, cl);

        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        snd.s_rawend = snd.s_soundtime;

        return cl.currentHandle;
    }
    Com_DPrintf(view.common, "trFMV::play(), invalid RoQ ID\n");

    RoQShutdown(view, cl);
    -1
}

/// Raven `SCR_StopCinematic`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1464-1470`
pub fn SCR_StopCinematic(view: &mut EngineHostView, cl: &mut Client) {
    if cl.CL_handle >= 0 && cl.CL_handle < MAX_VIDEO_HANDLES as c_int {
        CIN_StopCinematic(view, cl, cl.CL_handle);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_StopAllSounds(view.common, snd);
    }
        cl.CL_handle = -1;
    }
}

/// Raven `RoQInterrupt`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:922-1052`
/// Borrow the decoded RoQ block as the `S_RawSamples` byte slice.
/// The RoQ decoders always write 16-bit samples, so `width` is 2 at both call sites.
fn raw_sample_bytes(sbuf: &[i16], samples: usize, width: usize, channels: usize) -> &[u8] {
    let bytes = samples * width * channels;
    // SAFETY: the caller's `sbuf` is a live `i16` array, and the decoder wrote
    // `samples * channels` of its entries; the byte view never outlives it.
    unsafe { core::slice::from_raw_parts(sbuf.as_ptr() as *const u8, bytes.min(sbuf.len() * 2)) }
}

pub fn RoQInterrupt(view: &mut EngineHostView, cl: &mut Client) {
    if cl.currentHandle < 0 {
        return;
    }
    let handle = cl.currentHandle as usize;

    let mut sbuf = [0i16; SBUF_LEN];

    Sys_StreamedRead(
        view.common,
        cl.cin.file.as_mut_ptr() as *mut (),
        (cl.cinTable[handle].RoQFrameSize + 8) as c_int,
        1,
        cl.cinTable[handle].iFile,
    );
    if cl.cinTable[handle].RoQPlayed >= cl.cinTable[handle].ROQSize {
        if cl.cinTable[handle].holdAtEnd == qfalse {
            if cl.cinTable[handle].looping != qfalse {
                RoQReset(view, cl);
            } else {
                cl.cinTable[handle].status = FMV_EOF;
            }
        } else {
            cl.cinTable[handle].status = FMV_IDLE;
        }
        return;
    }

    let mut framedata: *mut byte = cl.cin.file.as_mut_ptr();

    // new frame is ready
    'redump: loop {
        match cl.cinTable[handle].roq_id {
            ROQ_QUAD_VQ => {
                if (cl.cinTable[handle].numQuads & 1) != 0 {
                    cl.cinTable[handle].normalBuffer0 = cl.cinTable[handle].t[1];
                    RoQPrepMcomp(cl, cl.cinTable[handle].roqF0, cl.cinTable[handle].roqF1);
                    let status = cl.cin.qStatus[1].as_mut_ptr();
                    if cl.cinTable[handle].VQ1 == VqBlitter::BlitVQQuad32fs {
                        blitVQQuad32fs(cl, status, framedata);
                    }
                    unsafe {
                        cl.cinTable[handle].buf = cl
                            .cin
                            .linbuf
                            .as_mut_ptr()
                            .offset(cl.cinTable[handle].screenDelta as isize);
                    }
                } else {
                    cl.cinTable[handle].normalBuffer0 = cl.cinTable[handle].t[0];
                    RoQPrepMcomp(cl, cl.cinTable[handle].roqF0, cl.cinTable[handle].roqF1);
                    let status = cl.cin.qStatus[0].as_mut_ptr();
                    if cl.cinTable[handle].VQ0 == VqBlitter::BlitVQQuad32fs {
                        blitVQQuad32fs(cl, status, framedata);
                    }
                    cl.cinTable[handle].buf = cl.cin.linbuf.as_mut_ptr();
                }
                if cl.cinTable[handle].numQuads == 0 {
                    // first frame
                    unsafe {
                        Com_Memcpy(
                            cl.cin
                                .linbuf
                                .as_mut_ptr()
                                .offset(cl.cinTable[handle].screenDelta as isize)
                                as *mut (),
                            cl.cin.linbuf.as_ptr() as *const (),
                            (cl.cinTable[handle].samplesPerLine
                                * cl.cinTable[handle].ysize as c_long)
                                as usize,
                        );
                    }
                }
                cl.cinTable[handle].numQuads += 1;
                cl.cinTable[handle].dirty = qtrue;
            }
            ROQ_CODEBOOK => {
                decodeCodeBook(cl, framedata, cl.cinTable[handle].roq_flags as c_ushort);
            }
            ZA_SOUND_MONO => {
                if cl.cinTable[handle].silent == qfalse {
                    // §19: Raven lets a mono chunk over 16384 bytes run off the
                    // end of `sbuf`; the clamp picks the one defined behavior.
                    let size = cl.cinTable[handle].RoQFrameSize.min(SBUF_LEN as c_uint / 2);
                    let ssize = RllDecodeMonoToStereo(
                        cl,
                        framedata,
                        sbuf.as_mut_ptr(),
                        size,
                        0,
                        cl.cinTable[handle].roq_flags as c_ushort,
                    );
                    // SAFETY: view-constructor slot, single-threaded, no other live cast.
                    let snd = unsafe { snd_from_view(view) };
                    let volume = view.common.cvar(snd.s_volume).value;
                    S_RawSamples(
                        view.common,
                        snd,
                        ssize as c_int,
                        22050,
                        2,
                        1,
                        raw_sample_bytes(&sbuf, ssize as usize, 2, 1),
                        volume,
                        true,
                    );
                }
            }
            ZA_SOUND_STEREO => {
                if cl.cinTable[handle].silent == qfalse {
                    if cl.cinTable[handle].numQuads == -1 {
                        // SAFETY: view-constructor slot, single-threaded, no other live cast.
                        let snd = unsafe { snd_from_view(view) };
                        S_Update(view, snd);
                        snd.s_rawend = snd.s_soundtime;
                    }
                    // §19: same `sbuf` overrun guard as the mono arm, one
                    // sample per input byte here.
                    let size = cl.cinTable[handle].RoQFrameSize.min(SBUF_LEN as c_uint);
                    let ssize = RllDecodeStereoToStereo(
                        cl,
                        framedata,
                        sbuf.as_mut_ptr(),
                        size,
                        0,
                        cl.cinTable[handle].roq_flags as c_ushort,
                    );
                    // SAFETY: view-constructor slot, single-threaded, no other live cast.
                    let snd = unsafe { snd_from_view(view) };
                    let volume = view.common.cvar(snd.s_volume).value;
                    S_RawSamples(
                        view.common,
                        snd,
                        ssize as c_int,
                        22050,
                        2,
                        2,
                        raw_sample_bytes(&sbuf, ssize as usize, 2, 2),
                        volume,
                        true,
                    );
                }
            }
            ROQ_QUAD_INFO => {
                if cl.cinTable[handle].numQuads == -1 {
                    readQuadInfo(view.common, cl, framedata);
                    setupQuad(cl, 0, 0);
                    let start = (sys_milliseconds(view.common) as f32
                        * view.common.cvar(view.common.com_timescale).value)
                        as c_uint;
                    cl.cinTable[handle].startTime = start;
                    cl.cinTable[handle].lastTime = start;
                }
                if cl.cinTable[handle].numQuads != 1 {
                    cl.cinTable[handle].numQuads = 0;
                }
            }
            ROQ_PACKET => {
                cl.cinTable[handle].inMemory = cl.cinTable[handle].roq_flags as qboolean;
                cl.cinTable[handle].RoQFrameSize = 0; // for header
            }
            ROQ_QUAD_HANG => {
                cl.cinTable[handle].RoQFrameSize = 0;
            }
            ROQ_QUAD_JPEG => {}
            _ => {
                cl.cinTable[handle].status = FMV_EOF;
            }
        }

        // read in next frame data
        if cl.cinTable[handle].RoQPlayed >= cl.cinTable[handle].ROQSize {
            if cl.cinTable[handle].holdAtEnd == qfalse {
                if cl.cinTable[handle].looping != qfalse {
                    RoQReset(view, cl);
                } else {
                    cl.cinTable[handle].status = FMV_EOF;
                }
            } else {
                cl.cinTable[handle].status = FMV_IDLE;
            }
            return;
        }

        unsafe {
            framedata = framedata.offset(cl.cinTable[handle].RoQFrameSize as isize);
            cl.cinTable[handle].roq_id = *framedata as c_uint + *framedata.add(1) as c_uint * 256;
            cl.cinTable[handle].RoQFrameSize = *framedata.add(2) as c_uint
                + *framedata.add(3) as c_uint * 256
                + *framedata.add(4) as c_uint * 65536;
            cl.cinTable[handle].roq_flags =
                *framedata.add(6) as c_long + *framedata.add(7) as c_long * 256;
            // Raven casts both through `(char)`, so the motion vectors are
            // signed byte offsets, not the raw 0..255 header bytes.
            cl.cinTable[handle].roqF0 = *framedata.add(7) as i8 as c_long;
            cl.cinTable[handle].roqF1 = *framedata.add(6) as i8 as c_long;
        }

        if cl.cinTable[handle].RoQFrameSize > 65536 || cl.cinTable[handle].roq_id == 0x1084 {
            Com_DPrintf(view.common, "roq_size>65536||roq_id==0x1084\n");
            cl.cinTable[handle].status = FMV_EOF;
            if cl.cinTable[handle].looping != qfalse {
                RoQReset(view, cl);
            }
            return;
        }
        if cl.cinTable[handle].inMemory != qfalse && cl.cinTable[handle].status != FMV_EOF {
            cl.cinTable[handle].inMemory = cl.cinTable[handle].inMemory - 1;
            unsafe {
                framedata = framedata.add(8);
            }
            continue 'redump;
        }

        // one more frame hits the dust
        cl.cinTable[handle].RoQPlayed += cl.cinTable[handle].RoQFrameSize as c_long + 8;
        break;
    }
}

/// Raven `CIN_RunCinematic`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1165-1229`
pub fn CIN_RunCinematic(view: &mut EngineHostView, cl: &mut Client, handle: c_int) -> e_status {
    if handle < 0
        || handle >= MAX_VIDEO_HANDLES as c_int
        || cl.cinTable[handle as usize].status == FMV_EOF
    {
        return FMV_EOF;
    }

    if cl.currentHandle != handle {
        cl.currentHandle = handle;
        cl.cinTable[cl.currentHandle as usize].status = FMV_EOF;
        RoQReset(view, cl);
    }

    if cl.cinTable[handle as usize].playonwalls < -1 {
        return cl.cinTable[handle as usize].status;
    }

    cl.currentHandle = handle;
    let ch = cl.currentHandle as usize;

    if cl.cinTable[ch].alterGameState != qfalse {
        if cl.cls.state != connstate_t::CA_CINEMATIC {
            return cl.cinTable[ch].status;
        }
    }

    if cl.cinTable[ch].status == FMV_IDLE {
        return cl.cinTable[ch].status;
    }

    let thisTime = (sys_milliseconds(view.common) as f32
        * view.common.cvar(view.common.com_timescale).value) as c_uint;
    if cl.cinTable[ch].shader != qfalse
        && (thisTime as i64 - cl.cinTable[ch].lastTime as i64).abs() > 100
    {
        // Raven does this in wrapping 32-bit unsigned arithmetic, so a clock
        // that steps backwards must wrap here too, not panic in a debug build.
        cl.cinTable[ch].startTime = cl.cinTable[ch]
            .startTime
            .wrapping_add(thisTime.wrapping_sub(cl.cinTable[ch].lastTime));
    }
    cl.cinTable[ch].tfps = (((sys_milliseconds(view.common) as f32
        * view.common.cvar(view.common.com_timescale).value) as c_uint
        - cl.cinTable[ch].startTime) as c_long
        * cl.cinTable[ch].roqFPS)
        / 1000;

    let mut start = cl.cinTable[ch].startTime;
    while cl.cinTable[ch].tfps != cl.cinTable[ch].numQuads && cl.cinTable[ch].status == FMV_PLAY {
        RoQInterrupt(view, cl);
        if start != cl.cinTable[ch].startTime {
            cl.cinTable[ch].tfps = (((sys_milliseconds(view.common) as f32
                * view.common.cvar(view.common.com_timescale).value)
                as c_uint
                - cl.cinTable[ch].startTime) as c_long
                * cl.cinTable[ch].roqFPS)
                / 1000;
            start = cl.cinTable[ch].startTime;
        }
    }

    cl.cinTable[ch].lastTime = thisTime;

    if cl.cinTable[ch].status == FMV_LOOPED {
        cl.cinTable[ch].status = FMV_PLAY;
    }

    if cl.cinTable[ch].status == FMV_EOF {
        if cl.cinTable[ch].looping != qfalse {
            RoQReset(view, cl);
        } else {
            RoQShutdown(view, cl);
        }
    }

    cl.cinTable[ch].status
}

/// Raven `SCR_RunCinematic`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1457-1462`
pub fn SCR_RunCinematic(view: &mut EngineHostView, cl: &mut Client) {
    if cl.CL_handle >= 0 && cl.CL_handle < MAX_VIDEO_HANDLES as c_int {
        CIN_RunCinematic(view, cl, cl.CL_handle);
    }
}

/// Raven `CL_PlayCinematic_f`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1417-1448`
pub fn CL_PlayCinematic_f(view: &mut EngineHostView, cl: &mut Client) {
    let mut bits: c_int = CIN_SYSTEM;

    Com_DPrintf(view.common, "CL_PlayCinematic_f\n");
    if cl.cls.state == connstate_t::CA_CINEMATIC {
        SCR_StopCinematic(view, cl);
    }

    let arg = Cmd_Argv(view.common, 1).to_string();
    let s = Cmd_Argv(view.common, 2).to_string();

    if (!s.is_empty() && s.as_bytes()[0] == b'1')
        || Q_stricmp(&arg, "demoend.roq") == 0
        || Q_stricmp(&arg, "end.roq") == 0
    {
        bits |= CIN_HOLD;
    }
    if !s.is_empty() && s.as_bytes()[0] == b'2' {
        bits |= CIN_LOOP;
    }

    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_StopAllSounds(view.common, snd);
    }

    let arg_c = std::ffi::CString::new(arg.clone()).unwrap_or_default();
    cl.CL_handle = CIN_PlayCinematic(
        view,
        cl,
        arg_c.as_ptr(),
        0,
        0,
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        bits,
    );
    if cl.CL_handle >= 0 {
        loop {
            SCR_RunCinematic(view, cl);
            let ch = cl.currentHandle as usize;
            if !(cl.cinTable[ch].buf.is_null() && cl.cinTable[ch].status == FMV_PLAY) {
                break;
            }
        }
    } else {
        com_printf(
            view.common,
            &format!(
                "{}PlayCinematic(): Failed to open \"{}\"\n",
                S_COLOR_RED, arg
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{blit2_32, blit4_32, blit8_32, move4_32, move8_32};
    use native_types::byte;

    /// Raven's motion vectors put a blitter row on a 4-byte boundary whenever
    /// the horizontal displacement is odd, because `RoQPrepMcomp` scales by 4.
    /// The shared cin-oracle fixtures round every displacement to an even texel
    /// count (porting-rules §19), so this is the only cover for the odd case. An
    /// aligned `f64` dereference aborts a debug build here.
    ///
    /// Source: `oracle/codemp/client/cl_cin.cpp:315-446,852`
    #[test]
    fn blitters_take_a_four_byte_aligned_row() {
        // The backing stores are 8-byte aligned, so the `+ 4` below is the only
        // misalignment under test.
        let mut src_store = vec![0f64; 1024];
        let mut dst_store = vec![0f64; 1024];

        let src_base = src_store.as_mut_ptr() as *mut byte;
        let dst_base = dst_store.as_mut_ptr() as *mut byte;
        // SAFETY: both stores hold 8192 bytes, past every offset used below.
        let src = unsafe { src_base.add(4) };
        let dst = unsafe { dst_base.add(4) };

        // SAFETY: filling the source rows the blitters read.
        unsafe {
            for i in 0..4096 {
                *src.add(i) = (i % 251) as byte;
            }
        }

        // `spl` is Raven's `samplesPerLine`: 64 texels of 4 bytes.
        let spl = 256;
        move8_32(src, dst, spl);
        move4_32(src, dst, spl);
        blit8_32(src, dst, spl);
        blit4_32(src, dst, spl);
        blit2_32(src, dst, spl);

        // `blit2_32` ran last: lane 0 lands at `dst`, lane 1 one scanline down.
        // SAFETY: every write above landed inside `dst_store`.
        unsafe {
            for i in 0..8 {
                assert_eq!(*dst.add(i), *src.add(i), "lane 0 byte {i}");
                assert_eq!(
                    *dst.add(spl as usize + i),
                    *src.add(8 + i),
                    "lane 1 byte {i}"
                );
            }
        }
    }
}

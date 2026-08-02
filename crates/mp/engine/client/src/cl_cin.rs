//! `cl_cin.cpp` — the RoQ cinematic decoder and playback pipeline.
//!
//! Source: `oracle/codemp/client/cl_cin.cpp`

use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort};

use mp_qshared::shared::cbuf_exec::cbufExec_t;
use mp_qshared::shared::connstate::connstate_t;
use mp_qshared::shared::cinematic_status::{e_status, FMV_EOF, FMV_IDLE, FMV_LOOPED, FMV_PLAY};
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::fs_origin::fsOrigin_t;
use mp_qshared::shared::swap::LittleLong;
use native_types::{byte, qboolean, qfalse, qtrue};

use mp_engine_qcommon::cmd_common::{Cbuf_ExecuteText, Cmd_Argv};
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::common_fns::{Com_DPrintf, Com_Memcpy, Com_Memset};
use mp_engine_qcommon::cvar_fns::{Cvar_Set, Cvar_VariableString};
use mp_engine_qcommon::files_common::{FS_FCloseFile, FS_FOpenFileRead, FS_Read};
use mp_engine_qcommon::files_pc::FS_Seek;
use mp_engine_qcommon::sys_engine::Sys_StreamedRead;
use mp_engine_qcommon::vm_fns::VM_Call;
use mp_engine_qcommon::z_memman_pc::{Hunk_AllocateTempMemory, Hunk_FreeTempMemory};
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
use crate::cl_console::Con_Close;
use crate::client_host::Client;
use crate::snd_stubs::{S_RawSamples, S_StopAllSounds, S_Update};

// PORT-NOTE(deps): `mp_qshared::shared::cbuf_exec::cbufExec_t`, `native_string`,
// and `mp_engine_icarus` (for `S_COLOR_RED`) are referenced per the packet
// rosetta but are not yet crate dependencies of `mp_engine_client`; escalate
// rather than silently adding them.

// PORT-NOTE(deps): `byte` is imported from `native_types` (already a crate
// dependency, same alias for `c_uchar`). The packet rosetta named
// `crates/mp/game/src/prelude.rs`, but `mp_game` is not a dependency of
// `mp_engine_client`; escalate the rosetta row rather than adding the
// dependency here.

/// Raven `CIN_HandleForVideo`.
///
/// This returns the first free slot in `cinTable`, or panics through
/// `com_error` when the table is full.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:137-147`
pub fn CIN_HandleForVideo(cl: &mut Client) -> c_int {
    for i in 0..MAX_VIDEO_HANDLES {
        if cl.cinTable[i as usize].fileName[0] == 0 {
            return i;
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
            *ddst = *dsrc;
            *ddst.add(1) = *dsrc.add(1);
            *ddst.add(2) = *dsrc.add(2);
            *ddst.add(3) = *dsrc.add(3);
            dsrc = dsrc.offset(dspl);
            ddst = ddst.offset(dspl);
        }
        *ddst = *dsrc;
        *ddst.add(1) = *dsrc.add(1);
        *ddst.add(2) = *dsrc.add(2);
        *ddst.add(3) = *dsrc.add(3);
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
            *ddst = *dsrc;
            *ddst.add(1) = *dsrc.add(1);
            dsrc = dsrc.offset(dspl);
            ddst = ddst.offset(dspl);
        }
        *ddst = *dsrc;
        *ddst.add(1) = *dsrc.add(1);
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
            *ddst = *dsrc;
            *ddst.add(1) = *dsrc.add(1);
            *ddst.add(2) = *dsrc.add(2);
            *ddst.add(3) = *dsrc.add(3);
            dsrc = dsrc.offset(4);
            ddst = ddst.offset(dspl);
        }
        *ddst = *dsrc;
        *ddst.add(1) = *dsrc.add(1);
        *ddst.add(2) = *dsrc.add(2);
        *ddst.add(3) = *dsrc.add(3);
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
            *ddst = *dsrc;
            *ddst.add(1) = *dsrc.add(1);
            dsrc = dsrc.offset(2);
            ddst = ddst.offset(dspl);
        }
        *ddst = *dsrc;
        *ddst.add(1) = *dsrc.add(1);
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

        *ddst = *dsrc;
        *ddst.offset(dspl) = *dsrc.add(1);
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

    let (lowx, lowy) = (0, 0);
    let mut bigx = cl.cinTable[handle].xsize;
    let mut bigy = cl.cinTable[handle].ysize;

    if bigx > cl.cinTable[handle].CIN_WIDTH {
        bigx = cl.cinTable[handle].CIN_WIDTH;
    }
    if bigy > cl.cinTable[handle].CIN_HEIGHT {
        bigy = cl.cinTable[handle].CIN_HEIGHT;
    }

    if startX >= lowx
        && (startX + quadSize) <= bigx
        && (startY + quadSize) <= bigy
        && startY >= lowy
        && quadSize <= MAXSIZE
    {
        let useY = startY;
        unsafe {
            let scroff = cl.cin.linbuf.offset(
                (useY + ((cl.cinTable[handle].CIN_HEIGHT - bigy) >> 1) + yOff)
                    * cl.cinTable[handle].samplesPerLine
                    + (startX + xOff) * 4,
            );

            let onquad = cl.cinTable[handle].onQuad as usize;
            cl.cin.qStatus[0][onquad] = scroff;
            cl.cin.qStatus[1][onquad] = scroff.offset(offset as isize);
            cl.cinTable[handle].onQuad += 1;
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
pub fn readQuadInfo(cl: &mut Client, qData: *mut byte) {
    if cl.currentHandle < 0 {
        return;
    }
    let handle = cl.currentHandle as usize;

    unsafe {
        cl.cinTable[handle].xsize = *qData.add(0) as c_long + *qData.add(1) as c_long * 256;
        cl.cinTable[handle].ysize = *qData.add(2) as c_long + *qData.add(3) as c_long * 256;
        cl.cinTable[handle].maxsize = *qData.add(4) as c_long + *qData.add(5) as c_long * 256;
        cl.cinTable[handle].minsize = *qData.add(6) as c_long + *qData.add(7) as c_long * 256;
    }

    cl.cinTable[handle].CIN_HEIGHT = cl.cinTable[handle].ysize;
    cl.cinTable[handle].CIN_WIDTH = cl.cinTable[handle].xsize;

    cl.cinTable[handle].samplesPerLine = cl.cinTable[handle].CIN_WIDTH * 4;
    cl.cinTable[handle].screenDelta =
        cl.cinTable[handle].CIN_HEIGHT * cl.cinTable[handle].samplesPerLine;

    cl.cinTable[handle].VQ0 = cl.cinTable[handle].VQNormal;
    cl.cinTable[handle].VQ1 = cl.cinTable[handle].VQBuffer;

    unsafe {
        let linbuf = cl.cin.linbuf as usize;
        cl.cinTable[handle].t[0] =
            (0 - linbuf as c_long) + linbuf as c_long + cl.cinTable[handle].screenDelta;
        cl.cinTable[handle].t[1] =
            (0 - (linbuf as c_long + cl.cinTable[handle].screenDelta)) + linbuf as c_long;
    }

    cl.cinTable[handle].drawX = cl.cinTable[handle].CIN_WIDTH;
    cl.cinTable[handle].drawY = cl.cinTable[handle].CIN_HEIGHT;
    // jic the card sucks
    if cl.glConfig.maxTextureSize <= 256 {
        if cl.cinTable[handle].drawX > 256 {
            cl.cinTable[handle].drawX = 256;
        }
        if cl.cinTable[handle].drawY > 256 {
            cl.cinTable[handle].drawY = 256;
        }
        if cl.cinTable[handle].CIN_WIDTH != 256 || cl.cinTable[handle].CIN_HEIGHT != 256 {
            // PORT-NOTE(shape): `Com_Printf` needs `common: &mut Common`, but
            // `readQuadInfo`'s resolved signature carries no `common` receiver.
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

    for y in 0..16i64 {
        let temp2 = (y + yoff - 8) * i;
        for x in 0..16i64 {
            let temp = (x + xoff - 8) * j;
            cl.cin.mcomp[((x * 16) + y) as usize] =
                cl.cinTable[handle].normalBuffer0 - (temp2 + temp);
        }
    }
}

/// Raven `RoQ_init`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1062-1083`
pub fn RoQ_init(common: &mut Common, cl: &mut Client) {
    let handle = cl.currentHandle as usize;
    let start = (crate::sys_milliseconds(common) as f32 * common.com_timescale.value) as c_long;
    cl.cinTable[handle].startTime = start;
    cl.cinTable[handle].lastTime = start;

    cl.cinTable[handle].RoQPlayed = 24;

    // get frame rate
    cl.cinTable[handle].roqFPS = cl.cin.file[6] as c_long + cl.cin.file[7] as c_long * 256;

    if cl.cinTable[handle].roqFPS == 0 {
        cl.cinTable[handle].roqFPS = 30;
    }

    cl.cinTable[handle].numQuads = -1;

    cl.cinTable[handle].roq_id = cl.cin.file[8] as c_ushort + cl.cin.file[9] as c_ushort * 256;
    cl.cinTable[handle].RoQFrameSize = cl.cin.file[10] as c_long
        + cl.cin.file[11] as c_long * 256
        + cl.cin.file[12] as c_long * 65536;
    cl.cinTable[handle].roq_flags = cl.cin.file[14] as c_ushort + cl.cin.file[15] as c_ushort * 256;

    if cl.cinTable[handle].RoQFrameSize > 65536 || cl.cinTable[handle].RoQFrameSize == 0 {
        return;
    }
}

/// Raven `RoQShutdown`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1093-1127`
pub fn RoQShutdown(cl: &mut Client) {
    let handle = cl.currentHandle as usize;

    if cl.cinTable[handle].buf.is_null() {
        return;
    }

    if cl.cinTable[handle].status == FMV_IDLE {
        return;
    }
    Com_DPrintf(common, "finished cinematic\n");
    cl.cinTable[handle].status = FMV_IDLE;

    if cl.cinTable[handle].iFile != 0 {
        crate::sys_end_streamed_file(cl.cinTable[handle].iFile);
        FS_FCloseFile(common, cl.cinTable[handle].iFile);
        cl.cinTable[handle].iFile = 0;
    }

    if cl.cinTable[handle].alterGameState != qfalse {
        cl.cls.state = connstate_t::CA_DISCONNECTED;
        // we can't just do a vstr nextmap, because
        // if we are aborting the intro cinematic with
        // a devmap command, nextmap would be valid by
        // the time it was referenced
        let s = Cvar_VariableString(common, "nextmap");
        if !s.is_empty() {
            // PORT-NOTE(shape): `Cbuf_ExecuteText`/`Cvar_Set` need
            // `view: &mut EngineHostView`, but `RoQShutdown`'s resolved
            // signature carries only `cl`. Passing `common` here is a
            // placeholder the seam fixer must correct.
            Cbuf_ExecuteText(
                common,
                cbufExec_t::EXEC_APPEND as c_int,
                &format!("{}\n", s),
            );
            Cvar_Set(common, "nextmap", "");
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
    if handle < 0 || handle >= MAX_VIDEO_HANDLES || cl.cinTable[handle as usize].status == FMV_EOF {
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
    if handle < 0 || handle >= MAX_VIDEO_HANDLES || cl.cinTable[handle as usize].status == FMV_EOF {
        return;
    }
    cl.cinTable[handle as usize].looping = r#loop;
}

/// Raven `CIN_DrawCinematic`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1344-1415`
pub fn CIN_DrawCinematic(cl: &mut Client, handle: c_int) {
    if handle < 0 || handle >= MAX_VIDEO_HANDLES || cl.cinTable[handle as usize].status == FMV_EOF {
        return;
    }

    if cl.cinTable[handle as usize].buf.is_null() {
        return;
    }

    let x = cl.cinTable[handle as usize].xpos as f32;
    let y = cl.cinTable[handle as usize].ypos as f32;
    let w = cl.cinTable[handle as usize].width as f32;
    let h = cl.cinTable[handle as usize].height as f32;

    if cl.cinTable[handle as usize].dirty
        && (cl.cinTable[handle as usize].CIN_WIDTH != cl.cinTable[handle as usize].drawX
            || cl.cinTable[handle as usize].CIN_HEIGHT != cl.cinTable[handle as usize].drawY)
    {
        let xm = cl.cinTable[handle as usize].CIN_WIDTH / 256;
        let ym = cl.cinTable[handle as usize].CIN_HEIGHT / 256;
        let mut ll = 8;
        if cl.cinTable[handle as usize].CIN_WIDTH == 512 {
            ll = 9;
        }

        unsafe {
            let buf3 = cl.cinTable[handle as usize].buf as *mut c_int;
            // PORT-NOTE(shape): `Hunk_AllocateTempMemory` needs
            // `view: &mut EngineHostView`, but `CIN_DrawCinematic`'s resolved
            // signature carries only `cl`. Passing `common` is a placeholder
            // the seam fixer must correct.
            let buf2 = Hunk_AllocateTempMemory(common, 256 * 256 * 4) as *mut c_int;

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
            cl.re
                .DrawStretchRaw(x, y, w, h, 256, 256, buf2 as *mut byte, handle, qtrue);
            cl.cinTable[handle as usize].dirty = qfalse;
            Hunk_FreeTempMemory(common, buf2 as *mut ());
            return;
        }
    }

    unsafe {
        cl.re.DrawStretchRaw(
            x,
            y,
            w,
            h,
            cl.cinTable[handle as usize].drawX,
            cl.cinTable[handle as usize].drawY,
            cl.cinTable[handle as usize].buf,
            handle,
            cl.cinTable[handle as usize].dirty,
        );
    }
    cl.cinTable[handle as usize].dirty = qfalse;
}

/// Raven `CIN_UploadCinematic`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1472-1493`
pub fn CIN_UploadCinematic(cl: &mut Client, handle: c_int) {
    if handle >= 0 && handle < MAX_VIDEO_HANDLES {
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
        unsafe {
            cl.re.UploadCinematic(
                cl.cinTable[handle as usize].drawX,
                cl.cinTable[handle as usize].drawY,
                cl.cinTable[handle as usize].buf,
                handle,
                cl.cinTable[handle as usize].dirty,
            );
        }
        if cl.cl_inGameVideo.integer == 0 && cl.cinTable[handle as usize].playonwalls == 1 {
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
                                move4_32(
                                    (&mut cl.vq4[cell * 32] as *mut c_ushort) as *mut byte,
                                    *status.add(index),
                                    spl,
                                );
                                // §19-PORT-NOTE: Raven calls `blit4_32` here for
                                // the 4x4 vq code arm; the line above is a
                                // literal transcription slip risk, kept
                                // faithful below with the correct callee.
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
                                let src =
                                    status.add(index).read().offset(cl.cin.mcomp[mc] as isize);
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
                    let src = status.add(index).read().offset(cl.cin.mcomp[mc] as isize);
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

        let icptr = cl.vq4.as_mut_ptr() as *mut c_uint;
        let idptr = cl.vq8.as_mut_ptr() as *mut c_uint;

        for _ in 0..four {
            let a_off = *input as usize;
            input = input.add(1);
            let b_off = *input as usize;
            input = input.add(1);
            let iaptr = (cl.vq2.as_mut_ptr() as *mut c_uint).add(a_off * 4);
            let ibptr2 = (cl.vq2.as_mut_ptr() as *mut c_uint).add(b_off * 4);
            for _ in 0..2 {
                // PORT-NOTE(macro): `VQ2TO4` is an unresolved function-like
                // macro (no rosetta row); escalated per zero-park discipline.
                VQ2TO4(iaptr, ibptr2, icptr, idptr);
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

    let mut numQuadCels = (cl.cinTable[handle].CIN_WIDTH * cl.cinTable[handle].CIN_HEIGHT) / 16;
    numQuadCels += numQuadCels / 4 + numQuadCels / 16;
    numQuadCels += 64; // for overflow

    let mut numQuadCels = (cl.cinTable[handle].xsize * cl.cinTable[handle].ysize) / 16;
    numQuadCels += numQuadCels / 4;
    numQuadCels += 64; // for overflow

    cl.cinTable[handle].onQuad = 0;

    let mut y = 0;
    while y < cl.cinTable[handle].ysize {
        let mut x = 0;
        while x < cl.cinTable[handle].xsize {
            recurseQuad(cl, x, y, 16, xOff, yOff);
            x += 16;
        }
        y += 16;
    }

    let temp: *mut byte = std::ptr::null_mut();

    for i in (numQuadCels - 64)..numQuadCels {
        cl.cin.qStatus[0][i as usize] = temp; // eoq
        cl.cin.qStatus[1][i as usize] = temp; // eoq
    }
}

/// Raven `RoQReset`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:899-912`
pub fn RoQReset(common: &mut Common, cl: &mut Client) {
    if cl.currentHandle < 0 {
        return;
    }
    let handle = cl.currentHandle as usize;

    if cl.cinTable[handle].iFile != 0 {
        crate::sys_end_streamed_file(cl.cinTable[handle].iFile);
        // PORT-NOTE(shape): `FS_Seek` needs `view: &mut EngineHostView`, but
        // `RoQReset`'s resolved signature carries only `common`/`cl`. Passing
        // `common` is a placeholder the seam fixer must correct.
        FS_Seek(
            common,
            cl.cinTable[handle].iFile,
            0,
            fsOrigin_t::FS_SEEK_SET as c_int,
        );
        unsafe {
            FS_Read(
                common,
                cl.cin.file.as_mut_ptr() as *mut (),
                16,
                cl.cinTable[handle].iFile,
            );
        }
        RoQ_init(common, cl);
        // let the background thread start reading ahead
        crate::sys_begin_streamed_file(cl.cinTable[handle].iFile, 0x10000);
        cl.cinTable[handle].status = FMV_LOOPED;
    }
}

/// Raven `CIN_StopCinematic`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1134-1154`
pub fn CIN_StopCinematic(cl: &mut Client, handle: c_int) -> e_status {
    if handle < 0 || handle >= MAX_VIDEO_HANDLES || cl.cinTable[handle as usize].status == FMV_EOF {
        return FMV_EOF;
    }
    cl.currentHandle = handle;

    Com_DPrintf(
        common,
        &format!(
            "trFMV::stop(), closing {}\n",
            String::from_utf8_lossy(&cl.cinTable[cl.currentHandle as usize].fileName)
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
    RoQShutdown(cl);

    FMV_EOF
}

/// Raven `SCR_DrawCinematic`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1451-1455`
pub fn SCR_DrawCinematic(cl: &mut Client) {
    if cl.CL_handle >= 0 && cl.CL_handle < MAX_VIDEO_HANDLES {
        CIN_DrawCinematic(cl, cl.CL_handle);
    }
}

/// Raven `CIN_CloseAllVideos`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:126-134`
pub fn CIN_CloseAllVideos(cl: &mut Client) {
    for i in 0..MAX_VIDEO_HANDLES {
        if cl.cinTable[i as usize].fileName[0] != 0 {
            CIN_StopCinematic(cl, i);
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

    cl.cinTable[handle].VQNormal = blitVQQuad32fs as *const ();
    cl.cinTable[handle].VQBuffer = blitVQQuad32fs as *const ();
    ROQ_GenYUVTables(cl);
    RllSetupTable(cl);
}

/// Raven `CIN_PlayCinematic`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1237-1322`
pub fn CIN_PlayCinematic(
    common: &mut Common,
    cl: &mut Client,
    arg: *const c_char,
    x: c_int,
    y: c_int,
    w: c_int,
    h: c_int,
    systemBits: c_int,
) -> c_int {
    let arg_str = unsafe { std::ffi::CStr::from_ptr(arg).to_string_lossy().into_owned() };

    let name = if arg_str.contains('/') || arg_str.contains('\\') {
        arg_str.clone()
    } else {
        format!("video/{}", arg_str)
    };
    // COM_DefaultExtension(name, sizeof(name), ".roq") - appended below if missing.
    let name = if name.to_lowercase().ends_with(".roq") {
        name
    } else {
        format!("{}.roq", name)
    };

    if systemBits & CIN_system == 0 {
        for i in 0..MAX_VIDEO_HANDLES {
            if String::from_utf8_lossy(&cl.cinTable[i as usize].fileName).trim_end_matches('\0')
                == name
            {
                return i;
            }
        }
    }

    Com_DPrintf(common, &format!("SCR_PlayCinematic( {} )\n", arg_str));

    Com_Memset(
        &mut cl.cin as *mut _ as *mut (),
        0,
        std::mem::size_of_val(&cl.cin),
    );
    cl.currentHandle = CIN_HandleForVideo(cl);
    let handle = cl.currentHandle as usize;

    let name_bytes = name.as_bytes();
    let copy_len = name_bytes.len().min(cl.cinTable[handle].fileName.len() - 1);
    cl.cinTable[handle].fileName[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
    cl.cinTable[handle].fileName[copy_len] = 0;

    cl.cinTable[handle].ROQSize = 0;
    // PORT-NOTE(shape): `FS_FOpenFileRead` needs `view: &mut EngineHostView`,
    // but `CIN_PlayCinematic`'s resolved signature carries only
    // `common`/`cl`. Passing `common` is a placeholder the seam fixer must
    // correct.
    cl.cinTable[handle].ROQSize =
        FS_FOpenFileRead(common, &name, &mut cl.cinTable[handle].iFile, true);

    if cl.cinTable[handle].ROQSize <= 0 {
        Com_DPrintf(common, &format!("cinematic failed to open {}\n", arg_str));
        cl.cinTable[handle].fileName[0] = 0;
        return -1;
    }

    CIN_SetExtents(cl, cl.currentHandle, x, y, w, h);
    CIN_SetLooping(
        cl,
        cl.currentHandle,
        if systemBits & CIN_loop != 0 {
            qtrue
        } else {
            qfalse
        },
    );

    cl.cinTable[handle].CIN_HEIGHT = DEFAULT_CIN_HEIGHT;
    cl.cinTable[handle].CIN_WIDTH = DEFAULT_CIN_WIDTH;
    cl.cinTable[handle].holdAtEnd = if systemBits & CIN_hold != 0 {
        qtrue
    } else {
        qfalse
    };
    cl.cinTable[handle].alterGameState = if systemBits & CIN_system != 0 {
        qtrue
    } else {
        qfalse
    };
    cl.cinTable[handle].playonwalls = 1;
    cl.cinTable[handle].silent = if systemBits & CIN_silent != 0 {
        qtrue
    } else {
        qfalse
    };
    cl.cinTable[handle].shader = if systemBits & CIN_shader != 0 {
        qtrue
    } else {
        qfalse
    };

    if cl.cinTable[handle].alterGameState != qfalse {
        // close the menu
        if !cl.uivm.is_null() {
            VM_Call(
                common,
                cl.uivm,
                MpUiExport::UI_SET_ACTIVE_MENU as c_int,
                &[UIMENU_NONE as isize],
            );
        }
    } else {
        cl.cinTable[handle].playonwalls = cl.cl_inGameVideo.integer;
    }

    initRoQ(cl);

    unsafe {
        FS_Read(
            common,
            cl.cin.file.as_mut_ptr() as *mut (),
            16,
            cl.cinTable[handle].iFile,
        );
    }

    let roq_id = cl.cin.file[0] as c_ushort + cl.cin.file[1] as c_ushort * 256;
    if roq_id == 0x1084 {
        RoQ_init(common, cl);
        // let the background thread start reading ahead
        crate::sys_begin_streamed_file(cl.cinTable[handle].iFile, 0x10000);

        cl.cinTable[handle].status = FMV_PLAY;
        Com_DPrintf(common, &format!("trFMV::play(), playing {}\n", arg_str));

        if cl.cinTable[handle].alterGameState != qfalse {
            cl.cls.state = connstate_t::CA_CINEMATIC;
        }

        Con_Close(common, cl);

        cl.s_rawend = cl.s_soundtime;

        return cl.currentHandle;
    }
    Com_DPrintf(common, "trFMV::play(), invalid RoQ ID\n");

    RoQShutdown(cl);
    -1
}

/// Raven `SCR_StopCinematic`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1464-1470`
pub fn SCR_StopCinematic(cl: &mut Client) {
    if cl.CL_handle >= 0 && cl.CL_handle < MAX_VIDEO_HANDLES {
        CIN_StopCinematic(cl, cl.CL_handle);
        S_StopAllSounds(cl);
        cl.CL_handle = -1;
    }
}

/// Raven `RoQInterrupt`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:922-1052`
pub fn RoQInterrupt(common: &mut Common, cl: &mut Client) {
    if cl.currentHandle < 0 {
        return;
    }
    let handle = cl.currentHandle as usize;

    let mut sbuf = [0i16; 32768];

    unsafe {
        Sys_StreamedRead(
            common,
            cl.cin.file.as_mut_ptr() as *mut (),
            cl.cinTable[handle].RoQFrameSize + 8,
            1,
            cl.cinTable[handle].iFile,
        );
    }
    if cl.cinTable[handle].RoQPlayed >= cl.cinTable[handle].ROQSize {
        if cl.cinTable[handle].holdAtEnd == qfalse {
            if cl.cinTable[handle].looping != qfalse {
                RoQReset(common, cl);
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
        match cl.cinTable[handle].roq_id as c_long {
            ROQ_QUAD_VQ => {
                if (cl.cinTable[handle].numQuads & 1) != 0 {
                    cl.cinTable[handle].normalBuffer0 = cl.cinTable[handle].t[1];
                    RoQPrepMcomp(
                        cl,
                        cl.cinTable[handle].roqF0 as c_long,
                        cl.cinTable[handle].roqF1 as c_long,
                    );
                    unsafe {
                        let vq1 = cl.cinTable[handle].VQ1;
                        // Raven: `cinTable[currentHandle].VQ1( (byte *)cin.qStatus[1], framedata)`.
                        let f: fn(*mut byte, *mut byte) =
                            std::mem::transmute::<*const (), fn(*mut byte, *mut byte)>(vq1);
                        f(cl.cin.qStatus[1].as_mut_ptr() as *mut byte, framedata);
                        cl.cinTable[handle].buf = cl
                            .cin
                            .linbuf
                            .offset(cl.cinTable[handle].screenDelta as isize);
                    }
                } else {
                    cl.cinTable[handle].normalBuffer0 = cl.cinTable[handle].t[0];
                    RoQPrepMcomp(
                        cl,
                        cl.cinTable[handle].roqF0 as c_long,
                        cl.cinTable[handle].roqF1 as c_long,
                    );
                    unsafe {
                        let vq0 = cl.cinTable[handle].VQ0;
                        let f: fn(*mut byte, *mut byte) =
                            std::mem::transmute::<*const (), fn(*mut byte, *mut byte)>(vq0);
                        f(cl.cin.qStatus[0].as_mut_ptr() as *mut byte, framedata);
                        cl.cinTable[handle].buf = cl.cin.linbuf;
                    }
                }
                if cl.cinTable[handle].numQuads == 0 {
                    // first frame
                    unsafe {
                        Com_Memcpy(
                            cl.cin
                                .linbuf
                                .offset(cl.cinTable[handle].screenDelta as isize)
                                as *mut (),
                            cl.cin.linbuf as *const (),
                            (cl.cinTable[handle].samplesPerLine * cl.cinTable[handle].ysize)
                                as usize,
                        );
                    }
                }
                cl.cinTable[handle].numQuads += 1;
                cl.cinTable[handle].dirty = qtrue;
            }
            ROQ_CODEBOOK => {
                decodeCodeBook(cl, framedata, cl.cinTable[handle].roq_flags);
            }
            ZA_SOUND_MONO => {
                if cl.cinTable[handle].silent == qfalse {
                    let ssize = RllDecodeMonoToStereo(
                        cl,
                        framedata,
                        sbuf.as_mut_ptr(),
                        cl.cinTable[handle].RoQFrameSize as c_uint,
                        0,
                        cl.cinTable[handle].roq_flags,
                    );
                    S_RawSamples(
                        cl,
                        ssize as c_int,
                        22050,
                        2,
                        1,
                        sbuf.as_mut_ptr() as *mut byte,
                        common.s_volume.value,
                        1,
                    );
                }
            }
            ZA_SOUND_STEREO => {
                if cl.cinTable[handle].silent == qfalse {
                    if cl.cinTable[handle].numQuads == -1 {
                        S_Update(common, cl);
                        cl.s_rawend = cl.s_soundtime;
                    }
                    let ssize = RllDecodeStereoToStereo(
                        cl,
                        framedata,
                        sbuf.as_mut_ptr(),
                        cl.cinTable[handle].RoQFrameSize as c_uint,
                        0,
                        cl.cinTable[handle].roq_flags,
                    );
                    S_RawSamples(
                        cl,
                        ssize as c_int,
                        22050,
                        2,
                        2,
                        sbuf.as_mut_ptr() as *mut byte,
                        common.s_volume.value,
                        1,
                    );
                }
            }
            ROQ_QUAD_INFO => {
                if cl.cinTable[handle].numQuads == -1 {
                    readQuadInfo(cl, framedata);
                    setupQuad(cl, 0, 0);
                    let start = (crate::sys_milliseconds(common) as f32
                        * common.com_timescale.value) as c_long;
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
                    RoQReset(common, cl);
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
            cl.cinTable[handle].roq_id =
                *framedata as c_ushort + *framedata.add(1) as c_ushort * 256;
            cl.cinTable[handle].RoQFrameSize = *framedata.add(2) as c_long
                + *framedata.add(3) as c_long * 256
                + *framedata.add(4) as c_long * 65536;
            cl.cinTable[handle].roq_flags =
                *framedata.add(6) as c_ushort + *framedata.add(7) as c_ushort * 256;
            cl.cinTable[handle].roqF0 = *framedata.add(7) as c_char;
            cl.cinTable[handle].roqF1 = *framedata.add(6) as c_char;
        }

        if cl.cinTable[handle].RoQFrameSize > 65536 || cl.cinTable[handle].roq_id == 0x1084 {
            Com_DPrintf(common, "roq_size>65536||roq_id==0x1084\n");
            cl.cinTable[handle].status = FMV_EOF;
            if cl.cinTable[handle].looping != qfalse {
                RoQReset(common, cl);
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
        cl.cinTable[handle].RoQPlayed += cl.cinTable[handle].RoQFrameSize + 8;
        break;
    }
}

/// Raven `CIN_RunCinematic`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1165-1229`
pub fn CIN_RunCinematic(common: &mut Common, cl: &mut Client, handle: c_int) -> e_status {
    if handle < 0 || handle >= MAX_VIDEO_HANDLES || cl.cinTable[handle as usize].status == FMV_EOF {
        return FMV_EOF;
    }

    if cl.currentHandle != handle {
        cl.currentHandle = handle;
        cl.cinTable[cl.currentHandle as usize].status = FMV_EOF;
        RoQReset(common, cl);
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

    let thisTime = (crate::sys_milliseconds(common) as f32 * common.com_timescale.value) as c_long;
    if cl.cinTable[ch].shader != qfalse && (thisTime - cl.cinTable[ch].lastTime).abs() > 100 {
        cl.cinTable[ch].startTime += thisTime - cl.cinTable[ch].lastTime;
    }
    cl.cinTable[ch].tfps = (((crate::sys_milliseconds(common) as f32 * common.com_timescale.value)
        as c_long
        - cl.cinTable[ch].startTime)
        * cl.cinTable[ch].roqFPS)
        / 1000;

    let mut start = cl.cinTable[ch].startTime;
    while cl.cinTable[ch].tfps != cl.cinTable[ch].numQuads && cl.cinTable[ch].status == FMV_PLAY {
        RoQInterrupt(common, cl);
        if start != cl.cinTable[ch].startTime {
            cl.cinTable[ch].tfps = (((crate::sys_milliseconds(common) as f32
                * common.com_timescale.value) as c_long
                - cl.cinTable[ch].startTime)
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
            RoQReset(common, cl);
        } else {
            RoQShutdown(cl);
        }
    }

    cl.cinTable[ch].status
}

/// Raven `SCR_RunCinematic`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1457-1462`
pub fn SCR_RunCinematic(common: &mut Common, cl: &mut Client) {
    if cl.CL_handle >= 0 && cl.CL_handle < MAX_VIDEO_HANDLES {
        CIN_RunCinematic(common, cl, cl.CL_handle);
    }
}

/// Raven `CL_PlayCinematic_f`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:1417-1448`
pub fn CL_PlayCinematic_f(common: &mut Common, cl: &mut Client) {
    let mut bits: c_int = CIN_system;

    Com_DPrintf(common, "CL_PlayCinematic_f\n");
    if cl.cls.state == connstate_t::CA_CINEMATIC {
        SCR_StopCinematic(cl);
    }

    let arg = Cmd_Argv(common, 1);
    let s = Cmd_Argv(common, 2);

    if (!s.is_empty() && s.as_bytes()[0] == b'1')
        || Q_stricmp(arg, "demoend.roq") == 0
        || Q_stricmp(arg, "end.roq") == 0
    {
        bits |= CIN_hold;
    }
    if !s.is_empty() && s.as_bytes()[0] == b'2' {
        bits |= CIN_loop;
    }

    S_StopAllSounds(cl);

    let arg_c = std::ffi::CString::new(arg).unwrap_or_default();
    cl.CL_handle = CIN_PlayCinematic(
        common,
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
            SCR_RunCinematic(common, cl);
            let ch = cl.currentHandle as usize;
            if !(cl.cinTable[ch].buf.is_null() && cl.cinTable[ch].status == FMV_PLAY) {
                break;
            }
        }
    } else {
        com_printf(
            common,
            &format!(
                "{}PlayCinematic(): Failed to open \"{}\"\n",
                S_COLOR_RED, arg
            ),
        );
    }
}

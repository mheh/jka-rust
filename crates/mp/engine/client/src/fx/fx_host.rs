//! Where the FX system reaches the rest of the engine.
//!
//! Raven put this behind `SFxHelper`, a wrapper of inline forwarders over the
//! `re`, `S_*`, `FS_*`, and `VM_Call` globals.
//! DEC-61.3 dissolves the wrapper, so each method below is the direct call the
//! forwarder wrapped, with the receivers threaded in.
//!
//! The `Harness` arm exists because porting-rules §18 requires the port to be
//! provable differentially. It gives the parity test the scripted trace, bolt,
//! and registration replies the C++ rig scripts, and it captures the emission
//! stream the goldens hold. A live client always runs the `Engine` arm.
//!
//! Source: `oracle/codemp/client/FxSystem.h:49-219`,
//! `oracle/codemp/client/FxSystem.cpp:39-129`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;
use std::sync::Arc;

use mp_abi::cgame::exports::MpCgameExport;
use mp_abi::cgame::shared_buffer::{
    TCGCameraShake, TCGG2Mark, TCGGetBoltData, TCGPointContents, TCGTrace, TCGVectorData,
};
use mp_engine_ghoul2::api_bolts::g2api_get_bolt_matrix;
use mp_engine_ghoul2::shared::cghoul2_info_v::CGhoul2Info_v;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::{com_printf, Common};
use mp_engine_qcommon::files_common::FS_FCloseFile;
use mp_engine_qcommon::files_pc::{FS_FOpenFileByMode, FS_Read2};
use mp_engine_qcommon::vm_fns::VM_Call;
use mp_qshared::common::mp::cgame::mini_ref_entity_s::miniRefEntity_t;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::{fileHandle_t, CHAN_AUTO, ENTITYNUM_NONE, FS_READ};
use mp_renderer::hook_install::{re_from_view, rm_from_view};
use mp_renderer::tr_cmds::RE_StretchPic;
use mp_renderer::tr_model::frontend::RE_RegisterModel;
use mp_renderer::tr_scene::{RE_AddLightToScene, RE_AddMiniRefEntityToScene, RE_AddPolyToScene};
use mp_renderer::tr_shader::RE_RegisterShader;
use native_math::rng::QRand;
use native_math::vector::vec3_t;
use native_types::mdxaBone_t;

use crate::client_host::{g2_from_view, snd_from_view, Client};
use crate::fx::fx_harness::{fx_f32, fx_v3, fx_zero_trace, FxHarness};
use crate::snd_dma::{S_RegisterSound, S_StartLocalSound, S_StartSound};

/// The two worlds the FX system can run against.
///
/// - `Engine`: the live client. Every method makes the direct call DEC-61.3 pins.
/// - `Harness`: the `tools/fx-oracle` parity rig. Inputs are scripted and every
///   outbound call lands in the capture stream instead.
pub enum FxHost<'a, 'v> {
    Engine {
        view: &'a mut EngineHostView<'v>,
        cl: &'a mut Client,
    },
    Harness(&'a mut FxHarness),
}

impl FxHost<'_, '_> {
    /// The generator every FX draw comes from.
    ///
    /// The live client shares `Common.qrand` with the rest of the engine island,
    /// which is what Raven's file-scope `holdrand` did.
    pub fn rng(&mut self) -> &mut QRand {
        match self {
            FxHost::Engine { view, .. } => &mut view.common.qrand,
            FxHost::Harness(h) => &mut h.rng,
        }
    }

    /// Raven `SFxHelper::Print` — a developer-gated print.
    ///
    /// Source: `oracle/codemp/client/FxSystem.cpp:40-50`
    pub fn Print(&mut self, msg: &str) {
        match self {
            FxHost::Engine { view, .. } => {
                let developer = match view.common.com_developer {
                    Some(h) => view.common.cvar(h).integer,
                    None => 0,
                };
                if developer != 0 {
                    com_printf(view.common, msg);
                }
            }
            FxHost::Harness(h) => {
                let text = msg.strip_suffix('\n').unwrap_or(msg).to_string();
                h.emit(format!("PRINT {text}"));
            }
        }
    }

    /// An always-on print. Raven reaches `Com_Printf` directly for the `fx_debug 2` trace.
    ///
    /// Source: `oracle/codemp/client/FxScheduler.cpp:854`
    pub fn Printf(&mut self, msg: &str) {
        match self {
            FxHost::Engine { view, .. } => com_printf(view.common, msg),
            FxHost::Harness(h) => {
                let text = msg.strip_suffix('\n').unwrap_or(msg).to_string();
                h.emit(format!("PRINT {text}"));
            }
        }
    }

    /// Raven `SFxHelper::OpenFile` — always `FS_READ`, whatever mode the caller passes.
    ///
    /// Source: `oracle/codemp/client/FxSystem.h:76-79`
    pub fn OpenFile(&mut self, path: &str) -> (c_int, fileHandle_t) {
        match self {
            FxHost::Engine { view, .. } => {
                let mut fh: fileHandle_t = 0;
                let len = FS_FOpenFileByMode(view, path, &mut fh, FS_READ);
                (len, fh)
            }
            FxHost::Harness(h) => match h.files.get(path) {
                Some(text) => {
                    let handle = h.next_file_handle;
                    h.next_file_handle += 1;
                    let len = text.len() as c_int;
                    h.open_files.insert(handle, (text.clone(), 0));
                    (len, handle)
                }
                None => (-1, 0),
            },
        }
    }

    /// Raven `SFxHelper::ReadFile`.
    ///
    /// Source: `oracle/codemp/client/FxSystem.h:80-84`
    pub fn ReadFile(&mut self, len: usize, fh: fileHandle_t) -> Vec<u8> {
        match self {
            FxHost::Engine { view, .. } => {
                let mut data = vec![0u8; len];
                // SAFETY: `FS_Read2` writes `len` bytes into the buffer we just sized.
                unsafe {
                    FS_Read2(view.common, data.as_mut_ptr() as *mut (), len as c_int, fh);
                }
                data
            }
            FxHost::Harness(h) => match h.open_files.get_mut(&fh) {
                Some((text, pos)) => {
                    let end = (*pos + len).min(text.len());
                    let out = text.as_bytes()[*pos..end].to_vec();
                    *pos = end;
                    out
                }
                None => Vec::new(),
            },
        }
    }

    /// Raven `SFxHelper::CloseFile`.
    ///
    /// Source: `oracle/codemp/client/FxSystem.h:85-88`
    pub fn CloseFile(&mut self, fh: fileHandle_t) {
        match self {
            FxHost::Engine { view, .. } => FS_FCloseFile(view.common, fh),
            FxHost::Harness(h) => {
                h.open_files.remove(&fh);
            }
        }
    }

    /// Raven `SFxHelper::RegisterShader`.
    ///
    /// Source: `oracle/codemp/client/FxSystem.h:198-201`
    pub fn RegisterShader(&mut self, name: &str) -> c_int {
        match self {
            FxHost::Engine { view, .. } => {
                // SAFETY: view-constructor slots, single-threaded, no other live cast.
                let re = unsafe { re_from_view(view) };
                let rm = unsafe { rm_from_view(view) };
                RE_RegisterShader(
                    name,
                    &mut re.qs,
                    &mut re.world_load,
                    Arc::make_mut(&mut re.sim.published),
                    view,
                    &re.cvars,
                    rm,
                    &mut re.img_state,
                    &mut re.sky_view,
                    &mut re.sky,
                )
            }
            FxHost::Harness(h) => {
                let FxHarness { shaders, out, .. } = &mut **h;
                FxHarness::register("REGSHADER", shaders, out, name)
            }
        }
    }

    /// Raven `SFxHelper::RegisterModel`.
    ///
    /// Source: `oracle/codemp/client/FxSystem.h:202-205`
    pub fn RegisterModel(&mut self, name: &str) -> c_int {
        match self {
            FxHost::Engine { view, .. } => {
                // SAFETY: view-constructor slots, single-threaded, no other live cast.
                let re = unsafe { re_from_view(view) };
                let rm = unsafe { rm_from_view(view) };
                RE_RegisterModel(
                    &mut re.qs,
                    &mut re.world_load,
                    Arc::make_mut(&mut re.sim.published),
                    view,
                    &re.cvars,
                    rm,
                    &mut re.img_state,
                    &mut re.sky_view,
                    &mut re.sky,
                    &mut re.world_effects,
                    name,
                )
            }
            FxHost::Harness(h) => {
                let FxHarness { models, out, .. } = &mut **h;
                FxHarness::register("REGMODEL", models, out, name)
            }
        }
    }

    /// Raven `SFxHelper::RegisterSound`.
    ///
    /// Source: `oracle/codemp/client/FxSystem.h:101-104`
    pub fn RegisterSound(&mut self, name: &str) -> c_int {
        match self {
            FxHost::Engine { view, .. } => {
                // SAFETY: view-constructor slot, single-threaded, no other live cast.
                let snd = unsafe { snd_from_view(view) };
                S_RegisterSound(view, snd, name)
            }
            FxHost::Harness(h) => {
                let FxHarness { sounds, out, .. } = &mut **h;
                FxHarness::register("REGSOUND", sounds, out, name)
            }
        }
    }

    /// Raven `SFxHelper::PlaySound`.
    ///
    /// Raven's body drops `entityNum`, `entchannel`, `volume`, and `radius` and
    /// passes `ENTITYNUM_NONE`/`CHAN_AUTO` instead. DEC-61.3 keeps that quirk.
    ///
    /// Source: `oracle/codemp/client/FxSystem.h:91-95`
    pub fn PlaySound(
        &mut self,
        origin: &mut vec3_t,
        _entity_num: c_int,
        _entchannel: c_int,
        sfx_handle: c_int,
        _volume: c_int,
        _radius: c_int,
    ) {
        match self {
            FxHost::Engine { view, .. } => {
                // SAFETY: view-constructor slot, single-threaded, no other live cast.
                let snd = unsafe { snd_from_view(view) };
                S_StartSound(
                    view,
                    snd,
                    Some(*origin),
                    ENTITYNUM_NONE,
                    CHAN_AUTO,
                    sfx_handle,
                );
            }
            // The four dropped arguments reach the record as the values Raven's
            // call actually carries: the rig declares `S_StartSound` with the
            // volume and radius defaulted to -1, so the loss stays visible.
            // Source: `tools/fx-oracle/host.cpp:317-326`
            FxHost::Harness(h) => h.emit(format!(
                "SOUND origin {} entnum {} entchannel {} sfx {} volume -1 radius -1",
                fx_v3(origin),
                ENTITYNUM_NONE,
                CHAN_AUTO,
                sfx_handle,
            )),
        }
    }

    /// Raven `SFxHelper::PlayLocalSound`.
    ///
    /// Source: `oracle/codemp/client/FxSystem.h:96-100`
    pub fn PlayLocalSound(&mut self, sfx_handle: c_int, entchannel: c_int) {
        match self {
            FxHost::Engine { view, .. } => {
                // SAFETY: view-constructor slot, single-threaded, no other live cast.
                let snd = unsafe { snd_from_view(view) };
                S_StartLocalSound(view, snd, sfx_handle, entchannel);
            }
            FxHost::Harness(h) => h.emit(format!(
                "LOCALSOUND sfx {sfx_handle} entchannel {entchannel}"
            )),
        }
    }

    /// Raven `SFxHelper::AddFxToScene(miniRefEntity_t*)`.
    ///
    /// The emitter passes a null pointer once per attached-model draw, which the
    /// `None` case carries.
    ///
    /// Source: `oracle/codemp/client/FxSystem.h:182-190`
    pub fn AddFxToScene(&mut self, ent: Option<&miniRefEntity_t>) {
        match self {
            FxHost::Engine { view, .. } => {
                // SAFETY: view-constructor slot, single-threaded, no other live cast.
                let re = unsafe { re_from_view(view) };
                RE_AddMiniRefEntityToScene(&mut re.frame_data, &re.sim.published, &mut re.scene, ent);
            }
            FxHost::Harness(h) => match ent {
                Some(e) => {
                    let record = format!("MINIREFENT {}", FxHarness::refent_fields(e));
                    h.emit(record);
                }
                None => h.emit("NULLREFENT".to_string()),
            },
        }
    }

    /// Raven `SFxHelper::AddLightToScene`.
    ///
    /// Source: `oracle/codemp/client/FxSystem.h:192-195`
    pub fn AddLightToScene(&mut self, org: vec3_t, radius: f32, red: f32, green: f32, blue: f32) {
        match self {
            FxHost::Engine { view, .. } => {
                // SAFETY: view-constructor slot, single-threaded, no other live cast.
                let re = unsafe { re_from_view(view) };
                RE_AddLightToScene(
                    &mut re.frame_data,
                    &re.sim.published,
                    org,
                    radius,
                    red,
                    green,
                    blue,
                );
            }
            FxHost::Harness(h) => h.emit(format!(
                "LIGHT origin {} radius {} rgb {} {} {}",
                fx_v3(&org),
                fx_f32(radius),
                fx_f32(red),
                fx_f32(green),
                fx_f32(blue)
            )),
        }
    }

    /// Raven `SFxHelper::AddPolyToScene` — always one poly.
    ///
    /// Source: `oracle/codemp/client/FxSystem.h:207-210`
    pub fn AddPolyToScene(&mut self, shader: c_int, verts: &[polyVert_t]) {
        match self {
            FxHost::Engine { view, .. } => {
                // SAFETY: view-constructor slot, single-threaded, no other live cast.
                let re = unsafe { re_from_view(view) };
                RE_AddPolyToScene(
                    &mut re.frame_data,
                    &re.sim.published,
                    view.common,
                    shader,
                    verts,
                    verts.len(),
                    1,
                );
            }
            FxHost::Harness(h) => {
                h.emit(format!("POLY shader {} count {}", shader, verts.len()));
                for (i, v) in verts.iter().enumerate() {
                    let record = FxHarness::polyvert_fields(i, v);
                    h.emit(record);
                }
            }
        }
    }

    /// Raven `SFxHelper::AddDecalToScene`.
    ///
    //TODO: Port RE_AddDecalToScene world root
    // Source: oracle/codemp/client/FxSystem.h:212-215
    // `RE_AddDecalToScene` takes a `world_root: &mut MarkNode`, and no carrier
    // owns a root until the renderer census merges that node arena (gh#31). The
    // `CG_R_ADDDECALTOSCENE` arm in `cl_cgame.rs` is parked on the same owner,
    // so the FX decal follows it and adds nothing on the live client. The parity
    // arm still records the call, so the decal computation stays gated.
    ///
    /// Source: `oracle/codemp/client/FxSystem.h:212-215`
    #[allow(clippy::too_many_arguments)]
    pub fn AddDecalToScene(
        &mut self,
        shader: c_int,
        origin: vec3_t,
        dir: vec3_t,
        orientation: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
        alpha_fade: bool,
        radius: f32,
        temporary: bool,
    ) {
        match self {
            FxHost::Engine { .. } => {}
            FxHost::Harness(h) => h.emit(format!(
                "DECAL shader {} origin {} dir {} orientation {} rgba {} {} {} {} alphaFade {} \
                 radius {} temporary {}",
                shader,
                fx_v3(&origin),
                fx_v3(&dir),
                fx_f32(orientation),
                fx_f32(r),
                fx_f32(g),
                fx_f32(b),
                fx_f32(a),
                alpha_fade as i32,
                fx_f32(radius),
                temporary as i32
            )),
        }
    }

    /// Raven `SFxHelper::AddGhoul2Decal` — a `CG_G2MARK` round trip into cgame.
    ///
    /// Source: `oracle/codemp/client/FxSystem.h:161-171`
    pub fn AddGhoul2Decal(&mut self, shader: c_int, start: vec3_t, dir: vec3_t, size: f32) {
        match self {
            FxHost::Engine { view, cl } => {
                let td = TCGG2Mark {
                    shader,
                    size,
                    start,
                    dir,
                };
                write_shared(cl, &td);
                let cgvm = cl.cgvm;
                VM_Call(view.common, cgvm, MpCgameExport::CG_G2MARK as c_int, &[]);
            }
            FxHost::Harness(h) => h.emit(format!(
                "G2DECAL shader {} start {} dir {} size {}",
                shader,
                fx_v3(&start),
                fx_v3(&dir),
                fx_f32(size)
            )),
        }
    }

    /// Raven `SFxHelper::CameraShake` — a `CG_FX_CAMERASHAKE` round trip into cgame.
    ///
    /// Source: `oracle/codemp/client/FxSystem.cpp:83-93`
    pub fn CameraShake(&mut self, origin: vec3_t, intensity: f32, radius: c_int, time: c_int) {
        match self {
            FxHost::Engine { view, cl } => {
                let data = TCGCameraShake {
                    mOrigin: origin,
                    mIntensity: intensity,
                    mRadius: radius,
                    mTime: time,
                };
                write_shared(cl, &data);
                let cgvm = cl.cgvm;
                VM_Call(
                    view.common,
                    cgvm,
                    MpCgameExport::CG_FX_CAMERASHAKE as c_int,
                    &[],
                );
            }
            FxHost::Harness(h) => h.emit(format!(
                "SHAKE origin {} intensity {} radius {} time {}",
                fx_v3(&origin),
                fx_f32(intensity),
                radius,
                time
            )),
        }
    }

    /// Raven `re.DrawStretchPic`, the one renderer call `Draw2DEffects` makes.
    ///
    /// Source: `oracle/codemp/client/FxScheduler.cpp:1200`
    pub fn DrawStretchPic(&mut self, x: f32, y: f32, w: f32, h: f32, shader: c_int) {
        match self {
            FxHost::Engine { view, .. } => {
                // SAFETY: view-constructor slot, single-threaded, no other live cast.
                let re = unsafe { re_from_view(view) };
                RE_StretchPic(
                    &mut re.frame_data,
                    &re.sim.published,
                    view.common,
                    x,
                    y,
                    w,
                    h,
                    0.0,
                    0.0,
                    1.0,
                    1.0,
                    shader,
                );
            }
            FxHost::Harness(hz) => hz.emit(format!(
                "STRETCHPIC x {} y {} w {} h {} shader {}",
                fx_f32(x),
                fx_f32(y),
                fx_f32(w),
                fx_f32(h),
                shader
            )),
        }
    }

    /// Raven `SFxHelper::Trace` and `SFxHelper::G2Trace` — one `CG_TRACE`/`CG_G2TRACE` round trip.
    ///
    /// Raven's `memset(td, sizeof(*td), 0)` has its two arguments swapped, so it
    /// clears nothing. Every input field is written below anyway, so the port
    /// drops the call rather than transcribing a no-op.
    ///
    /// Source: `oracle/codemp/client/FxSystem.h:107-159`
    #[allow(clippy::too_many_arguments)]
    pub fn Trace(
        &mut self,
        start: vec3_t,
        min: Option<vec3_t>,
        max: Option<vec3_t>,
        end: vec3_t,
        skip_ent_num: c_int,
        flags: c_int,
        ghoul2: bool,
    ) -> trace_t {
        let mins = min.unwrap_or([0.0, 0.0, 0.0]);
        let maxs = max.unwrap_or([0.0, 0.0, 0.0]);
        match self {
            FxHost::Engine { view, cl } => {
                let mut td = TCGTrace {
                    mResult: fx_zero_trace(),
                    mStart: start,
                    mMins: mins,
                    mMaxs: maxs,
                    mEnd: end,
                    mSkipNumber: skip_ent_num,
                    mMask: flags,
                };
                write_shared(cl, &td);
                let cgvm = cl.cgvm;
                let call = if ghoul2 {
                    MpCgameExport::CG_G2TRACE
                } else {
                    MpCgameExport::CG_TRACE
                };
                VM_Call(view.common, cgvm, call as c_int, &[]);
                read_shared(cl, &mut td);
                td.mResult
            }
            FxHost::Harness(h) => {
                let record = format!(
                    "TRACE start {} mins {} maxs {} end {} skip {} mask {} g2 {}",
                    fx_v3(&start),
                    fx_v3(&mins),
                    fx_v3(&maxs),
                    fx_v3(&end),
                    skip_ent_num,
                    flags,
                    ghoul2 as i32
                );
                h.emit(record);
                h.next_trace(end)
            }
        }
    }

    /// Raven's inline `CG_POINT_CONTENTS` round trip in `CParticle::UpdateOrigin`.
    ///
    /// Source: `oracle/codemp/client/FxPrimitives.cpp:234-240`
    pub fn PointContents(&mut self, point: vec3_t, pass_entity_num: c_int) -> c_int {
        match self {
            FxHost::Engine { view, cl } => {
                let data = TCGPointContents {
                    mPoint: point,
                    mPassEntityNum: pass_entity_num,
                };
                write_shared(cl, &data);
                let cgvm = cl.cgvm;
                VM_Call(
                    view.common,
                    cgvm,
                    MpCgameExport::CG_POINT_CONTENTS as c_int,
                    &[],
                ) as c_int
            }
            FxHost::Harness(h) => {
                let contents = h.next_point_contents();
                h.emit(format!(
                    "POINTCONTENTS point {} passent {} -> {}",
                    fx_v3(&point),
                    pass_entity_num,
                    contents
                ));
                contents
            }
        }
    }

    /// Raven's inline `CG_GET_LERP_ORIGIN` round trip.
    ///
    /// Source: `oracle/codemp/client/FxScheduler.cpp:130-135`
    pub fn GetLerpOrigin(&mut self, entity_num: c_int) -> vec3_t {
        match self {
            FxHost::Engine { view, cl } => {
                let mut data = TCGVectorData {
                    mEntityNum: entity_num,
                    mPoint: [0.0, 0.0, 0.0],
                };
                write_shared(cl, &data);
                let cgvm = cl.cgvm;
                VM_Call(
                    view.common,
                    cgvm,
                    MpCgameExport::CG_GET_LERP_ORIGIN as c_int,
                    &[],
                );
                read_shared(cl, &mut data);
                data.mPoint
            }
            FxHost::Harness(h) => {
                let point = h.next_lerp_origin();
                h.emit(format!(
                    "LERPORIGIN ent {} -> {}",
                    entity_num,
                    fx_v3(&point)
                ));
                point
            }
        }
    }

    /// Raven `CGhoul2Info_v::IsValid` — whether the bolted instance still exists.
    ///
    /// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:399-408`
    pub fn Ghoul2IsValid(&mut self, ghoul2: i32) -> bool {
        match self {
            FxHost::Engine { view, .. } => {
                // SAFETY: view-constructor slot, single-threaded, no other live cast.
                let g2 = unsafe { g2_from_view(view) };
                g2.info_array.is_valid(ghoul2)
            }
            // The rig's `CGhoul2Info_v` stub answers from the handle alone, and
            // never from the scripted bolt queue.
            // Source: `tools/fx-oracle/stubs/G2_local.h:30`
            FxHost::Harness(_) => ghoul2 != 0,
        }
    }

    /// Raven `SFxHelper::GetOriginAxisFromBolt`.
    ///
    /// The `CG_GET_LERP_DATA` round trip zeroes pitch and roll for players and
    /// ridable vehicles before the bolt matrix lookup.
    ///
    /// Source: `oracle/codemp/client/FxSystem.cpp:96-129`
    pub fn GetOriginAxisFromBolt(
        &mut self,
        ghoul2: i32,
        ent_num: c_int,
        model_num: c_int,
        bolt_num: c_int,
        old_time: c_int,
    ) -> Option<(vec3_t, [vec3_t; 3])> {
        match self {
            FxHost::Engine { view, cl } => {
                let mut data = TCGGetBoltData {
                    mOrigin: [0.0; 3],
                    mAngles: [0.0; 3],
                    mScale: [0.0; 3],
                    mEntityNum: ent_num,
                };
                write_shared(cl, &data);
                let cgvm = cl.cgvm;
                VM_Call(
                    view.common,
                    cgvm,
                    MpCgameExport::CG_GET_LERP_DATA as c_int,
                    &[],
                );
                read_shared(cl, &mut data);

                let mut bolt_matrix = mdxaBone_t {
                    matrix: [[0.0; 4]; 3],
                };
                let mut handle = CGhoul2Info_v { mItem: ghoul2 };
                // SAFETY: view-constructor slot, single-threaded, no other live cast.
                let g2 = unsafe { g2_from_view(view) };
                let exists = g2api_get_bolt_matrix(
                    g2,
                    *view,
                    &mut handle,
                    model_num,
                    bolt_num,
                    data.mAngles,
                    data.mOrigin,
                    old_time,
                    &[],
                    data.mScale,
                    &mut bolt_matrix,
                );
                if !exists {
                    return None;
                }
                Some(bolt_matrix_to_origin_axis(&bolt_matrix))
            }
            FxHost::Harness(h) => {
                let (exists, origin, axis) = h.next_bolt();
                h.emit(format!(
                    "BOLT ent {} model {} bolt {} -> {}",
                    ent_num, model_num, bolt_num, exists as i32
                ));
                if exists {
                    Some((origin, axis))
                } else {
                    None
                }
            }
        }
    }
}

/// Unpack a bolt matrix into the origin and axis the FX spawner wants.
///
/// Source: `oracle/codemp/client/FxSystem.cpp:110-127`
fn bolt_matrix_to_origin_axis(bolt_matrix: &mdxaBone_t) -> (vec3_t, [vec3_t; 3]) {
    let m = &bolt_matrix.matrix;
    let origin = [m[0][3], m[1][3], m[2][3]];
    let mut axis = [[0.0f32; 3]; 3];
    axis[1] = [m[0][0], m[1][0], m[2][0]];
    axis[0] = [m[0][1], m[1][1], m[2][1]];
    axis[2] = [m[0][2], m[1][2], m[2][2]];
    (origin, axis)
}

/// Write one shared-buffer payload into `cl.mSharedMemory`, the way Raven's
/// pointer cast plus field stores do.
fn write_shared<T: Copy>(cl: &mut Client, value: &T) {
    // SAFETY: `mSharedMemory` is the module's `SHARED_BUFFER_SIZE` scratch block,
    // set by `CG_SET_SHARED_BUFFER`. Every `TCG*` payload fits it (§D11).
    unsafe {
        core::ptr::write_unaligned(cl.cl.mSharedMemory as *mut T, *value);
    }
}

/// Read one shared-buffer payload back after the module answered.
fn read_shared<T: Copy>(cl: &mut Client, value: &mut T) {
    // SAFETY: same block as `write_shared`, written by the module during the call.
    unsafe {
        *value = core::ptr::read_unaligned(cl.cl.mSharedMemory as *const T);
    }
}

/// The `Common` receiver the FX system prints and draws through.
///
/// Only the live-engine arm has one.
pub fn host_common<'a>(host: &'a mut FxHost<'_, '_>) -> Option<&'a mut Common> {
    match host {
        FxHost::Engine { view, .. } => Some(view.common),
        FxHost::Harness(_) => None,
    }
}

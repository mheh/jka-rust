//! Raven `CFxScheduler` — template storage, the delayed-spawn schedule, and the
//! looped-effect table (DEC-61.4).
//!
//! Handle arithmetic is parity surface: a template handle is its slot index, slot
//! zero is the bogus handle, and the name map decides whether a re-registration
//! allocates. The caps are Raven's 256 templates, 64 two-dimensional effects, 32
//! looped effects, and 24 primitives per effect, each with its own overflow arm.
//!
//! Source: `oracle/codemp/client/FxScheduler.cpp`, `oracle/codemp/client/FxScheduler.h`

#![allow(non_camel_case_types, non_snake_case)]

use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;

use mp_engine_qcommon::gp2::generic_parser2::GenericParser2;
use mp_engine_qcommon::gp2::gp_group::GpGroup;
use mp_qshared::shared::q_string::COM_StripExtension;
use mp_qshared::shared::surface_flags::MASK_SOLID;
use mp_qshared::shared::{CHAN_AUTO, ENTITYNUM_NONE};
use native_math::qmath::{
    vectoangles, CrossProduct, MakeNormalVectors, RotatePointAroundVector, VectorNormalize2,
};
use native_math::vector::vec3_t;

use crate::fx::cparticle::{distance_squared, vector_add, vector_ma_in_place, vector_scale};
use crate::fx::cprimitive_template::CPrimitiveTemplate;
use crate::fx::eprim_type::EPrimType;
use crate::fx::fx_flags::{
    FX_ACCEL_IS_ABSOLUTE, FX_AXIS_FROM_SPHERE, FX_CHEAP_ORG2_CALC, FX_CHEAP_ORG_CALC,
    FX_EVEN_DISTRIBUTION, FX_GHOUL2_DECALS, FX_MAX_TRACE_DIST, FX_ORG2_FROM_TRACE,
    FX_ORG2_IS_OFFSET, FX_ORG_ON_CYLINDER, FX_ORG_ON_SPHERE, FX_RAND_ROT_AROUND_FWD, FX_RELATIVE,
    FX_RGB_COMPONENT_INTERP, FX_TRACE_IMPACT_FX, FX_VEL_IS_ABSOLUTE,
};
use crate::fx::fx_host::FxHost;
use crate::fx::fx_system::FxSystem;
use crate::fx::fx_util::{
    FX_Add, FX_AddCylinder, FX_AddElectricity, FX_AddEmitter, FX_AddFlash, FX_AddLight, FX_AddLine,
    FX_AddOrientedParticle, FX_AddParticle, FX_AddTail,
};
use crate::fx::seffect_template::{PrimitiveRef, SEffectTemplate, FX_MAX_EFFECT_COMPONENTS};

/// How many effects the system can store.
///
/// Source: `oracle/codemp/client/FxScheduler.h:26`
pub const FX_MAX_EFFECTS: usize = 256;

/// How many two-dimensional effects the system can store.
///
/// Source: `oracle/codemp/client/FxScheduler.h:27`
pub const FX_MAX_2DEFFECTS: usize = 64;

/// How many looped effects reschedule themselves at once.
///
/// Source: `oracle/codemp/client/FxScheduler.h:398`
pub const MAX_LOOPED_FX: usize = 32;

/// Bolt-index packing, shared with ghoul2.
///
/// Source: `oracle/codemp/ghoul2/G2.h:30-40`
const ENTITY_WIDTH: i32 = 12;
const MODEL_WIDTH: i32 = 10;
const BOLT_WIDTH: i32 = 10;
const MODEL_AND: i32 = (1 << MODEL_WIDTH) - 1;
const BOLT_AND: i32 = (1 << BOLT_WIDTH) - 1;
const ENTITY_AND: i32 = (1 << ENTITY_WIDTH) - 1;
const BOLT_SHIFT: i32 = 0;
const MODEL_SHIFT: i32 = BOLT_SHIFT + BOLT_WIDTH;
const ENTITY_SHIFT: i32 = MODEL_SHIFT + MODEL_WIDTH;

/// One spawn waiting for its start time.
///
/// Source: `oracle/codemp/client/FxScheduler.h:378-395`
#[derive(Clone, Debug)]
pub struct SScheduledEffect {
    pub mpTemplate: PrimitiveRef,
    pub mStartTime: i32,
    pub mModelNum: i8,
    pub mBoltNum: i8,
    pub mEntNum: i16,
    /// rww - render this before skyportals, and not in the normal world view.
    pub mPortalEffect: bool,
    /// bolt this puppy on keep it updated
    pub mIsRelative: bool,
    pub iGhoul2: i32,
    pub mOrigin: vec3_t,
    pub mAxis: [vec3_t; 3],
}

/// One looped effect, rescheduled every `mRepeatDelay`.
///
/// Source: `oracle/codemp/client/FxScheduler.h:400-409`
#[derive(Clone, Debug, Default)]
pub struct SLoopedEffect {
    pub mId: i32,
    pub mBoltInfo: i32,
    pub mGhoul2: i32,
    pub mNextTime: i32,
    pub mLoopStopTime: i32,
    pub mPortalEffect: bool,
    pub mIsRelative: bool,
}

/// One screen-space effect, drawn after the scene.
///
/// Source: `oracle/codemp/client/FxScheduler.h:417-435`
#[derive(Clone, Copy, Debug)]
pub struct CScheduled2DEffect {
    pub mScreenX: f32,
    pub mScreenY: f32,
    pub mWidth: f32,
    pub mHeight: f32,
    /// bytes A, G, B, R -- see class paletteRGBA_c
    pub mColor: [f32; 4],
    pub mShaderHandle: i32,
}

impl Default for CScheduled2DEffect {
    fn default() -> Self {
        CScheduled2DEffect {
            mScreenX: 0.0,
            mScreenY: 0.0,
            mWidth: 0.0,
            mHeight: 0.0,
            mColor: [1.0, 1.0, 1.0, 1.0],
            mShaderHandle: 0,
        }
    }
}

/// Raven `CFxScheduler` — everything the scheduler owns.
///
/// Source: `oracle/codemp/client/FxScheduler.h:373-498`
#[derive(Clone, Debug)]
pub struct CFxScheduler {
    pub mEffectTemplates: Vec<SEffectTemplate>,
    /// If you only have the unique effect name, you'll have to use this to get the ID.
    pub mEffectIDs: BTreeMap<String, i32>,

    pub m2DEffects: Vec<CScheduled2DEffect>,
    pub mNextFree2DEffect: usize,

    /// Scheduled effects that need creating at the correct time, newest first.
    pub mFxSchedule: VecDeque<SScheduledEffect>,

    pub mLoopedEffectArray: Vec<SLoopedEffect>,
}

impl Default for CFxScheduler {
    /// Raven `CFxScheduler::CFxScheduler` — everything zeroed.
    ///
    /// Source: `oracle/codemp/client/FxScheduler.cpp:33-38`
    fn default() -> Self {
        CFxScheduler {
            mEffectTemplates: vec![SEffectTemplate::default(); FX_MAX_EFFECTS],
            mEffectIDs: BTreeMap::new(),
            m2DEffects: vec![CScheduled2DEffect::default(); FX_MAX_2DEFFECTS],
            mNextFree2DEffect: 0,
            mFxSchedule: VecDeque::new(),
            mLoopedEffectArray: vec![SLoopedEffect::default(); MAX_LOOPED_FX],
        }
    }
}

impl CFxScheduler {
    /// Source: `oracle/codemp/client/FxScheduler.h:488`
    pub fn NumScheduledFx(&self) -> usize {
        self.mFxSchedule.len()
    }
}

/// Raven's `Round` macro.
///
/// Source: `oracle/codemp/game/q_shared.h`
fn round_to_int(x: f32) -> i32 {
    if x < 0.0 {
        (x - 0.5) as i32
    } else {
        (x + 0.5) as i32
    }
}

/// Raven `CFxScheduler::Clean` — drop the schedule, and optionally the templates.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:175-241`
pub fn fx_scheduler_clean(
    fx: &mut FxSystem,
    _host: &mut FxHost<'_, '_>,
    b_remove_templates: bool,
    id_to_preserve: i32,
) {
    fx.scheduler.mFxSchedule.clear();

    if b_remove_templates {
        for i in 1..FX_MAX_EFFECTS {
            if i as i32 == id_to_preserve {
                continue;
            }
            fx.scheduler.mEffectTemplates[i].mPrimitives.clear();
            fx.scheduler.mEffectTemplates[i].mInUse = false;
        }

        if id_to_preserve == 0 {
            fx.scheduler.mEffectIDs.clear();
        } else {
            // Clear the effect names, but first get the name of the effect to
            // preserve, and restore it after clearing.
            let mut preserved = String::new();
            for (name, id) in fx.scheduler.mEffectIDs.iter() {
                if *id == id_to_preserve {
                    preserved = name.clone();
                    break;
                }
            }

            fx.scheduler.mEffectIDs.clear();
            fx.scheduler.mEffectIDs.insert(preserved, id_to_preserve);
        }
    }
}

/// Raven `CFxScheduler::GetNewEffectTemplate` — the first free slot above zero.
///
/// Handle zero stays the bogus handle, so the search starts at one.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:519-549`
pub fn fx_get_new_effect_template(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    file: Option<&str>,
) -> i32 {
    for i in 1..FX_MAX_EFFECTS {
        if !fx.scheduler.mEffectTemplates[i].mInUse {
            fx.scheduler.mEffectTemplates[i] = SEffectTemplate::default();

            // If we are a copy, we really won't have a name that we care about saving for later
            if let Some(name) = file {
                fx.scheduler.mEffectIDs.insert(name.to_string(), i as i32);
                fx.scheduler.mEffectTemplates[i].mEffectName = name.to_string();
            }

            fx.scheduler.mEffectTemplates[i].mInUse = true;
            fx.scheduler.mEffectTemplates[i].mRepeatDelay = 300;
            return i as i32;
        }
    }

    host.Print("FxScheduler:  Error--reached max effects\n");
    0
}

/// Raven `CFxScheduler::AddPrimitiveToEffect`.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:492-505`
pub fn fx_add_primitive_to_effect(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    handle: i32,
    prim: CPrimitiveTemplate,
) {
    let ct = fx.scheduler.mEffectTemplates[handle as usize].mPrimitiveCount as usize;

    if ct >= FX_MAX_EFFECT_COMPONENTS {
        host.Print("FxScheduler:  Error--too many primitives in an effect\n");
    } else {
        fx.scheduler.mEffectTemplates[handle as usize]
            .mPrimitives
            .push(Rc::new(RefCell::new(prim)));
        fx.scheduler.mEffectTemplates[handle as usize].mPrimitiveCount += 1;
    }
}

/// Raven `CFxScheduler::ParseEffect` — one group per primitive, keyed by group name.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:369-478`
pub fn fx_parse_effect(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    file: &str,
    parser: &GenericParser2,
) -> i32 {
    let handle = fx_get_new_effect_template(fx, host, Some(file));

    if handle == 0 {
        // failure
        return 0;
    }

    let base = parser.top_level();

    if let Some(pair) = base.pairs().next() {
        if pair.name().eq_ignore_ascii_case("repeatDelay") {
            let value = pair.top_value().unwrap_or("");
            fx.scheduler.mEffectTemplates[handle as usize].mRepeatDelay =
                value.trim().parse::<i32>().unwrap_or(0);
        }
    }

    let groups: Vec<GpGroup<'_>> = base.subgroups().collect();
    for group in groups {
        // Huge stricmp lists suxor
        let ty = match group.name().to_ascii_lowercase().as_str() {
            "particle" => EPrimType::Particle,
            "line" => EPrimType::Line,
            "tail" => EPrimType::Tail,
            "sound" => EPrimType::Sound,
            "cylinder" => EPrimType::Cylinder,
            "electricity" => EPrimType::Electricity,
            "emitter" => EPrimType::Emitter,
            "decal" => EPrimType::Decal,
            "orientedparticle" => EPrimType::OrientedParticle,
            "fxrunner" => EPrimType::FxRunner,
            "light" => EPrimType::Light,
            "camerashake" => EPrimType::CameraShake,
            "flash" => EPrimType::ScreenFlash,
            _ => EPrimType::None,
        };

        if ty != EPrimType::None {
            let mut prim = CPrimitiveTemplate::default();
            prim.mType = ty;
            prim.ParsePrimitive(host, fx, &group);

            // Add our primitive template to the effect list
            fx_add_primitive_to_effect(fx, host, handle, prim);
        }
    }

    handle
}

/// Raven `CFxScheduler::RegisterEffect` — open the `.efx`, parse it, hand back the handle.
///
/// A file already in the name map returns its old handle without re-reading.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:254-353`
pub fn fx_register_effect(fx: &mut FxSystem, host: &mut FxHost<'_, '_>, file: &str) -> i32 {
    let sfile = COM_StripExtension(file).to_ascii_lowercase();

    host.Print(&format!("Registering effect : {sfile}\n"));

    // see if the specified file is already registered. If it is, just return the id of that file
    if let Some(id) = fx.scheduler.mEffectIDs.get(&sfile) {
        return *id;
    }

    // if our file doesn't have an extension, add one
    let mut final_filename = file.to_string();
    if !final_filename.contains('.') {
        // didn't find an extension so add one
        final_filename.push_str(".efx");
    }

    // kef - grr. this angers me. every filename everywhere should start from the base dir
    let effects_substr: String = final_filename.chars().take(7).collect();
    if effects_substr != "effects" {
        final_filename = format!("effects/{final_filename}");
    }

    let (len, fh) = host.OpenFile(&final_filename);

    if len < 0 {
        host.Print(&format!("Effect file load failed: {final_filename}\n"));
        return 0;
    }

    if len == 0 {
        host.Print(&format!("INVALID Effect file: {final_filename}\n"));
        host.CloseFile(fh);
        return 0;
    }

    // If we'll overflow our buffer, bail out--not a particularly elegant solution
    if len as usize >= EFX_BUFFER_SIZE - 1 {
        host.CloseFile(fh);
        return 0;
    }

    // Get the goods and ensure Null termination
    let data = host.ReadFile(len as usize, fh);
    host.CloseFile(fh);

    // Raven reads the file as bytes and terminates it. The `.efx` grammar is
    // Latin-1 text, so a lossy decode keeps every byte addressable.
    let text = data.iter().map(|b| *b as char).collect::<String>();

    // Let the generic parser process the whole file
    let mut parser = GenericParser2::new();
    if parser.parse(&text, true).is_err() {
        // Raven's parser reports failure through the same empty base group, and
        // `ParseEffect` then registers an effect with no primitives.
    }

    fx_parse_effect(fx, host, &sfile, &parser)
}

/// Raven's `char data[65536]` read buffer.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:282`
const EFX_BUFFER_SIZE: usize = 65536;

/// Raven `CFxScheduler::GetEffectCopy` by handle.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:582-626`
pub fn fx_get_effect_copy(fx: &mut FxSystem, host: &mut FxHost<'_, '_>, fx_handle: i32) -> i32 {
    if fx_handle < 1 || fx_handle as usize >= FX_MAX_EFFECTS {
        // Didn't even request a valid effect to copy!!!
        host.Print(&format!(
            "FxScheduler: Bad effect file copy request: id = {fx_handle}\n"
        ));
        return 0;
    }

    if !fx.scheduler.mEffectTemplates[fx_handle as usize].mInUse {
        host.Print(&format!(
            "FxScheduler: Bad effect file copy request: id {fx_handle} not inuse\n"
        ));
        return 0;
    }

    // Copies shouldn't have names, otherwise they could trash our stl map used
    // for getting ID from name
    let new_handle = fx_get_new_effect_template(fx, host, None);

    if new_handle == 0 {
        // No space left to return an effect
        return 0;
    }

    let source = fx.scheduler.mEffectTemplates[fx_handle as usize].clone();
    let mut copy = SEffectTemplate {
        mInUse: true,
        mRepeatDelay: fx.scheduler.mEffectTemplates[new_handle as usize].mRepeatDelay,
        ..SEffectTemplate::default()
    };
    copy.copy_from(&source);
    copy.mInUse = true;
    copy.mCopy = true;
    fx.scheduler.mEffectTemplates[new_handle as usize] = copy;

    new_handle
}

/// Raven `CFxScheduler::GetPrimitiveCopy` — the named component of an effect copy.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:639-657`
pub fn fx_get_primitive_copy(
    fx: &FxSystem,
    effect_copy: i32,
    component_name: &str,
) -> Option<PrimitiveRef> {
    if effect_copy < 1 || effect_copy as usize >= FX_MAX_EFFECTS {
        return None;
    }
    let effect = &fx.scheduler.mEffectTemplates[effect_copy as usize];
    if !effect.mInUse {
        return None;
    }

    for i in 0..effect.mPrimitiveCount as usize {
        if effect.mPrimitives[i]
            .borrow()
            .mName
            .eq_ignore_ascii_case(component_name)
        {
            return Some(Rc::clone(&effect.mPrimitives[i]));
        }
    }

    None
}

/// Raven `CFxScheduler::ScheduleLoopedEffect`.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:40-86`
pub fn fx_schedule_looped_effect(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    id: i32,
    bolt_info: i32,
    i_ghoul2: i32,
    is_portal: bool,
    i_loop_time: i32,
    is_relative: bool,
) -> i32 {
    // see if it's already playing so we can just update it
    let mut i = MAX_LOOPED_FX;
    for slot in 0..MAX_LOOPED_FX {
        let e = &fx.scheduler.mLoopedEffectArray[slot];
        if e.mId == id && e.mBoltInfo == bolt_info && e.mPortalEffect == is_portal {
            i = slot;
            break;
        }
    }

    if i == MAX_LOOPED_FX {
        // didn't find it existing, so find a free spot
        for slot in 0..MAX_LOOPED_FX {
            if fx.scheduler.mLoopedEffectArray[slot].mId == 0 {
                i = slot;
                break;
            }
        }
    }

    if i == MAX_LOOPED_FX {
        let name = fx.scheduler.mEffectTemplates[id as usize]
            .mEffectName
            .clone();
        host.Print(&format!(
            "CFxScheduler::AddLoopedEffect- No Free Slots available for {name}\n"
        ));
        return -1;
    }

    let repeat = fx.scheduler.mEffectTemplates[id as usize].mRepeatDelay;
    let now = fx.clock.mTime;
    let e = &mut fx.scheduler.mLoopedEffectArray[i];
    e.mId = id;
    e.mBoltInfo = bolt_info;
    e.mGhoul2 = i_ghoul2;
    e.mPortalEffect = is_portal;
    e.mIsRelative = is_relative;
    e.mNextTime = now + repeat;
    e.mLoopStopTime = if i_loop_time == 1 {
        0
    } else {
        now + i_loop_time
    };
    i as i32
}

/// Raven `CFxScheduler::StopEffect` — drop a looped entry that matches.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:88-118`
pub fn fx_stop_effect(fx: &mut FxSystem, file: &str, bolt_info: i32, is_portal: bool) {
    // Get an extenstion stripped version of the file
    let sfile = COM_StripExtension(file);
    // Retail defines FINAL_BUILD, so the unregistered-name guard compiles out
    // and the `mEffectIDs[sfile]` lookup inserts the name with id 0 (DEC-63.3).
    let id = *fx.scheduler.mEffectIDs.entry(sfile).or_insert(0);

    for i in 0..MAX_LOOPED_FX {
        let e = &fx.scheduler.mLoopedEffectArray[i];
        if e.mId == id && e.mBoltInfo == bolt_info && e.mPortalEffect == is_portal {
            fx.scheduler.mLoopedEffectArray[i] = SLoopedEffect::default();
            return;
        }
    }
}

/// Raven `CFxScheduler::AddLoopedEffects` — replay every looped entry that is due.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:120-144`
pub fn fx_add_looped_effects(fx: &mut FxSystem, host: &mut FxHost<'_, '_>) {
    for i in 0..MAX_LOOPED_FX {
        let e = fx.scheduler.mLoopedEffectArray[i].clone();
        if e.mId != 0 && e.mNextTime < fx.clock.mTime {
            let ent_num = (e.mBoltInfo >> ENTITY_SHIFT) & ENTITY_AND;
            // Find out where the entity currently is
            let point = host.GetLerpOrigin(ent_num);

            // very important to send FALSE to not recursively add me!
            //
            // Raven hands nine arguments to an eleven-parameter signature whose
            // sixth is `fxParm`, so every later argument lands one place early:
            // the portal flag reaches `vol`, `false` reaches `rad`, and the
            // relative flag reaches `isPortal`. Every reschedule from a relative
            // loop therefore belongs to the portal pass, and only a portal pass
            // drains it.
            // Source: `oracle/codemp/client/FxScheduler.cpp:135`
            fx_play_effect_axis(
                fx,
                host,
                e.mId,
                Some(point),
                [[0.0; 3]; 3],
                e.mBoltInfo,
                e.mGhoul2,
                -1,
                e.mPortalEffect as i32,
                0,
                e.mIsRelative,
                0,
                false,
            );
            let repeat = fx.scheduler.mEffectTemplates[e.mId as usize].mRepeatDelay;
            fx.scheduler.mLoopedEffectArray[i].mNextTime = fx.clock.mTime + repeat;
            if e.mLoopStopTime != 0 && e.mLoopStopTime < fx.clock.mTime {
                // time's up, kill this entry
                fx.scheduler.mLoopedEffectArray[i] = SLoopedEffect::default();
            }
        }
    }
}

/// Raven `GetRGB_Colors` — six draws, or one plus six interpolations.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:774-790`
fn get_rgb_colors(t: &CPrimitiveTemplate, host: &mut FxHost<'_, '_>) -> (vec3_t, vec3_t) {
    if t.mSpawnFlags & FX_RGB_COMPONENT_INTERP != 0 {
        let percent = host.rng().flrand(0.0, 1.0);
        (
            [
                t.mRedStart.GetValFraction(percent),
                t.mGreenStart.GetValFraction(percent),
                t.mBlueStart.GetValFraction(percent),
            ],
            [
                t.mRedEnd.GetValFraction(percent),
                t.mGreenEnd.GetValFraction(percent),
                t.mBlueEnd.GetValFraction(percent),
            ],
        )
    } else {
        (
            [
                t.mRedStart.GetVal(host.rng()),
                t.mGreenStart.GetVal(host.rng()),
                t.mBlueStart.GetVal(host.rng()),
            ],
            [
                t.mRedEnd.GetVal(host.rng()),
                t.mGreenEnd.GetVal(host.rng()),
                t.mBlueEnd.GetVal(host.rng()),
            ],
        )
    }
}

/// Raven `CFxScheduler::PlayEffect( int id, vec3_t origin, vec3_t forward, ... )`.
///
/// Builds two arbitrary perpendicular vectors from the forward vector.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:727-736`
#[allow(clippy::too_many_arguments)]
pub fn fx_play_effect_fwd(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    id: i32,
    origin: vec3_t,
    forward: vec3_t,
    vol: i32,
    rad: i32,
    is_portal: bool,
) {
    let mut axis = [[0.0f32; 3]; 3];
    axis[0] = forward;
    let (mut right, mut up) = ([0.0f32; 3], [0.0f32; 3]);
    MakeNormalVectors(forward, &mut right, &mut up);
    axis[1] = right;
    axis[2] = up;

    fx_play_effect_axis(
        fx,
        host,
        id,
        Some(origin),
        axis,
        -1,
        0,
        -1,
        vol,
        rad,
        is_portal,
        0,
        false,
    );
}

/// Raven `CFxScheduler::PlayEffect( const char *file, vec3_t origin, vec3_t forward, ... )`.
///
/// An unregistered name maps to handle zero, and the id overload reports it.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:1042-1050`
pub fn fx_play_effect_file_fwd(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    file: &str,
    origin: vec3_t,
    forward: vec3_t,
    vol: i32,
    rad: i32,
) {
    // Get an extenstion stripped version of the file
    let sfile = COM_StripExtension(file);
    // Raven's `mEffectIDs[sfile]` inserts an unregistered name with id 0.
    let id = *fx.scheduler.mEffectIDs.entry(sfile).or_insert(0);

    fx_play_effect_fwd(fx, host, id, origin, forward, vol, rad, false);
}

/// Raven `CFxScheduler::PlayEffect( const char *file, vec3_t origin, vec3_t axis[3], ... )`.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:752-769`
#[allow(clippy::too_many_arguments)]
pub fn fx_play_effect_file_axis(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    file: &str,
    origin: Option<vec3_t>,
    axis: [vec3_t; 3],
    bolt_info: i32,
    i_ghoul2: i32,
    fx_parm: i32,
    vol: i32,
    rad: i32,
    i_loop_time: i32,
    is_relative: bool,
) {
    // Get an extenstion stripped version of the file
    let sfile = COM_StripExtension(file);

    // Retail defines FINAL_BUILD, so the guard compiles out and the id overload
    // reports an unregistered name as id 0 (DEC-63.3).
    let id = *fx.scheduler.mEffectIDs.entry(sfile).or_insert(0);

    fx_play_effect_axis(
        fx,
        host,
        id,
        origin,
        axis,
        bolt_info,
        i_ghoul2,
        fx_parm,
        vol,
        rad,
        false,
        i_loop_time,
        is_relative,
    );
}

/// Raven `CFxScheduler::PlayEffect( int id, vec3_t origin, vec3_t axis[3], ... )`.
///
/// Every primitive of the effect spawns `count` bits, each either right now or on
/// the schedule. A bolted effect always schedules, so it never plays a frame early.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:805-1005`
#[allow(clippy::too_many_arguments)]
pub fn fx_play_effect_axis(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    id: i32,
    origin: Option<vec3_t>,
    axis: [vec3_t; 3],
    bolt_info: i32,
    i_ghoul2: i32,
    fx_parm: i32,
    vol: i32,
    rad: i32,
    is_portal: bool,
    i_loop_time: i32,
    is_relative: bool,
) {
    if id < 1 || id as usize >= FX_MAX_EFFECTS || !fx.scheduler.mEffectTemplates[id as usize].mInUse
    {
        // Now you've done it!
        host.Print(&format!(
            "CFxScheduler::PlayEffect called with invalid effect ID: {id}\n"
        ));
        return;
    }

    let mut model_num = 0;
    let mut bolt_num = -1;
    let mut entity_num = -1;
    let mut force_scheduling = false;

    if bolt_info > 0 {
        // extract the wraith ID from the bolt info
        model_num = (bolt_info >> MODEL_SHIFT) & MODEL_AND;
        bolt_num = (bolt_info >> BOLT_SHIFT) & BOLT_AND;
        entity_num = (bolt_info >> ENTITY_SHIFT) & ENTITY_AND;

        // We always force ghoul bolted objects to be scheduled so that they don't play right away.
        force_scheduling = true;

        if i_loop_time != 0 {
            // 0 = not looping, 1 for infinite, else duration
            fx_schedule_looped_effect(
                fx,
                host,
                id,
                bolt_info,
                i_ghoul2,
                is_portal,
                i_loop_time,
                is_relative,
            );
        }
    }

    // The `fx_debug 2` effect-name print sits under `#ifndef FINAL_BUILD`, so
    // the retail build the FX goldens pin compiles it out.
    // Source: `oracle/codemp/client/FxScheduler.cpp:851-856`

    let prim_count = fx.scheduler.mEffectTemplates[id as usize].mPrimitiveCount as usize;

    // Loop through the primitives and schedule each bit
    for i in 0..prim_count {
        fx.totalPrimitives += 1;
        let prim = Rc::clone(&fx.scheduler.mEffectTemplates[id as usize].mPrimitives[i]);

        {
            let mut p = prim.borrow_mut();
            p.mSoundRadius = rad;
            p.mSoundVolume = vol;
        }

        let (cull_range, spawn_min, spawn_max, is_copy, spawn_flags) = {
            let p = prim.borrow();
            (
                p.mCullRange,
                p.mSpawnCount.GetMin(),
                p.mSpawnCount.GetMax(),
                p.mCopy,
                p.mSpawnFlags,
            )
        };

        if cull_range != 0 {
            let org = origin.unwrap_or([0.0; 3]);
            // cullrange gets squared on load
            if distance_squared(&org, &fx.refdef.vieworg) > cull_range as f32 {
                // is too far away
                continue;
            }
        }

        // Scale the particles based on the countscale factor. Never, ever scale
        // the particles upwards, however.
        let mut fxscale = fx.fx_countScale;
        if fxscale > 1.0 {
            fxscale = 1.0;
        }
        // Only use scalability if there is a range.
        // Temp fix until I have time to reweight all the scalability files.
        let mut count = if (spawn_max - spawn_min).abs() > 1.0 {
            round_to_int(prim.borrow().mSpawnCount.GetVal(host.rng()) * fxscale)
        } else {
            round_to_int(prim.borrow().mSpawnCount.GetVal(host.rng()))
        };
        // Make sure we have at least one particle after scaling
        if spawn_min >= 1.0 && count < 1 {
            count = 1;
        }

        if is_copy {
            // If we are a copy, we need to store a "how many references count" so
            // that we can keep the primitive template around for the correct amount of time.
            prim.borrow_mut().mRefCount = count;
        }

        let mut factor = 0.0f32;
        if spawn_flags & FX_EVEN_DISTRIBUTION != 0 {
            let p = prim.borrow();
            factor = (p.mSpawnDelay.GetMax() - p.mSpawnDelay.GetMin()).abs() / count as f32;
        }

        // Schedule the random number of bits
        for t in 0..count {
            fx.totalEffects += 1;
            let delay = if spawn_flags & FX_EVEN_DISTRIBUTION != 0 {
                (t as f32 * factor) as i32
            } else {
                prim.borrow().mSpawnDelay.GetVal(host.rng()) as i32
            };

            // if the delay is so small, we may as well just create this bit right now
            if delay < 1 && !force_scheduling && !is_portal {
                if bolt_info == -1 && entity_num != -1 {
                    // Find out where the entity currently is
                    let point = host.GetLerpOrigin(entity_num);
                    fx_create_effect(fx, host, &prim, point, axis, -delay, fx_parm, 0, -1, -1, -1);
                } else {
                    let org = origin.unwrap_or([0.0; 3]);
                    fx_create_effect(fx, host, &prim, org, axis, -delay, fx_parm, 0, -1, -1, -1);
                }
            } else {
                // We have to create a new scheduled effect so that we can create it
                // at a later point. You should avoid this because it's much more expensive.
                let mut sfx = SScheduledEffect {
                    mpTemplate: Rc::clone(&prim),
                    mStartTime: fx.clock.mTime + delay,
                    mModelNum: 0,
                    mBoltNum: -1,
                    mEntNum: ENTITYNUM_NONE as i16,
                    mPortalEffect: is_portal,
                    mIsRelative: is_relative,
                    iGhoul2: 0,
                    mOrigin: [0.0; 3],
                    mAxis: axis,
                };

                if bolt_info == -1 {
                    sfx.iGhoul2 = 0;
                    if entity_num == -1 {
                        // we aren't bolting, so make sure the spawn system knows this
                        // by putting -1's in these fields
                        sfx.mBoltNum = -1;
                        sfx.mEntNum = ENTITYNUM_NONE as i16;
                        sfx.mModelNum = 0;
                        sfx.mOrigin = origin.unwrap_or([0.0; 3]);
                    } else {
                        // we are doing bolting onto the origin of the entity, so use
                        // a cheaper method
                        sfx.mBoltNum = -1;
                        sfx.mEntNum = entity_num as i16;
                        sfx.mModelNum = 0;
                    }
                } else {
                    // we are bolting, so store the extra info
                    sfx.mBoltNum = bolt_num as i8;
                    sfx.mEntNum = entity_num as i16;
                    sfx.mModelNum = model_num as i8;
                    sfx.iGhoul2 = i_ghoul2;

                    // Also, the ghoul bolt may not be around yet, so delay the creation one frame
                    sfx.mStartTime += 1;
                }

                fx.scheduler.mFxSchedule.push_front(sfx);
            }
        }
    }

    // We track effect templates and primitive templates separately.
    if fx.scheduler.mEffectTemplates[id as usize].mCopy {
        // We don't use dynamic memory allocation, so just mark us as dead
        fx.scheduler.mEffectTemplates[id as usize].mInUse = false;
    }
}

/// Raven `CFxScheduler::AddScheduledEffects` — spawn everything that came due, then draw.
///
/// A spawn made during this pass lands at the front of the schedule, which
/// Raven's list iteration never revisits in the same frame.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:1066-1161`
pub fn fx_add_scheduled_effects(fx: &mut FxSystem, host: &mut FxHost<'_, '_>, portal: bool) {
    let mut old_ent_num = -1;
    let mut old_bolt_index = -1;
    let mut old_model_num = -1;
    let mut bolt: Option<(vec3_t, [vec3_t; 3])> = None;

    if portal {
        fx.gEffectsInPortal = true;
    } else {
        fx_add_looped_effects(fx, host);
    }

    let pass: Vec<SScheduledEffect> = core::mem::take(&mut fx.scheduler.mFxSchedule).into();
    let mut kept: Vec<SScheduledEffect> = Vec::new();

    for item in pass.into_iter() {
        // only render portal fx on the skyportal pass and vice versa
        if portal != item.mPortalEffect || item.mStartTime > fx.clock.mTime {
            kept.push(item);
            continue;
        }

        if item.mBoltNum == -1 {
            // ok, are we spawning a bolt on effect or a normal one?
            if item.mEntNum as i32 != ENTITYNUM_NONE {
                // Find out where the entity currently is
                let point = host.GetLerpOrigin(item.mEntNum as i32);
                let late = fx.clock.mTime - item.mStartTime;
                fx_create_effect(
                    fx,
                    host,
                    &item.mpTemplate,
                    point,
                    item.mAxis,
                    late,
                    -1,
                    0,
                    -1,
                    -1,
                    -1,
                );
            } else {
                let late = fx.clock.mTime - item.mStartTime;
                fx_create_effect(
                    fx,
                    host,
                    &item.mpTemplate,
                    item.mOrigin,
                    item.mAxis,
                    late,
                    -1,
                    0,
                    -1,
                    -1,
                    -1,
                );
            }
        } else {
            // bolted on effect. Re-getting the bolt matrix costs time, so do it only once.
            if item.mModelNum as i32 != old_model_num
                || item.mEntNum as i32 != old_ent_num
                || item.mBoltNum as i32 != old_bolt_index
            {
                old_model_num = item.mModelNum as i32;
                old_ent_num = item.mEntNum as i32;
                old_bolt_index = item.mBoltNum as i32;
                let old_time = fx.clock.mOldTime;
                bolt = host.GetOriginAxisFromBolt(
                    item.iGhoul2,
                    item.mEntNum as i32,
                    item.mModelNum as i32,
                    item.mBoltNum as i32,
                    old_time,
                );
            }

            // only do this if we found the bolt
            if let Some((origin, ax)) = bolt {
                if item.mIsRelative {
                    fx_create_effect(
                        fx,
                        host,
                        &item.mpTemplate,
                        origin,
                        ax,
                        0,
                        -1,
                        item.iGhoul2,
                        item.mEntNum as i32,
                        item.mModelNum as i32,
                        item.mBoltNum as i32,
                    );
                } else {
                    let late = fx.clock.mTime - item.mStartTime;
                    fx_create_effect(
                        fx,
                        host,
                        &item.mpTemplate,
                        origin,
                        ax,
                        late,
                        -1,
                        0,
                        -1,
                        -1,
                        -1,
                    );
                }
            }
        }
    }

    // Anything a spawn pushed during the pass is already at the front.
    fx.scheduler.mFxSchedule.extend(kept);

    // Add all active effects into the scene
    FX_Add(fx, host, portal);

    fx.gEffectsInPortal = false;
}

/// Raven `CFxScheduler::Add2DEffect`.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:1163-1181`
pub fn fx_add_2d_effect(
    fx: &mut FxSystem,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: [f32; 4],
    shader_handle: i32,
) -> bool {
    // need some sort of scale here because the effect was created using world
    // units, not pixels
    let fx_scale_2d = 10.0f32;

    if fx.scheduler.mNextFree2DEffect < FX_MAX_2DEFFECTS {
        let slot = &mut fx.scheduler.m2DEffects[fx.scheduler.mNextFree2DEffect];
        slot.mScreenX = x;
        slot.mScreenY = y;
        slot.mWidth = w * fx_scale_2d;
        slot.mHeight = h * fx_scale_2d;
        slot.mColor = color;
        slot.mShaderHandle = shader_handle;

        fx.scheduler.mNextFree2DEffect += 1;
        return true;
    }
    false
}

/// Raven `CFxScheduler::Draw2DEffects` — flush the screen-space list.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:1183-1204`
pub fn fx_draw_2d_effects(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    screen_x_scale: f32,
    screen_y_scale: f32,
) {
    for i in 0..fx.scheduler.mNextFree2DEffect {
        let e = fx.scheduler.m2DEffects[i];
        let mut x = e.mScreenX;
        let mut y = e.mScreenY;
        let mut w = e.mWidth;
        let mut h = e.mHeight;

        x *= screen_x_scale;
        w *= screen_x_scale;
        y *= screen_y_scale;
        h *= screen_y_scale;

        host.DrawStretchPic(x - (w * 0.5), y - (h * 0.5), w, h, e.mShaderHandle);
    }
    // now that all 2D effects have been drawn we can consider the entire array to be free
    fx.scheduler.mNextFree2DEffect = 0;
}

/// Raven `CFxScheduler::CreateEffect` — turn one primitive template into one live primitive.
///
/// Every `GetVal` call below draws a random number, so the call order is parity
/// surface. Do not hoist one.
///
/// Source: `oracle/codemp/client/FxScheduler.cpp:1219-1727`
#[allow(clippy::too_many_arguments)]
pub fn fx_create_effect(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    prim: &PrimitiveRef,
    origin: vec3_t,
    axis: [vec3_t; 3],
    late_time: i32,
    fx_parm: i32,
    i_ghoul2: i32,
    ent_num: i32,
    model_num: i32,
    bolt_num: i32,
) {
    // CreateEffect never writes the template, so a snapshot keeps the shared
    // handle free for a recursive spawn of the same effect.
    let t = prim.borrow().clone();

    // We may modify the axis, so make a work copy
    let mut ax = axis;

    let mut flags = t.mFlags;
    if i_ghoul2 > 0 && model_num >= 0 && bolt_num >= 0 {
        // since you passed in these values, mark as relative to use them if it is supported
        match t.mType {
            EPrimType::Particle
            | EPrimType::Line
            | EPrimType::Tail
            | EPrimType::Electricity
            | EPrimType::Cylinder
            | EPrimType::Emitter
            | EPrimType::OrientedParticle
            | EPrimType::Light => flags |= FX_RELATIVE,
            // decals, fx runners, and flashes are not supported yet; sounds and
            // camera shakes do not work bolted
            _ => {}
        }
    }

    if t.mSpawnFlags & FX_RAND_ROT_AROUND_FWD != 0 {
        let degrees = host.rng().flrand(0.0, 360.0);
        let mut rotated = [0.0f32; 3];
        RotatePointAroundVector(&mut rotated, ax[0], axis[1], degrees);
        ax[1] = rotated;
        let mut cross = [0.0f32; 3];
        CrossProduct(ax[0], ax[1], &mut cross);
        ax[2] = cross;
    }

    // Origin calculations
    let mut org: vec3_t;
    if t.mSpawnFlags & FX_CHEAP_ORG_CALC != 0 || flags & FX_RELATIVE != 0 {
        // let's take the easy way out
        org = [
            t.mOrigin1X.GetVal(host.rng()),
            t.mOrigin1Y.GetVal(host.rng()),
            t.mOrigin1Z.GetVal(host.rng()),
        ];
    } else {
        // time for some extra work
        //
        // `VectorScale` and `VectorMA` are macros, so each `GetVal` below runs
        // once per component. A ranged origin draws three times per axis.
        // Source: `oracle/codemp/game/q_shared.h:1361,1365`
        org = [0.0; 3];
        for i in 0..3 {
            org[i] = ax[0][i] * t.mOrigin1X.GetVal(host.rng());
        }
        for i in 0..3 {
            org[i] += ax[1][i] * t.mOrigin1Y.GetVal(host.rng());
        }
        for i in 0..3 {
            org[i] += ax[2][i] * t.mOrigin1Z.GetVal(host.rng());
        }
    }

    // We always add our calculated offset to the passed in origin, unless relative!
    if flags & FX_RELATIVE == 0 {
        org = vector_add(&org, &origin);
    }

    // Now, we may need to calc a point on a sphere/ellipsoid/cylinder/disk and add that to it
    if t.mSpawnFlags & FX_ORG_ON_SPHERE != 0 {
        let x = deg2rad(host.rng().flrand(0.0, 360.0));
        let y = deg2rad(host.rng().flrand(0.0, 180.0));

        let width = t.mRadius.GetVal(host.rng());
        let height = t.mHeight.GetVal(host.rng());

        // calculate point on ellipse
        let temp: vec3_t = [
            x.sin() * width * y.sin(),
            x.cos() * width * y.sin(),
            y.cos() * height,
        ];
        org = vector_add(&org, &temp);

        if t.mSpawnFlags & FX_AXIS_FROM_SPHERE != 0 {
            // well, we will now override the axis at the users request
            let mut normal = [0.0f32; 3];
            VectorNormalize2(temp, &mut normal);
            ax[0] = normal;
            let (mut right, mut up) = ([0.0f32; 3], [0.0f32; 3]);
            MakeNormalVectors(ax[0], &mut right, &mut up);
            ax[1] = right;
            ax[2] = up;
        }
    } else if t.mSpawnFlags & FX_ORG_ON_CYLINDER != 0 {
        // set up our point, then rotate around the current direction to.
        // Make unrotated cylinder centered around 0,0,0
        //
        // Raven's `VectorScale` and `VectorMA` are macros that expand their
        // scale argument once per component, so each draw below happens three
        // times. Hoisting either one changes the generator stream.
        // Source: `oracle/codemp/game/q_shared.h:1361,1365`
        let mut pt: vec3_t = [0.0; 3];
        for i in 0..3 {
            pt[i] = ax[1][i] * t.mRadius.GetVal(host.rng());
        }
        for i in 0..3 {
            let scale = host.rng().flrand(-1.0, 1.0) * 0.5 * t.mHeight.GetVal(host.rng());
            pt[i] += ax[0][i] * scale;
        }
        let degrees = host.rng().flrand(0.0, 360.0);
        let mut temp = [0.0f32; 3];
        RotatePointAroundVector(&mut temp, ax[0], pt, degrees);

        org = vector_add(&org, &temp);

        if t.mSpawnFlags & FX_AXIS_FROM_SPHERE != 0 {
            let mut up: vec3_t = [0.0, 0.0, 1.0];

            // well, we will now override the axis at the users request
            let mut normal = [0.0f32; 3];
            VectorNormalize2(temp, &mut normal);
            ax[0] = normal;

            if ax[0][2] == 1.0 {
                // readjust up
                up = [0.0, 1.0, 0.0];
            }

            let mut right = [0.0f32; 3];
            CrossProduct(up, ax[0], &mut right);
            ax[1] = right;
            let mut third = [0.0f32; 3];
            CrossProduct(ax[0], ax[1], &mut third);
            ax[2] = third;
        }
    }

    if t.mType == EPrimType::OrientedParticle && flags & FX_RELATIVE != 0 {
        // bolted oriented particles use origin2 as an angular rotation offset
        ax[0] = [
            t.mOrigin2X.GetVal(host.rng()),
            t.mOrigin2Y.GetVal(host.rng()),
            t.mOrigin2Z.GetVal(host.rng()),
        ];
    }

    // There are only a few types that really use velocity and acceleration
    let mut vel: vec3_t = [0.0; 3];
    let mut accel: vec3_t = [0.0; 3];
    if matches!(
        t.mType,
        EPrimType::Particle | EPrimType::OrientedParticle | EPrimType::Tail | EPrimType::Emitter
    ) {
        if t.mSpawnFlags & FX_VEL_IS_ABSOLUTE != 0 || flags & FX_RELATIVE != 0 {
            vel = [
                t.mVelX.GetVal(host.rng()),
                t.mVelY.GetVal(host.rng()),
                t.mVelZ.GetVal(host.rng()),
            ];
        } else {
            // bah, do some extra work to coerce it. The two macros expand their
            // scale argument once per component, so a ranged velocity draws
            // three times per axis.
            // Source: `oracle/codemp/game/q_shared.h:1361,1365`
            for i in 0..3 {
                vel[i] = ax[0][i] * t.mVelX.GetVal(host.rng());
            }
            for i in 0..3 {
                vel[i] += ax[1][i] * t.mVelY.GetVal(host.rng());
            }
            for i in 0..3 {
                vel[i] += ax[2][i] * t.mVelZ.GetVal(host.rng());
            }
        }

        // Raven's wind query is commented out in the oracle, so `FX_AFFECTED_BY_WIND`
        // changes nothing.

        if t.mSpawnFlags & FX_ACCEL_IS_ABSOLUTE != 0 || flags & FX_RELATIVE != 0 {
            accel = [
                t.mAccelX.GetVal(host.rng()),
                t.mAccelY.GetVal(host.rng()),
                t.mAccelZ.GetVal(host.rng()),
            ];
        } else {
            // The same macro expansion applies here.
            // Source: `oracle/codemp/game/q_shared.h:1361,1365`
            for i in 0..3 {
                accel[i] = ax[0][i] * t.mAccelX.GetVal(host.rng());
            }
            for i in 0..3 {
                accel[i] += ax[1][i] * t.mAccelY.GetVal(host.rng());
            }
            for i in 0..3 {
                accel[i] += ax[2][i] * t.mAccelZ.GetVal(host.rng());
            }
        }

        // Gravity is completely decoupled from acceleration since it is __always__
        // absolute. NOTE: I only effect Z ( up/down in the Quake world )
        accel[2] += t.mGravity.GetVal(host.rng());

        // There may be a lag between when the effect should be created and when it
        // actually gets created. Since we know what the discrepancy is, we can
        // attempt to compensate.
        if late_time > 0 {
            // Calc the time differences
            let ftime = late_time as f32 * 0.001;
            let time2 = ftime * ftime * 0.5;

            vector_ma_in_place(&mut vel, ftime, &accel);

            // Predict the new position
            for i in 0..3 {
                org[i] = org[i] + ftime * vel[i] + time2 * vel[i];
            }
        }
    }

    // Line type primitives work with an origin2
    let mut org2: vec3_t = [0.0; 3];
    if t.mType == EPrimType::Line || t.mType == EPrimType::Electricity {
        // We may have to do a trace to find our endpoint
        if t.mSpawnFlags & FX_ORG2_FROM_TRACE != 0 {
            let mut temp = org;
            vector_ma_in_place(&mut temp, FX_MAX_TRACE_DIST, &ax[0]);

            if t.mSpawnFlags & FX_ORG2_IS_OFFSET != 0 {
                // add a random flair to the endpoint. Note: org2 will have to be
                // pretty large to affect this much. We also do this pre-trace, since
                // we may have to render an impact effect and we want the normal at
                // the exact endpos.
                if t.mSpawnFlags & FX_CHEAP_ORG2_CALC != 0 || flags & FX_RELATIVE != 0 {
                    org2 = [
                        t.mOrigin2X.GetVal(host.rng()),
                        t.mOrigin2Y.GetVal(host.rng()),
                        t.mOrigin2Z.GetVal(host.rng()),
                    ];
                    temp = vector_add(&org2, &temp);
                } else {
                    // I can only imagine a few cases where you might want to do this
                    let x = t.mOrigin2X.GetVal(host.rng());
                    vector_ma_in_place(&mut temp, x, &ax[0]);
                    let y = t.mOrigin2Y.GetVal(host.rng());
                    vector_ma_in_place(&mut temp, y, &ax[1]);
                    let z = t.mOrigin2Z.GetVal(host.rng());
                    vector_ma_in_place(&mut temp, z, &ax[2]);
                }
            }

            let tr = host.Trace(org, None, None, temp, -1, MASK_SOLID, false);

            org2 = tr.endpos;

            if t.mSpawnFlags & FX_TRACE_IMPACT_FX != 0 {
                let handle = t.mImpactFxHandles.GetHandle(host.rng());
                fx_play_effect_fwd(fx, host, handle, org2, tr.plane.normal, -1, -1, false);
            }
        } else {
            if t.mSpawnFlags & FX_CHEAP_ORG2_CALC != 0 || flags & FX_RELATIVE != 0 {
                org2 = [
                    t.mOrigin2X.GetVal(host.rng()),
                    t.mOrigin2Y.GetVal(host.rng()),
                    t.mOrigin2Z.GetVal(host.rng()),
                ];
            } else {
                org2 = vector_scale(&ax[0], t.mOrigin2X.GetVal(host.rng()));
                let y = t.mOrigin2Y.GetVal(host.rng());
                vector_ma_in_place(&mut org2, y, &ax[1]);
                let z = t.mOrigin2Z.GetVal(host.rng());
                vector_ma_in_place(&mut org2, z, &ax[2]);
            }
            if flags & FX_RELATIVE == 0 {
                org2 = vector_add(&org2, &origin);
            }
        }
    }

    // handle RGB color, but only for types that will use it
    let (s_rgb, e_rgb) = if t.mType != EPrimType::Sound
        && t.mType != EPrimType::FxRunner
        && t.mType != EPrimType::CameraShake
    {
        get_rgb_colors(&t, host)
    } else {
        ([0.0; 3], [0.0; 3])
    };

    // Now create the appropriate effect entity
    match t.mType {
        EPrimType::Particle => {
            // The draws run in one tuple so the call itself does not hold a second
            // borrow of the host. The evaluation order is Raven's argument order.
            let (
                size1,
                size2,
                size_parm,
                alpha1,
                alpha2,
                alpha_parm,
                rgb_parm,
                rotation,
                rotation_delta,
                elasticity,
                death_id,
                impact_id,
                life,
                media,
            ) = (
                t.mSizeStart.GetVal(host.rng()),
                t.mSizeEnd.GetVal(host.rng()),
                t.mSizeParm.GetVal(host.rng()),
                t.mAlphaStart.GetVal(host.rng()),
                t.mAlphaEnd.GetVal(host.rng()),
                t.mAlphaParm.GetVal(host.rng()),
                t.mRGBParm.GetVal(host.rng()),
                t.mRotation.GetVal(host.rng()),
                t.mRotationDelta.GetVal(host.rng()),
                t.mElasticity.GetVal(host.rng()),
                t.mDeathFxHandles.GetHandle(host.rng()),
                t.mImpactFxHandles.GetHandle(host.rng()),
                t.mLife.GetVal(host.rng()) as i32,
                t.mMediaHandles.GetHandle(host.rng()),
            );
            FX_AddParticle(
                fx,
                host,
                org,
                vel,
                accel,
                size1,
                size2,
                size_parm,
                alpha1,
                alpha2,
                alpha_parm,
                s_rgb,
                e_rgb,
                rgb_parm,
                rotation,
                rotation_delta,
                t.mMin,
                t.mMax,
                elasticity,
                death_id,
                impact_id,
                life,
                media,
                flags,
                t.mMatImpactFX,
                fx_parm,
                i_ghoul2,
                ent_num,
                model_num,
                bolt_num,
            );
        }
        EPrimType::Line => {
            let (size1, size2, size_parm, alpha1, alpha2, alpha_parm, rgb_parm, life, media) = (
                t.mSizeStart.GetVal(host.rng()),
                t.mSizeEnd.GetVal(host.rng()),
                t.mSizeParm.GetVal(host.rng()),
                t.mAlphaStart.GetVal(host.rng()),
                t.mAlphaEnd.GetVal(host.rng()),
                t.mAlphaParm.GetVal(host.rng()),
                t.mRGBParm.GetVal(host.rng()),
                t.mLife.GetVal(host.rng()) as i32,
                t.mMediaHandles.GetHandle(host.rng()),
            );
            FX_AddLine(
                fx,
                host,
                org,
                org2,
                size1,
                size2,
                size_parm,
                alpha1,
                alpha2,
                alpha_parm,
                s_rgb,
                e_rgb,
                rgb_parm,
                life,
                media,
                flags,
                t.mMatImpactFX,
                fx_parm,
                i_ghoul2,
                ent_num,
                model_num,
                bolt_num,
            );
        }
        EPrimType::Tail => {
            let (
                size1,
                size2,
                size_parm,
                length1,
                length2,
                length_parm,
                alpha1,
                alpha2,
                alpha_parm,
                rgb_parm,
                elasticity,
                death_id,
                impact_id,
                life,
                media,
            ) = (
                t.mSizeStart.GetVal(host.rng()),
                t.mSizeEnd.GetVal(host.rng()),
                t.mSizeParm.GetVal(host.rng()),
                t.mLengthStart.GetVal(host.rng()),
                t.mLengthEnd.GetVal(host.rng()),
                t.mLengthParm.GetVal(host.rng()),
                t.mAlphaStart.GetVal(host.rng()),
                t.mAlphaEnd.GetVal(host.rng()),
                t.mAlphaParm.GetVal(host.rng()),
                t.mRGBParm.GetVal(host.rng()),
                t.mElasticity.GetVal(host.rng()),
                t.mDeathFxHandles.GetHandle(host.rng()),
                t.mImpactFxHandles.GetHandle(host.rng()),
                t.mLife.GetVal(host.rng()) as i32,
                t.mMediaHandles.GetHandle(host.rng()),
            );
            FX_AddTail(
                fx,
                host,
                org,
                vel,
                accel,
                size1,
                size2,
                size_parm,
                length1,
                length2,
                length_parm,
                alpha1,
                alpha2,
                alpha_parm,
                s_rgb,
                e_rgb,
                rgb_parm,
                t.mMin,
                t.mMax,
                elasticity,
                death_id,
                impact_id,
                life,
                media,
                flags,
                t.mMatImpactFX,
                fx_parm,
                i_ghoul2,
                ent_num,
                model_num,
                bolt_num,
            );
        }
        EPrimType::Electricity => {
            let (size1, size2, size_parm, alpha1, alpha2, alpha_parm, rgb_parm, chaos, life, media) = (
                t.mSizeStart.GetVal(host.rng()),
                t.mSizeEnd.GetVal(host.rng()),
                t.mSizeParm.GetVal(host.rng()),
                t.mAlphaStart.GetVal(host.rng()),
                t.mAlphaEnd.GetVal(host.rng()),
                t.mAlphaParm.GetVal(host.rng()),
                t.mRGBParm.GetVal(host.rng()),
                t.mElasticity.GetVal(host.rng()),
                t.mLife.GetVal(host.rng()) as i32,
                t.mMediaHandles.GetHandle(host.rng()),
            );
            FX_AddElectricity(
                fx,
                host,
                org,
                org2,
                size1,
                size2,
                size_parm,
                alpha1,
                alpha2,
                alpha_parm,
                s_rgb,
                e_rgb,
                rgb_parm,
                chaos,
                life,
                media,
                flags,
                t.mMatImpactFX,
                fx_parm,
                i_ghoul2,
                ent_num,
                model_num,
                bolt_num,
            );
        }
        EPrimType::Cylinder => {
            let trace_end = t.mSpawnFlags & FX_ORG2_FROM_TRACE != 0;
            let (
                size1s,
                size1e,
                size1_parm,
                size2s,
                size2e,
                size2_parm,
                length1,
                length2,
                length_parm,
                alpha1,
                alpha2,
                alpha_parm,
                rgb_parm,
                life,
                media,
            ) = (
                t.mSizeStart.GetVal(host.rng()),
                t.mSizeEnd.GetVal(host.rng()),
                t.mSizeParm.GetVal(host.rng()),
                t.mSize2Start.GetVal(host.rng()),
                t.mSize2End.GetVal(host.rng()),
                t.mSize2Parm.GetVal(host.rng()),
                t.mLengthStart.GetVal(host.rng()),
                t.mLengthEnd.GetVal(host.rng()),
                t.mLengthParm.GetVal(host.rng()),
                t.mAlphaStart.GetVal(host.rng()),
                t.mAlphaEnd.GetVal(host.rng()),
                t.mAlphaParm.GetVal(host.rng()),
                t.mRGBParm.GetVal(host.rng()),
                t.mLife.GetVal(host.rng()) as i32,
                t.mMediaHandles.GetHandle(host.rng()),
            );
            FX_AddCylinder(
                fx,
                host,
                org,
                ax[0],
                size1s,
                size1e,
                size1_parm,
                size2s,
                size2e,
                size2_parm,
                length1,
                length2,
                length_parm,
                alpha1,
                alpha2,
                alpha_parm,
                s_rgb,
                e_rgb,
                rgb_parm,
                life,
                media,
                flags,
                t.mMatImpactFX,
                fx_parm,
                i_ghoul2,
                ent_num,
                model_num,
                bolt_num,
                trace_end,
            );
        }
        EPrimType::Emitter => {
            // for chunk angles, you don't really need much control over the end
            // result, you just want variation
            let mut ang: vec3_t = [
                t.mAngle1.GetVal(host.rng()),
                t.mAngle2.GetVal(host.rng()),
                t.mAngle3.GetVal(host.rng()),
            ];

            let mut temp = [0.0f32; 3];
            vectoangles(ax[0], &mut temp);
            ang = vector_add(&ang, &temp);

            let ang_delta: vec3_t = [
                t.mAngle1Delta.GetVal(host.rng()),
                t.mAngle2Delta.GetVal(host.rng()),
                t.mAngle3Delta.GetVal(host.rng()),
            ];

            let emitter_model = t.mMediaHandles.GetHandle(host.rng());

            let (
                size1,
                size2,
                size_parm,
                alpha1,
                alpha2,
                alpha_parm,
                rgb_parm,
                elasticity,
                death_id,
                impact_id,
                emitter_id,
                density,
                variance,
                life,
            ) = (
                t.mSizeStart.GetVal(host.rng()),
                t.mSizeEnd.GetVal(host.rng()),
                t.mSizeParm.GetVal(host.rng()),
                t.mAlphaStart.GetVal(host.rng()),
                t.mAlphaEnd.GetVal(host.rng()),
                t.mAlphaParm.GetVal(host.rng()),
                t.mRGBParm.GetVal(host.rng()),
                t.mElasticity.GetVal(host.rng()),
                t.mDeathFxHandles.GetHandle(host.rng()),
                t.mImpactFxHandles.GetHandle(host.rng()),
                t.mEmitterFxHandles.GetHandle(host.rng()),
                t.mDensity.GetVal(host.rng()),
                t.mVariance.GetVal(host.rng()),
                t.mLife.GetVal(host.rng()) as i32,
            );
            FX_AddEmitter(
                fx,
                host,
                org,
                vel,
                accel,
                size1,
                size2,
                size_parm,
                alpha1,
                alpha2,
                alpha_parm,
                s_rgb,
                e_rgb,
                rgb_parm,
                ang,
                ang_delta,
                t.mMin,
                t.mMax,
                elasticity,
                death_id,
                impact_id,
                emitter_id,
                density,
                variance,
                life,
                emitter_model,
                flags,
                t.mMatImpactFX,
                fx_parm,
            );
        }
        EPrimType::Decal => {
            let shader = t.mMediaHandles.GetHandle(host.rng());
            let rotation = t.mRotation.GetVal(host.rng());
            let alpha = t.mAlphaStart.GetVal(host.rng());
            let size = t.mSizeStart.GetVal(host.rng());
            host.AddDecalToScene(
                shader, org, ax[0], rotation, s_rgb[0], s_rgb[1], s_rgb[2], alpha, true, size,
                false,
            );

            if t.mFlags & FX_GHOUL2_DECALS != 0 {
                let g2shader = t.mMediaHandles.GetHandle(host.rng());
                let g2size = t.mSizeStart.GetVal(host.rng());
                host.AddGhoul2Decal(g2shader, org, ax[0], g2size);
            }
        }
        EPrimType::OrientedParticle => {
            let (
                size1,
                size2,
                size_parm,
                alpha1,
                alpha2,
                alpha_parm,
                rgb_parm,
                rotation,
                rotation_delta,
                bounce,
                death_id,
                impact_id,
                life,
                media,
            ) = (
                t.mSizeStart.GetVal(host.rng()),
                t.mSizeEnd.GetVal(host.rng()),
                t.mSizeParm.GetVal(host.rng()),
                t.mAlphaStart.GetVal(host.rng()),
                t.mAlphaEnd.GetVal(host.rng()),
                t.mAlphaParm.GetVal(host.rng()),
                t.mRGBParm.GetVal(host.rng()),
                t.mRotation.GetVal(host.rng()),
                t.mRotationDelta.GetVal(host.rng()),
                t.mElasticity.GetVal(host.rng()),
                t.mDeathFxHandles.GetHandle(host.rng()),
                t.mImpactFxHandles.GetHandle(host.rng()),
                t.mLife.GetVal(host.rng()) as i32,
                t.mMediaHandles.GetHandle(host.rng()),
            );
            FX_AddOrientedParticle(
                fx,
                host,
                org,
                ax[0],
                vel,
                accel,
                size1,
                size2,
                size_parm,
                alpha1,
                alpha2,
                alpha_parm,
                s_rgb,
                e_rgb,
                rgb_parm,
                rotation,
                rotation_delta,
                t.mMin,
                t.mMax,
                bounce,
                death_id,
                impact_id,
                life,
                media,
                flags,
                t.mMatImpactFX,
                fx_parm,
                i_ghoul2,
                ent_num,
                model_num,
                bolt_num,
            );
        }
        EPrimType::Sound => {
            let handle = t.mMediaHandles.GetHandle(host.rng());
            if fx.gEffectsInPortal {
                // could orient this anyway for panning, but eh. It's going to appear
                // to the player in the sky the same place no matter what, so just make
                // it a local sound.
                host.PlayLocalSound(handle, CHAN_AUTO);
            } else {
                let mut sound_org = org;
                host.PlaySound(
                    &mut sound_org,
                    ENTITYNUM_NONE,
                    CHAN_AUTO,
                    handle,
                    t.mSoundVolume,
                    t.mSoundRadius,
                );
            }
        }
        EPrimType::FxRunner => {
            let handle = t.mPlayFxHandles.GetHandle(host.rng());
            fx_play_effect_axis(
                fx,
                host,
                handle,
                Some(org),
                ax,
                -1,
                0,
                -1,
                -1,
                -1,
                false,
                0,
                false,
            );
        }
        EPrimType::Light => {
            let (size1, size2, size_parm, rgb_parm, life) = (
                t.mSizeStart.GetVal(host.rng()),
                t.mSizeEnd.GetVal(host.rng()),
                t.mSizeParm.GetVal(host.rng()),
                t.mRGBParm.GetVal(host.rng()),
                t.mLife.GetVal(host.rng()) as i32,
            );
            FX_AddLight(
                fx,
                host,
                org,
                size1,
                size2,
                size_parm,
                s_rgb,
                e_rgb,
                rgb_parm,
                life,
                flags,
                t.mMatImpactFX,
                fx_parm,
                i_ghoul2,
                ent_num,
                model_num,
                bolt_num,
            );
        }
        EPrimType::CameraShake => {
            // Elasticity is the intensity, radius is the distance the shake reaches,
            // and life is how long it lasts.
            let intensity = t.mElasticity.GetVal(host.rng());
            let radius = t.mRadius.GetVal(host.rng()) as i32;
            let time = t.mLife.GetVal(host.rng()) as i32;
            host.CameraShake(org, intensity, radius, time);
        }
        EPrimType::ScreenFlash => {
            let (size1, size2, size_parm, alpha1, alpha2, alpha_parm, rgb_parm, life, media) = (
                t.mSizeStart.GetVal(host.rng()),
                t.mSizeEnd.GetVal(host.rng()),
                t.mSizeParm.GetVal(host.rng()),
                t.mAlphaStart.GetVal(host.rng()),
                t.mAlphaEnd.GetVal(host.rng()),
                t.mAlphaParm.GetVal(host.rng()),
                t.mRGBParm.GetVal(host.rng()),
                t.mLife.GetVal(host.rng()) as i32,
                t.mMediaHandles.GetHandle(host.rng()),
            );
            FX_AddFlash(
                fx,
                host,
                org,
                size1,
                size2,
                size_parm,
                alpha1,
                alpha2,
                alpha_parm,
                s_rgb,
                e_rgb,
                rgb_parm,
                life,
                media,
                flags,
                t.mMatImpactFX,
                fx_parm,
            );
        }
        EPrimType::None => {}
    }

    // Track when we need to clean ourselves up if we are a copy
    if t.mCopy {
        let mut p = prim.borrow_mut();
        p.mRefCount -= 1;
        // Raven frees the primitive here. The shared handle drops with its last
        // holder instead, which is the same lifetime.
    }
}

/// Raven's `DEG2RAD` macro.
fn deg2rad(degrees: f32) -> f32 {
    // Raven's `M_PI` is a double, so the multiply and the divide both happen at
    // double width and round to float once, at the store.
    ((degrees as f64 * core::f64::consts::PI) / 180.0) as f32
}

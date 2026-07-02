#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_ulong, c_void};

use sp_qshared::common::sp::gentity::gentity_t;
use sp_qshared::common::sp::ghoul2::crag_doll_params::CRagDollParams;
use sp_qshared::common::sp::ghoul2::eg2_collision::EG2_Collision;
use sp_qshared::common::sp::qcommon::collision_record::CCollisionRecord;
use sp_qshared::common::sp::qcommon::shared_set_bone_ik_state_params::sharedSetBoneIKStateParams_t;
use sp_qshared::common::sp::qcommon::tags::memtag_t;
use sp_qshared::common::sp::trace_t::trace_t;
use sp_qshared::shared::{
    fileHandle_t, fsMode_t, mdxaBone_t, qboolean, qhandle_t, sharedIKMoveParams_t, vec3_t,
    Eorientations,
};

use crate::cgame::types::CGhoul2Info_v;

/// Raven `game_import_t` — engine import table handed to the statically-linked SP `jagame` module.
///
/// Raven: general Quake services, savegame, server, collision, memory, and Ghoul2/weather calls
/// the SP server-game code needs from the engine. Like SP's UI (see `uiimport_t`), `jagame` is
/// linked directly into the engine binary rather than routed through a VM syscall table, so this
/// is a plain function-pointer struct.
/// Type definition source: `oracle/oracle/code/game/g_public.h:168-471`
#[repr(C)]
pub struct game_import_t {
    //============== general Quake services ==================

    /// print message on the local console
    //TODO: Port Printf variadic args
    // Source: oracle/oracle/code/game/g_public.h:172
    pub Printf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,

    /// Write a camera ref_tag to cameras.map
    pub WriteCam: Option<unsafe extern "C" fn(text: *const c_char)>,
    pub FlushCamFile: Option<unsafe extern "C" fn()>,

    /// abort the game
    //TODO: Port Error variadic args
    // Source: oracle/oracle/code/game/g_public.h:179
    pub Error: Option<unsafe extern "C" fn(level: c_int, fmt: *const c_char, ...)>,

    /// get current time for profiling reasons
    /// this should NOT be used for any game related tasks,
    /// because it is not journaled
    pub Milliseconds: Option<unsafe extern "C" fn() -> c_int>,

    // console variable interaction
    //TODO: Port cvar_t
    // Source: oracle/oracle/code/game/q_shared.h:1310
    // The engine-side `cvar_s` registry node is deferred to the engine/qcommon tier
    // (see `sp_qshared::shared::cvar`); kept opaque here since only the pointer
    // crosses this ABI seam.
    pub cvar: Option<
        unsafe extern "C" fn(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut c_void,
    >,
    pub cvar_set: Option<unsafe extern "C" fn(var_name: *const c_char, value: *const c_char)>,
    pub Cvar_VariableIntegerValue: Option<unsafe extern "C" fn(var_name: *const c_char) -> c_int>,
    pub Cvar_VariableStringBuffer: Option<
        unsafe extern "C" fn(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int),
    >,

    // ClientCommand and ServerCommand parameter access
    pub argc: Option<unsafe extern "C" fn() -> c_int>,
    pub argv: Option<unsafe extern "C" fn(n: c_int) -> *mut c_char>,

    pub FS_FOpenFile: Option<
        unsafe extern "C" fn(qpath: *const c_char, file: *mut fileHandle_t, mode: fsMode_t) -> c_int,
    >,
    pub FS_Read:
        Option<unsafe extern "C" fn(buffer: *mut c_void, len: c_int, f: fileHandle_t) -> c_int>,
    pub FS_Write:
        Option<unsafe extern "C" fn(buffer: *const c_void, len: c_int, f: fileHandle_t) -> c_int>,
    pub FS_FCloseFile: Option<unsafe extern "C" fn(f: fileHandle_t)>,
    pub FS_ReadFile:
        Option<unsafe extern "C" fn(name: *const c_char, buf: *mut *mut c_void) -> c_int>,
    pub FS_FreeFile: Option<unsafe extern "C" fn(buf: *mut c_void)>,
    pub FS_GetFileList: Option<
        unsafe extern "C" fn(
            path: *const c_char,
            extension: *const c_char,
            listbuf: *mut c_char,
            bufsize: c_int,
        ) -> c_int,
    >,

    // Savegame handling
    //
    pub AppendToSaveGame: Option<
        unsafe extern "C" fn(chid: c_ulong, data: *const c_void, length: c_int) -> qboolean,
    >,
    // Raven's `#ifdef _XBOX` branch inlines these two as default-argument member
    // functions; dead on the shipping PC engine. Only the `#else` function-pointer
    // branch (with the C++ default `ppvAddressPtr = NULL` dropped, since Rust fn
    // pointers have no default args) is faithful here.
    pub ReadFromSaveGame: Option<
        unsafe extern "C" fn(
            chid: c_ulong,
            pvAddress: *mut c_void,
            iLength: c_int,
            ppvAddressPtr: *mut *mut c_void,
        ) -> c_int,
    >,
    pub ReadFromSaveGameOptional: Option<
        unsafe extern "C" fn(
            chid: c_ulong,
            pvAddress: *mut c_void,
            iLength: c_int,
            ppvAddressPtr: *mut *mut c_void,
        ) -> c_int,
    >,
    /// add commands to the console as if they were typed in
    /// for map changing, etc
    pub SendConsoleCommand: Option<unsafe extern "C" fn(text: *const c_char)>,

    //=========== server specific functionality =============

    /// kick a client off the server with a message
    pub DropClient: Option<unsafe extern "C" fn(clientNum: c_int, reason: *const c_char)>,

    /// reliably sends a command string to be interpreted by the given
    /// client.  If clientNum is -1, it will be sent to all clients
    //TODO: Port SendServerCommand variadic args
    // Source: oracle/oracle/code/game/g_public.h:233
    pub SendServerCommand: Option<unsafe extern "C" fn(clientNum: c_int, fmt: *const c_char, ...)>,

    // config strings hold all the index strings, and various other information
    // that is reliably communicated to all clients
    // All of the current configstrings are sent to clients when
    // they connect, and changes are sent to all connected clients.
    // All confgstrings are cleared at each level start.
    pub SetConfigstring: Option<unsafe extern "C" fn(num: c_int, string: *const c_char)>,
    pub GetConfigstring:
        Option<unsafe extern "C" fn(num: c_int, buffer: *mut c_char, bufferSize: c_int)>,

    // userinfo strings are maintained by the server system, so they
    // are persistant across level loads, while all other game visible
    // data is completely reset
    pub GetUserinfo:
        Option<unsafe extern "C" fn(num: c_int, buffer: *mut c_char, bufferSize: c_int)>,
    pub SetUserinfo: Option<unsafe extern "C" fn(num: c_int, buffer: *const c_char)>,

    /// the serverinfo info string has all the cvars visible to server browsers
    pub GetServerinfo: Option<unsafe extern "C" fn(buffer: *mut c_char, bufferSize: c_int)>,

    /// sets mins and maxs based on the brushmodel name
    pub SetBrushModel: Option<unsafe extern "C" fn(ent: *mut gentity_t, name: *const c_char)>,

    // collision detection against all linked entities
    pub trace: Option<
        unsafe extern "C" fn(
            results: *mut trace_t,
            start: *const vec3_t,
            mins: *const vec3_t,
            maxs: *const vec3_t,
            end: *const vec3_t,
            passEntityNum: c_int,
            contentmask: c_int,
            eG2TraceType: EG2_Collision,
            useLod: c_int,
        ),
    >,

    /// point contents against all linked entities
    pub pointcontents: Option<unsafe extern "C" fn(point: *const vec3_t, passEntityNum: c_int) -> c_int>,
    /// what contents are on the map?
    pub totalMapContents: Option<unsafe extern "C" fn() -> c_int>,

    pub inPVS: Option<unsafe extern "C" fn(p1: *const vec3_t, p2: *const vec3_t) -> qboolean>,
    pub inPVSIgnorePortals:
        Option<unsafe extern "C" fn(p1: *const vec3_t, p2: *const vec3_t) -> qboolean>,
    pub AdjustAreaPortalState: Option<unsafe extern "C" fn(ent: *mut gentity_t, open: qboolean)>,
    pub AreasConnected: Option<unsafe extern "C" fn(area1: c_int, area2: c_int) -> qboolean>,

    // an entity will never be sent to a client or used for collision
    // if it is not passed to linkentity.  If the size, position, or
    // solidity changes, it must be relinked.
    pub linkentity: Option<unsafe extern "C" fn(ent: *mut gentity_t)>,
    /// call before removing an interactive entity
    pub unlinkentity: Option<unsafe extern "C" fn(ent: *mut gentity_t)>,

    // EntitiesInBox will return brush models based on their bounding box,
    // so exact determination must still be done with EntityContact
    pub EntitiesInBox: Option<
        unsafe extern "C" fn(
            mins: *const vec3_t,
            maxs: *const vec3_t,
            list: *mut *mut gentity_t,
            maxcount: c_int,
        ) -> c_int,
    >,

    /// perform an exact check against inline brush models of non-square shape
    pub EntityContact: Option<
        unsafe extern "C" fn(mins: *const vec3_t, maxs: *const vec3_t, ent: *const gentity_t) -> qboolean,
    >,

    /// sound volume values
    pub VoiceVolume: *mut c_int,

    /// dynamic memory allocator for things that need to be freed
    // see qcommon/tags.h for choices
    pub Malloc: Option<unsafe extern "C" fn(iSize: c_int, eTag: memtag_t, bZeroIt: qboolean) -> *mut c_void>,
    pub Free: Option<unsafe extern "C" fn(buf: *mut c_void) -> c_int>,
    // see qcommon/tags.h for choices
    pub bIsFromZone: Option<unsafe extern "C" fn(buf: *mut c_void, eTag: memtag_t) -> qboolean>,

    /*
    Ghoul2 Insert Start
    */
    pub G2API_PrecacheGhoul2Model: Option<unsafe extern "C" fn(fileName: *const c_char) -> qhandle_t>,

    // Raven's `#ifdef _XBOX` branch inlines the Ghoul2 API as default-argument member
    // functions; dead on the shipping PC engine. Only the `#else` function-pointer
    // branch (with C++ default args dropped, since Rust fn pointers have no default
    // args) is faithful here.
    pub G2API_InitGhoul2Model: Option<
        unsafe extern "C" fn(
            ghoul2: *mut CGhoul2Info_v,
            fileName: *const c_char,
            modelIndex: c_int,
            customSkin: qhandle_t,
            customShader: qhandle_t,
            modelFlags: c_int,
            lodBias: c_int,
        ) -> c_int,
    >,
    //TODO: Port CGhoul2Info
    // Source: oracle/oracle/code/game/ghoul2_shared.h:240
    // `CGhoul2Info` is a distinct (non-vector) Ghoul2 C++ class from `CGhoul2Info_v`;
    // pointer-only dep kept opaque per house rules.
    pub G2API_SetSkin: Option<
        unsafe extern "C" fn(ghlInfo: *mut c_void, customSkin: qhandle_t, renderSkin: qhandle_t) -> qboolean,
    >,
    pub G2API_SetBoneAnim: Option<
        unsafe extern "C" fn(
            ghlInfo: *mut c_void,
            boneName: *const c_char,
            startFrame: c_int,
            endFrame: c_int,
            flags: c_int,
            animSpeed: f32,
            currentTime: c_int,
            setFrame: f32,
            blendTime: c_int,
        ) -> qboolean,
    >,
    pub G2API_SetBoneAngles: Option<
        unsafe extern "C" fn(
            ghlInfo: *mut c_void,
            boneName: *const c_char,
            angles: *const vec3_t,
            flags: c_int,
            up: Eorientations,
            right: Eorientations,
            forward: Eorientations,
            modelList: *mut qhandle_t,
            blendTime: c_int,
            blendStart: c_int,
        ) -> qboolean,
    >,
    pub G2API_SetBoneAnglesIndex: Option<
        unsafe extern "C" fn(
            ghlInfo: *mut c_void,
            index: c_int,
            angles: *const vec3_t,
            flags: c_int,
            yaw: c_int,
            pitch: c_int,
            roll: c_int,
            modelList: *mut qhandle_t,
            blendTime: c_int,
            currentTime: c_int,
        ) -> qboolean,
    >,
    pub G2API_SetBoneAnglesMatrix: Option<
        unsafe extern "C" fn(
            ghlInfo: *mut c_void,
            boneName: *const c_char,
            matrix: *const mdxaBone_t,
            flags: c_int,
            modelList: *mut qhandle_t,
            blendTime: c_int,
            currentTime: c_int,
        ) -> qboolean,
    >,
    pub G2API_CopyGhoul2Instance: Option<
        unsafe extern "C" fn(ghoul2From: *mut CGhoul2Info_v, ghoul2To: *mut CGhoul2Info_v, modelIndex: c_int),
    >,
    pub G2API_SetBoneAnimIndex: Option<
        unsafe extern "C" fn(
            ghlInfo: *mut c_void,
            index: c_int,
            startFrame: c_int,
            endFrame: c_int,
            flags: c_int,
            animSpeed: f32,
            currentTime: c_int,
            setFrame: f32,
            blendTime: c_int,
        ) -> qboolean,
    >,

    pub G2API_SetLodBias: Option<unsafe extern "C" fn(ghlInfo: *mut c_void, lodBias: c_int) -> qboolean>,
    pub G2API_SetShader:
        Option<unsafe extern "C" fn(ghlInfo: *mut c_void, customShader: qhandle_t) -> qboolean>,
    pub G2API_RemoveGhoul2Model:
        Option<unsafe extern "C" fn(ghlInfo: *mut CGhoul2Info_v, modelIndex: c_int) -> qboolean>,
    pub G2API_SetSurfaceOnOff: Option<
        unsafe extern "C" fn(ghlInfo: *mut c_void, surfaceName: *const c_char, flags: c_int) -> qboolean,
    >,
    pub G2API_SetRootSurface: Option<
        unsafe extern "C" fn(
            ghlInfo: *mut CGhoul2Info_v,
            modelIndex: c_int,
            surfaceName: *const c_char,
        ) -> qboolean,
    >,
    pub G2API_RemoveSurface: Option<unsafe extern "C" fn(ghlInfo: *mut c_void, index: c_int) -> qboolean>,
    pub G2API_AddSurface: Option<
        unsafe extern "C" fn(
            ghlInfo: *mut c_void,
            surfaceNumber: c_int,
            polyNumber: c_int,
            BarycentricI: f32,
            BarycentricJ: f32,
            lod: c_int,
        ) -> c_int,
    >,
    pub G2API_GetBoneAnim: Option<
        unsafe extern "C" fn(
            ghlInfo: *mut c_void,
            boneName: *const c_char,
            currentTime: c_int,
            currentFrame: *mut f32,
            startFrame: *mut c_int,
            endFrame: *mut c_int,
            flags: *mut c_int,
            animSpeed: *mut f32,
            modelList: *mut c_int,
        ) -> qboolean,
    >,
    pub G2API_GetBoneAnimIndex: Option<
        unsafe extern "C" fn(
            ghlInfo: *mut c_void,
            iBoneIndex: c_int,
            currentTime: c_int,
            currentFrame: *mut f32,
            startFrame: *mut c_int,
            endFrame: *mut c_int,
            flags: *mut c_int,
            animSpeed: *mut f32,
            modelList: *mut c_int,
        ) -> qboolean,
    >,
    pub G2API_GetAnimRange: Option<
        unsafe extern "C" fn(
            ghlInfo: *mut c_void,
            boneName: *const c_char,
            startFrame: *mut c_int,
            endFrame: *mut c_int,
        ) -> qboolean,
    >,
    pub G2API_GetAnimRangeIndex: Option<
        unsafe extern "C" fn(
            ghlInfo: *mut c_void,
            boneIndex: c_int,
            startFrame: *mut c_int,
            endFrame: *mut c_int,
        ) -> qboolean,
    >,

    pub G2API_PauseBoneAnim: Option<
        unsafe extern "C" fn(ghlInfo: *mut c_void, boneName: *const c_char, currentTime: c_int) -> qboolean,
    >,
    pub G2API_PauseBoneAnimIndex: Option<
        unsafe extern "C" fn(ghlInfo: *mut c_void, boneIndex: c_int, currentTime: c_int) -> qboolean,
    >,
    pub G2API_IsPaused:
        Option<unsafe extern "C" fn(ghlInfo: *mut c_void, boneName: *const c_char) -> qboolean>,
    pub G2API_StopBoneAnim:
        Option<unsafe extern "C" fn(ghlInfo: *mut c_void, boneName: *const c_char) -> qboolean>,
    pub G2API_StopBoneAngles:
        Option<unsafe extern "C" fn(ghlInfo: *mut c_void, boneName: *const c_char) -> qboolean>,
    pub G2API_RemoveBone:
        Option<unsafe extern "C" fn(ghlInfo: *mut c_void, boneName: *const c_char) -> qboolean>,
    pub G2API_RemoveBolt: Option<unsafe extern "C" fn(ghlInfo: *mut c_void, index: c_int) -> qboolean>,
    pub G2API_AddBolt:
        Option<unsafe extern "C" fn(ghlInfo: *mut c_void, boneName: *const c_char) -> c_int>,
    pub G2API_AddBoltSurfNum: Option<unsafe extern "C" fn(ghlInfo: *mut c_void, surfIndex: c_int) -> c_int>,
    pub G2API_AttachG2Model: Option<
        unsafe extern "C" fn(
            ghlInfo: *mut c_void,
            ghlInfoTo: *mut c_void,
            toBoltIndex: c_int,
            toModel: c_int,
        ) -> qboolean,
    >,
    pub G2API_DetachG2Model: Option<unsafe extern "C" fn(ghlInfo: *mut c_void) -> qboolean>,
    pub G2API_AttachEnt: Option<
        unsafe extern "C" fn(
            boltInfo: *mut c_int,
            ghlInfoTo: *mut c_void,
            toBoltIndex: c_int,
            entNum: c_int,
            toModelNum: c_int,
        ) -> qboolean,
    >,
    pub G2API_DetachEnt: Option<unsafe extern "C" fn(boltInfo: *mut c_int)>,

    pub G2API_GetBoltMatrix: Option<
        unsafe extern "C" fn(
            ghoul2: *mut CGhoul2Info_v,
            modelIndex: c_int,
            boltIndex: c_int,
            matrix: *mut mdxaBone_t,
            angles: *const vec3_t,
            position: *const vec3_t,
            frameNum: c_int,
            modelList: *mut qhandle_t,
            scale: *const vec3_t,
        ) -> qboolean,
    >,

    pub G2API_ListSurfaces: Option<unsafe extern "C" fn(ghlInfo: *mut c_void)>,
    pub G2API_ListBones: Option<unsafe extern "C" fn(ghlInfo: *mut c_void, frame: c_int)>,
    pub G2API_HaveWeGhoul2Models: Option<unsafe extern "C" fn(ghoul2: *mut CGhoul2Info_v) -> qboolean>,
    pub G2API_SetGhoul2ModelFlags:
        Option<unsafe extern "C" fn(ghlInfo: *mut c_void, flags: c_int) -> qboolean>,
    pub G2API_GetGhoul2ModelFlags: Option<unsafe extern "C" fn(ghlInfo: *mut c_void) -> c_int>,

    pub G2API_GetAnimFileName:
        Option<unsafe extern "C" fn(ghlInfo: *mut c_void, filename: *mut *mut c_char) -> qboolean>,
    //TODO: Port CMiniHeap
    // Source: oracle/oracle/code/game/g_public.h:164
    pub G2API_CollisionDetect: Option<
        unsafe extern "C" fn(
            collRecMap: *mut CCollisionRecord,
            ghoul2: *mut CGhoul2Info_v,
            angles: *const vec3_t,
            position: *const vec3_t,
            frameNumber: c_int,
            entNum: c_int,
            rayStart: *mut vec3_t,
            rayEnd: *mut vec3_t,
            scale: *mut vec3_t,
            G2VertSpace: *mut c_void,
            eG2TraceType: c_int,
            useLod: c_int,
            fRadius: f32,
        ),
    >,
    pub G2API_GiveMeVectorFromMatrix: Option<
        unsafe extern "C" fn(boltMatrix: *mut mdxaBone_t, flags: c_int, vec: *mut vec3_t),
    >,
    pub G2API_CleanGhoul2Models: Option<unsafe extern "C" fn(ghoul2: *mut CGhoul2Info_v)>,
    //TODO: Port IGhoul2InfoArray
    // Source: oracle/oracle/code/game/ghoul2_shared.h:313
    // Returns a C++ reference (`&`), which crosses the ABI as a bare pointer; kept
    // opaque since `IGhoul2InfoArray` itself is unported.
    pub TheGhoul2InfoArray: Option<unsafe extern "C" fn() -> *mut c_void>,
    pub G2API_GetParentSurface: Option<unsafe extern "C" fn(ghlInfo: *mut c_void, index: c_int) -> c_int>,
    pub G2API_GetSurfaceIndex:
        Option<unsafe extern "C" fn(ghlInfo: *mut c_void, surfaceName: *const c_char) -> c_int>,
    pub G2API_GetSurfaceName:
        Option<unsafe extern "C" fn(ghlInfo: *mut c_void, surfNumber: c_int) -> *mut c_char>,
    pub G2API_GetGLAName: Option<unsafe extern "C" fn(ghlInfo: *mut c_void) -> *mut c_char>,
    pub G2API_SetNewOrigin: Option<unsafe extern "C" fn(ghlInfo: *mut c_void, boltIndex: c_int) -> qboolean>,
    pub G2API_GetBoneIndex: Option<
        unsafe extern "C" fn(ghlInfo: *mut c_void, boneName: *const c_char, bAddIfNotFound: qboolean) -> c_int,
    >,
    pub G2API_StopBoneAnglesIndex:
        Option<unsafe extern "C" fn(ghlInfo: *mut c_void, index: c_int) -> qboolean>,
    pub G2API_StopBoneAnimIndex: Option<unsafe extern "C" fn(ghlInfo: *mut c_void, index: c_int) -> qboolean>,
    pub G2API_SetBoneAnglesMatrixIndex: Option<
        unsafe extern "C" fn(
            ghlInfo: *mut c_void,
            index: c_int,
            matrix: *const mdxaBone_t,
            flags: c_int,
            modelList: *mut qhandle_t,
            blendTime: c_int,
            currentTime: c_int,
        ) -> qboolean,
    >,
    pub G2API_SetAnimIndex: Option<unsafe extern "C" fn(ghlInfo: *mut c_void, index: c_int) -> qboolean>,
    pub G2API_GetAnimIndex: Option<unsafe extern "C" fn(ghlInfo: *mut c_void) -> c_int>,
    pub G2API_SaveGhoul2Models: Option<unsafe extern "C" fn(ghoul2: *mut CGhoul2Info_v)>,
    pub G2API_LoadGhoul2Models: Option<unsafe extern "C" fn(ghoul2: *mut CGhoul2Info_v, buffer: *mut c_char)>,
    pub G2API_LoadSaveCodeDestructGhoul2Info: Option<unsafe extern "C" fn(ghoul2: *mut CGhoul2Info_v)>,
    pub G2API_GetAnimFileNameIndex: Option<unsafe extern "C" fn(modelIndex: qhandle_t) -> *mut c_char>,
    pub G2API_GetAnimFileInternalNameIndex:
        Option<unsafe extern "C" fn(modelIndex: qhandle_t) -> *mut c_char>,
    pub G2API_GetSurfaceRenderStatus:
        Option<unsafe extern "C" fn(ghlInfo: *mut c_void, surfaceName: *const c_char) -> c_int>,

    //rww - RAGDOLL_BEGIN
    pub G2API_SetRagDoll:
        Option<unsafe extern "C" fn(ghoul2: *mut CGhoul2Info_v, parms: *mut CRagDollParams)>,
    //TODO: Port CRagDollUpdateParams
    // Source: oracle/oracle/code/game/g_public.h:47
    pub G2API_AnimateG2Models: Option<
        unsafe extern "C" fn(ghoul2: *mut CGhoul2Info_v, AcurrentTime: c_int, params: *mut c_void),
    >,
    //rww - RAGDOLL_END

    // additional ragdoll options -rww
    pub G2API_RagPCJConstraint: Option<
        unsafe extern "C" fn(
            ghoul2: *mut CGhoul2Info_v,
            boneName: *const c_char,
            min: *mut vec3_t,
            max: *mut vec3_t,
        ) -> qboolean,
    >,
    pub G2API_RagPCJGradientSpeed: Option<
        unsafe extern "C" fn(ghoul2: *mut CGhoul2Info_v, boneName: *const c_char, speed: f32) -> qboolean,
    >,
    pub G2API_RagEffectorGoal: Option<
        unsafe extern "C" fn(ghoul2: *mut CGhoul2Info_v, boneName: *const c_char, pos: *mut vec3_t) -> qboolean,
    >,
    pub G2API_GetRagBonePos: Option<
        unsafe extern "C" fn(
            ghoul2: *mut CGhoul2Info_v,
            boneName: *const c_char,
            pos: *mut vec3_t,
            entAngles: *mut vec3_t,
            entPos: *mut vec3_t,
            entScale: *mut vec3_t,
        ) -> qboolean,
    >,
    pub G2API_RagEffectorKick: Option<
        unsafe extern "C" fn(
            ghoul2: *mut CGhoul2Info_v,
            boneName: *const c_char,
            velocity: *mut vec3_t,
        ) -> qboolean,
    >,
    pub G2API_RagForceSolve:
        Option<unsafe extern "C" fn(ghoul2: *mut CGhoul2Info_v, force: qboolean) -> qboolean>,

    // rww - ik move method, allows you to specify a bone and move it to a world point (within
    // joint constraints) by using the majority of gil's existing bone angling stuff from the
    // ragdoll code.
    pub G2API_SetBoneIKState: Option<
        unsafe extern "C" fn(
            ghoul2: *mut CGhoul2Info_v,
            time: c_int,
            boneName: *const c_char,
            ikState: c_int,
            params: *mut sharedSetBoneIKStateParams_t,
        ) -> qboolean,
    >,
    pub G2API_IKMove: Option<
        unsafe extern "C" fn(
            ghoul2: *mut CGhoul2Info_v,
            time: c_int,
            params: *mut sharedIKMoveParams_t,
        ) -> qboolean,
    >,

    //TODO: Port SSkinGoreData
    // Source: oracle/oracle/code/game/q_shared.h:2530
    pub G2API_AddSkinGore:
        Option<unsafe extern "C" fn(ghoul2: *mut CGhoul2Info_v, gore: *mut c_void)>,
    pub G2API_ClearSkinGore: Option<unsafe extern "C" fn(ghoul2: *mut CGhoul2Info_v)>,

    pub RMG_Init: Option<unsafe extern "C" fn(terrainID: c_int)>,
    // non-XBOX branch (`#ifndef _XBOX`)
    pub CM_RegisterTerrain: Option<unsafe extern "C" fn(info: *const c_char) -> c_int>,
    pub SetActiveSubBSP: Option<unsafe extern "C" fn(index: c_int) -> *const c_char>,

    pub RE_RegisterSkin: Option<unsafe extern "C" fn(name: *const c_char) -> c_int>,
    pub RE_GetAnimationCFG: Option<
        unsafe extern "C" fn(psCFGFilename: *const c_char, psDest: *mut c_char, iDestSize: c_int) -> c_int,
    >,

    // Raven writes these as C++ `bool`/`float`, not `qboolean`; `bool` matches the
    // source's declared return type and is FFI-compatible with C99 `_Bool`.
    pub WE_GetWindVector:
        Option<unsafe extern "C" fn(windVector: *mut vec3_t, atpoint: *mut vec3_t) -> bool>,
    pub WE_GetWindGusting: Option<unsafe extern "C" fn(atpoint: *mut vec3_t) -> bool>,
    pub WE_IsOutside: Option<unsafe extern "C" fn(pos: *mut vec3_t) -> bool>,
    pub WE_IsOutsideCausingPain: Option<unsafe extern "C" fn(pos: *mut vec3_t) -> f32>,
    pub WE_GetChanceOfSaberFizz: Option<unsafe extern "C" fn() -> f32>,
    pub WE_IsShaking: Option<unsafe extern "C" fn(pos: *mut vec3_t) -> bool>,
    pub WE_AddWeatherZone: Option<unsafe extern "C" fn(mins: *mut vec3_t, maxs: *mut vec3_t)>,
    pub WE_SetTempGlobalFogColor: Option<unsafe extern "C" fn(color: *mut vec3_t) -> bool>,
    /*
    Ghoul2 Insert End
    */
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<game_import_t>() == 1048);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, Printf) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, WriteCam) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, FlushCamFile) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, Error) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, Milliseconds) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, cvar) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, cvar_set) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, Cvar_VariableIntegerValue) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, Cvar_VariableStringBuffer) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, argc) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, argv) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, FS_FOpenFile) == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, FS_Read) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, FS_Write) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, FS_FCloseFile) == 112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, FS_ReadFile) == 120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, FS_FreeFile) == 128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, FS_GetFileList) == 136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, AppendToSaveGame) == 144);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, ReadFromSaveGame) == 152);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, ReadFromSaveGameOptional) == 160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, SendConsoleCommand) == 168);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, DropClient) == 176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, SendServerCommand) == 184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, SetConfigstring) == 192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, GetConfigstring) == 200);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, GetUserinfo) == 208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, SetUserinfo) == 216);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, GetServerinfo) == 224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, SetBrushModel) == 232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, trace) == 240);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, pointcontents) == 248);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, totalMapContents) == 256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, inPVS) == 264);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, inPVSIgnorePortals) == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, AdjustAreaPortalState) == 280);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, AreasConnected) == 288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, linkentity) == 296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, unlinkentity) == 304);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, EntitiesInBox) == 312);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, EntityContact) == 320);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, VoiceVolume) == 328);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, Malloc) == 336);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, Free) == 344);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, bIsFromZone) == 352);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_PrecacheGhoul2Model) == 360);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_InitGhoul2Model) == 368);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_SetSkin) == 376);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_SetBoneAnim) == 384);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_SetBoneAngles) == 392);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_SetBoneAnglesIndex) == 400);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_SetBoneAnglesMatrix) == 408);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_CopyGhoul2Instance) == 416);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_SetBoneAnimIndex) == 424);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_SetLodBias) == 432);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_SetShader) == 440);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_RemoveGhoul2Model) == 448);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_SetSurfaceOnOff) == 456);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_SetRootSurface) == 464);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_RemoveSurface) == 472);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_AddSurface) == 480);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_GetBoneAnim) == 488);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_GetBoneAnimIndex) == 496);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_GetAnimRange) == 504);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_GetAnimRangeIndex) == 512);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_PauseBoneAnim) == 520);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_PauseBoneAnimIndex) == 528);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_IsPaused) == 536);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_StopBoneAnim) == 544);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_StopBoneAngles) == 552);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_RemoveBone) == 560);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_RemoveBolt) == 568);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_AddBolt) == 576);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_AddBoltSurfNum) == 584);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_AttachG2Model) == 592);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_DetachG2Model) == 600);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_AttachEnt) == 608);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_DetachEnt) == 616);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_GetBoltMatrix) == 624);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_ListSurfaces) == 632);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_ListBones) == 640);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_HaveWeGhoul2Models) == 648);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_SetGhoul2ModelFlags) == 656);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_GetGhoul2ModelFlags) == 664);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_GetAnimFileName) == 672);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_CollisionDetect) == 680);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_GiveMeVectorFromMatrix) == 688);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_CleanGhoul2Models) == 696);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, TheGhoul2InfoArray) == 704);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_GetParentSurface) == 712);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_GetSurfaceIndex) == 720);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_GetSurfaceName) == 728);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_GetGLAName) == 736);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_SetNewOrigin) == 744);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_GetBoneIndex) == 752);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_StopBoneAnglesIndex) == 760);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_StopBoneAnimIndex) == 768);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_SetBoneAnglesMatrixIndex) == 776);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_SetAnimIndex) == 784);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_GetAnimIndex) == 792);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_SaveGhoul2Models) == 800);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_LoadGhoul2Models) == 808);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_LoadSaveCodeDestructGhoul2Info) == 816);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_GetAnimFileNameIndex) == 824);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_GetAnimFileInternalNameIndex) == 832);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_GetSurfaceRenderStatus) == 840);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_SetRagDoll) == 848);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_AnimateG2Models) == 856);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_RagPCJConstraint) == 864);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_RagPCJGradientSpeed) == 872);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_RagEffectorGoal) == 880);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_GetRagBonePos) == 888);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_RagEffectorKick) == 896);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_RagForceSolve) == 904);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_SetBoneIKState) == 912);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_IKMove) == 920);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_AddSkinGore) == 928);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, G2API_ClearSkinGore) == 936);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, RMG_Init) == 944);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, CM_RegisterTerrain) == 952);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, SetActiveSubBSP) == 960);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, RE_RegisterSkin) == 968);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, RE_GetAnimationCFG) == 976);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, WE_GetWindVector) == 984);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, WE_GetWindGusting) == 992);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, WE_IsOutside) == 1000);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, WE_IsOutsideCausingPain) == 1008);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, WE_GetChanceOfSaberFizz) == 1016);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, WE_IsShaking) == 1024);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, WE_AddWeatherZone) == 1032);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(game_import_t, WE_SetTempGlobalFogColor) == 1040);

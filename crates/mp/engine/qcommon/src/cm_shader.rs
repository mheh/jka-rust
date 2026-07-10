#![allow(non_snake_case, non_camel_case_types, dead_code, unused_variables)]

//! MP `cm_shader.cpp` — the collision-model shader table: parses
//! `scripts/*.shader` text into `CCMShader` surface/content-flag records
//! (`svInfoParms`/`svMaterialNames` keyword tables, vector/material parse
//! helpers), caches per-name shader text in `shaderTextTable`, and resolves
//! `CCMShader` lookups (by BSP index or by name) through `cmShaderTable`.
//!
//! Source: `oracle/codemp/qcommon/cm_shader.cpp`
//!
//! PORT-NOTE(rm-types): `RenderModels`/`RmManager` are the state-receiver
//! types pinned by the engine-fork-discovery preamble's receiver order
//! (rmg-terrain.md owns their shape); neither has landed in the tree yet.
//! Referenced by their exact resolved-signature names per the no-stub rule
//! (`common_fns.rs` precedent); reported as missing symbols/shape mismatches
//! for the finisher.
#[allow(dead_code)]
struct RenderModels;
#[allow(dead_code)]
struct RmManager;

use core::ffi::{c_char, c_int};

use mp_engine_icarus::q3_interface::S_COLOR_YELLOW;
use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::surface_flags::{CONTENTS_OPAQUE, CONTENTS_SOLID, MATERIAL_LAST, SURF_SKY};
use mp_qshared::shared::{qboolean, vec3_t, MAX_QPATH};

use crate::cm::ccmshader::CCMShader;
use crate::cm::cm_shader_consts::MAX_SHADER_FILES;
use crate::collision_world::CollisionWorld;
use crate::common::Common;

/// Raven `SV_ParseSurfaceParm` — match the next token against `svInfoParms`,
/// OR/AND-ing the shader's surface/content flags from the matching row.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:261-278`
pub fn SV_ParseSurfaceParm(
    cm: &mut CollisionWorld,
    shader: *mut CCMShader,
    text: *mut *const c_char,
) {
    //TODO: Port svInfoParms
    // Source: oracle/codemp/qcommon/cm_shader.cpp:226-259
    let numsvInfoParms: c_int = cm.svInfoParms.len() as c_int;

    let token = crate::qcommon::com_parse::COM_ParseExt(text, false);
    for i in 0..numsvInfoParms {
        let row = &cm.svInfoParms[i as usize];
        if crate::qcommon::q_shared::Q_stricmp(token, row.name) == 0 {
            unsafe {
                (*shader).surfaceFlags |= row.surfaceFlags;
                (*shader).contentFlags |= row.contents;
                (*shader).contentFlags &= row.clearSolid;
            }
            break;
        }
    }
}

/// Raven `CM_ShutdownShaderProperties` — clear the per-name shader table.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:489-496`
pub fn CM_ShutdownShaderProperties(cm: &mut CollisionWorld) {
    //TODO: Port cmShaderTable
    // Source: oracle/codemp/qcommon/cm_shader.cpp:30
    if cm.cmShaderTable.count() != 0 {
        //		Com_Printf("Shutting down cmShaderTable .....\n");
        cm.cmShaderTable.clear();
    }
}

/// Raven `CM_GetShaderInfo( int shaderNum )` — the BSP-index overload:
/// bounds-checked pointer into `cmg.shaders`.
///
/// PORT-NOTE(overload): Raven overloads `CM_GetShaderInfo` by parameter type
/// (`int` vs `const char *`); Rust has no overloading, so the by-name form
/// below is `CM_GetShaderInfo_ByName` (packet `qcommon__2130_CM_GetShaderInfo.md`).
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:526-536`
pub fn CM_GetShaderInfo(cm: &mut CollisionWorld, shaderNum: c_int) -> *mut CCMShader {
    //TODO: Port cmg
    // Source: oracle/codemp/qcommon/cm_local.h:220
    if shaderNum < 0 || shaderNum >= cm.cmg.numShaders {
        return core::ptr::null_mut();
    }
    unsafe { cm.cmg.shaders.add(shaderNum as usize) }
}

/// Raven `CM_CreateShaderTextHash` — walk the loaded `shaderText` buffer,
/// building one `CCMShaderText` per labeled block and inserting it into
/// `shaderTextTable`.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:37-59`
pub fn CM_CreateShaderTextHash(
    cm: &mut CollisionWorld,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
) {
    //TODO: Port shaderText
    // Source: oracle/codemp/qcommon/cm_shader.cpp:28
    let mut p: *const c_char = cm.shaderText;
    // look for label
    while !p.is_null() {
        let mut hasNewLines: bool = false;
        p = crate::qcommon::com_parse::SkipWhitespace(p, &mut hasNewLines);
        let token = crate::qcommon::com_parse::COM_ParseExt(&mut p, true);
        if unsafe { *token } == 0 {
            break;
        }
        //TODO: Port CCMShaderText
        // Source: oracle/codemp/qcommon/cm_local.h (CCMShaderText)
        let shader = crate::cm::ccmshader_text::CCMShaderText::CCMShaderText(rmg, host, token, p);
        //TODO: Port shaderTextTable
        // Source: oracle/codemp/qcommon/cm_shader.cpp:29
        cm.shaderTextTable.insert(shader);

        crate::qcommon::com_parse::SkipBracedSection(&mut p);
    }
}

/// Raven `CM_GetShaderText` — look up the raw shader-text block for `key`.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:158-168`
pub fn CM_GetShaderText(
    cm: &mut CollisionWorld,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    key: *const c_char,
) -> *const c_char {
    //TODO: Port shaderTextTable
    // Source: oracle/codemp/qcommon/cm_shader.cpp:29
    let st = cm.shaderTextTable[key];
    if !st.is_null() {
        return unsafe { (*st).GetData(rmg, host) };
    }
    core::ptr::null()
}

/// Raven `CM_LoadShaderFiles` — scan `shaders/*.shader` (+ `shaders/test/*`
/// outside `FINAL_BUILD`), read and concatenate them into one `shaderText`
/// buffer.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:71-150`
pub fn CM_LoadShaderFiles(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    let mut numShaders1: c_int = 0;
    // scan for shader files
    let shaderFiles1 =
        crate::files::FS_ListFiles(common, cm, rm, host, "shaders", ".shader", &mut numShaders1);
    //TODO: Port FINAL_BUILD
    // Source: oracle/codemp/qcommon/cm_shader.cpp:75
    let mut numShaders2: c_int = 0;
    let shaderFiles2 = crate::files::FS_ListFiles(
        common,
        cm,
        rm,
        host,
        "shaders/test",
        ".shader",
        &mut numShaders2,
    );

    if shaderFiles1.is_null() || numShaders1 == 0 {
        //TODO: Port WARNING
        // Source: oracle/codemp/qcommon/cm_shader.cpp:92
        crate::common::com_printf::Com_Printf(
            common,
            &format!("{}WARNING: no shader files found\n", S_COLOR_YELLOW),
        );
        return;
    }

    let mut numShaders = numShaders1 + numShaders2;
    if numShaders > MAX_SHADER_FILES as c_int {
        numShaders = MAX_SHADER_FILES as c_int;
    }

    let mut buffers: [*mut c_char; MAX_SHADER_FILES] = [core::ptr::null_mut(); MAX_SHADER_FILES];
    let mut sum: c_int = 0;
    let mut i: c_int = 0;

    // load and parse shader files
    while i < numShaders1 {
        let mut filename: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
        crate::qcommon::q_shared::Com_sprintf(
            filename.as_mut_ptr(),
            core::mem::size_of_val(&filename),
            &format!("shaders/{}", unsafe { *shaderFiles1.add(i as usize) }),
        );
        crate::common::com_printf::Com_DPrintf(common, &format!("...loading '{}'\n", "filename"));
        sum += crate::files::FS_ReadFile(common, cm, rm, host, "filename", unsafe {
            buffers.as_mut_ptr().add(i as usize) as *mut *mut core::ffi::c_void
        });
        if unsafe { *buffers.as_ptr().add(i as usize) }.is_null() {
            crate::common::com_error::Com_Error(
                common,
                cm,
                rm,
                host,
                errorParm_t::ERR_FATAL,
                &format!("Couldn't load {}", "filename"),
            );
        }
        i += 1;
    }
    while i < numShaders {
        let mut filename: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
        crate::qcommon::q_shared::Com_sprintf(
            filename.as_mut_ptr(),
            core::mem::size_of_val(&filename),
            &format!("shaders/test/{}", "shaderFiles2[i - numShaders1]"),
        );
        crate::common::com_printf::Com_DPrintf(common, &format!("...loading '{}'\n", "filename"));
        sum += crate::files::FS_ReadFile(common, cm, rm, host, "filename", unsafe {
            buffers.as_mut_ptr().add(i as usize) as *mut *mut core::ffi::c_void
        });
        if unsafe { *buffers.as_ptr().add(i as usize) }.is_null() {
            crate::common::com_error::Com_Error(
                common,
                cm,
                rm,
                host,
                errorParm_t::ERR_DROP,
                &format!("Couldn't load {}", "filename"),
            );
        }
        i += 1;
    }

    // build single large buffer
    //TODO: Port shaderText
    // Source: oracle/codemp/qcommon/cm_shader.cpp:28
    cm.shaderText = crate::z_memman::Z_Malloc(
        common,
        cm,
        rm,
        host,
        (sum + numShaders * 2) as usize,
        memtag_t::TAG_SHADERTEXT,
        true,
    ) as *mut c_char;

    // free in reverse order, so the temp files are all dumped
    let mut j = numShaders - 1;
    while j >= 0 {
        unsafe {
            crate::qcommon::q_shared::strcat(cm.shaderText, c"\n".as_ptr());
            crate::qcommon::q_shared::strcat(cm.shaderText, buffers[j as usize]);
        }
        crate::files::FS_FreeFile(common, buffers[j as usize]);
        j -= 1;
    }

    // free up memory
    crate::files::FS_FreeFileList(common, shaderFiles1);
    crate::files::FS_FreeFileList(common, shaderFiles2);
}

/// Raven `CM_FreeShaderText` — release the cached shader-text buffer and
/// per-name hash.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:176-184`
pub fn CM_FreeShaderText(common: &mut Common, cm: &mut CollisionWorld) {
    //TODO: Port shaderTextTable
    // Source: oracle/codemp/qcommon/cm_shader.cpp:29
    cm.shaderTextTable.clear();
    //TODO: Port shaderText
    // Source: oracle/codemp/qcommon/cm_shader.cpp:28
    if !cm.shaderText.is_null() {
        crate::z_memman::Z_Free(common, cm.shaderText);
        cm.shaderText = core::ptr::null_mut();
    }
}

/// Raven `SV_ParseMaterial` — match the next token against `svMaterialNames`,
/// OR-ing its index into the shader's surface flags.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:290-309`
pub fn SV_ParseMaterial(
    common: &mut Common,
    cm: &mut CollisionWorld,
    shader: *mut CCMShader,
    text: *mut *const c_char,
) {
    let token = crate::qcommon::com_parse::COM_ParseExt(text, false);
    if unsafe { *token } == 0 {
        crate::common::com_printf::Com_Printf(
            common,
            &format!(
                "{}WARNING: missing material in shader '{}'\n",
                S_COLOR_YELLOW,
                unsafe { core::ffi::CStr::from_ptr((*shader).shader.as_ptr()).to_string_lossy() }
            ),
        );
        return;
    }
    for i in 0..MATERIAL_LAST {
        //TODO: Port svMaterialNames
        // Source: oracle/codemp/qcommon/cm_shader.cpp:285-288
        if crate::qcommon::q_shared::Q_stricmp(token, cm.svMaterialNames[i as usize]) == 0 {
            unsafe {
                (*shader).surfaceFlags |= i;
            }
            break;
        }
    }
}

/// Raven `CM_ParseVector` — parse a parenthesized `count`-float vector.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:316-347`
// FIXME: spaces are currently required after parens, should change parseext...
pub fn CM_ParseVector(
    common: &mut Common,
    shader: *mut CCMShader,
    text: *mut *const c_char,
    count: c_int,
    v: *mut f32,
) -> qboolean {
    let mut token = crate::qcommon::com_parse::COM_ParseExt(text, false);
    if unsafe { crate::qcommon::q_shared::strcmp(token, c"(".as_ptr()) } != 0 {
        crate::common::com_printf::Com_Printf(
            common,
            &format!(
                "{}WARNING: missing parenthesis in shader '{}'\n",
                S_COLOR_YELLOW,
                unsafe { core::ffi::CStr::from_ptr((*shader).shader.as_ptr()).to_string_lossy() }
            ),
        );
        return qboolean::qfalse as qboolean;
    }

    for i in 0..count {
        token = crate::qcommon::com_parse::COM_ParseExt(text, false);
        if unsafe { *token } == 0 {
            crate::common::com_printf::Com_Printf(
                common,
                &format!(
                    "{}WARNING: missing vector element in shader '{}'\n",
                    S_COLOR_YELLOW,
                    unsafe {
                        core::ffi::CStr::from_ptr((*shader).shader.as_ptr()).to_string_lossy()
                    }
                ),
            );
            return qboolean::qfalse as qboolean;
        }
        unsafe {
            *v.add(i as usize) = crate::qcommon::q_shared::atof(token);
        }
    }

    token = crate::qcommon::com_parse::COM_ParseExt(text, false);
    if unsafe { crate::qcommon::q_shared::strcmp(token, c")".as_ptr()) } != 0 {
        crate::common::com_printf::Com_Printf(
            common,
            &format!(
                "{}WARNING: missing parenthesis in shader '{}'\n",
                S_COLOR_YELLOW,
                unsafe { core::ffi::CStr::from_ptr((*shader).shader.as_ptr()).to_string_lossy() }
            ),
        );
        return qboolean::qfalse as qboolean;
    }
    qboolean::qtrue as qboolean
}

/// Raven `CM_LoadShaderText` — (re)build the cached shader-text buffer if
/// missing (or forced).
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:195-210`
pub fn CM_LoadShaderText(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    forceReload: qboolean,
) {
    if forceReload != 0 {
        CM_FreeShaderText(common, cm);
    }
    //TODO: Port shaderText
    // Source: oracle/codemp/qcommon/cm_shader.cpp:28
    if !cm.shaderText.is_null() {
        return;
    }
    //	Com_Printf("Loading shader text .....\n");
    CM_LoadShaderFiles(common, cm, rm, host);
    CM_CreateShaderTextHash(cm, rmg, host);

    //Com_Printf("..... %d shader definitions loaded\n", shaderTextTable.count());
}

/// Raven `CM_ParseShader` — parse one `{ ... }` shader body: `surfaceParm`,
/// `material`/`q3map_material`, `sun`/`q3map_sun` (values discarded — dead
/// per Raven's own comments), and `fogParms` keywords.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:361-454`
pub fn CM_ParseShader(
    common: &mut Common,
    cm: &mut CollisionWorld,
    shader: *mut CCMShader,
    text: *mut *const c_char,
) {
    let mut token = crate::qcommon::com_parse::COM_ParseExt(text, true);
    if unsafe { *token } != b'{' as c_char {
        crate::common::com_printf::Com_Printf(
            common,
            &format!(
                "{}WARNING: expecting '{{', found '{}' instead in shader '{}'\n",
                S_COLOR_YELLOW,
                unsafe { core::ffi::CStr::from_ptr(token).to_string_lossy() },
                unsafe { core::ffi::CStr::from_ptr((*shader).shader.as_ptr()).to_string_lossy() }
            ),
        );
        return;
    }

    loop {
        token = crate::qcommon::com_parse::COM_ParseExt(text, true);
        if unsafe { *token } == 0 {
            crate::common::com_printf::Com_Printf(
                common,
                &format!(
                    "{}WARNING: no concluding '}}' in shader {}\n",
                    S_COLOR_YELLOW,
                    unsafe {
                        core::ffi::CStr::from_ptr((*shader).shader.as_ptr()).to_string_lossy()
                    }
                ),
            );
            return;
        }

        let c0 = unsafe { *token };
        // end of shader definition
        if c0 == b'}' as c_char {
            break;
        }
        // stage definition
        else if c0 == b'{' as c_char {
            crate::qcommon::com_parse::SkipBracedSection(text);
            continue;
        }
        // material deprecated as of 11 Jan 01
        // material undeprecated as of 7 May 01 - q3map_material deprecated
        else if unsafe {
            crate::qcommon::q_shared::Q_stricmp(token, c"material".as_ptr()) == 0
                || crate::qcommon::q_shared::Q_stricmp(token, c"q3map_material".as_ptr()) == 0
        } {
            SV_ParseMaterial(common, cm, shader, text);
        }
        // sun parms
        // q3map_sun deprecated as of 11 Jan 01
        else if unsafe {
            crate::qcommon::q_shared::Q_stricmp(token, c"sun".as_ptr()) == 0
                || crate::qcommon::q_shared::Q_stricmp(token, c"q3map_sun".as_ptr()) == 0
        } {
            //			float	a, b;

            token = crate::qcommon::com_parse::COM_ParseExt(text, false);
            //			shader->sunLight[0] = atof( token );
            token = crate::qcommon::com_parse::COM_ParseExt(text, false);
            //			shader->sunLight[1] = atof( token );
            token = crate::qcommon::com_parse::COM_ParseExt(text, false);
            //			shader->sunLight[2] = atof( token );

            //			VectorNormalize( shader->sunLight );

            token = crate::qcommon::com_parse::COM_ParseExt(text, false);
            //			a = atof( token );
            //			VectorScale( shader->sunLight, a, shader->sunLight);

            token = crate::qcommon::com_parse::COM_ParseExt(text, false);
            //			a = DEG2RAD(atof( token ));

            token = crate::qcommon::com_parse::COM_ParseExt(text, false);
            //			b = DEG2RAD(atof( token ));

            //			shader->sunDirection[0] = cos( a ) * cos( b );
            //			shader->sunDirection[1] = sin( a ) * cos( b );
            //			shader->sunDirection[2] = sin( b );
        } else if unsafe {
            crate::qcommon::q_shared::Q_stricmp(token, c"surfaceParm".as_ptr()) == 0
        } {
            SV_ParseSurfaceParm(cm, shader, text);
            continue;
        } else if unsafe { crate::qcommon::q_shared::Q_stricmp(token, c"fogParms".as_ptr()) == 0 } {
            let mut fogColor: vec3_t = vec3_t::default();
            if CM_ParseVector(common, shader, text, 3, fogColor.as_mut_ptr()) == 0 {
                return;
            }

            token = crate::qcommon::com_parse::COM_ParseExt(text, false);
            if unsafe { *token } == 0 {
                crate::common::com_printf::Com_Printf(
                    common,
                    &format!(
                        "{}WARNING: missing parm for 'fogParms' keyword in shader '{}'\n",
                        S_COLOR_YELLOW,
                        unsafe {
                            core::ffi::CStr::from_ptr((*shader).shader.as_ptr()).to_string_lossy()
                        }
                    ),
                );
                continue;
            }
            //			shader->depthForOpaque = atof( token );

            // skip any old gradient directions
            crate::qcommon::com_parse::SkipRestOfLine(text);
            continue;
        }
    }
}

/// Raven `CM_SetupShaderProperties` — populate `cmShaderTable` from every
/// loaded BSP shader, then parse each one's shader-script text.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:466-487`
pub fn CM_SetupShaderProperties(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
) {
    //TODO: Port cmg
    // Source: oracle/codemp/qcommon/cm_local.h:220
    // Add all basic shaders to the cmShaderTable
    let numShaders = cm.cmg.numShaders;
    for i in 0..numShaders {
        let s = CM_GetShaderInfo(cm, i);
        //TODO: Port cmShaderTable
        // Source: oracle/codemp/qcommon/cm_shader.cpp:30
        cm.cmShaderTable.insert(s);
    }
    // Go through and parse evaluate shader names to shadernums
    for i in 0..numShaders {
        let shader = CM_GetShaderInfo(cm, i);
        let def = CM_GetShaderText(cm, rmg, host, unsafe { (*shader).shader.as_ptr() });
        if !def.is_null() {
            CM_ParseShader(common, cm, shader, &mut { def } as *mut *const c_char);
        }
    }
}

/// Raven `CM_GetShaderInfo( const char *name )` — the by-name overload:
/// looks up (or lazily allocates + parses) the `CCMShader` for `name`.
///
/// PORT-NOTE(overload): disambiguated from the by-index overload
/// (`CM_GetShaderInfo` above, packet `qcommon__0110_CM_GetShaderInfo.md` /
/// `qcommon__2130_CM_GetShaderInfo.md`) since Rust has no overloading.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:498-524`
pub fn CM_GetShaderInfo_ByName(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    name: *const c_char,
) -> *mut CCMShader {
    //TODO: Port cmShaderTable
    // Source: oracle/codemp/qcommon/cm_shader.cpp:30
    let mut out = cm.cmShaderTable[name];
    if !out.is_null() {
        return out;
    }

    // Create a new CCMShader class
    //TODO: Port h_high
    // Source: oracle/codemp/qcommon/cm_shader.cpp:510
    out = crate::z_memman::Hunk_Alloc(
        common,
        cm,
        rm,
        host,
        core::mem::size_of::<CCMShader>(),
        "h_high",
    ) as *mut CCMShader;
    // Set defaults
    unsafe {
        crate::qcommon::q_shared::Q_strncpyz((*out).shader.as_mut_ptr(), name, MAX_QPATH);
        (*out).contentFlags = CONTENTS_SOLID | CONTENTS_OPAQUE;
    }

    // Parse in any text if it exists
    let def = CM_GetShaderText(cm, rmg, host, name);
    if !def.is_null() {
        CM_ParseShader(common, cm, out, &mut { def } as *mut *const c_char);
    }

    cm.cmShaderTable.insert(out);
    out
}

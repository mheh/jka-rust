#![allow(non_snake_case, non_camel_case_types, dead_code, unused_variables)]

//! MP `cm_shader.cpp` — the collision-model shader table: parses
//! `scripts/*.shader` text into `CCMShader` surface/content-flag records
//! (`svInfoParms`/`svMaterialNames` keyword tables, vector/material parse
//! helpers), caches per-name shader text in `shaderTextTable`, and resolves
//! `CCMShader` lookups (by BSP index or by name) through `cmShaderTable`.
//!
//! Source: `oracle/codemp/qcommon/cm_shader.cpp`

use core::ffi::{c_char, c_int};

use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::surface_flags::{CONTENTS_OPAQUE, CONTENTS_SOLID, MATERIAL_LAST};
use mp_qshared::shared::{qboolean, qfalse, qtrue, vec3_t, MAX_QPATH};

use crate::cm::ccmshader::CCMShader;
use crate::cm::cm_shader_consts::MAX_SHADER_FILES;
use crate::collision_world::CollisionWorld;
use crate::common::engine_host_view::EngineHostView;
use crate::common::Common;
use crate::common_fns::Com_DPrintf;

// Raven `S_COLOR_YELLOW` — not reachable from this crate (icarus/game keep
// their own private copies of this same literal; qshared has no public
// home for it yet), so this file keeps its own copy per that precedent.
// Source: `oracle/codemp/game/q_shared.h:1163`
const S_COLOR_YELLOW: &str = "^3";

// Sweep: extern forward-declares eliminated. Real qshared/in-crate callees
// imported (`Q_stricmp`/`Q_strncpyz`/`Hunk_Alloc`/`com_error`). q_shared
// parse helpers (`COM_Parse`/`Skip*`/`Com_sprintf`), this crate's own
// unported `FS_*` (files.cpp subject) and `Z_Malloc`/`Z_Free` (z_memman_pc.cpp
// subject) referenced at their canonical homes; reported.
use crate::common::com_error;
use crate::files_common::{FS_FreeFile, FS_ListFiles, FS_ReadFile};
use crate::z_memman_pc::Hunk_Alloc;
use crate::z_memman_pc::{Z_Free, Z_Malloc};
use mp_qshared::shared::ha_pref;
use mp_qshared::shared::q_string::Q_strncpyz;
use mp_qshared::shared::q_string::{COM_Parse, SkipBracedSection, SkipRestOfLine};
use native_string::atof;
use native_string::q_string::Q_stricmp;

/// Raven `SV_ParseSurfaceParm` — match the next token against `svInfoParms`,
/// OR/AND-ing the shader's surface/content flags from the matching row.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:261-278`
pub fn SV_ParseSurfaceParm(cm: &CollisionWorld, shader: *mut CCMShader, text: &mut &str) {
    let numsvInfoParms: c_int = cm.svInfoParms.len() as c_int;

    let (token, rest) = COM_Parse(*text, false);
    *text = rest;
    for i in 0..numsvInfoParms {
        let row = &cm.svInfoParms[i as usize];
        if Q_stricmp(&token, row.name) == 0 {
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
    if cm.cmShaderTable.count() != 0 {
        //		Com_Printf("Shutting down cmShaderTable .....\n");
        cm.cmShaderTable.clear();
    }
}

/// Raven `CM_GetShaderInfo( int shaderNum )` — the BSP-index overload:
/// bounds-checked pointer into `cmg.shaders`.
///
/// Raven overloads `CM_GetShaderInfo` by parameter type (`int` vs
/// `const char *`); Rust has no overloading, so the by-name form below is
/// `CM_GetShaderInfo_ByName`.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:526-536`
pub fn CM_GetShaderInfo(cm: &mut CollisionWorld, shaderNum: c_int) -> *mut CCMShader {
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
pub fn CM_CreateShaderTextHash(view: &mut EngineHostView) {
    if view.cm.shaderText.is_null() {
        return;
    }
    let base = view.cm.shaderText as usize;
    // Borrow the raw `shaderText` buffer as `&str` so the native `COM_Parse`
    // cursor can walk it; shader files are ASCII, so a byte offset into the
    // view maps 1:1 onto `shaderText` (§19: non-UTF-8 shader text hashes as
    // empty). Raven's redundant `SkipWhitespace` before `COM_ParseExt` leaves
    // the post-token cursor identical, so it is dropped. Raven
    // `new CCMShaderText(token, p)` → name → byte offset of the cursor past the
    // label (§17).
    // Source: `oracle/codemp/qcommon/cm_shader.cpp:37-59`
    let text = unsafe { core::ffi::CStr::from_ptr(view.cm.shaderText) }
        .to_str()
        .unwrap_or("");
    let mut cursor = text;
    // look for label
    loop {
        let (token, rest) = COM_Parse(cursor, true);
        if token.is_empty() {
            break;
        }
        let offset = (rest.as_ptr() as usize) - base;
        view.cm.shaderTextTable.insert(token, offset);
        cursor = SkipBracedSection(rest);
    }
}

/// Raven `CM_GetShaderText` — look up the raw shader-text block for `key`.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:158-168`
pub fn CM_GetShaderText(view: &mut EngineHostView, key: *const c_char) -> *const c_char {
    // Raven `st = shaderTextTable[key]; return st ? st->GetData() : NULL`.
    // The map yields the stored byte offset; `GetData` (the captured `mData`)
    // is `shaderText + offset`.
    // Source: `oracle/codemp/qcommon/cm_shader.cpp:158-168`
    let key_str = unsafe { core::ffi::CStr::from_ptr(key) }.to_string_lossy();
    if let Some(&offset) = view.cm.shaderTextTable.get(key_str.as_ref()) {
        return unsafe { view.cm.shaderText.add(offset) } as *const c_char;
    }
    core::ptr::null()
}

/// Raven `CM_LoadShaderFiles` — scan `shaders/*.shader` (+ `shaders/test/*`
/// outside `FINAL_BUILD`), read and concatenate them into one `shaderText`
/// buffer.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:71-150`
pub fn CM_LoadShaderFiles(view: &mut EngineHostView) {
    // scan for shader files
    let shaderFiles1 = FS_ListFiles(view, "shaders", ".shader");
    let shaderFiles2 = FS_ListFiles(view, "shaders/test", ".shader");

    if shaderFiles1.is_empty() {
        crate::common::com_printf(
            view.common,
            &format!("{}WARNING: no shader files found\n", S_COLOR_YELLOW),
        );
        return;
    }

    let numShaders1 = shaderFiles1.len() as c_int;
    let mut numShaders = numShaders1 + shaderFiles2.len() as c_int;
    if numShaders > MAX_SHADER_FILES as c_int {
        numShaders = MAX_SHADER_FILES as c_int;
    }

    let mut buffers: [*mut c_char; MAX_SHADER_FILES] = [core::ptr::null_mut(); MAX_SHADER_FILES];
    let mut sum: c_int = 0;
    let mut i: c_int = 0;

    // load and parse shader files
    while i < numShaders1 {
        let filename = format!("shaders/{}", shaderFiles1[i as usize]);
        Com_DPrintf(view.common, &format!("...loading '{filename}'\n"));
        sum += unsafe {
            FS_ReadFile(
                view,
                &filename,
                buffers.as_mut_ptr().add(i as usize) as *mut *mut (),
            )
        };
        if unsafe { *buffers.as_ptr().add(i as usize) }.is_null() {
            com_error(errorParm_t::ERR_FATAL, format!("Couldn't load {filename}"));
        }
        i += 1;
    }
    while i < numShaders {
        let filename = format!("shaders/test/{}", shaderFiles2[(i - numShaders1) as usize]);
        Com_DPrintf(view.common, &format!("...loading '{filename}'\n"));
        sum += unsafe {
            FS_ReadFile(
                view,
                &filename,
                buffers.as_mut_ptr().add(i as usize) as *mut *mut (),
            )
        };
        if unsafe { *buffers.as_ptr().add(i as usize) }.is_null() {
            com_error(errorParm_t::ERR_DROP, format!("Couldn't load {filename}"));
        }
        i += 1;
    }

    // build single large buffer
    view.cm.shaderText = Z_Malloc(
        view,
        (sum + numShaders * 2) as c_int,
        memtag_t::TAG_SHADERTEXT,
        mp_qshared::shared::qtrue,
        4,
    ) as *mut c_char;

    // free in reverse order, so the temp files are all dumped
    let mut j = numShaders - 1;
    while j >= 0 {
        unsafe {
            libc::strcat(view.cm.shaderText, c"\n".as_ptr());
            libc::strcat(view.cm.shaderText, buffers[j as usize]);
        }
        FS_FreeFile(view.common, buffers[j as usize] as *mut ());
        j -= 1;
    }

    // (Raven's FS_FreeFileList pair: the owned lists free themselves.)
}

/// Raven `CM_FreeShaderText` — release the cached shader-text buffer and
/// per-name hash.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:176-184`
pub fn CM_FreeShaderText(common: &mut Common, cm: &mut CollisionWorld) {
    cm.shaderTextTable.clear();
    if !cm.shaderText.is_null() {
        Z_Free(common, cm.shaderText as *mut ());
        cm.shaderText = core::ptr::null_mut();
    }
}

/// Raven `SV_ParseMaterial` — match the next token against `svMaterialNames`,
/// OR-ing its index into the shader's surface flags.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:290-309`
pub fn SV_ParseMaterial(
    common: &mut Common,
    cm: &CollisionWorld,
    shader: *mut CCMShader,
    text: &mut &str,
) {
    let (token, rest) = COM_Parse(*text, false);
    *text = rest;
    if token.is_empty() {
        crate::common::com_printf(
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
        if Q_stricmp(&token, cm.svMaterialNames[i as usize]) == 0 {
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
    text: &mut &str,
    count: c_int,
    v: *mut f32,
) -> qboolean {
    let (token, rest) = COM_Parse(*text, false);
    *text = rest;
    if token != "(" {
        crate::common::com_printf(
            common,
            &format!(
                "{}WARNING: missing parenthesis in shader '{}'\n",
                S_COLOR_YELLOW,
                unsafe { core::ffi::CStr::from_ptr((*shader).shader.as_ptr()).to_string_lossy() }
            ),
        );
        return qfalse;
    }

    for i in 0..count {
        let (token, rest) = COM_Parse(*text, false);
        *text = rest;
        if token.is_empty() {
            crate::common::com_printf(
                common,
                &format!(
                    "{}WARNING: missing vector element in shader '{}'\n",
                    S_COLOR_YELLOW,
                    unsafe {
                        core::ffi::CStr::from_ptr((*shader).shader.as_ptr()).to_string_lossy()
                    }
                ),
            );
            return qfalse;
        }
        unsafe {
            *v.add(i as usize) = atof(&token) as f32;
        }
    }

    let (token, rest) = COM_Parse(*text, false);
    *text = rest;
    if token != ")" {
        crate::common::com_printf(
            common,
            &format!(
                "{}WARNING: missing parenthesis in shader '{}'\n",
                S_COLOR_YELLOW,
                unsafe { core::ffi::CStr::from_ptr((*shader).shader.as_ptr()).to_string_lossy() }
            ),
        );
        return qfalse;
    }
    qtrue
}

/// Raven `CM_LoadShaderText` — (re)build the cached shader-text buffer if
/// missing (or forced).
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:195-210`
pub fn CM_LoadShaderText(view: &mut EngineHostView, forceReload: qboolean) {
    if forceReload != 0 {
        CM_FreeShaderText(view.common, view.cm);
    }
    if !view.cm.shaderText.is_null() {
        return;
    }
    //	Com_Printf("Loading shader text .....\n");
    CM_LoadShaderFiles(view);
    CM_CreateShaderTextHash(view);

    //Com_Printf("..... %d shader definitions loaded\n", shaderTextTable.count());
}

/// Raven `CM_ParseShader` — parse one `{ ... }` shader body: `surfaceParm`,
/// `material`/`q3map_material`, `sun`/`q3map_sun` (values discarded — dead
/// per Raven's own comments), and `fogParms` keywords.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:361-454`
pub fn CM_ParseShader(
    common: &mut Common,
    cm: &CollisionWorld,
    shader: *mut CCMShader,
    text: &mut &str,
) {
    let (token, rest) = COM_Parse(*text, true);
    *text = rest;
    if token.as_bytes().first() != Some(&b'{') {
        crate::common::com_printf(
            common,
            &format!(
                "{}WARNING: expecting '{{', found '{}' instead in shader '{}'\n",
                S_COLOR_YELLOW,
                token,
                unsafe { core::ffi::CStr::from_ptr((*shader).shader.as_ptr()).to_string_lossy() }
            ),
        );
        return;
    }

    loop {
        let (token, rest) = COM_Parse(*text, true);
        *text = rest;
        if token.is_empty() {
            crate::common::com_printf(
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

        let c0 = token.as_bytes().first().copied().unwrap_or(0);
        // end of shader definition
        if c0 == b'}' {
            break;
        }
        // stage definition
        else if c0 == b'{' {
            *text = SkipBracedSection(*text);
            continue;
        }
        // material deprecated as of 11 Jan 01
        // material undeprecated as of 7 May 01 - q3map_material deprecated
        else if (Q_stricmp(&token, "material") == 0) || (Q_stricmp(&token, "q3map_material") == 0) {
            SV_ParseMaterial(common, cm, shader, text);
        }
        // sun parms
        // q3map_sun deprecated as of 11 Jan 01
        else if (Q_stricmp(&token, "sun") == 0) || (Q_stricmp(&token, "q3map_sun") == 0) {
            //			float	a, b;

            let (_, rest) = COM_Parse(*text, false);
            *text = rest;
            //			shader->sunLight[0] = atof( token );
            let (_, rest) = COM_Parse(*text, false);
            *text = rest;
            //			shader->sunLight[1] = atof( token );
            let (_, rest) = COM_Parse(*text, false);
            *text = rest;
            //			shader->sunLight[2] = atof( token );

            //			VectorNormalize( shader->sunLight );

            let (_, rest) = COM_Parse(*text, false);
            *text = rest;
            //			a = atof( token );
            //			VectorScale( shader->sunLight, a, shader->sunLight);

            let (_, rest) = COM_Parse(*text, false);
            *text = rest;
            //			a = DEG2RAD(atof( token ));

            let (_, rest) = COM_Parse(*text, false);
            *text = rest;
            //			b = DEG2RAD(atof( token ));

            //			shader->sunDirection[0] = cos( a ) * cos( b );
            //			shader->sunDirection[1] = sin( a ) * cos( b );
            //			shader->sunDirection[2] = sin( b );
        } else if Q_stricmp(&token, "surfaceParm") == 0 {
            SV_ParseSurfaceParm(cm, shader, text);
            continue;
        } else if Q_stricmp(&token, "fogParms") == 0 {
            let mut fogColor: vec3_t = vec3_t::default();
            if CM_ParseVector(common, shader, text, 3, fogColor.as_mut_ptr()) == 0 {
                return;
            }

            let (token, rest) = COM_Parse(*text, false);
            *text = rest;
            if token.is_empty() {
                crate::common::com_printf(
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
            *text = SkipRestOfLine(*text);
            continue;
        }
    }
}

/// Raven `CM_SetupShaderProperties` — populate `cmShaderTable` from every
/// loaded BSP shader, then parse each one's shader-script text.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:466-487`
pub fn CM_SetupShaderProperties(view: &mut EngineHostView) {
    // Add all basic shaders to the cmShaderTable
    let numShaders = view.cm.cmg.numShaders;
    for i in 0..numShaders {
        let s = CM_GetShaderInfo(view.cm, i);
        view.cm.cmShaderTable.insert(s);
    }
    // Go through and parse evaluate shader names to shadernums
    for i in 0..numShaders {
        let shader = CM_GetShaderInfo(view.cm, i);
        let def = CM_GetShaderText(view, unsafe { (*shader).shader.as_ptr() });
        if !def.is_null() {
            let def_str = unsafe { core::ffi::CStr::from_ptr(def) }
                .to_string_lossy()
                .into_owned();
            let mut cursor: &str = &def_str;
            CM_ParseShader(view.common, view.cm, shader, &mut cursor);
        }
    }
}

/// Raven `CM_GetShaderInfo( const char *name )` — the by-name overload:
/// looks up (or lazily allocates + parses) the `CCMShader` for `name`.
///
/// Disambiguated from the by-index overload (`CM_GetShaderInfo` above) since
/// Rust has no overloading.
///
/// Source: `oracle/codemp/qcommon/cm_shader.cpp:498-524`
pub fn CM_GetShaderInfo_ByName(view: &mut EngineHostView, name: *const c_char) -> *mut CCMShader {
    let mut out = view.cm.cmShaderTable[name];
    if !out.is_null() {
        return out;
    }

    // Create a new CCMShader class
    out = Hunk_Alloc(
        view,
        core::mem::size_of::<CCMShader>() as c_int,
        ha_pref::h_high,
    ) as *mut CCMShader;
    // Set defaults
    unsafe {
        Q_strncpyz((*out).shader.as_mut_ptr(), name, MAX_QPATH as c_int);
        (*out).contentFlags = CONTENTS_SOLID | CONTENTS_OPAQUE;
    }

    // Parse in any text if it exists
    let def = CM_GetShaderText(view, name);
    if !def.is_null() {
        let def_str = unsafe { core::ffi::CStr::from_ptr(def) }
            .to_string_lossy()
            .into_owned();
        let mut cursor: &str = &def_str;
        CM_ParseShader(view.common, view.cm, out, &mut cursor);
    }

    view.cm.cmShaderTable.insert(out);
    out
}

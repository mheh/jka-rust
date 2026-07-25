//! `DisplayContext` — the trait that replaces Raven's `displayContextDef_t`.

use core::ffi::{c_int, c_void};

use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::shared::{pc_token_t, qhandle_t, sfxHandle_t, vec3_t, vec4_t};

use super::item_id::ItemId;

/// Everything the menu framework needs from the module hosting it.
///
/// Raven's `displayContextDef_t` was a ~52-entry function-pointer vtable each
/// host filled at init (`Init_Display(&uiInfo.uiDC)` in ui, `cgDC` in cgame)
/// and `ui_shared.c` called through a file-scope `DC` pointer. DEC-36 D3
/// retires that struct: the function pointers become this trait — an open set
/// with two implementors, which is exactly the case the translation dictionary
/// reserves a trait for — and the data tail becomes `DisplayState`.
///
/// The signatures are the dictionary's: `const char *` → `&str`, `qboolean` →
/// `bool`, out-params → returns, `char *buf, int buflen` → returned `String`.
///
/// The trait carries ONLY the callback surface (U3 ruling, 2026-07-24):
/// `MenuSystem` and `DisplayState` are NOT reached through it — they thread
/// beside it as struct fields of the host's context (the `GameContext.world` /
/// `.engine` precedent), so framework code gets field-level split borrows: a
/// paint loop can hold an arena item while calling a draw callback. Raven
/// reached both through `DC->` and file-scope globals; the data tail
/// (`DC->realTime`, `DC->Assets.*`, `DC->cursorx`) lives in `DisplayState`.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:400-477`
#[allow(non_snake_case)]
pub trait DisplayContext {
    // ---- Raven `displayContextDef_t` function pointers ----

    /// Raven `qhandle_t (*registerShaderNoMip)(const char *p)`.
    fn registerShaderNoMip(&mut self, p: &str) -> qhandle_t;

    /// Raven `void (*setColor)(const vec4_t v)` — `NULL` resets to white.
    fn setColor(&mut self, v: Option<vec4_t>);

    /// Raven `void (*drawHandlePic)(float x, float y, float w, float h, qhandle_t asset)`.
    fn drawHandlePic(&mut self, x: f32, y: f32, w: f32, h: f32, asset: qhandle_t);

    /// Raven `void (*drawStretchPic)(...)`.
    #[allow(clippy::too_many_arguments)]
    fn drawStretchPic(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        s1: f32,
        t1: f32,
        s2: f32,
        t2: f32,
        hShader: qhandle_t,
    );

    /// Raven `void (*drawText)(float x, float y, float scale, vec4_t color,
    /// const char *text, float adjust, int limit, int style, int iMenuFont)`.
    #[allow(clippy::too_many_arguments)]
    fn drawText(
        &mut self,
        x: f32,
        y: f32,
        scale: f32,
        color: vec4_t,
        text: &str,
        adjust: f32,
        limit: c_int,
        style: c_int,
        iMenuFont: c_int,
    );

    /// Raven `int (*textWidth)(const char *text, float scale, int iMenuFont)`.
    fn textWidth(&mut self, text: &str, scale: f32, iMenuFont: c_int) -> c_int;

    /// Raven `int (*textHeight)(const char *text, float scale, int iMenuFont)`.
    fn textHeight(&mut self, text: &str, scale: f32, iMenuFont: c_int) -> c_int;

    /// Raven `qhandle_t (*registerModel)(const char *p)`.
    fn registerModel(&mut self, p: &str) -> qhandle_t;

    /// Raven `void (*modelBounds)(qhandle_t model, vec3_t min, vec3_t max)` —
    /// the two out-params become the return value.
    fn modelBounds(&mut self, model: qhandle_t) -> (vec3_t, vec3_t);

    /// Raven `void (*fillRect)(float x, float y, float w, float h, const vec4_t color)`.
    fn fillRect(&mut self, x: f32, y: f32, w: f32, h: f32, color: vec4_t);

    /// Raven `void (*drawRect)(float x, float y, float w, float h, float size, const vec4_t color)`.
    fn drawRect(&mut self, x: f32, y: f32, w: f32, h: f32, size: f32, color: vec4_t);

    /// Raven `void (*drawSides)(float x, float y, float w, float h, float size)`.
    fn drawSides(&mut self, x: f32, y: f32, w: f32, h: f32, size: f32);

    /// Raven `void (*drawTopBottom)(float x, float y, float w, float h, float size)`.
    fn drawTopBottom(&mut self, x: f32, y: f32, w: f32, h: f32, size: f32);

    /// Raven `void (*clearScene)()`.
    fn clearScene(&mut self);

    /// Raven `void (*addRefEntityToScene)(const refEntity_t *re)`.
    fn addRefEntityToScene(&mut self, re: &refEntity_t);

    /// Raven `void (*renderScene)(const refdef_t *fd)`.
    fn renderScene(&mut self, fd: &refdef_t);

    /// Raven `qhandle_t (*RegisterFont)(const char *fontName)`.
    fn RegisterFont(&mut self, fontName: &str) -> qhandle_t;

    /// Raven `int (*Font_StrLenPixels)(const char *text, const int iFontIndex, const float scale)`.
    fn Font_StrLenPixels(&mut self, text: &str, iFontIndex: c_int, scale: f32) -> c_int;

    /// Raven `int (*Font_StrLenChars)(const char *text)`.
    fn Font_StrLenChars(&mut self, text: &str) -> c_int;

    /// Raven `int (*Font_HeightPixels)(const int iFontIndex, const float scale)`.
    fn Font_HeightPixels(&mut self, iFontIndex: c_int, scale: f32) -> c_int;

    /// Raven `void (*Font_DrawString)(int ox, int oy, const char *text,
    /// const float *rgba, const int setIndex, int iCharLimit, const float scale)`.
    #[allow(clippy::too_many_arguments)]
    fn Font_DrawString(
        &mut self,
        ox: c_int,
        oy: c_int,
        text: &str,
        rgba: vec4_t,
        setIndex: c_int,
        iCharLimit: c_int,
        scale: f32,
    );

    /// Raven `qboolean (*Language_IsAsian)(void)`.
    fn Language_IsAsian(&mut self) -> bool;

    /// Raven `qboolean (*Language_UsesSpaces)(void)`.
    fn Language_UsesSpaces(&mut self) -> bool;

    /// Raven `unsigned int (*AnyLanguage_ReadCharFromString)(const char *psText,
    /// int *piAdvanceCount, qboolean *pbIsTrailingPunctuation)` — returns
    /// `(character, advance count, is trailing punctuation)`. The text argument
    /// stays bytes: the string package is Latin-1/multi-byte and this call
    /// decodes it.
    fn AnyLanguage_ReadCharFromString(&mut self, psText: &[u8]) -> (u32, c_int, bool);

    /// Raven `void (*ownerDrawItem)(...)`.
    #[allow(clippy::too_many_arguments)]
    fn ownerDrawItem(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        text_x: f32,
        text_y: f32,
        ownerDraw: c_int,
        ownerDrawFlags: c_int,
        align: c_int,
        special: f32,
        scale: f32,
        color: vec4_t,
        shader: qhandle_t,
        textStyle: c_int,
        iMenuFont: c_int,
    );

    /// Raven `float (*getValue)(int ownerDraw)`.
    fn getValue(&mut self, ownerDraw: c_int) -> f32;

    /// Raven `qboolean (*ownerDrawVisible)(int flags)`.
    fn ownerDrawVisible(&mut self, flags: c_int) -> bool;

    /// Raven `void (*runScript)(char **p)` — the callee consumes tokens off the
    /// script cursor, so the `char **` becomes a `&str` cursor.
    fn runScript(&mut self, p: &mut &str);

    /// Raven `qboolean (*deferScript)(char **p)`.
    fn deferScript(&mut self, p: &mut &str) -> bool;

    /// Raven `void (*getTeamColor)(vec4_t *color)` — out-param becomes the
    /// return value.
    fn getTeamColor(&mut self) -> vec4_t;

    /// Raven `void (*getCVarString)(const char *cvar, char *buffer, int bufsize)`
    /// — `bufsize` keeps the engine-side truncation width at the call site.
    fn getCVarString(&mut self, cvar: &str, bufsize: usize) -> String;

    /// Raven `float (*getCVarValue)(const char *cvar)`.
    fn getCVarValue(&mut self, cvar: &str) -> f32;

    /// Raven `void (*setCVar)(const char *cvar, const char *value)`.
    fn setCVar(&mut self, cvar: &str, value: &str);

    /// Raven `void (*drawTextWithCursor)(float x, float y, float scale,
    /// vec4_t color, const char *text, int cursorPos, char cursor, int limit,
    /// int style, int iFontIndex)`.
    #[allow(clippy::too_many_arguments)]
    fn drawTextWithCursor(
        &mut self,
        x: f32,
        y: f32,
        scale: f32,
        color: vec4_t,
        text: &str,
        cursorPos: c_int,
        cursor: u8,
        limit: c_int,
        style: c_int,
        iFontIndex: c_int,
    );

    /// Raven `void (*setOverstrikeMode)(qboolean b)`.
    fn setOverstrikeMode(&mut self, b: bool);

    /// Raven `qboolean (*getOverstrikeMode)()`.
    fn getOverstrikeMode(&mut self) -> bool;

    /// Raven `void (*startLocalSound)(sfxHandle_t sfx, int channelNum)`.
    fn startLocalSound(&mut self, sfx: sfxHandle_t, channelNum: c_int);

    /// Raven `qboolean (*ownerDrawHandleKey)(int ownerDraw, int flags,
    /// float *special, int key)` — `special` stays an in/out borrow, the
    /// handlers both read and rewrite it.
    fn ownerDrawHandleKey(
        &mut self,
        ownerDraw: c_int,
        flags: c_int,
        special: &mut f32,
        key: c_int,
    ) -> bool;

    /// Raven `int (*feederCount)(float feederID)`.
    fn feederCount(&mut self, feederID: f32) -> c_int;

    /// Raven `const char *(*feederItemText)(float feederID, int index,
    /// int column, qhandle_t *handle1, qhandle_t *handle2, qhandle_t *handle3)`
    /// — the three out-param handles join the return value; `None` is Raven's
    /// NULL return.
    fn feederItemText(
        &mut self,
        feederID: f32,
        index: c_int,
        column: c_int,
    ) -> (Option<String>, qhandle_t, qhandle_t, qhandle_t);

    /// Raven `qhandle_t (*feederItemImage)(float feederID, int index)`.
    fn feederItemImage(&mut self, feederID: f32, index: c_int) -> qhandle_t;

    /// Raven `qboolean (*feederSelection)(float feederID, int index, itemDef_t *item)`.
    fn feederSelection(&mut self, feederID: f32, index: c_int, item: Option<ItemId>) -> bool;

    /// Raven `void (*keynumToStringBuf)(int keynum, char *buf, int buflen)`.
    fn keynumToStringBuf(&mut self, keynum: c_int, buflen: usize) -> String;

    /// Raven `void (*getBindingBuf)(int keynum, char *buf, int buflen)`.
    fn getBindingBuf(&mut self, keynum: c_int, buflen: usize) -> String;

    /// Raven `void (*setBinding)(int keynum, const char *binding)`.
    fn setBinding(&mut self, keynum: c_int, binding: &str);

    /// Raven `void (*executeText)(int exec_when, const char *text)`.
    fn executeText(&mut self, exec_when: c_int, text: &str);

    /// Raven `void (*Error)(int level, const char *error, ...)` — the variadic
    /// tail is formatted at the call site (`format!`), as everywhere else in
    /// this port.
    fn Error(&mut self, level: c_int, error: &str);

    /// Raven `void (*Print)(const char *msg, ...)`.
    fn Print(&mut self, msg: &str);

    /// Raven `void (*Pause)(qboolean b)`.
    fn Pause(&mut self, b: bool);

    /// Raven `int (*ownerDrawWidth)(int ownerDraw, float scale)`.
    fn ownerDrawWidth(&mut self, ownerDraw: c_int, scale: f32) -> c_int;

    /// Raven `sfxHandle_t (*registerSound)(const char *name)`.
    fn registerSound(&mut self, name: &str) -> sfxHandle_t;

    /// Raven `void (*startBackgroundTrack)(const char *intro, const char *loop,
    /// qboolean bReturnWithoutStarting)`.
    fn startBackgroundTrack(&mut self, intro: &str, loop_: &str, bReturnWithoutStarting: bool);

    /// Raven `void (*stopBackgroundTrack)()`.
    fn stopBackgroundTrack(&mut self);

    /// Raven `int (*playCinematic)(const char *name, float x, float y, float w, float h)`.
    fn playCinematic(&mut self, name: &str, x: f32, y: f32, w: f32, h: f32) -> c_int;

    /// Raven `void (*stopCinematic)(int handle)`.
    fn stopCinematic(&mut self, handle: c_int);

    /// Raven `void (*drawCinematic)(int handle, float x, float y, float w, float h)`.
    fn drawCinematic(&mut self, handle: c_int, x: f32, y: f32, w: f32, h: f32);

    /// Raven `void (*runCinematicFrame)(int handle)`.
    fn runCinematicFrame(&mut self, handle: c_int);

    // ---- Host trap surface beyond the fn-pointer table ----
    //
    // Raven's `ui_shared.c` also calls these module traps directly (both hosts
    // compile the TU against their own syscall table). `mp_uishared` is
    // host-agnostic, so they route through this trait too — same D3 seam, same
    // dictionary shapes as the host's `trap.rs` wrappers, so each impl is pure
    // delegation. Census: `grep -o "trap_[A-Za-z0-9_]*" oracle/codemp/ui/ui_shared.c`.

    /// Raven `trap_Milliseconds()`.
    fn Milliseconds(&mut self) -> c_int;

    /// Raven `trap_Cvar_SetValue(const char *var_name, float value)`.
    fn setCVarValue(&mut self, cvar: &str, value: f32);

    /// Raven `trap_Key_IsDown(int keynum)`.
    fn Key_IsDown(&mut self, keynum: c_int) -> bool;

    /// Raven `trap_Key_GetCatcher()`.
    fn Key_GetCatcher(&mut self) -> c_int;

    /// Raven `trap_Key_SetCatcher(int catcher)`.
    fn Key_SetCatcher(&mut self, catcher: c_int);

    /// Raven `trap_Key_ClearStates()`.
    fn Key_ClearStates(&mut self);

    /// Raven `trap_PC_ReadToken(int handle, pc_token_t *pc_token)`.
    fn PC_ReadToken(&mut self, handle: c_int, pc_token: &mut pc_token_t) -> bool;

    /// Raven `trap_PC_SourceFileAndLine(int handle, char *filename, int *line)`
    /// — returns `(status, filename, line)`.
    fn PC_SourceFileAndLine(&mut self, handle: c_int, buffer_len: usize) -> (c_int, String, c_int);

    /// Raven `trap_SP_GetStringTextString(const char *text, char *buffer, int
    /// bufferLength)` — `None` when the lookup fails.
    fn SP_GetStringTextString(&mut self, text: &str, buffer_len: usize) -> Option<String>;

    /// Raven `trap_R_RegisterSkin(const char *name)`.
    fn R_RegisterSkin(&mut self, name: &str) -> qhandle_t;

    /// Raven `trap_GetLanguageName(const int languageIndex, char *buffer)`.
    fn GetLanguageName(&mut self, languageIndex: c_int, buffer_len: usize) -> String;

    /// Raven `trap_G2API_InitGhoul2Model(...)` — ghoul2 handles stay opaque
    /// `*mut c_void` (U1 convention).
    #[allow(clippy::too_many_arguments)]
    fn G2API_InitGhoul2Model(
        &mut self,
        ghoul2Ptr: *mut *mut c_void,
        fileName: &str,
        modelIndex: c_int,
        customSkin: qhandle_t,
        customShader: qhandle_t,
        modelFlags: c_int,
        lodBias: c_int,
    ) -> c_int;

    /// Raven `trap_G2API_SetSkin(void *ghoul2, int modelIndex, qhandle_t
    /// customSkin, qhandle_t renderSkin)`.
    fn G2API_SetSkin(
        &mut self,
        ghoul2: *mut c_void,
        modelIndex: c_int,
        customSkin: qhandle_t,
        renderSkin: qhandle_t,
    ) -> bool;

    /// Raven `trap_G2API_CleanGhoul2Models(void **ghoul2Ptr)`.
    fn G2API_CleanGhoul2Models(&mut self, ghoul2Ptr: *mut *mut c_void);

    /// Raven `trap_G2API_SetBoneAnim(...)`.
    #[allow(clippy::too_many_arguments)]
    fn G2API_SetBoneAnim(
        &mut self,
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boneName: &str,
        startFrame: c_int,
        endFrame: c_int,
        flags: c_int,
        animSpeed: f32,
        currentTime: c_int,
        setFrame: f32,
        blendTime: c_int,
    ) -> bool;

    /// Raven `trap_G2API_GetGLAName(void *ghoul2, int modelIndex, char *fillBuf)`.
    fn G2API_GetGLAName(&mut self, ghoul2: *mut c_void, modelIndex: c_int, buffer_len: usize)
        -> String;

    /// Raven `trap_G2_HaveWeGhoul2Models(void *ghoul2)`.
    fn G2_HaveWeGhoul2Models(&mut self, ghoul2: *mut c_void) -> bool;
}

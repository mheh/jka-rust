//! `impl DisplayContext for CgContext` — Raven's `CG_LoadHudMenu` vtable wiring
//! (DEC-36 D3, DEC-46.1), the cgame twin of `mp_ui`'s `ui_display_context`.
//!
//! Raven filled a file-scope `displayContextDef_t cgDC` at hud-menu load
//! (`cgDC.<slot> = &CG_Xxx` / `= trap_Xxx`,
//! `oracle/codemp/cgame/cg_main.c:3149-3205`, then `Init_Display(&cgDC)`) and
//! `ui_shared.c` — the same TU cgame compiles with `CGAME` defined — called
//! through its `DC` pointer. This impl IS that table: every method is a
//! `trap_*` forwarder, the `cg_*.c` callback Raven installed, or one of the
//! slots cgame leaves empty.
//!
//! Three dispositions beyond plain forwarding, each named at its method:
//!
//! - **Seven slots cgame never fills.** `//cgDC.setOverstrikeMode`,
//!   `getOverstrikeMode`, `setBinding`, `getBindingBuf`, `keynumToStringBuf`,
//!   `executeText` and `Pause` are commented out in `CG_LoadHudMenu`, so
//!   `cgDC` holds NULL there (the struct is file-scope, zero-initialized).
//!   Raven itself null-checks only two of them before calling (`if (DC &&
//!   DC->getBindingBuf)` at `ui_shared.c:375`, `if (DC->Pause)` at
//!   `ui_shared.c:4481`) - the other five (`setOverstrikeMode`,
//!   `getOverstrikeMode`, `setBinding`, `keynumToStringBuf`, `executeText`)
//!   are called unconditionally from `ui_shared.c`, which would deref NULL in
//!   Raven if cgame's huds ever walked those paths. `mp_uishared` is
//!   host-agnostic and calls all seven unconditionally, so each lands here as
//!   a no-op returning the neutral value - a deliberate defined-behavior pick
//!   for the five unguarded slots (§F19), not a path-unreachable no-op; a
//!   panic would invent a crash Raven's null derefs never got the chance to
//!   have either.
//!
//! - **Six `#ifndef CGAME` hooks.** The saber, character-animation and
//!   species/language slots at the tail of the trait are blocks `ui_shared.c`
//!   compiles out of the cgame TU entirely. The cgame arm of each guard is
//!   "nothing happens", so they are no-ops / `None` / empty lists here, cited
//!   to the guard.
//!
//! - **Seven §20 dead slots.** `drawStretchPic`, the four `Font_*`,
//!   `Language_IsAsian` and `Error` are filled by `CG_LoadHudMenu` exactly as
//!   `_UI_Init` fills them, and no `DC->` call site for any of them exists
//!   anywhere in `oracle/codemp` — the same census `mp_ui`'s impl ran. They
//!   panic with their subject rather than forwarding a trap nothing calls.
//!
//! Targets that land with a later C5 wave carry `todo!()` naming the Raven fn
//! and its source lines; they are never quietly neutralized.

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};

use mp_bg::public::animation::animation_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::shared::{pc_token_t, qhandle_t, sfxHandle_t, vec3_t, vec4_t};
use mp_uishared::shared::display_context::DisplayContext;
use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::item_def_s::ItemDef;
use mp_uishared::shared::item_id::ItemId;
use mp_uishared::shared::menu_system::MenuSystem;

use crate::cg_draw::{CG_Text_Height, CG_Text_Paint, CG_Text_Width};
use crate::cg_drawtools::{CG_DrawPic, CG_DrawRect, CG_DrawSides, CG_DrawTopBottom, CG_FillRect};
use crate::cg_main::{
    CG_Cvar_Get, CG_DrawCinematic, CG_FeederCount, CG_FeederItemImage, CG_FeederSelection,
    CG_OwnerDrawHandleKey, CG_PlayCinematic, CG_Printf, CG_RunCinematicFrame, CG_StopCinematic,
};
use crate::cg_new_draw::{CG_DeferMenuScript, CG_GetTeamColor, CG_OwnerDraw, CG_RunMenuScript};
use crate::trap;
use crate::world::cg_context::CgContext;

impl<'e> DisplayContext for CgContext<'e> {
    /// Raven `cgDC.registerShaderNoMip = &trap_R_RegisterShaderNoMip`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3149`
    fn registerShaderNoMip(&mut self, p: &str) -> qhandle_t {
        trap::R_RegisterShaderNoMip(self.engine, p)
    }

    /// Raven `cgDC.setColor = &trap_R_SetColor` — cgame wires the trap
    /// directly where ui goes through `UI_SetColor`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3150`
    fn setColor(&mut self, v: Option<vec4_t>) {
        trap::R_SetColor(self.engine, v.as_ref())
    }

    /// Raven `cgDC.drawHandlePic = &CG_DrawPic`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3151`
    fn drawHandlePic(&mut self, x: f32, y: f32, w: f32, h: f32, asset: qhandle_t) {
        CG_DrawPic(self, x, y, w, h, asset)
    }

    /// Raven `cgDC.drawStretchPic = &trap_R_DrawStretchPic` — §20 DEAD
    /// SURFACE: the slot is filled and nothing ever calls `DC->drawStretchPic`
    /// (zero hits across `oracle/codemp`).
    /// Source: `oracle/codemp/cgame/cg_main.c:3152`, slot
    /// `oracle/codemp/ui/ui_shared.h:404`
    fn drawStretchPic(
        &mut self,
        _x: f32,
        _y: f32,
        _w: f32,
        _h: f32,
        _s1: f32,
        _t1: f32,
        _s2: f32,
        _t2: f32,
        _hShader: qhandle_t,
    ) {
        unreachable!(
            "DisplayContext::drawStretchPic — dead vtable slot, no DC-> call site in Raven"
        )
    }

    /// Raven `cgDC.drawText = &CG_Text_Paint`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3153`
    fn drawText(
        &mut self,
        ds: &DisplayState,
        x: f32,
        y: f32,
        scale: f32,
        color: vec4_t,
        text: &str,
        adjust: f32,
        limit: c_int,
        style: c_int,
        iMenuFont: c_int,
    ) {
        CG_Text_Paint(
            self, ds, x, y, scale, color, text, adjust, limit, style, iMenuFont,
        )
    }

    /// Raven `cgDC.textWidth = &CG_Text_Width`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3154`
    fn textWidth(&mut self, ds: &DisplayState, text: &str, scale: f32, iMenuFont: c_int) -> c_int {
        CG_Text_Width(self, ds, text, scale, iMenuFont)
    }

    /// Raven `cgDC.textHeight = &CG_Text_Height`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3155`
    fn textHeight(&mut self, ds: &DisplayState, text: &str, scale: f32, iMenuFont: c_int) -> c_int {
        CG_Text_Height(self, ds, text, scale, iMenuFont)
    }

    /// Raven `cgDC.registerModel = &trap_R_RegisterModel`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3156`
    fn registerModel(&mut self, p: &str) -> qhandle_t {
        trap::R_RegisterModel(self.engine, p)
    }

    /// Raven `cgDC.modelBounds = &trap_R_ModelBounds` — cgame's trap keeps
    /// Raven's two out-params, so they are filled here and returned.
    /// Source: `oracle/codemp/cgame/cg_main.c:3157`
    fn modelBounds(&mut self, model: qhandle_t) -> (vec3_t, vec3_t) {
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        trap::R_ModelBounds(self.engine, model, &mut mins, &mut maxs);
        (mins, maxs)
    }

    /// Raven `cgDC.fillRect = &CG_FillRect`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3158`
    fn fillRect(&mut self, _ds: &DisplayState, x: f32, y: f32, w: f32, h: f32, color: vec4_t) {
        CG_FillRect(self, x, y, w, h, &color)
    }

    /// Raven `cgDC.drawRect = &CG_DrawRect`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3159`
    fn drawRect(
        &mut self,
        _ds: &DisplayState,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        size: f32,
        color: vec4_t,
    ) {
        CG_DrawRect(self, x, y, w, h, size, &color)
    }

    /// Raven `cgDC.drawSides = &CG_DrawSides`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3160`
    fn drawSides(&mut self, _ds: &DisplayState, x: f32, y: f32, w: f32, h: f32, size: f32) {
        CG_DrawSides(self, x, y, w, h, size)
    }

    /// Raven `cgDC.drawTopBottom = &CG_DrawTopBottom`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3161`
    fn drawTopBottom(&mut self, _ds: &DisplayState, x: f32, y: f32, w: f32, h: f32, size: f32) {
        CG_DrawTopBottom(self, x, y, w, h, size)
    }

    /// Raven `cgDC.clearScene = &trap_R_ClearScene`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3162`
    fn clearScene(&mut self) {
        trap::R_ClearScene(self.engine)
    }

    /// Raven `cgDC.addRefEntityToScene = &trap_R_AddRefEntityToScene`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3163`
    fn addRefEntityToScene(&mut self, re: &refEntity_t) {
        trap::R_AddRefEntityToScene(self.engine, re)
    }

    /// Raven `cgDC.renderScene = &trap_R_RenderScene`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3164`
    fn renderScene(&mut self, fd: &refdef_t) {
        trap::R_RenderScene(self.engine, fd)
    }

    /// Raven `cgDC.RegisterFont = &trap_R_RegisterFont`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3165`
    fn RegisterFont(&mut self, fontName: &str) -> qhandle_t {
        trap::R_RegisterFont(self.engine, fontName)
    }

    /// Raven `cgDC.Font_StrLenPixels = &trap_R_Font_StrLenPixels` — §20 DEAD
    /// SURFACE (no `DC->Font_StrLenPixels` anywhere in `oracle/codemp`).
    /// Source: `oracle/codemp/cgame/cg_main.c:3166`, slot
    /// `oracle/codemp/ui/ui_shared.h:419`
    fn Font_StrLenPixels(&mut self, _text: &str, _iFontIndex: c_int, _scale: f32) -> c_int {
        unreachable!(
            "DisplayContext::Font_StrLenPixels — dead vtable slot, no DC-> call site in Raven"
        )
    }

    /// Raven `cgDC.Font_StrLenChars = &trap_R_Font_StrLenChars` — §20 DEAD
    /// SURFACE (no `DC->Font_StrLenChars` anywhere in `oracle/codemp`).
    /// Source: `oracle/codemp/cgame/cg_main.c:3167`, slot
    /// `oracle/codemp/ui/ui_shared.h:420`
    fn Font_StrLenChars(&mut self, _text: &str) -> c_int {
        unreachable!(
            "DisplayContext::Font_StrLenChars — dead vtable slot, no DC-> call site in Raven"
        )
    }

    /// Raven `cgDC.Font_HeightPixels = &trap_R_Font_HeightPixels` — §20 DEAD
    /// SURFACE (no `DC->Font_HeightPixels` anywhere in `oracle/codemp`).
    /// Source: `oracle/codemp/cgame/cg_main.c:3168`, slot
    /// `oracle/codemp/ui/ui_shared.h:421`
    fn Font_HeightPixels(&mut self, _iFontIndex: c_int, _scale: f32) -> c_int {
        unreachable!(
            "DisplayContext::Font_HeightPixels — dead vtable slot, no DC-> call site in Raven"
        )
    }

    /// Raven `cgDC.Font_DrawString = &trap_R_Font_DrawString` — §20 DEAD
    /// SURFACE (no `DC->Font_DrawString` anywhere in `oracle/codemp`).
    /// Source: `oracle/codemp/cgame/cg_main.c:3169`, slot
    /// `oracle/codemp/ui/ui_shared.h:422`
    fn Font_DrawString(
        &mut self,
        _ox: c_int,
        _oy: c_int,
        _text: &str,
        _rgba: vec4_t,
        _setIndex: c_int,
        _iCharLimit: c_int,
        _scale: f32,
    ) {
        unreachable!(
            "DisplayContext::Font_DrawString — dead vtable slot, no DC-> call site in Raven"
        )
    }

    /// Raven `cgDC.Language_IsAsian = &trap_Language_IsAsian` — §20 DEAD
    /// SURFACE (no `DC->Language_IsAsian` anywhere in `oracle/codemp`).
    /// Source: `oracle/codemp/cgame/cg_main.c:3170`, slot
    /// `oracle/codemp/ui/ui_shared.h:423`
    fn Language_IsAsian(&mut self) -> bool {
        unreachable!(
            "DisplayContext::Language_IsAsian — dead vtable slot, no DC-> call site in Raven"
        )
    }

    /// Raven `cgDC.Language_UsesSpaces = &trap_Language_UsesSpaces`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3171`
    fn Language_UsesSpaces(&mut self) -> bool {
        trap::Language_UsesSpaces(self.engine)
    }

    /// Raven `cgDC.AnyLanguage_ReadCharFromString = &trap_AnyLanguage_ReadCharFromString`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3172`
    fn AnyLanguage_ReadCharFromString(&mut self, psText: &[u8]) -> (u32, c_int, bool) {
        trap::AnyLanguage_ReadCharFromString(self.engine, psText)
    }

    /// Raven `cgDC.ownerDrawItem = &CG_OwnerDraw` — cgame's ownerdraw switch
    /// sits inside `#if 0`, so the call draws nothing.
    /// Source: `oracle/codemp/cgame/cg_main.c:3173`
    fn ownerDrawItem(
        &mut self,
        _menus: &mut MenuSystem,
        _ds: &DisplayState,
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
    ) {
        CG_OwnerDraw(
            x,
            y,
            w,
            h,
            text_x,
            text_y,
            ownerDraw,
            ownerDrawFlags,
            align,
            special,
            scale,
            color,
            shader,
            textStyle,
            iMenuFont,
        )
    }

    /// Raven `cgDC.getValue = &CG_GetValue`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3174`
    fn getValue(&mut self, _ownerDraw: c_int) -> f32 {
        todo!("CG_GetValue — oracle/codemp/cgame/cg_newDraw.c:46-91, lands with its C5 wave")
    }

    /// Raven `cgDC.ownerDrawVisible = &CG_OwnerDrawVisible`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3175`
    fn ownerDrawVisible(&mut self, _ds: &DisplayState, _flags: c_int) -> bool {
        todo!(
            "CG_OwnerDrawVisible — oracle/codemp/cgame/cg_newDraw.c:123-201, lands with its C5 wave"
        )
    }

    /// Raven `cgDC.runScript = &CG_RunMenuScript` — cgame's hook does nothing;
    /// item scripts run entirely inside `ui_shared.c`'s own dispatch.
    /// Source: `oracle/codemp/cgame/cg_main.c:3176`
    fn runScript(&mut self, _menus: &mut MenuSystem, _ds: &DisplayState, p: &mut &str) {
        CG_RunMenuScript(p)
    }

    /// Raven `cgDC.deferScript = &CG_DeferMenuScript`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3177`
    fn deferScript(&mut self, _menus: &mut MenuSystem, _ds: &DisplayState, p: &mut &str) -> bool {
        CG_DeferMenuScript(p)
    }

    /// Raven `cgDC.getTeamColor = &CG_GetTeamColor` — the out-param becomes
    /// the return value.
    /// Source: `oracle/codemp/cgame/cg_main.c:3178`
    fn getTeamColor(&mut self) -> vec4_t {
        CG_GetTeamColor(self.world)
    }

    /// Raven `cgDC.getCVarString = trap_Cvar_VariableStringBuffer`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3180`
    fn getCVarString(&mut self, cvar: &str, bufsize: usize) -> String {
        trap::Cvar_VariableStringBuffer(self.engine, cvar, bufsize)
    }

    /// Raven `cgDC.getCVarValue = CG_Cvar_Get` — cgame reads the string and
    /// `atof`s it where ui wires `trap_Cvar_VariableValue`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3181`
    fn getCVarValue(&mut self, cvar: &str) -> f32 {
        CG_Cvar_Get(self, cvar)
    }

    /// Raven `cgDC.setCVar = trap_Cvar_Set`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3179`
    fn setCVar(&mut self, cvar: &str, value: &str) {
        trap::Cvar_Set(self.engine, cvar, value)
    }

    /// Raven `cgDC.drawTextWithCursor = &CG_Text_PaintWithCursor`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3182`
    fn drawTextWithCursor(
        &mut self,
        _ds: &DisplayState,
        _x: f32,
        _y: f32,
        _scale: f32,
        _color: vec4_t,
        _text: &str,
        _cursorPos: c_int,
        _cursor: u8,
        _limit: c_int,
        _style: c_int,
        _iFontIndex: c_int,
    ) {
        todo!(
            "CG_Text_PaintWithCursor — oracle/codemp/cgame/cg_main.c:3027-3029, lands with its C5 wave"
        )
    }

    /// Raven `//cgDC.setOverstrikeMode = &trap_Key_SetOverstrikeMode` — the
    /// assignment is commented out, so cgame's slot is NULL and its only
    /// callers are the text-field editing paths cgame's huds never focus.
    /// cgame's syscall table has no `trap_Key_SetOverstrikeMode` either.
    /// Source: `oracle/codemp/cgame/cg_main.c:3183`
    fn setOverstrikeMode(&mut self, _b: bool) {}

    /// Raven `//cgDC.getOverstrikeMode = &trap_Key_GetOverstrikeMode` — NULL
    /// slot, same text-field paths as `setOverstrikeMode`; `false` is the
    /// neutral read.
    /// Source: `oracle/codemp/cgame/cg_main.c:3184`
    fn getOverstrikeMode(&mut self) -> bool {
        false
    }

    /// Raven `cgDC.startLocalSound = &trap_S_StartLocalSound`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3185`
    fn startLocalSound(&mut self, sfx: sfxHandle_t, channelNum: c_int) {
        trap::S_StartLocalSound(self.engine, sfx, channelNum)
    }

    /// Raven `cgDC.ownerDrawHandleKey = &CG_OwnerDrawHandleKey` — cgame's
    /// handler eats nothing, ever.
    /// Source: `oracle/codemp/cgame/cg_main.c:3186`
    fn ownerDrawHandleKey(
        &mut self,
        _menus: &mut MenuSystem,
        _ds: &DisplayState,
        ownerDraw: c_int,
        flags: c_int,
        special: &mut f32,
        key: c_int,
    ) -> bool {
        CG_OwnerDrawHandleKey(ownerDraw, flags, special, key)
    }

    /// Raven `cgDC.feederCount = &CG_FeederCount`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3187`
    fn feederCount(&mut self, _menus: &mut MenuSystem, _ds: &DisplayState, feederID: f32) -> c_int {
        CG_FeederCount(self.world, feederID)
    }

    /// Raven `cgDC.feederItemText = &CG_FeederItemText`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3189`
    fn feederItemText(
        &mut self,
        _ds: &DisplayState,
        _feederID: f32,
        _index: c_int,
        _column: c_int,
    ) -> (Option<String>, qhandle_t, qhandle_t, qhandle_t) {
        todo!("CG_FeederItemText — oracle/codemp/cgame/cg_main.c:2909-2994, lands with its C5 wave")
    }

    /// Raven `cgDC.feederItemImage = &CG_FeederItemImage` — cgame's feeders
    /// are text-only, so every row image is the null handle.
    /// Source: `oracle/codemp/cgame/cg_main.c:3188`
    fn feederItemImage(
        &mut self,
        _menus: &mut MenuSystem,
        _ds: &DisplayState,
        feederID: f32,
        index: c_int,
    ) -> qhandle_t {
        CG_FeederItemImage(feederID, index)
    }

    /// Raven `cgDC.feederSelection = &CG_FeederSelection`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3190`
    fn feederSelection(
        &mut self,
        _menus: &mut MenuSystem,
        _ds: &DisplayState,
        feederID: f32,
        index: c_int,
        item: Option<ItemId>,
    ) -> bool {
        CG_FeederSelection(self.world, feederID, index, item)
    }

    /// Raven `//cgDC.keynumToStringBuf = &trap_Key_KeynumToStringBuf` — NULL
    /// slot; the key-name lookup belongs to ui's controls menu and cgame's
    /// syscall table has no such trap. The empty string is Raven's "no name".
    /// Source: `oracle/codemp/cgame/cg_main.c:3193`
    fn keynumToStringBuf(&mut self, _keynum: c_int, _buflen: usize) -> String {
        String::new()
    }

    /// Raven `//cgDC.getBindingBuf = &trap_Key_GetBindingBuf` — NULL slot, and
    /// the one slot Raven checks before use: `String_Init`'s `if (DC &&
    /// DC->getBindingBuf)` skips `Controls_GetConfig` outright in cgame. The
    /// empty string reproduces that — every binding scan finds nothing.
    /// Source: `oracle/codemp/cgame/cg_main.c:3192`, null-check
    /// `oracle/codemp/ui/ui_shared.c:375`
    fn getBindingBuf(&mut self, _keynum: c_int, _buflen: usize) -> String {
        String::new()
    }

    /// Raven `//cgDC.setBinding = &trap_Key_SetBinding` — NULL slot; only the
    /// controls menu rebinds keys, and cgame has no `trap_Key_SetBinding`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3191`
    fn setBinding(&mut self, _keynum: c_int, _binding: &str) {}

    /// Raven `//cgDC.executeText = &trap_Cmd_ExecuteText` — NULL slot; cgame
    /// pushes console text through `trap_SendConsoleCommand` instead, and the
    /// framework's two `DC->executeText` sites are ui menu actions.
    /// Source: `oracle/codemp/cgame/cg_main.c:3194`
    fn executeText(&mut self, _exec_when: c_int, _text: &str) {}

    /// Raven `cgDC.Error = &Com_Error` — §20 DEAD SURFACE (no `DC->Error`
    /// anywhere in `oracle/codemp`; `ui_shared.c` reports through
    /// `PC_SourceError`/`DC->Print`).
    /// Source: `oracle/codemp/cgame/cg_main.c:3195`, slot
    /// `oracle/codemp/ui/ui_shared.h:448`
    fn Error(&mut self, _level: c_int, _error: &str) {
        unreachable!("DisplayContext::Error — dead vtable slot, no DC-> call site in Raven")
    }

    /// Raven `cgDC.Print = &Com_Printf`, cgame's `Com_Printf` being a
    /// `CG_Printf` forwarder.
    /// Source: `oracle/codemp/cgame/cg_main.c:3196`, forwarder
    /// `oracle/codemp/cgame/cg_main.c:1245-1254`
    fn Print(&mut self, msg: &str) {
        CG_Printf(self, msg)
    }

    /// Raven `//cgDC.Pause = &CG_Pause` — NULL slot, and Raven guards it at
    /// its one call site (`if (DC->Pause)`), so cgame simply doesn't pause on
    /// the last menu closing.
    /// Source: `oracle/codemp/cgame/cg_main.c:3198`, null-check
    /// `oracle/codemp/ui/ui_shared.c:4481-4482`
    fn Pause(&mut self, _b: bool) {}

    /// Raven `cgDC.ownerDrawWidth = &CG_OwnerDrawWidth`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3197`
    fn ownerDrawWidth(
        &mut self,
        _menus: &mut MenuSystem,
        _ds: &DisplayState,
        _ownerDraw: c_int,
        _scale: f32,
    ) -> c_int {
        todo!("CG_OwnerDrawWidth — oracle/codemp/cgame/cg_main.c:3031-3051, lands with its C5 wave")
    }

    /// Raven `cgDC.registerSound = &trap_S_RegisterSound`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3199`
    fn registerSound(&mut self, name: &str) -> sfxHandle_t {
        trap::S_RegisterSound(self.engine, name)
    }

    /// Raven `cgDC.startBackgroundTrack = &trap_S_StartBackgroundTrack`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3200`
    fn startBackgroundTrack(&mut self, intro: &str, loop_: &str, bReturnWithoutStarting: bool) {
        trap::S_StartBackgroundTrack(self.engine, intro, loop_, bReturnWithoutStarting)
    }

    /// Raven `cgDC.stopBackgroundTrack = &trap_S_StopBackgroundTrack`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3201`
    fn stopBackgroundTrack(&mut self) {
        trap::S_StopBackgroundTrack(self.engine)
    }

    /// Raven `cgDC.playCinematic = &CG_PlayCinematic`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3202`
    fn playCinematic(&mut self, name: &str, x: f32, y: f32, w: f32, h: f32) -> c_int {
        CG_PlayCinematic(self, name, x, y, w, h)
    }

    /// Raven `cgDC.stopCinematic = &CG_StopCinematic`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3203`
    fn stopCinematic(&mut self, handle: c_int) {
        CG_StopCinematic(self, handle)
    }

    /// Raven `cgDC.drawCinematic = &CG_DrawCinematic`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3204`
    fn drawCinematic(&mut self, handle: c_int, x: f32, y: f32, w: f32, h: f32) {
        CG_DrawCinematic(self, handle, x, y, w, h)
    }

    /// Raven `cgDC.runCinematicFrame = &CG_RunCinematicFrame`.
    /// Source: `oracle/codemp/cgame/cg_main.c:3205`
    fn runCinematicFrame(&mut self, handle: c_int) {
        CG_RunCinematicFrame(self, handle)
    }

    // ---- Host trap surface beyond the fn-pointer table ----
    //
    // `ui_shared.c` calls these `trap_*` directly, against whichever host's
    // syscall table the TU is compiled into. cgame's table is smaller than
    // ui's: `trap_Cvar_SetValue`, `trap_Key_ClearStates` and
    // `trap_GetLanguageName` have no `CG_*` syscall at all, and every call
    // site that uses them is either commented out or inside `#ifndef CGAME`.
    // Those three land as documented no-ops; the rest are pure delegation.
    // Census: `oracle/codemp/cgame/cg_public.h:57-200`.

    /// Raven `trap_Milliseconds` (`ui_shared.c` direct call).
    fn Milliseconds(&mut self) -> c_int {
        trap::Milliseconds(self.engine)
    }

    /// Raven `trap_Cvar_SetValue` — no cgame syscall, and every `ui_shared.c`
    /// call site is commented out (the old controls-menu writes).
    /// Source: `oracle/codemp/ui/ui_shared.c:5381-5389`
    fn setCVarValue(&mut self, _cvar: &str, _value: f32) {}

    /// Raven `trap_Key_IsDown` (`ui_shared.c` direct call).
    fn Key_IsDown(&mut self, keynum: c_int) -> bool {
        trap::Key_IsDown(self.engine, keynum)
    }

    /// Raven `trap_Key_GetCatcher` (`ui_shared.c` direct call, one of the
    /// three externs `ui_shared.c`'s `#ifdef CGAME` block declares).
    /// Source: `oracle/codemp/ui/ui_shared.c:59-65`
    fn Key_GetCatcher(&mut self) -> c_int {
        trap::Key_GetCatcher(self.engine)
    }

    /// Raven `trap_Key_SetCatcher` (`ui_shared.c` direct call).
    fn Key_SetCatcher(&mut self, catcher: c_int) {
        trap::Key_SetCatcher(self.engine, catcher)
    }

    /// Raven `trap_Key_ClearStates` — no cgame syscall; both call sites sit
    /// under `#ifndef CGAME`, right after the `Key_SetCatcher` line that does
    /// run in cgame.
    /// Source: `oracle/codemp/ui/ui_shared.c:10024-10025,10103-10104`
    fn Key_ClearStates(&mut self) {}

    /// Raven `trap_PC_ReadToken` (`ui_shared.c` direct call).
    fn PC_ReadToken(&mut self, handle: c_int, pc_token: &mut pc_token_t) -> bool {
        trap::PC_ReadToken(self.engine, handle, pc_token)
    }

    /// Raven `trap_PC_SourceFileAndLine` (`ui_shared.c` direct call).
    fn PC_SourceFileAndLine(&mut self, handle: c_int, buffer_len: usize) -> (c_int, String, c_int) {
        trap::PC_SourceFileAndLine(self.engine, handle, buffer_len)
    }

    /// Raven `trap_SP_GetStringTextString` (`ui_shared.c` direct call).
    fn SP_GetStringTextString(&mut self, text: &str, buffer_len: usize) -> Option<String> {
        trap::SP_GetStringTextString(self.engine, text, buffer_len)
    }

    /// Raven `trap_R_RegisterSkin` (`ui_shared.c` direct call).
    fn R_RegisterSkin(&mut self, name: &str) -> qhandle_t {
        trap::R_RegisterSkin(self.engine, name)
    }

    /// Raven `trap_GetLanguageName` — no cgame syscall; its only call site is
    /// the `#ifndef CGAME` language feeder, whose cgame arm builds no list.
    /// Source: `oracle/codemp/ui/ui_shared.c:8631-8644`
    fn GetLanguageName(&mut self, _languageIndex: c_int, _buffer_len: usize) -> String {
        String::new()
    }

    /// Raven `trap_G2API_InitGhoul2Model` (`ui_shared.c` direct call).
    fn G2API_InitGhoul2Model(
        &mut self,
        ghoul2Ptr: *mut *mut c_void,
        fileName: &str,
        modelIndex: c_int,
        customSkin: qhandle_t,
        customShader: qhandle_t,
        modelFlags: c_int,
        lodBias: c_int,
    ) -> c_int {
        trap::G2API_InitGhoul2Model(
            self.engine,
            ghoul2Ptr,
            fileName,
            modelIndex,
            customSkin,
            customShader,
            modelFlags,
            lodBias,
        )
    }

    /// Raven `trap_G2API_SetSkin` (`ui_shared.c` direct call).
    fn G2API_SetSkin(
        &mut self,
        ghoul2: *mut c_void,
        modelIndex: c_int,
        customSkin: qhandle_t,
        renderSkin: qhandle_t,
    ) -> bool {
        trap::G2API_SetSkin(self.engine, ghoul2, modelIndex, customSkin, renderSkin)
    }

    /// Raven `trap_G2API_CleanGhoul2Models` (`ui_shared.c` direct call).
    fn G2API_CleanGhoul2Models(&mut self, ghoul2Ptr: *mut *mut c_void) {
        trap::G2API_CleanGhoul2Models(self.engine, ghoul2Ptr)
    }

    /// Raven `trap_G2API_SetBoneAnim` (`ui_shared.c` direct call).
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
    ) -> bool {
        trap::G2API_SetBoneAnim(
            self.engine,
            ghoul2,
            modelIndex,
            boneName,
            startFrame,
            endFrame,
            flags,
            animSpeed,
            currentTime,
            setFrame,
            blendTime,
        )
    }

    /// Raven `trap_G2API_GetGLAName` (`ui_shared.c` direct call).
    fn G2API_GetGLAName(
        &mut self,
        ghoul2: *mut c_void,
        modelIndex: c_int,
        buffer_len: usize,
    ) -> String {
        trap::G2API_GetGLAName(self.engine, ghoul2, modelIndex, buffer_len)
    }

    /// Raven `trap_G2_HaveWeGhoul2Models` (`ui_shared.c` direct call).
    fn G2_HaveWeGhoul2Models(&mut self, ghoul2: *mut c_void) -> bool {
        trap::G2_HaveWeGhoul2Models(self.engine, ghoul2)
    }

    // ---- The `#ifndef CGAME` hooks ----
    //
    // Six trait methods bundle blocks `ui_shared.c` compiles out of the cgame
    // TU: the saber cache/draw pair, the character-animation pair, and the
    // species/language cycle lists. cgame's arm of each guard is an empty
    // block, so each hook does nothing and returns the empty value. The
    // framework calls them unconditionally (`mp_uishared` is host-agnostic and
    // ported the ui arm), which is exactly why they must be quiet rather than
    // loud - a panic here would invent a crash the retail cgame doesn't have.

    /// Raven's `UI_CacheSaberGlowGraphics()` + `ui_saber_parms_parsed`-gated
    /// `UI_SaberLoadParms()` — inside `ItemParse_isSaber`/`ItemParse_isSaber2`,
    /// whose whole bodies are `#ifndef CGAME` (cgame's arm falls straight to
    /// `return qfalse`).
    /// Source: `oracle/codemp/ui/ui_shared.c:8835-8858,8867-8886`
    fn UI_CacheSaberGlowGraphics(&mut self) {}

    /// Raven `UI_SaberDrawBlades(item, origin, angles)` — inside
    /// `Item_Model_Paint`'s `#ifndef CGAME` block, alongside the
    /// `ITF_ISCHARACTER` color wash; cgame draws the model without blades.
    /// Source: `oracle/codemp/ui/ui_shared.c:5871-5884`
    fn UI_SaberDrawBlades(
        &mut self,
        _ds: &DisplayState,
        _item: &ItemDef,
        _origin: vec3_t,
        _angles: vec3_t,
    ) {
    }

    /// Raven's `UI_ParseAnimationFile` + `bgAllAnims` lookup in
    /// `ItemParse_asset_model_go` — that fn's entire body is `#ifndef CGAME`,
    /// so cgame never reaches the anim branch. `None` is the caller's "skip
    /// the `G2API_SetBoneAnim` call, leave `*runTimeLength` at 0" path.
    ///
    /// cgame does carry its own `UI_ParseAnimationFile`
    /// (`crate::cg_draw::UI_ParseAnimationFile`, a `BG_ParseAnimationFile`
    /// passthrough Raven kept "called from UI shared code") - the `#ifndef
    /// CGAME` guard around the one call site left it stranded, so this hook
    /// deliberately doesn't route to it.
    /// Source: `oracle/codemp/ui/ui_shared.c:7569-7611`; stranded passthrough
    /// `oracle/codemp/cgame/cg_draw.c:98-105`
    fn UI_ParseAnimationFile(&mut self, _filename: &str, _g2anim: c_int) -> Option<animation_t> {
        None
    }

    /// Raven's "a moves datapad anim is playing" block at the top of
    /// `Item_Model_Paint` — `#ifndef CGAME`, and all `uiInfo`-only state
    /// besides.
    /// Source: `oracle/codemp/ui/ui_shared.c:5709-5769`
    fn UI_MovesDatapadAnimTick(
        &mut self,
        _ds: &DisplayState,
        _menus: &mut MenuSystem,
        _item: ItemId,
    ) {
    }

    /// Raven's `feeder == FEEDER_PLAYER_SPECIES` population loop in
    /// `ItemParse_cvarStrList` — `#ifndef CGAME`, so cgame adds no pairs and
    /// the item's `MultiDef` stays as parsed.
    /// Source: `oracle/codemp/ui/ui_shared.c:8623-8629`
    fn UI_PlayerSpeciesCvarStrList(&mut self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// Raven's `feeder == FEEDER_LANGUAGES` population loop in
    /// `ItemParse_cvarStrList` — `#ifndef CGAME`, same empty cgame arm as the
    /// species list above.
    /// Source: `oracle/codemp/ui/ui_shared.c:8631-8644`
    fn UI_LanguageCvarStrList(&mut self) -> Vec<(String, String)> {
        Vec::new()
    }
}

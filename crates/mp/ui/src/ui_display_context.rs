//! `impl DisplayContext for UiContext` — Raven's `_UI_Init` vtable wiring
//! (DEC-36 D3, DEC-38 ruling 1 revised).
//!
//! Raven filled a `displayContextDef_t` function-pointer table at init
//! (`uiInfo.uiDC.<slot> = &UI_Xxx` / `= trap_Xxx`,
//! `oracle/codemp/ui/ui_main.c:10701-10758`) and `ui_shared.c` called through
//! its file-scope `DC` pointer. This impl IS that table: every method is either
//! a `trap_*` forwarder or the `ui_main.c` callback Raven installed, and the
//! receiver is the module's own [`UiContext`] — so the framework's `dc` and the
//! ported logic's `ctx` are one object and there is nothing to alias (§B4).
//!
//! Re-entrant slots take the caller's `menus: &mut MenuSystem` / `ds:
//! &DisplayState` and hand them straight to the target, so mutations are
//! visible on return. Only the slots whose target genuinely needs them are
//! widened — measured, not assumed; no dc-routed target writes `DisplayState`,
//! which is why `ds` is shared throughout.
//!
//! Seven slots `_UI_Init` fills are never reached: `grep -rn "DC->drawStretchPic"
//! oracle/codemp` (and the six siblings named below) has no hit anywhere in the
//! oracle tree — `DC->` appears only in `ui/ui_shared.c` and `ui/ui_saber.c` —
//! so they are §20 dead surface and panic with their subject rather than
//! forwarding a trap nothing calls.

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use std::ffi::CString;
use std::ptr::null_mut;

use mp_bg::bg_panimate::BG_ParseAnimationFile;
use mp_bg::public::animation::animation_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::shared::{pc_token_t, qfalse, qhandle_t, sfxHandle_t, vec3_t, vec4_t};
use mp_uishared::shared::display_context::DisplayContext;
use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::item_def_s::ItemDef;
use mp_uishared::shared::item_id::ItemId;
use mp_uishared::shared::menu_system::MenuSystem;

use crate::bg_channel::{UiBgTraps, UiGameCallbacks};
use crate::trap;
use crate::ui_atoms::{UI_DrawHandlePic, UI_FillRect, UI_SetColor};
use crate::ui_main::{
    _UI_DrawRect, _UI_DrawSides, _UI_DrawTopBottom, Text_Height, Text_Paint, Text_PaintWithCursor,
    Text_Width, UI_DeferMenuScript, UI_DrawCinematic, UI_FeederCount, UI_FeederItemImage,
    UI_FeederItemText, UI_FeederSelection, UI_GetTeamColor, UI_GetValue, UI_MovesDatapadAnimTick,
    UI_OwnerDraw, UI_OwnerDrawHandleKey, UI_OwnerDrawVisible, UI_OwnerDrawWidth, UI_Pause,
    UI_PlayCinematic, UI_RunCinematicFrame, UI_RunMenuScript, UI_StopCinematic,
};
use crate::ui_saber::{UI_CacheSaberGlowGraphics, UI_SaberDrawBlades, UI_SaberLoadParms};
use crate::world::ui_context::UiContext;

impl<'e> DisplayContext for UiContext<'e> {
    /// Raven `uiDC.registerShaderNoMip = &trap_R_RegisterShaderNoMip`.
    /// Source: `oracle/codemp/ui/ui_main.c:10701`
    fn registerShaderNoMip(&mut self, p: &str) -> qhandle_t {
        trap::R_RegisterShaderNoMip(self.engine, p)
    }

    /// Raven `uiDC.setColor = &UI_SetColor`.
    /// Source: `oracle/codemp/ui/ui_main.c:10702`
    fn setColor(&mut self, v: Option<vec4_t>) {
        UI_SetColor(self, v.as_ref())
    }

    /// Raven `uiDC.drawHandlePic = &UI_DrawHandlePic`.
    /// Source: `oracle/codemp/ui/ui_main.c:10703`
    fn drawHandlePic(&mut self, x: f32, y: f32, w: f32, h: f32, asset: qhandle_t) {
        UI_DrawHandlePic(self, x, y, w, h, asset)
    }

    /// Raven `uiDC.drawStretchPic = &trap_R_DrawStretchPic` — §20 DEAD SURFACE:
    /// `_UI_Init` fills the slot and nothing ever calls `DC->drawStretchPic`
    /// (zero hits across `oracle/codemp`).
    /// Source: `oracle/codemp/ui/ui_main.c:10704`, slot
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

    /// Raven `uiDC.drawText = &Text_Paint`.
    /// Source: `oracle/codemp/ui/ui_main.c:10705`
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
        Text_Paint(
            self, ds, x, y, scale, color, text, adjust, limit, style, iMenuFont,
        )
    }

    /// Raven `uiDC.textWidth = &Text_Width`.
    /// Source: `oracle/codemp/ui/ui_main.c:10706`
    fn textWidth(&mut self, ds: &DisplayState, text: &str, scale: f32, iMenuFont: c_int) -> c_int {
        Text_Width(self, ds, text, scale, iMenuFont)
    }

    /// Raven `uiDC.textHeight = &Text_Height`.
    /// Source: `oracle/codemp/ui/ui_main.c:10707`
    fn textHeight(&mut self, ds: &DisplayState, text: &str, scale: f32, iMenuFont: c_int) -> c_int {
        Text_Height(self, ds, text, scale, iMenuFont)
    }

    /// Raven `uiDC.registerModel = &trap_R_RegisterModel`.
    /// Source: `oracle/codemp/ui/ui_main.c:10708`
    fn registerModel(&mut self, p: &str) -> qhandle_t {
        trap::R_RegisterModel(self.engine, p)
    }

    /// Raven `uiDC.modelBounds = &trap_R_ModelBounds`.
    /// Source: `oracle/codemp/ui/ui_main.c:10709`
    fn modelBounds(&mut self, model: qhandle_t) -> (vec3_t, vec3_t) {
        trap::R_ModelBounds(self.engine, model)
    }

    /// Raven `uiDC.fillRect = &UI_FillRect`.
    /// Source: `oracle/codemp/ui/ui_main.c:10710`
    fn fillRect(&mut self, ds: &DisplayState, x: f32, y: f32, w: f32, h: f32, color: vec4_t) {
        UI_FillRect(self, ds, x, y, w, h, &color)
    }

    /// Raven `uiDC.drawRect = &_UI_DrawRect`.
    /// Source: `oracle/codemp/ui/ui_main.c:10711`
    fn drawRect(
        &mut self,
        ds: &DisplayState,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        size: f32,
        color: vec4_t,
    ) {
        _UI_DrawRect(self, ds, x, y, w, h, size, &color)
    }

    /// Raven `uiDC.drawSides = &_UI_DrawSides`.
    /// Source: `oracle/codemp/ui/ui_main.c:10712`
    fn drawSides(&mut self, ds: &DisplayState, x: f32, y: f32, w: f32, h: f32, size: f32) {
        _UI_DrawSides(self, ds, x, y, w, h, size)
    }

    /// Raven `uiDC.drawTopBottom = &_UI_DrawTopBottom`.
    /// Source: `oracle/codemp/ui/ui_main.c:10713`
    fn drawTopBottom(&mut self, ds: &DisplayState, x: f32, y: f32, w: f32, h: f32, size: f32) {
        _UI_DrawTopBottom(self, ds, x, y, w, h, size)
    }

    /// Raven `uiDC.clearScene = &trap_R_ClearScene`.
    /// Source: `oracle/codemp/ui/ui_main.c:10714`
    fn clearScene(&mut self) {
        trap::R_ClearScene(self.engine)
    }

    /// Raven `uiDC.addRefEntityToScene = &trap_R_AddRefEntityToScene`.
    /// Source: `oracle/codemp/ui/ui_main.c:10716`
    fn addRefEntityToScene(&mut self, re: &refEntity_t) {
        trap::R_AddRefEntityToScene(self.engine, re)
    }

    /// Raven `uiDC.renderScene = &trap_R_RenderScene`.
    /// Source: `oracle/codemp/ui/ui_main.c:10717`
    fn renderScene(&mut self, fd: &refdef_t) {
        trap::R_RenderScene(self.engine, fd)
    }

    /// Raven `uiDC.RegisterFont = &trap_R_RegisterFont`.
    /// Source: `oracle/codemp/ui/ui_main.c:10718`
    fn RegisterFont(&mut self, fontName: &str) -> qhandle_t {
        trap::R_RegisterFont(self.engine, fontName)
    }

    /// Raven `uiDC.Font_StrLenPixels = trap_R_Font_StrLenPixels` — §20 DEAD
    /// SURFACE (no `DC->Font_StrLenPixels` anywhere in `oracle/codemp`).
    /// Source: `oracle/codemp/ui/ui_main.c:10719`, slot
    /// `oracle/codemp/ui/ui_shared.h:419`
    fn Font_StrLenPixels(&mut self, _text: &str, _iFontIndex: c_int, _scale: f32) -> c_int {
        unreachable!(
            "DisplayContext::Font_StrLenPixels — dead vtable slot, no DC-> call site in Raven"
        )
    }

    /// Raven `uiDC.Font_StrLenChars = trap_R_Font_StrLenChars` — §20 DEAD
    /// SURFACE (no `DC->Font_StrLenChars` anywhere in `oracle/codemp`).
    /// Source: `oracle/codemp/ui/ui_main.c:10720`, slot
    /// `oracle/codemp/ui/ui_shared.h:420`
    fn Font_StrLenChars(&mut self, _text: &str) -> c_int {
        unreachable!(
            "DisplayContext::Font_StrLenChars — dead vtable slot, no DC-> call site in Raven"
        )
    }

    /// Raven `uiDC.Font_HeightPixels = trap_R_Font_HeightPixels` — §20 DEAD
    /// SURFACE (no `DC->Font_HeightPixels` anywhere in `oracle/codemp`).
    /// Source: `oracle/codemp/ui/ui_main.c:10721`, slot
    /// `oracle/codemp/ui/ui_shared.h:421`
    fn Font_HeightPixels(&mut self, _iFontIndex: c_int, _scale: f32) -> c_int {
        unreachable!(
            "DisplayContext::Font_HeightPixels — dead vtable slot, no DC-> call site in Raven"
        )
    }

    /// Raven `uiDC.Font_DrawString = trap_R_Font_DrawString` — §20 DEAD SURFACE
    /// (no `DC->Font_DrawString` anywhere in `oracle/codemp`).
    /// Source: `oracle/codemp/ui/ui_main.c:10722`, slot
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

    /// Raven `uiDC.Language_IsAsian = trap_Language_IsAsian` — §20 DEAD SURFACE
    /// (no `DC->Language_IsAsian` anywhere in `oracle/codemp`).
    /// Source: `oracle/codemp/ui/ui_main.c:10723`, slot
    /// `oracle/codemp/ui/ui_shared.h:423`
    fn Language_IsAsian(&mut self) -> bool {
        unreachable!(
            "DisplayContext::Language_IsAsian — dead vtable slot, no DC-> call site in Raven"
        )
    }

    /// Raven `uiDC.Language_UsesSpaces = trap_Language_UsesSpaces`.
    /// Source: `oracle/codemp/ui/ui_main.c:10724`
    fn Language_UsesSpaces(&mut self) -> bool {
        trap::Language_UsesSpaces(self.engine)
    }

    /// Raven `uiDC.AnyLanguage_ReadCharFromString = trap_AnyLanguage_ReadCharFromString`.
    /// Source: `oracle/codemp/ui/ui_main.c:10725`
    fn AnyLanguage_ReadCharFromString(&mut self, psText: &[u8]) -> (u32, c_int, bool) {
        trap::AnyLanguage_ReadCharFromString(self.engine, psText)
    }

    /// Raven `uiDC.ownerDrawItem = &UI_OwnerDraw`.
    /// Source: `oracle/codemp/ui/ui_main.c:10726`
    fn ownerDrawItem(
        &mut self,
        menus: &mut MenuSystem,
        ds: &DisplayState,
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
        UI_OwnerDraw(
            self,
            menus,
            ds,
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

    /// Raven `uiDC.getValue = &UI_GetValue`.
    /// Source: `oracle/codemp/ui/ui_main.c:10727`
    fn getValue(&mut self, ownerDraw: c_int) -> f32 {
        UI_GetValue(ownerDraw)
    }

    /// Raven `uiDC.ownerDrawVisible = &UI_OwnerDrawVisible`.
    /// Source: `oracle/codemp/ui/ui_main.c:10728`
    fn ownerDrawVisible(&mut self, ds: &DisplayState, flags: c_int) -> bool {
        UI_OwnerDrawVisible(self, ds, flags)
    }

    /// Raven `uiDC.runScript = &UI_RunMenuScript`.
    /// Source: `oracle/codemp/ui/ui_main.c:10729`
    fn runScript(&mut self, menus: &mut MenuSystem, ds: &DisplayState, p: &mut &str) {
        UI_RunMenuScript(self, menus, ds, p)
    }

    /// Raven `uiDC.deferScript = &UI_DeferMenuScript`.
    /// Source: `oracle/codemp/ui/ui_main.c:10730`
    fn deferScript(&mut self, menus: &mut MenuSystem, ds: &DisplayState, p: &mut &str) -> bool {
        UI_DeferMenuScript(self, menus, ds, p)
    }

    /// Raven `uiDC.getTeamColor = &UI_GetTeamColor` — the out-param becomes the
    /// return value; Raven's body is empty, so the color comes back untouched.
    /// Source: `oracle/codemp/ui/ui_main.c:10731`
    fn getTeamColor(&mut self) -> vec4_t {
        let mut color: vec4_t = [0.0; 4];
        UI_GetTeamColor(&mut color);
        color
    }

    /// Raven `uiDC.getCVarString = trap_Cvar_VariableStringBuffer`.
    /// Source: `oracle/codemp/ui/ui_main.c:10733`
    fn getCVarString(&mut self, cvar: &str, bufsize: usize) -> String {
        trap::Cvar_VariableStringBuffer(self.engine, cvar, bufsize)
    }

    /// Raven `uiDC.getCVarValue = trap_Cvar_VariableValue`.
    /// Source: `oracle/codemp/ui/ui_main.c:10734`
    fn getCVarValue(&mut self, cvar: &str) -> f32 {
        trap::Cvar_VariableValue(self.engine, cvar)
    }

    /// Raven `uiDC.setCVar = trap_Cvar_Set`.
    /// Source: `oracle/codemp/ui/ui_main.c:10732`
    fn setCVar(&mut self, cvar: &str, value: &str) {
        trap::Cvar_Set(self.engine, cvar, value)
    }

    /// Raven `uiDC.drawTextWithCursor = &Text_PaintWithCursor`.
    /// Source: `oracle/codemp/ui/ui_main.c:10735`
    fn drawTextWithCursor(
        &mut self,
        ds: &DisplayState,
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
    ) {
        Text_PaintWithCursor(
            self,
            ds,
            x,
            y,
            scale,
            color,
            text,
            cursorPos,
            cursor as char,
            limit,
            style,
            iFontIndex,
        )
    }

    /// Raven `uiDC.setOverstrikeMode = &trap_Key_SetOverstrikeMode`.
    /// Source: `oracle/codemp/ui/ui_main.c:10736`
    fn setOverstrikeMode(&mut self, b: bool) {
        trap::Key_SetOverstrikeMode(self.engine, b)
    }

    /// Raven `uiDC.getOverstrikeMode = &trap_Key_GetOverstrikeMode`.
    /// Source: `oracle/codemp/ui/ui_main.c:10737`
    fn getOverstrikeMode(&mut self) -> bool {
        trap::Key_GetOverstrikeMode(self.engine)
    }

    /// Raven `uiDC.startLocalSound = &trap_S_StartLocalSound`.
    /// Source: `oracle/codemp/ui/ui_main.c:10738`
    fn startLocalSound(&mut self, sfx: sfxHandle_t, channelNum: c_int) {
        trap::S_StartLocalSound(self.engine, sfx, channelNum)
    }

    /// Raven `uiDC.ownerDrawHandleKey = &UI_OwnerDrawHandleKey`.
    /// Source: `oracle/codemp/ui/ui_main.c:10739`
    fn ownerDrawHandleKey(
        &mut self,
        menus: &mut MenuSystem,
        ds: &DisplayState,
        ownerDraw: c_int,
        flags: c_int,
        special: &mut f32,
        key: c_int,
    ) -> bool {
        UI_OwnerDrawHandleKey(self, menus, ds, ownerDraw, flags, special, key)
    }

    /// Raven `uiDC.feederCount = &UI_FeederCount`.
    /// Source: `oracle/codemp/ui/ui_main.c:10740`
    fn feederCount(&mut self, menus: &mut MenuSystem, ds: &DisplayState, feederID: f32) -> c_int {
        UI_FeederCount(self, menus, ds, feederID)
    }

    /// Raven `uiDC.feederItemText = &UI_FeederItemText` — the three out-param
    /// handles join the return value, and Raven's `NULL` return becomes `None`
    /// (the port's `String::new()` is that `NULL`).
    /// Source: `oracle/codemp/ui/ui_main.c:10742`
    fn feederItemText(
        &mut self,
        ds: &DisplayState,
        feederID: f32,
        index: c_int,
        column: c_int,
    ) -> (Option<String>, qhandle_t, qhandle_t, qhandle_t) {
        let mut handle1: qhandle_t = -1;
        let mut handle2: qhandle_t = -1;
        let mut handle3: qhandle_t = -1;
        let text = UI_FeederItemText(
            self,
            ds,
            feederID,
            index,
            column,
            &mut handle1,
            &mut handle2,
            &mut handle3,
        );
        let text = if text.is_empty() { None } else { Some(text) };
        (text, handle1, handle2, handle3)
    }

    /// Raven `uiDC.feederItemImage = &UI_FeederItemImage`.
    /// Source: `oracle/codemp/ui/ui_main.c:10741`
    fn feederItemImage(
        &mut self,
        menus: &mut MenuSystem,
        ds: &DisplayState,
        feederID: f32,
        index: c_int,
    ) -> qhandle_t {
        UI_FeederItemImage(self, menus, ds, feederID, index)
    }

    /// Raven `uiDC.feederSelection = &UI_FeederSelection`.
    /// Source: `oracle/codemp/ui/ui_main.c:10743`
    fn feederSelection(
        &mut self,
        menus: &mut MenuSystem,
        ds: &DisplayState,
        feederID: f32,
        index: c_int,
        item: Option<ItemId>,
    ) -> bool {
        UI_FeederSelection(self, menus, ds, feederID, index, item)
    }

    /// Raven `uiDC.keynumToStringBuf = &trap_Key_KeynumToStringBuf`.
    /// Source: `oracle/codemp/ui/ui_main.c:10746`
    fn keynumToStringBuf(&mut self, keynum: c_int, buflen: usize) -> String {
        trap::Key_KeynumToStringBuf(self.engine, keynum, buflen)
    }

    /// Raven `uiDC.getBindingBuf = &trap_Key_GetBindingBuf`.
    /// Source: `oracle/codemp/ui/ui_main.c:10745`
    fn getBindingBuf(&mut self, keynum: c_int, buflen: usize) -> String {
        trap::Key_GetBindingBuf(self.engine, keynum, buflen)
    }

    /// Raven `uiDC.setBinding = &trap_Key_SetBinding`.
    /// Source: `oracle/codemp/ui/ui_main.c:10744`
    fn setBinding(&mut self, keynum: c_int, binding: &str) {
        trap::Key_SetBinding(self.engine, keynum, binding)
    }

    /// Raven `uiDC.executeText = &trap_Cmd_ExecuteText`.
    /// Source: `oracle/codemp/ui/ui_main.c:10747`
    fn executeText(&mut self, exec_when: c_int, text: &str) {
        trap::Cmd_ExecuteText(self.engine, exec_when, text)
    }

    /// Raven `uiDC.Error = &Com_Error` — §20 DEAD SURFACE (no `DC->Error`
    /// anywhere in `oracle/codemp`; `ui_shared.c` reports through
    /// `PC_SourceError`/`DC->Print`).
    /// Source: `oracle/codemp/ui/ui_main.c:10748`, slot
    /// `oracle/codemp/ui/ui_shared.h:448`
    fn Error(&mut self, _level: c_int, _error: &str) {
        unreachable!("DisplayContext::Error — dead vtable slot, no DC-> call site in Raven")
    }

    /// Raven `uiDC.Print = &Com_Printf` (a `trap_Print` forwarder in
    /// `ui_syscalls.c`).
    /// Source: `oracle/codemp/ui/ui_main.c:10749`
    fn Print(&mut self, msg: &str) {
        trap::Print(self.engine, msg)
    }

    /// Raven `uiDC.Pause = &UI_Pause`.
    /// Source: `oracle/codemp/ui/ui_main.c:10750`
    fn Pause(&mut self, b: bool) {
        UI_Pause(self, b)
    }

    /// Raven `uiDC.ownerDrawWidth = &UI_OwnerDrawWidth`.
    /// Source: `oracle/codemp/ui/ui_main.c:10751`
    fn ownerDrawWidth(
        &mut self,
        menus: &mut MenuSystem,
        ds: &DisplayState,
        ownerDraw: c_int,
        scale: f32,
    ) -> c_int {
        UI_OwnerDrawWidth(self, menus, ds, ownerDraw, scale)
    }

    /// Raven `uiDC.registerSound = &trap_S_RegisterSound`.
    /// Source: `oracle/codemp/ui/ui_main.c:10752`
    fn registerSound(&mut self, name: &str) -> sfxHandle_t {
        trap::S_RegisterSound(self.engine, name)
    }

    /// Raven `uiDC.startBackgroundTrack = &trap_S_StartBackgroundTrack`.
    /// Source: `oracle/codemp/ui/ui_main.c:10753`
    fn startBackgroundTrack(&mut self, intro: &str, loop_: &str, bReturnWithoutStarting: bool) {
        trap::S_StartBackgroundTrack(self.engine, intro, loop_, bReturnWithoutStarting)
    }

    /// Raven `uiDC.stopBackgroundTrack = &trap_S_StopBackgroundTrack`.
    /// Source: `oracle/codemp/ui/ui_main.c:10754`
    fn stopBackgroundTrack(&mut self) {
        trap::S_StopBackgroundTrack(self.engine)
    }

    /// Raven `uiDC.playCinematic = &UI_PlayCinematic`.
    /// Source: `oracle/codemp/ui/ui_main.c:10755`
    fn playCinematic(&mut self, name: &str, x: f32, y: f32, w: f32, h: f32) -> c_int {
        UI_PlayCinematic(self, name, x, y, w, h)
    }

    /// Raven `uiDC.stopCinematic = &UI_StopCinematic`.
    /// Source: `oracle/codemp/ui/ui_main.c:10756`
    fn stopCinematic(&mut self, handle: c_int) {
        UI_StopCinematic(self, handle)
    }

    /// Raven `uiDC.drawCinematic = &UI_DrawCinematic`.
    /// Source: `oracle/codemp/ui/ui_main.c:10757`
    fn drawCinematic(&mut self, handle: c_int, x: f32, y: f32, w: f32, h: f32) {
        UI_DrawCinematic(self, handle, x, y, w, h)
    }

    /// Raven `uiDC.runCinematicFrame = &UI_RunCinematicFrame`.
    /// Source: `oracle/codemp/ui/ui_main.c:10758`
    fn runCinematicFrame(&mut self, handle: c_int) {
        UI_RunCinematicFrame(self, handle)
    }

    // ---- Host trap surface beyond the fn-pointer table ----
    //
    // `ui_shared.c` calls these `trap_*` directly (both hosts compile the TU
    // against their own syscall table); `mp_uishared` is host-agnostic, so they
    // route through the trait and land here as pure delegation.

    /// Raven `trap_Milliseconds` (`ui_shared.c` direct call).
    fn Milliseconds(&mut self) -> c_int {
        trap::Milliseconds(self.engine)
    }

    /// Raven `trap_Cvar_SetValue` (`ui_shared.c` direct call).
    fn setCVarValue(&mut self, cvar: &str, value: f32) {
        trap::Cvar_SetValue(self.engine, cvar, value)
    }

    /// Raven `trap_Key_IsDown` (`ui_shared.c` direct call).
    fn Key_IsDown(&mut self, keynum: c_int) -> bool {
        trap::Key_IsDown(self.engine, keynum)
    }

    /// Raven `trap_Key_GetCatcher` (`ui_shared.c` direct call).
    fn Key_GetCatcher(&mut self) -> c_int {
        trap::Key_GetCatcher(self.engine)
    }

    /// Raven `trap_Key_SetCatcher` (`ui_shared.c` direct call).
    fn Key_SetCatcher(&mut self, catcher: c_int) {
        trap::Key_SetCatcher(self.engine, catcher)
    }

    /// Raven `trap_Key_ClearStates` (`ui_shared.c` direct call).
    fn Key_ClearStates(&mut self) {
        trap::Key_ClearStates(self.engine)
    }

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

    /// Raven `trap_GetLanguageName` (`ui_shared.c` direct call).
    fn GetLanguageName(&mut self, languageIndex: c_int, buffer_len: usize) -> String {
        trap::GetLanguageName(self.engine, languageIndex, buffer_len)
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

    /// Raven `UI_CacheSaberGlowGraphics()` + the `ui_saber_parms_parsed`-gated
    /// `UI_SaberLoadParms()` (`ItemParse_isSaber`/`ItemParse_isSaber2`,
    /// `ui_shared.c:8844-8847,8874-8877`).
    fn UI_CacheSaberGlowGraphics(&mut self) {
        UI_CacheSaberGlowGraphics(self);
        if !self.world.saber.ui_saber_parms_parsed {
            UI_SaberLoadParms(self);
        }
    }

    /// Raven `UI_SaberDrawBlades(item, origin, angles)` (`ui_shared.c:5882`).
    fn UI_SaberDrawBlades(
        &mut self,
        ds: &DisplayState,
        item: &ItemDef,
        origin: vec3_t,
        angles: vec3_t,
    ) {
        UI_SaberDrawBlades(self, ds, item, origin, angles);
    }

    /// Raven's `UI_ParseAnimationFile(GLAName, NULL, qfalse)` call plus the
    /// `bgAllAnims[animIndex].anims[modelPtr->g2anim]` lookup
    /// (`ItemParse_asset_model_go`, `ui_shared.c:7602-7611`) — routed to
    /// `mp_bg`'s `BG_ParseAnimationFile` (DEC-36 D5 reuse) against
    /// `self.world.bg_state`'s `bgAllAnims` cache, mirroring `UI_SiegeInit`'s
    /// `UiBgTraps`/`UiGameCallbacks` construction.
    fn UI_ParseAnimationFile(&mut self, filename: &str, g2anim: c_int) -> Option<animation_t> {
        let traps = UiBgTraps::new(self.engine);
        let mut callbacks = UiGameCallbacks::new(self.engine);
        // An interior NUL can't reach C's `const char*`; `None` here is the
        // caller's "skip the `G2API_SetBoneAnim` call" path.
        let filename_c = match CString::new(filename) {
            Ok(s) => s,
            Err(_) => return None,
        };

        let animIndex = BG_ParseAnimationFile(
            &mut self.world.bg_state,
            &traps,
            &mut callbacks,
            filename_c.as_ptr(),
            null_mut(),
            qfalse,
        );
        if animIndex < 0 {
            return None;
        }

        // Raven: `&bgAllAnims[animIndex].anims[modelPtr->g2anim]` — an
        // unchecked array-of-arrays index, ported as-is (no bounds check
        // added beyond Raven's own); `anims` is non-null whenever
        // `BG_ParseAnimationFile` returned a non-negative index.
        let anims = self.world.bg_state.bgAllAnims[animIndex as usize].anims;
        if anims.is_null() {
            return None;
        }
        Some(unsafe { *anims.add(g2anim as usize) })
    }

    /// Raven's "a moves datapad anim is playing" block (`Item_Model_Paint`,
    /// `ui_shared.c:5709-5769`).
    fn UI_MovesDatapadAnimTick(&mut self, ds: &DisplayState, menus: &mut MenuSystem, item: ItemId) {
        UI_MovesDatapadAnimTick(self, ds, menus, item);
    }

    /// Raven's `feeder == FEEDER_PLAYER_SPECIES` population loop
    /// (`ItemParse_cvarStrList`, `ui_shared.c:8623-8629`).
    fn UI_PlayerSpeciesCvarStrList(&mut self) -> Vec<(String, String)> {
        self.world
            .playerSpecies
            .iter()
            .map(|species| {
                let label = format!("@MENUS_{}", species.Name).to_ascii_uppercase();
                (label, species.Name.clone())
            })
            .collect()
    }

    /// Raven's `feeder == FEEDER_LANGUAGES` population loop
    /// (`ItemParse_cvarStrList`, `ui_shared.c:8631-8644`).
    fn UI_LanguageCvarStrList(&mut self) -> Vec<(String, String)> {
        (0..self.world.languageCount)
            .map(|i| {
                // Raven: "The displayed text" call, whose result is discarded
                // (the label is the constant key) — emitted for trace parity.
                let _ = trap::GetLanguageName(self.engine, i, 128);
                let name = trap::GetLanguageName(self.engine, i, 128);
                ("@MENUS_MYLANGUAGE".to_string(), name)
            })
            .collect()
    }
}

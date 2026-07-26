//! `UnbuiltDisplayContext` — the placeholder [`DisplayContext`] the shell
//! passes into `mp_ui::ui_main::vmMain` until the real implementor lands.
//!
//! //TODO: Port ui DisplayContext implementor
//! // Source: `crates/mp/uishared/src/shared/display_context.rs` (DEC-36
//! // addendum 12) — the trait's own doc names the concrete implementor a
//! // "U5-built carrier over split borrows of `UiWorld`"; that carrier has not
//! // been built yet (checked 2026-07-25, `crates/mp/ui/src` has zero `impl
//! // DisplayContext` blocks). `vmMain` cannot compile without SOME `&mut dyn
//! // DisplayContext`, so this type exists ONLY to satisfy that signature —
//! // it invents no behavior. Every method panics loudly, naming itself, the
//! // instant a live call reaches it (the same "real bug -> fatal" contract
//! // panicking module code always carries, porting-rules Unported-work
//! // markers).

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};

use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::shared::{pc_token_t, qhandle_t, sfxHandle_t, vec3_t, vec4_t};
use mp_uishared::shared::display_context::DisplayContext;
use mp_uishared::shared::item_id::ItemId;

/// Panics naming the missing `DisplayContext` method — see the module doc.
fn unbuilt(subject: &str) -> ! {
    todo!(
        "Port DisplayContext::{subject} — no ui DisplayContext implementor is \
         built yet (DEC-36 addendum 12 U5 slice); \
         crates/mp/uishared/src/shared/display_context.rs"
    )
}

/// The zero-sized placeholder [`DisplayContext`] implementor (SEAM-D10 slice).
pub struct UnbuiltDisplayContext;

impl DisplayContext for UnbuiltDisplayContext {
    fn registerShaderNoMip(&mut self, _p: &str) -> qhandle_t {
        unbuilt("registerShaderNoMip")
    }

    fn setColor(&mut self, _v: Option<vec4_t>) {
        unbuilt("setColor")
    }

    fn drawHandlePic(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _asset: qhandle_t) {
        unbuilt("drawHandlePic")
    }

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
        unbuilt("drawStretchPic")
    }

    fn drawText(
        &mut self,
        _x: f32,
        _y: f32,
        _scale: f32,
        _color: vec4_t,
        _text: &str,
        _adjust: f32,
        _limit: c_int,
        _style: c_int,
        _iMenuFont: c_int,
    ) {
        unbuilt("drawText")
    }

    fn textWidth(&mut self, _text: &str, _scale: f32, _iMenuFont: c_int) -> c_int {
        unbuilt("textWidth")
    }

    fn textHeight(&mut self, _text: &str, _scale: f32, _iMenuFont: c_int) -> c_int {
        unbuilt("textHeight")
    }

    fn registerModel(&mut self, _p: &str) -> qhandle_t {
        unbuilt("registerModel")
    }

    fn modelBounds(&mut self, _model: qhandle_t) -> (vec3_t, vec3_t) {
        unbuilt("modelBounds")
    }

    fn fillRect(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _color: vec4_t) {
        unbuilt("fillRect")
    }

    fn drawRect(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _size: f32, _color: vec4_t) {
        unbuilt("drawRect")
    }

    fn drawSides(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _size: f32) {
        unbuilt("drawSides")
    }

    fn drawTopBottom(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _size: f32) {
        unbuilt("drawTopBottom")
    }

    fn clearScene(&mut self) {
        unbuilt("clearScene")
    }

    fn addRefEntityToScene(&mut self, _re: &refEntity_t) {
        unbuilt("addRefEntityToScene")
    }

    fn renderScene(&mut self, _fd: &refdef_t) {
        unbuilt("renderScene")
    }

    fn RegisterFont(&mut self, _fontName: &str) -> qhandle_t {
        unbuilt("RegisterFont")
    }

    fn Font_StrLenPixels(&mut self, _text: &str, _iFontIndex: c_int, _scale: f32) -> c_int {
        unbuilt("Font_StrLenPixels")
    }

    fn Font_StrLenChars(&mut self, _text: &str) -> c_int {
        unbuilt("Font_StrLenChars")
    }

    fn Font_HeightPixels(&mut self, _iFontIndex: c_int, _scale: f32) -> c_int {
        unbuilt("Font_HeightPixels")
    }

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
        unbuilt("Font_DrawString")
    }

    fn Language_IsAsian(&mut self) -> bool {
        unbuilt("Language_IsAsian")
    }

    fn Language_UsesSpaces(&mut self) -> bool {
        unbuilt("Language_UsesSpaces")
    }

    fn AnyLanguage_ReadCharFromString(&mut self, _psText: &[u8]) -> (u32, c_int, bool) {
        unbuilt("AnyLanguage_ReadCharFromString")
    }

    fn ownerDrawItem(
        &mut self,
        _x: f32,
        _y: f32,
        _w: f32,
        _h: f32,
        _text_x: f32,
        _text_y: f32,
        _ownerDraw: c_int,
        _ownerDrawFlags: c_int,
        _align: c_int,
        _special: f32,
        _scale: f32,
        _color: vec4_t,
        _shader: qhandle_t,
        _textStyle: c_int,
        _iMenuFont: c_int,
    ) {
        unbuilt("ownerDrawItem")
    }

    fn getValue(&mut self, _ownerDraw: c_int) -> f32 {
        unbuilt("getValue")
    }

    fn ownerDrawVisible(&mut self, _flags: c_int) -> bool {
        unbuilt("ownerDrawVisible")
    }

    fn runScript(&mut self, _p: &mut &str) {
        unbuilt("runScript")
    }

    fn deferScript(&mut self, _p: &mut &str) -> bool {
        unbuilt("deferScript")
    }

    fn getTeamColor(&mut self) -> vec4_t {
        unbuilt("getTeamColor")
    }

    fn getCVarString(&mut self, _cvar: &str, _bufsize: usize) -> String {
        unbuilt("getCVarString")
    }

    fn getCVarValue(&mut self, _cvar: &str) -> f32 {
        unbuilt("getCVarValue")
    }

    fn setCVar(&mut self, _cvar: &str, _value: &str) {
        unbuilt("setCVar")
    }

    fn drawTextWithCursor(
        &mut self,
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
        unbuilt("drawTextWithCursor")
    }

    fn setOverstrikeMode(&mut self, _b: bool) {
        unbuilt("setOverstrikeMode")
    }

    fn getOverstrikeMode(&mut self) -> bool {
        unbuilt("getOverstrikeMode")
    }

    fn startLocalSound(&mut self, _sfx: sfxHandle_t, _channelNum: c_int) {
        unbuilt("startLocalSound")
    }

    fn ownerDrawHandleKey(
        &mut self,
        _ownerDraw: c_int,
        _flags: c_int,
        _special: &mut f32,
        _key: c_int,
    ) -> bool {
        unbuilt("ownerDrawHandleKey")
    }

    fn feederCount(&mut self, _feederID: f32) -> c_int {
        unbuilt("feederCount")
    }

    fn feederItemText(
        &mut self,
        _feederID: f32,
        _index: c_int,
        _column: c_int,
    ) -> (Option<String>, qhandle_t, qhandle_t, qhandle_t) {
        unbuilt("feederItemText")
    }

    fn feederItemImage(&mut self, _feederID: f32, _index: c_int) -> qhandle_t {
        unbuilt("feederItemImage")
    }

    fn feederSelection(&mut self, _feederID: f32, _index: c_int, _item: Option<ItemId>) -> bool {
        unbuilt("feederSelection")
    }

    fn keynumToStringBuf(&mut self, _keynum: c_int, _buflen: usize) -> String {
        unbuilt("keynumToStringBuf")
    }

    fn getBindingBuf(&mut self, _keynum: c_int, _buflen: usize) -> String {
        unbuilt("getBindingBuf")
    }

    fn setBinding(&mut self, _keynum: c_int, _binding: &str) {
        unbuilt("setBinding")
    }

    fn executeText(&mut self, _exec_when: c_int, _text: &str) {
        unbuilt("executeText")
    }

    fn Error(&mut self, _level: c_int, _error: &str) {
        unbuilt("Error")
    }

    fn Print(&mut self, _msg: &str) {
        unbuilt("Print")
    }

    fn Pause(&mut self, _b: bool) {
        unbuilt("Pause")
    }

    fn ownerDrawWidth(&mut self, _ownerDraw: c_int, _scale: f32) -> c_int {
        unbuilt("ownerDrawWidth")
    }

    fn registerSound(&mut self, _name: &str) -> sfxHandle_t {
        unbuilt("registerSound")
    }

    fn startBackgroundTrack(&mut self, _intro: &str, _loop_: &str, _bReturnWithoutStarting: bool) {
        unbuilt("startBackgroundTrack")
    }

    fn stopBackgroundTrack(&mut self) {
        unbuilt("stopBackgroundTrack")
    }

    fn playCinematic(&mut self, _name: &str, _x: f32, _y: f32, _w: f32, _h: f32) -> c_int {
        unbuilt("playCinematic")
    }

    fn stopCinematic(&mut self, _handle: c_int) {
        unbuilt("stopCinematic")
    }

    fn drawCinematic(&mut self, _handle: c_int, _x: f32, _y: f32, _w: f32, _h: f32) {
        unbuilt("drawCinematic")
    }

    fn runCinematicFrame(&mut self, _handle: c_int) {
        unbuilt("runCinematicFrame")
    }

    fn Milliseconds(&mut self) -> c_int {
        unbuilt("Milliseconds")
    }

    fn setCVarValue(&mut self, _cvar: &str, _value: f32) {
        unbuilt("setCVarValue")
    }

    fn Key_IsDown(&mut self, _keynum: c_int) -> bool {
        unbuilt("Key_IsDown")
    }

    fn Key_GetCatcher(&mut self) -> c_int {
        unbuilt("Key_GetCatcher")
    }

    fn Key_SetCatcher(&mut self, _catcher: c_int) {
        unbuilt("Key_SetCatcher")
    }

    fn Key_ClearStates(&mut self) {
        unbuilt("Key_ClearStates")
    }

    fn PC_ReadToken(&mut self, _handle: c_int, _pc_token: &mut pc_token_t) -> bool {
        unbuilt("PC_ReadToken")
    }

    fn PC_SourceFileAndLine(
        &mut self,
        _handle: c_int,
        _buffer_len: usize,
    ) -> (c_int, String, c_int) {
        unbuilt("PC_SourceFileAndLine")
    }

    fn SP_GetStringTextString(&mut self, _text: &str, _buffer_len: usize) -> Option<String> {
        unbuilt("SP_GetStringTextString")
    }

    fn R_RegisterSkin(&mut self, _name: &str) -> qhandle_t {
        unbuilt("R_RegisterSkin")
    }

    fn GetLanguageName(&mut self, _languageIndex: c_int, _buffer_len: usize) -> String {
        unbuilt("GetLanguageName")
    }

    fn G2API_InitGhoul2Model(
        &mut self,
        _ghoul2Ptr: *mut *mut c_void,
        _fileName: &str,
        _modelIndex: c_int,
        _customSkin: qhandle_t,
        _customShader: qhandle_t,
        _modelFlags: c_int,
        _lodBias: c_int,
    ) -> c_int {
        unbuilt("G2API_InitGhoul2Model")
    }

    fn G2API_SetSkin(
        &mut self,
        _ghoul2: *mut c_void,
        _modelIndex: c_int,
        _customSkin: qhandle_t,
        _renderSkin: qhandle_t,
    ) -> bool {
        unbuilt("G2API_SetSkin")
    }

    fn G2API_CleanGhoul2Models(&mut self, _ghoul2Ptr: *mut *mut c_void) {
        unbuilt("G2API_CleanGhoul2Models")
    }

    fn G2API_SetBoneAnim(
        &mut self,
        _ghoul2: *mut c_void,
        _modelIndex: c_int,
        _boneName: &str,
        _startFrame: c_int,
        _endFrame: c_int,
        _flags: c_int,
        _animSpeed: f32,
        _currentTime: c_int,
        _setFrame: f32,
        _blendTime: c_int,
    ) -> bool {
        unbuilt("G2API_SetBoneAnim")
    }

    fn G2API_GetGLAName(
        &mut self,
        _ghoul2: *mut c_void,
        _modelIndex: c_int,
        _buffer_len: usize,
    ) -> String {
        unbuilt("G2API_GetGLAName")
    }

    fn G2_HaveWeGhoul2Models(&mut self, _ghoul2: *mut c_void) -> bool {
        unbuilt("G2_HaveWeGhoul2Models")
    }
}

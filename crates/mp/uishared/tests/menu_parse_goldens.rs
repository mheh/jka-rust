//! Differential parity: the ported `Menu_New`/`Menu_Parse`/
//! `dispatch_menu_keyword`/`MenuParse_*`/`MenuParse_itemDef`/`Item_Parse`/
//! `dispatch_item_keyword`/`ItemParse_*` keyword-dispatch pipeline must
//! reproduce, byte for byte, the dumps produced by the UNMODIFIED Raven C
//! oracle compiled by `tools/ui-oracle/run.sh` (goldens under
//! `tools/ui-oracle/golden/`).
//!
//! The dump format mirrors `tools/ui-oracle/main.cpp` exactly — see that
//! file's header comment for the deterministic stand-ins this test's
//! [`TestDisplayContext`] must reproduce (same shader/model/sound/skin/font
//! handle counters, same "" for every cvar read, same G2 success values),
//! and `tools/ui-oracle/README.md` for the PC_* linking rationale: both
//! sides drive the SAME real, already-ported botlib tokenizer
//! (`mp_engine_botlib`'s `LoadSourceMemory`/`PC_ReadTokenHandle`), not a
//! reimplementation, so this is genuine end-to-end parity.

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use std::ffi::CStr;
use std::fmt::Write as _;
use std::os::raw::c_char;
use std::path::{Path, PathBuf};

use mp_engine_botlib::l_precomp_fns::{LoadSourceMemory, PC_ReadTokenHandle, PC_SourceFileAndLine};
use mp_engine_botlib::BotLib;
use mp_qshared::shared::{pc_token_t, qhandle_t, sfxHandle_t, vec3_t, vec4_t};
use mp_uishared::shared::display_context::DisplayContext;
use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::item_def_s::MAX_COLOR_RANGES;
use mp_uishared::shared::item_id::ItemId;
use mp_uishared::shared::item_payload::ItemPayload;
use mp_uishared::shared::list_box_def_s::MAX_LB_COLUMNS;
use mp_uishared::shared::menu_def_t::MAX_MENUITEMS;
use mp_uishared::shared::menu_id::MenuId;
use mp_uishared::shared::menu_system::MenuSystem;
use mp_uishared::shared::multi_def_s::MAX_MULTI_CVARS;
use mp_uishared::shared::rect_def_t::RectDef;
use mp_uishared::shared::text_scroll_def_s::MAX_TEXTSCROLL_LINES;
use mp_uishared::shared::window_def_t::WindowDef;
use mp_uishared::ui_shared::Menu_New;
use native_string::string_to_latin1;

// ============================================================================
// TestDisplayContext — drives the real mp_engine_botlib tokenizer for
// PC_ReadToken/PC_SourceFileAndLine; registerShaderNoMip/registerModel/
// registerSound/R_RegisterSkin/RegisterFont hand out the SAME
// deterministic counters (same bases) as tools/ui-oracle/main.cpp's DC/trap
// stand-ins; getCVarString/G2API_GetGLAName return "" and
// G2API_InitGhoul2Model always "succeeds", matching the C harness so the
// two sides' handle-valued/flag-derived fields agree. Every other method is
// off this pipeline's tested path and panics loudly, naming itself, if
// ever reached — mirroring the C harness's `aborting()` stand-ins.
// ============================================================================
struct TestDisplayContext {
    bot: BotLib,
    shader_counter: qhandle_t,
    model_counter: qhandle_t,
    sound_counter: sfxHandle_t,
    skin_counter: qhandle_t,
    font_counter: qhandle_t,
    g2_sentinel: i32,
}

impl TestDisplayContext {
    fn new() -> Self {
        TestDisplayContext {
            bot: BotLib::default(),
            shader_counter: 1000,
            model_counter: 2000,
            sound_counter: 3000,
            skin_counter: 4000,
            font_counter: 5000,
            g2_sentinel: 0,
        }
    }

    /// Installs `data` as source handle `handle` — `LoadSourceMemory` +
    /// direct `bot.sourceFiles[handle]` assignment, the same bypass of
    /// `trap_PC_LoadSource`'s filesystem path the C harness's
    /// `ui_oracle_install_source` uses.
    fn install_source(&mut self, handle: c_int, data: &[u8], name: &str) {
        let source = LoadSourceMemory(&mut self.bot, data, data.len() as c_int, name);
        self.bot.sourceFiles[handle as usize] = Some(source);
    }
}

/// Panics naming the missing method — see the struct doc.
fn not_on_tested_path(subject: &str) -> ! {
    panic!("TestDisplayContext::{subject} — not on the menu-parse pipeline's tested path");
}

impl DisplayContext for TestDisplayContext {
    fn registerShaderNoMip(&mut self, _p: &str) -> qhandle_t {
        let h = self.shader_counter;
        self.shader_counter += 1;
        h
    }

    fn setColor(&mut self, _v: Option<vec4_t>) {
        not_on_tested_path("setColor")
    }

    fn drawHandlePic(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _asset: qhandle_t) {
        not_on_tested_path("drawHandlePic")
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
        not_on_tested_path("drawStretchPic")
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
        not_on_tested_path("drawText")
    }

    fn textWidth(&mut self, text: &str, scale: f32, _iMenuFont: c_int) -> c_int {
        // Latin-1 byte length mirrors the C dumper's strlen, not UTF-8 .len().
        (string_to_latin1(text).len() as f32 * 8.0 * scale) as c_int
    }

    fn textHeight(&mut self, _text: &str, _scale: f32, _iMenuFont: c_int) -> c_int {
        not_on_tested_path("textHeight")
    }

    fn registerModel(&mut self, _p: &str) -> qhandle_t {
        let h = self.model_counter;
        self.model_counter += 1;
        h
    }

    fn modelBounds(&mut self, _model: qhandle_t) -> (vec3_t, vec3_t) {
        not_on_tested_path("modelBounds")
    }

    fn fillRect(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _color: vec4_t) {
        not_on_tested_path("fillRect")
    }

    fn drawRect(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _size: f32, _color: vec4_t) {
        not_on_tested_path("drawRect")
    }

    fn drawSides(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _size: f32) {
        not_on_tested_path("drawSides")
    }

    fn drawTopBottom(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _size: f32) {
        not_on_tested_path("drawTopBottom")
    }

    fn clearScene(&mut self) {
        not_on_tested_path("clearScene")
    }

    fn addRefEntityToScene(
        &mut self,
        _re: &mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t,
    ) {
        not_on_tested_path("addRefEntityToScene")
    }

    fn renderScene(&mut self, _fd: &mp_qshared::common::mp::cgame::refdef_t::refdef_t) {
        not_on_tested_path("renderScene")
    }

    fn RegisterFont(&mut self, _fontName: &str) -> qhandle_t {
        let h = self.font_counter;
        self.font_counter += 1;
        h
    }

    fn Font_StrLenPixels(&mut self, _text: &str, _iFontIndex: c_int, _scale: f32) -> c_int {
        not_on_tested_path("Font_StrLenPixels")
    }

    fn Font_StrLenChars(&mut self, _text: &str) -> c_int {
        not_on_tested_path("Font_StrLenChars")
    }

    fn Font_HeightPixels(&mut self, _iFontIndex: c_int, _scale: f32) -> c_int {
        not_on_tested_path("Font_HeightPixels")
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
        not_on_tested_path("Font_DrawString")
    }

    fn Language_IsAsian(&mut self) -> bool {
        not_on_tested_path("Language_IsAsian")
    }

    fn Language_UsesSpaces(&mut self) -> bool {
        true
    }

    fn AnyLanguage_ReadCharFromString(&mut self, psText: &[u8]) -> (u32, c_int, bool) {
        (psText[0] as u32, 1, false)
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
        not_on_tested_path("ownerDrawItem")
    }

    fn getValue(&mut self, _ownerDraw: c_int) -> f32 {
        not_on_tested_path("getValue")
    }

    fn ownerDrawVisible(&mut self, _flags: c_int) -> bool {
        not_on_tested_path("ownerDrawVisible")
    }

    fn runScript(&mut self, _p: &mut &str) {
        not_on_tested_path("runScript")
    }

    fn deferScript(&mut self, _p: &mut &str) -> bool {
        not_on_tested_path("deferScript")
    }

    fn getTeamColor(&mut self) -> vec4_t {
        not_on_tested_path("getTeamColor")
    }

    fn getCVarString(&mut self, _cvar: &str, _bufsize: usize) -> String {
        String::new()
    }

    fn getCVarValue(&mut self, _cvar: &str) -> f32 {
        not_on_tested_path("getCVarValue")
    }

    fn setCVar(&mut self, _cvar: &str, _value: &str) {
        not_on_tested_path("setCVar")
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
        not_on_tested_path("drawTextWithCursor")
    }

    fn setOverstrikeMode(&mut self, _b: bool) {
        not_on_tested_path("setOverstrikeMode")
    }

    fn getOverstrikeMode(&mut self) -> bool {
        not_on_tested_path("getOverstrikeMode")
    }

    fn startLocalSound(&mut self, _sfx: sfxHandle_t, _channelNum: c_int) {
        not_on_tested_path("startLocalSound")
    }

    fn ownerDrawHandleKey(
        &mut self,
        _ownerDraw: c_int,
        _flags: c_int,
        _special: &mut f32,
        _key: c_int,
    ) -> bool {
        not_on_tested_path("ownerDrawHandleKey")
    }

    fn feederCount(&mut self, _feederID: f32) -> c_int {
        not_on_tested_path("feederCount")
    }

    fn feederItemText(
        &mut self,
        _feederID: f32,
        _index: c_int,
        _column: c_int,
    ) -> (Option<String>, qhandle_t, qhandle_t, qhandle_t) {
        not_on_tested_path("feederItemText")
    }

    fn feederItemImage(&mut self, _feederID: f32, _index: c_int) -> qhandle_t {
        not_on_tested_path("feederItemImage")
    }

    fn feederSelection(&mut self, _feederID: f32, _index: c_int, _item: Option<ItemId>) -> bool {
        not_on_tested_path("feederSelection")
    }

    fn keynumToStringBuf(&mut self, _keynum: c_int, _buflen: usize) -> String {
        not_on_tested_path("keynumToStringBuf")
    }

    fn getBindingBuf(&mut self, _keynum: c_int, _buflen: usize) -> String {
        not_on_tested_path("getBindingBuf")
    }

    fn setBinding(&mut self, _keynum: c_int, _binding: &str) {
        not_on_tested_path("setBinding")
    }

    fn executeText(&mut self, _exec_when: c_int, _text: &str) {
        not_on_tested_path("executeText")
    }

    fn Error(&mut self, _level: c_int, _error: &str) {
        not_on_tested_path("Error")
    }

    fn Print(&mut self, _msg: &str) {
        // Matches the C harness's real (non-aborting) Com_Printf: PC_SourceError
        // routes through here and DOES run on the tested path (unknown-keyword
        // recovery, truncated-source edge cases). Not captured in the dump.
        eprint!("{_msg}");
    }

    fn Pause(&mut self, _b: bool) {
        not_on_tested_path("Pause")
    }

    fn ownerDrawWidth(&mut self, _ownerDraw: c_int, _scale: f32) -> c_int {
        not_on_tested_path("ownerDrawWidth")
    }

    fn registerSound(&mut self, _name: &str) -> sfxHandle_t {
        let h = self.sound_counter;
        self.sound_counter += 1;
        h
    }

    fn startBackgroundTrack(&mut self, _intro: &str, _loop_: &str, _bReturnWithoutStarting: bool) {
        not_on_tested_path("startBackgroundTrack")
    }

    fn stopBackgroundTrack(&mut self) {
        not_on_tested_path("stopBackgroundTrack")
    }

    fn playCinematic(&mut self, _name: &str, _x: f32, _y: f32, _w: f32, _h: f32) -> c_int {
        not_on_tested_path("playCinematic")
    }

    fn stopCinematic(&mut self, _handle: c_int) {
        not_on_tested_path("stopCinematic")
    }

    fn drawCinematic(&mut self, _handle: c_int, _x: f32, _y: f32, _w: f32, _h: f32) {
        not_on_tested_path("drawCinematic")
    }

    fn runCinematicFrame(&mut self, _handle: c_int) {
        not_on_tested_path("runCinematicFrame")
    }

    fn Milliseconds(&mut self) -> c_int {
        not_on_tested_path("Milliseconds")
    }

    fn setCVarValue(&mut self, _cvar: &str, _value: f32) {
        not_on_tested_path("setCVarValue")
    }

    fn Key_IsDown(&mut self, _keynum: c_int) -> bool {
        not_on_tested_path("Key_IsDown")
    }

    fn Key_GetCatcher(&mut self) -> c_int {
        not_on_tested_path("Key_GetCatcher")
    }

    fn Key_SetCatcher(&mut self, _catcher: c_int) {
        not_on_tested_path("Key_SetCatcher")
    }

    fn Key_ClearStates(&mut self) {
        not_on_tested_path("Key_ClearStates")
    }

    fn PC_ReadToken(&mut self, handle: c_int, pc_token: &mut pc_token_t) -> bool {
        PC_ReadTokenHandle(&mut self.bot, handle, pc_token as *mut pc_token_t) != 0
    }

    fn PC_SourceFileAndLine(&mut self, handle: c_int, buffer_len: usize) -> (c_int, String, c_int) {
        let mut buf = vec![0u8; buffer_len.max(1)];
        let mut line: c_int = 0;
        let status = PC_SourceFileAndLine(
            &mut self.bot,
            handle,
            buf.as_mut_ptr() as *mut c_char,
            &mut line as *mut c_int,
        );
        let filename = unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) }
            .to_string_lossy()
            .into_owned();
        (status, filename, line)
    }

    fn SP_GetStringTextString(&mut self, _text: &str, _buffer_len: usize) -> Option<String> {
        not_on_tested_path("SP_GetStringTextString")
    }

    fn R_RegisterSkin(&mut self, _name: &str) -> qhandle_t {
        let h = self.skin_counter;
        self.skin_counter += 1;
        h
    }

    fn GetLanguageName(&mut self, _languageIndex: c_int, _buffer_len: usize) -> String {
        not_on_tested_path("GetLanguageName")
    }

    fn G2API_InitGhoul2Model(
        &mut self,
        ghoul2Ptr: *mut *mut c_void,
        _fileName: &str,
        _modelIndex: c_int,
        _customSkin: qhandle_t,
        _customShader: qhandle_t,
        _modelFlags: c_int,
        _lodBias: c_int,
    ) -> c_int {
        // Fixed non-NULL sentinel — never dereferenced or dumped, only the
        // resulting ITF_G2VALID flag bit is observable (matches
        // tools/ui-oracle/main.cpp's g_g2Sentinel).
        unsafe { *ghoul2Ptr = &mut self.g2_sentinel as *mut i32 as *mut c_void };
        0
    }

    fn G2API_SetSkin(
        &mut self,
        _ghoul2: *mut c_void,
        _modelIndex: c_int,
        _customSkin: qhandle_t,
        _renderSkin: qhandle_t,
    ) -> bool {
        true
    }

    fn G2API_CleanGhoul2Models(&mut self, ghoul2Ptr: *mut *mut c_void) {
        if !ghoul2Ptr.is_null() {
            unsafe { *ghoul2Ptr = core::ptr::null_mut() };
        }
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
        true
    }

    fn G2API_GetGLAName(
        &mut self,
        _ghoul2: *mut c_void,
        _modelIndex: c_int,
        _buffer_len: usize,
    ) -> String {
        String::new()
    }

    fn G2_HaveWeGhoul2Models(&mut self, _ghoul2: *mut c_void) -> bool {
        false
    }
}

// ============================================================================
// Canonical dump — mirrors tools/ui-oracle/main.cpp's dumpMenu/dumpItem/
// dumpWindow/dumpTypeData field-by-field, in the same order, with the same
// text format (including its quirky per-line prefixes: window fields always
// get a "  " prefix regardless of menu/item nesting; item wrapper fields get
// "  ", menu wrapper fields get none; "== item N ==" has a leading space,
// "== menu N ==" does not — transcribed as-is for byte parity).
// ============================================================================
fn esc(out: &mut String, s: &str) {
    out.push('|');
    // Iterate Latin-1 bytes so the escape mirrors the C dumper's per-byte
    // `%02x` exactly (a codepoint > 0xFF would otherwise emit 3+ hex digits).
    for &b in &string_to_latin1(s) {
        if b == b'|' || b == b'\\' || b < 0x20 || b >= 0x7f {
            write!(out, "\\x{b:02x}").unwrap();
        } else {
            out.push(b as char);
        }
    }
    out.push('|');
}

fn pstr_opt(out: &mut String, label: &str, s: Option<&str>) {
    write!(out, "{label}: ").unwrap();
    match s {
        None => out.push_str("<NULL>\n"),
        Some(s) => {
            esc(out, s);
            out.push('\n');
        }
    }
}
fn pstr_def(out: &mut String, label: &str, s: &str) {
    pstr_opt(out, label, Some(s));
}
fn pint(out: &mut String, label: &str, v: c_int) {
    writeln!(out, "{label}: {v}").unwrap();
}
fn pfloat(out: &mut String, label: &str, v: f32) {
    writeln!(out, "{label}: {v:.6}").unwrap();
}
fn pbool(out: &mut String, label: &str, v: bool) {
    writeln!(out, "{label}: {}", if v { "true" } else { "false" }).unwrap();
}
fn prect(out: &mut String, label: &str, r: &RectDef) {
    writeln!(
        out,
        "{label}: ({:.6}, {:.6}, {:.6}, {:.6})",
        r.x, r.y, r.w, r.h
    )
    .unwrap();
}
fn pvec4(out: &mut String, label: &str, v: &vec4_t) {
    writeln!(
        out,
        "{label}: ({:.6}, {:.6}, {:.6}, {:.6})",
        v[0], v[1], v[2], v[3]
    )
    .unwrap();
}
fn pvec3(out: &mut String, label: &str, v: &vec3_t) {
    writeln!(out, "{label}: ({:.6}, {:.6}, {:.6})", v[0], v[1], v[2]).unwrap();
}

fn dump_window(out: &mut String, w: &WindowDef) {
    prect(out, "  window.rect", &w.rect);
    prect(out, "  window.rectClient", &w.rectClient);
    pstr_opt(out, "  window.name", w.name.as_deref());
    pstr_opt(out, "  window.group", w.group.as_deref());
    pstr_def(out, "  window.cinematicName", &w.cinematicName);
    pint(out, "  window.cinematic", w.cinematic);
    pint(out, "  window.style", w.style);
    pint(out, "  window.border", w.border);
    pint(out, "  window.ownerDraw", w.ownerDraw);
    pint(out, "  window.ownerDrawFlags", w.ownerDrawFlags);
    pfloat(out, "  window.borderSize", w.borderSize);
    pint(out, "  window.flags", w.flags);
    prect(out, "  window.rectEffects", &w.rectEffects);
    prect(out, "  window.rectEffects2", &w.rectEffects2);
    pint(out, "  window.offsetTime", w.offsetTime);
    pint(out, "  window.nextTime", w.nextTime);
    pvec4(out, "  window.foreColor", &w.foreColor);
    pvec4(out, "  window.backColor", &w.backColor);
    pvec4(out, "  window.borderColor", &w.borderColor);
    pvec4(out, "  window.outlineColor", &w.outlineColor);
    pint(out, "  window.background", w.background);
}

fn dump_type_data(out: &mut String, payload: &ItemPayload) {
    match payload {
        ItemPayload::None => {
            out.push_str("  typeData: none\n");
        }
        ItemPayload::ListBox(l) => {
            out.push_str("  typeData: listbox\n");
            pint(out, "    startPos", l.startPos);
            pint(out, "    endPos", l.endPos);
            pint(out, "    drawPadding", l.drawPadding);
            pint(out, "    cursorPos", l.cursorPos);
            pfloat(out, "    elementWidth", l.elementWidth);
            pfloat(out, "    elementHeight", l.elementHeight);
            pint(out, "    elementStyle", l.elementStyle);
            pint(out, "    numColumns", l.numColumns);
            for i in 0..(l.numColumns.max(0) as usize).min(MAX_LB_COLUMNS) {
                let c = &l.columnInfo[i];
                writeln!(
                    out,
                    "    columnInfo[{i}]: pos={} width={} maxChars={}",
                    c.pos, c.width, c.maxChars
                )
                .unwrap();
            }
            pstr_def(out, "    doubleClick", &l.doubleClick);
            pbool(out, "    notselectable", l.notselectable);
            pbool(out, "    scrollhidden", l.scrollhidden);
        }
        ItemPayload::EditField(e) => {
            out.push_str("  typeData: editfield\n");
            pfloat(out, "    minVal", e.minVal);
            pfloat(out, "    maxVal", e.maxVal);
            pfloat(out, "    defVal", e.defVal);
            pfloat(out, "    range", e.range);
            pint(out, "    maxChars", e.maxChars);
            pint(out, "    maxPaintChars", e.maxPaintChars);
            pint(out, "    paintOffset", e.paintOffset);
        }
        ItemPayload::Multi(m) => {
            out.push_str("  typeData: multi\n");
            pbool(out, "    strDef", m.strDef);
            pint(out, "    count", m.cvarList.len() as c_int);
            for i in 0..m.cvarList.len().min(MAX_MULTI_CVARS) {
                write!(out, "    cvarList[{i}]: ").unwrap();
                esc(out, &m.cvarList[i]);
                out.push('\n');
                if m.strDef {
                    write!(out, "    cvarStr[{i}]: ").unwrap();
                    esc(out, &m.cvarStr[i]);
                    out.push('\n');
                } else {
                    writeln!(out, "    cvarValue[{i}]: {:.6}", m.cvarValue[i]).unwrap();
                }
            }
        }
        ItemPayload::Model(md) => {
            out.push_str("  typeData: model\n");
            pint(out, "    angle", md.angle);
            pvec3(out, "    origin", &md.origin);
            pfloat(out, "    fov_x", md.fov_x);
            pfloat(out, "    fov_y", md.fov_y);
            pint(out, "    rotationSpeed", md.rotationSpeed);
            pvec3(out, "    g2mins", &md.g2mins);
            pvec3(out, "    g2maxs", &md.g2maxs);
            pvec3(out, "    g2scale", &md.g2scale);
            pint(out, "    g2skin", md.g2skin);
            pint(out, "    g2anim", md.g2anim);
            pvec3(out, "    g2mins2", &md.g2mins2);
            pvec3(out, "    g2maxs2", &md.g2maxs2);
            pvec3(out, "    g2minsEffect", &md.g2minsEffect);
            pvec3(out, "    g2maxsEffect", &md.g2maxsEffect);
            pfloat(out, "    fov_x2", md.fov_x2);
            pfloat(out, "    fov_y2", md.fov_y2);
            pfloat(out, "    fov_Effectx", md.fov_Effectx);
            pfloat(out, "    fov_Effecty", md.fov_Effecty);
        }
        ItemPayload::TextScroll(t) => {
            out.push_str("  typeData: textscroll\n");
            pint(out, "    startPos", t.startPos);
            pint(out, "    endPos", t.endPos);
            pfloat(out, "    lineHeight", t.lineHeight);
            pint(out, "    maxLineChars", t.maxLineChars);
            pint(out, "    drawPadding", t.drawPadding);
            pint(out, "    iLineCount", t.pLines.len() as c_int);
            for (i, line) in t.pLines.iter().enumerate().take(MAX_TEXTSCROLL_LINES) {
                write!(out, "    pLines[{i}]: ").unwrap();
                esc(out, line);
                out.push('\n');
            }
        }
    }
}

fn dump_item(out: &mut String, idx: usize, menus: &MenuSystem, id: ItemId) {
    let item = menus.item(id);
    writeln!(out, " == item {idx} ==").unwrap();
    dump_window(out, &item.window);
    prect(out, "  textRect", &item.textRect);
    pint(out, "  type", item.r#type);
    pint(out, "  alignment", item.alignment);
    pint(out, "  textalignment", item.textalignment);
    pfloat(out, "  textalignx", item.textalignx);
    pfloat(out, "  textaligny", item.textaligny);
    pfloat(out, "  textscale", item.textscale);
    pint(out, "  textStyle", item.textStyle);
    pstr_opt(out, "  text", item.text.as_deref());
    pstr_def(out, "  text2", &item.text2);
    pfloat(out, "  text2alignx", item.text2alignx);
    pfloat(out, "  text2aligny", item.text2aligny);
    pint(out, "  asset", item.asset);
    pint(out, "  flags", item.flags);
    pstr_def(out, "  mouseEnterText", &item.mouseEnterText);
    pstr_def(out, "  mouseExitText", &item.mouseExitText);
    pstr_def(out, "  mouseEnter", &item.mouseEnter);
    pstr_def(out, "  mouseExit", &item.mouseExit);
    pstr_def(out, "  action", &item.action);
    pstr_def(out, "  accept", &item.accept);
    pstr_def(out, "  selectionNext", &item.selectionNext);
    pstr_def(out, "  selectionPrev", &item.selectionPrev);
    pstr_def(out, "  onFocus", &item.onFocus);
    pstr_def(out, "  leaveFocus", &item.leaveFocus);
    pstr_opt(out, "  cvar", item.cvar.as_deref());
    pstr_def(out, "  cvarTest", &item.cvarTest);
    pstr_def(out, "  enableCvar", &item.enableCvar);
    pint(out, "  cvarFlags", item.cvarFlags);
    pint(out, "  focusSound", item.focusSound);
    pint(out, "  numColors", item.numColors);
    for i in 0..(item.numColors.max(0) as usize).min(MAX_COLOR_RANGES) {
        let c = &item.colorRanges[i];
        writeln!(
            out,
            "  colorRanges[{i}]: low={:.6} high={:.6} color=({:.6}, {:.6}, {:.6}, {:.6})",
            c.low, c.high, c.color[0], c.color[1], c.color[2], c.color[3]
        )
        .unwrap();
    }
    pfloat(out, "  special", item.special);
    pint(out, "  cursorPos", item.cursorPos);
    dump_type_data(out, &item.typeData);
    pstr_def(out, "  descText", &item.descText);
    pint(out, "  appearanceSlot", item.appearanceSlot);
    pint(out, "  iMenuFont", item.iMenuFont);
    pbool(out, "  disabled", item.disabled);
    pint(out, "  invertYesNo", item.invertYesNo);
    pint(out, "  xoffset", item.xoffset);
}

fn dump_menu(
    out: &mut String,
    idx: usize,
    menus: &MenuSystem,
    id: mp_uishared::shared::menu_id::MenuId,
) {
    let menu = menus.menu(id);
    writeln!(out, "== menu {idx} ==").unwrap();
    dump_window(out, &menu.window);
    pstr_def(out, "font", &menu.font);
    pbool(out, "fullScreen", menu.fullScreen);
    pint(out, "fontIndex", menu.fontIndex);
    pint(out, "cursorItem", menu.cursorItem);
    pint(out, "fadeCycle", menu.fadeCycle);
    pfloat(out, "fadeClamp", menu.fadeClamp);
    pfloat(out, "fadeAmount", menu.fadeAmount);
    pstr_def(out, "onOpen", &menu.onOpen);
    pstr_def(out, "onClose", &menu.onClose);
    pstr_def(out, "onAccept", &menu.onAccept);
    pstr_def(out, "onESC", &menu.onESC);
    pstr_def(out, "soundName", &menu.soundName);
    pvec4(out, "focusColor", &menu.focusColor);
    pvec4(out, "disableColor", &menu.disableColor);
    pint(out, "itemCount", menu.items.len() as c_int);
    pint(out, "descX", menu.descX);
    pint(out, "descY", menu.descY);
    pvec4(out, "descColor", &menu.descColor);
    pint(out, "descAlignment", menu.descAlignment);
    pfloat(out, "descScale", menu.descScale);
    pfloat(out, "appearanceTime", menu.appearanceTime);
    pint(out, "appearanceCnt", menu.appearanceCnt);
    pfloat(out, "appearanceIncrement", menu.appearanceIncrement);
    let itemIds = menu.items.clone();
    for (i, itemId) in itemIds.into_iter().enumerate().take(MAX_MENUITEMS) {
        dump_item(out, i, menus, itemId);
    }
}

/// Drives `attempts` `Menu_New(1)` calls over `fixture_bytes` and produces
/// the canonical dump — the Rust twin of `tools/ui-oracle/main.cpp`'s `main`.
fn dump(fixture_bytes: &[u8], fixture_name: &str, attempts: c_int) -> String {
    let mut dc = TestDisplayContext::new();
    let mut ds = DisplayState::default();
    let mut menus = MenuSystem::default();

    dc.install_source(1, fixture_bytes, fixture_name);

    let mut out = String::new();
    for i in 0..attempts {
        let before = menus.menus.len();
        Menu_New(&mut menus, &mut ds, &mut dc, 1);
        let ok = menus.menus.len() > before;
        writeln!(
            out,
            "== attempt {i}: {} (menuCount now {}) ==",
            if ok { "ok" } else { "error" },
            menus.menus.len()
        )
        .unwrap();
    }

    writeln!(out, "== menuCount {} ==", menus.menus.len()).unwrap();
    let menuIds: Vec<_> = (0..menus.menus.len()).map(MenuId::new).collect();
    for (i, menuId) in menuIds.into_iter().enumerate() {
        dump_menu(&mut out, i, &menus, menuId);
    }
    out.push_str("== end ==\n");
    out
}

/// (fixture stem, Menu_New attempts) — must match tools/ui-oracle/run.sh's
/// `run_one` calls exactly.
const FIXTURES: &[(&str, c_int)] = &[
    ("retail", 1),
    ("all_menu_keywords", 1),
    ("broad_item_keywords", 1),
    ("edge_cases", 8),
];

#[test]
fn matches_oracle_goldens() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../tools/ui-oracle");

    for &(name, attempts) in FIXTURES {
        let fixture_path = root.join("fixtures").join(format!("{name}.menu"));
        let fixture_bytes = std::fs::read(&fixture_path)
            .unwrap_or_else(|_| panic!("read fixture {fixture_path:?}"));
        let golden_path = root.join("golden").join(format!("{name}.txt"));
        let golden = std::fs::read_to_string(&golden_path).unwrap_or_else(|_| {
            panic!("missing golden {golden_path:?} — run tools/ui-oracle/run.sh --regen")
        });

        let got = dump(&fixture_bytes, name, attempts);
        assert_eq!(got, golden, "fixture {name} diverges from the C oracle");
    }
}

/// Sanity check that the fixture/golden directories this test reads from
/// actually resolve (fails loudly on a moved/renamed tools/ dir rather than
/// silently checking zero fixtures).
#[test]
fn fixture_root_resolves() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../tools/ui-oracle");
    assert!(
        Path::new(&root).join("fixtures").is_dir(),
        "expected {root:?}/fixtures to exist"
    );
    assert_eq!(FIXTURES.len(), 4);
}

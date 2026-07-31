//! `HarnessDc` — the dev harness's [`DisplayContext`] implementor.
//!
//! `mp_ui`'s `UiContext` backs every slot of this trait on a `trap_*` syscall
//! into a C engine. This one backs them on **our own stack**, and that is the
//! whole point of the wave: nothing between the menu framework and the pixels
//! is a double.
//!
//! * **Draw slots** append [`FrameEvent`]s to an owned [`FrameData`], the same
//!   stream [`crate::frame_exec`] already executes. `drawHandlePic`,
//!   `fillRect`, `drawRect`, `drawSides` and `drawTopBottom` are transcriptions
//!   of `ui_atoms.c`/`ui_main.c`'s bodies with `trap_R_DrawStretchPic` replaced
//!   by a pushed event, so the emitted geometry is Raven's.
//! * **Text slots** enter the real `mp_renderer` font pipeline —
//!   `RE_RegisterFont`, `RE_Font_StrLenPixels`, `RE_Font_HeightPixels`,
//!   `RE_Font_DrawString` — through `Text_Paint`'s own font-index and
//!   style-translation logic (`MenuFontToHandle`, the JK2→SOF2 ctrl-code
//!   `match`). `RE_Font_DrawString` writes its glyph events straight into this
//!   struct's `frame_data`.
//! * **Registration slots** enter `RE_RegisterShaderNoMip` / `RE_RegisterSkin`,
//!   so a menu's art is loaded off the retail pk3s by the ported image and
//!   shader pipeline.
//! * **Cvar/cmd/print slots** hit the real engine tables through
//!   [`EngineHostView`].
//! * **`PC_*` slots and [`HarnessDc::load_menus`]** drive the production
//!   `mp_engine_botlib` precompiler — the same tokenizer the live engine uses.
//!
//! **What is stubbed, and why.** Two families, both logged through
//! [`StubLog`] rather than panicked:
//!
//! 1. `ui_main.c`-owned callbacks (`ownerDraw*`, `feeder*`, `runScript`,
//!    `deferScript`, `getValue`, `getTeamColor`, the species/language cvar
//!    lists, the saber and datapad-animation hooks). Every one of them takes a
//!    concrete `&mut UiContext`, whose `engine` field is the module-side C
//!    syscall transport; reaching it needs the unported engine-side UI syscall
//!    dispatcher (see the [`super`] module doc).
//! 2. Engine services this harness deliberately never booted: sound,
//!    cinematics, key bindings, StringEd, ghoul2, and the 3D scene path
//!    (`clearScene`/`addRefEntityToScene`/`renderScene`) that backend #1 does
//!    not render yet.
//!
//! A stub returns a benign default and counts itself, so a run's tail can state
//! exactly what the menus asked for and did not get. Nothing here panics or
//! `unreachable!()`s: the harness exists to show how far a real menu gets, and
//! an honest degraded frame is more informative than an abort.

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use mp_bg::public::animation::animation_t;
use mp_engine_botlib::l_precomp_fns::{
    PC_FreeSourceHandle, PC_LoadGlobalDefines, PC_LoadSourceHandle, PC_ReadTokenHandle,
    PC_RemoveAllGlobalDefines, PC_SourceFileAndLine,
};
use mp_engine_botlib::BotLib;
use mp_engine_qcommon::cmd_common::Cbuf_AddText;
use mp_engine_qcommon::common::common::com_printf;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::cvar_fns::{
    Cvar_Set, Cvar_SetValue, Cvar_VariableString, Cvar_VariableValue,
};
use mp_engine_qcommon::qfiles::font_style::{STYLE_BLINK, STYLE_DROPSHADOW};
use mp_engine_qcommon::stringed::api::SE_GetString;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::shared::com_parse::QSharedScratch;
use mp_qshared::shared::{pc_token_t, qhandle_t, sfxHandle_t, vec3_t, vec4_t, MAX_TOKENLENGTH};
use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::frame_event::FrameEvent;
use mp_renderer::render_state::frame_state::FrameState;
use mp_renderer::render_state::gpu_resources::GpuResources;
use mp_renderer::render_state::render_assets::RenderAssets;
use mp_renderer::render_state::render_assets_sim::RenderAssetsSim;
use mp_renderer::render_state::renderer_cvars::RendererCvars;
use mp_renderer::render_state::shader_asset::ShaderHandle;
use mp_renderer::tr_font::{
    FontState, Language_e, RE_Font_DrawString, RE_Font_HeightPixels, RE_Font_StrLenChars,
    RE_Font_StrLenPixels, RE_RegisterFont,
};
use mp_renderer::tr_image::{RE_RegisterSkin, TrImageState};
use mp_renderer::tr_local::view_parms_t::viewParms_t;
use mp_renderer::tr_model::render_models::RenderModels;
use mp_renderer::tr_shader::RE_RegisterShaderNoMip;
use mp_renderer::tr_sky::SkyState;
use mp_ui::ui_main::MenuFontToHandle;
use mp_uishared::shared::display_context::DisplayContext;
use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::item_def_s::ItemDef;
use mp_uishared::shared::item_id::ItemId;
use mp_uishared::shared::menu_system::MenuSystem;
use mp_uishared::shared::menudef::{
    ITEM_TEXTSTYLE_BLINK, ITEM_TEXTSTYLE_NORMAL, ITEM_TEXTSTYLE_OUTLINED,
    ITEM_TEXTSTYLE_OUTLINESHADOWED, ITEM_TEXTSTYLE_PULSE, ITEM_TEXTSTYLE_SHADOWED,
    ITEM_TEXTSTYLE_SHADOWEDMORE,
};
use mp_uishared::ui_shared::Menu_New;
use native_string::{buf_to_string, latin1_to_string, string_to_latin1};

use crate::ui_host::state::{InputState, StubLog};

/// The language every text slot resolves against.
///
/// `GetLanguageEnum()`/`se_language->modificationCount` are unported (the
/// `tr_font` file-head DEFERRED note), and the harness loads no StringEd
/// package, so the western path — the one every retail English asset takes —
/// is the only reachable one.
const HARNESS_LANGUAGE: Language_e = Language_e::eWestern;

/// `iSE_Language_ModificationCount` for every font call: the harness never
/// switches language packages, so the count never moves off zero.
const HARNESS_LANGUAGE_MODCOUNT: i32 = 0;

/// The harness's [`DisplayContext`] — a bundle of disjoint `&mut` borrows out
/// of [`crate::ui_host::state::UiHost`], rebuilt every frame by
/// [`crate::ui_host::boot::with_dc`], plus the frame's own event stream.
///
/// The carrier list (`view`..`sky`) is exactly DEC-42.3's: what
/// `RE_RegisterShaderNoMip` and the `RE_Font_*` family need to reach the
/// filesystem, the cvar table, the image pool and the shader arena.
pub struct HarnessDc<'a> {
    /// The engine islands — cvars, cmd/cbuf, filesystem, hooks.
    pub view: EngineHostView<'a>,
    /// The precompiler island, used solely as the menu-file tokenizer.
    pub bot: &'a mut BotLib,
    pub cvars: &'a mut RendererCvars,
    pub assets: &'a mut RenderAssets,
    pub sim: &'a mut RenderAssetsSim,
    pub models: &'a mut RenderModels,
    pub img_state: &'a mut TrImageState,
    pub gpu: &'a mut GpuResources,
    pub frame: &'a mut FrameState,
    pub qs: &'a mut QSharedScratch,
    pub sky_view: &'a mut viewParms_t,
    pub sky: &'a mut SkyState,
    pub font: &'a mut FontState,
    /// The frame's event stream, drained by the caller after painting.
    pub frame_data: FrameData,
    /// The key/cursor state Raven's engine owned on the module's behalf.
    pub input: &'a mut InputState,
    /// One counter per stubbed slot (see the module doc).
    pub stubs: &'a mut StubLog,
    /// `trap_Milliseconds`' answer for this frame.
    pub millis: c_int,
}

// ============================================================================
// Harness-side helpers: the shared bodies the trait slots delegate to, plus
// the botlib menu-file driver `UI_LoadMenus`/`Load_Menu`/`UI_ParseMenu` would
// have provided if they were not `UiContext`-bound.
// ============================================================================

impl HarnessDc<'_> {
    /// Resolves a Raven `qhandle_t` shader index to an arena [`ShaderHandle`].
    ///
    /// DEC-42.2's "slot = index" rule: the menu framework stores plain ints, so
    /// the int addresses an arena slot. A non-positive handle (Raven's "no
    /// shader") and a stale index both fall back to slot zero, the default
    /// shader — the same white quad `frame_exec` degrades to.
    fn shader_handle(&self, h: qhandle_t) -> ShaderHandle {
        if h <= 0 {
            return ShaderHandle::slot_zero();
        }
        self.assets
            .shaders
            .handle_at_slot(h as u32)
            .unwrap_or_else(ShaderHandle::slot_zero)
    }

    /// Appends one `DrawStretchPic` event, resolving `hShader` on the way in.
    #[allow(clippy::too_many_arguments)]
    fn push_stretch_pic(
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
    ) {
        let shader = self.shader_handle(hShader);
        self.frame_data.events.push(FrameEvent::DrawStretchPic {
            x,
            y,
            w,
            h,
            s1,
            t1,
            s2,
            t2,
            shader,
        });
    }

    /// `RE_Font_StrLenPixels` over the harness's carriers.
    fn font_str_len_pixels(&mut self, text: &str, iFontIndex: c_int, scale: f32) -> c_int {
        let bytes = string_to_latin1(text);
        RE_Font_StrLenPixels(
            self.qs,
            self.frame,
            self.assets,
            &mut self.view,
            &*self.cvars,
            self.sim,
            &*self.models,
            self.img_state,
            self.gpu,
            self.sky_view,
            self.sky,
            self.font,
            HARNESS_LANGUAGE,
            HARNESS_LANGUAGE_MODCOUNT,
            &bytes,
            iFontIndex,
            scale,
        )
    }

    /// `RE_Font_DrawString` over the harness's carriers, emitting its glyph
    /// events into this frame's own [`FrameData`].
    #[allow(clippy::too_many_arguments)]
    fn font_draw_string(
        &mut self,
        ox: c_int,
        oy: c_int,
        text: &str,
        rgba: vec4_t,
        iFontHandle: c_int,
        iMaxPixelWidth: c_int,
        scale: f32,
    ) {
        let bytes = string_to_latin1(text);
        let millis = self.millis;
        RE_Font_DrawString(
            self.qs,
            self.frame,
            self.assets,
            &mut self.view,
            &*self.cvars,
            self.sim,
            &*self.models,
            self.img_state,
            self.gpu,
            self.sky_view,
            self.sky,
            self.font,
            HARNESS_LANGUAGE,
            HARNESS_LANGUAGE_MODCOUNT,
            &mut self.frame_data,
            ox,
            oy,
            &bytes,
            Some(rgba),
            iFontHandle,
            iMaxPixelWidth,
            scale,
            millis,
        );
    }

    /// Raven `Text_Paint`'s body — the menu-font lookup plus the JK2-menu-style
    /// to SOF2-printstring-ctrl-code translation, then the font draw.
    ///
    /// Source: `oracle/codemp/ui/ui_main.c:1103-1130`
    #[allow(clippy::too_many_arguments)]
    fn text_paint(
        &mut self,
        ds: &DisplayState,
        x: f32,
        y: f32,
        scale: f32,
        color: vec4_t,
        text: &str,
        limit: c_int,
        style: c_int,
        iMenuFont: c_int,
    ) {
        let iFontIndex = MenuFontToHandle(ds, iMenuFont);
        // kludge.. convert JK2 menu styles to SOF2 printstring ctrl codes...
        let iStyleOR: c_int = match style {
            ITEM_TEXTSTYLE_NORMAL => 0,                           // JK2 normal text
            ITEM_TEXTSTYLE_BLINK => STYLE_BLINK as c_int,         // JK2 fast blinking
            ITEM_TEXTSTYLE_PULSE => STYLE_BLINK as c_int,         // JK2 slow pulsing
            ITEM_TEXTSTYLE_SHADOWED => STYLE_DROPSHADOW as c_int, // JK2 drop shadow
            ITEM_TEXTSTYLE_OUTLINED => STYLE_DROPSHADOW as c_int, // JK2 drop shadow
            ITEM_TEXTSTYLE_OUTLINESHADOWED => STYLE_DROPSHADOW as c_int, // JK2 drop shadow
            ITEM_TEXTSTYLE_SHADOWEDMORE => STYLE_DROPSHADOW as c_int, // JK2 drop shadow
            _ => 0,
        };

        self.font_draw_string(
            x as c_int,
            y as c_int,
            text,
            color,
            iStyleOR | iFontIndex,
            if limit == 0 { -1 } else { limit }, // iCharLimit (-1 = none)
            scale,
        );
    }

    /// Raven `_UI_DrawSides` — the left/right edges of a `size`-thick border,
    /// `size` scaled horizontally.
    ///
    /// Source: `oracle/codemp/ui/ui_main.c:1047-1051`
    fn draw_sides(&mut self, ds: &DisplayState, x: f32, y: f32, w: f32, h: f32, size: f32) {
        let size = size * ds.xscale;
        let white = ds.whiteShader;
        self.push_stretch_pic(x, y, size, h, 0.0, 0.0, 0.0, 0.0, white);
        self.push_stretch_pic(x + w - size, y, size, h, 0.0, 0.0, 0.0, 0.0, white);
    }

    /// Raven `_UI_DrawTopBottom` — the top/bottom edges of a `size`-thick
    /// border, `size` scaled vertically.
    ///
    /// Source: `oracle/codemp/ui/ui_main.c:1053-1057`
    fn draw_top_bottom(&mut self, ds: &DisplayState, x: f32, y: f32, w: f32, h: f32, size: f32) {
        let size = size * ds.yscale;
        let white = ds.whiteShader;
        self.push_stretch_pic(x, y, w, size, 0.0, 0.0, 0.0, 0.0, white);
        self.push_stretch_pic(x, y + h - size, w, size, 0.0, 0.0, 0.0, 0.0, white);
    }

    /// One token off `handle`, decoded to an owned `String`; `None` is the
    /// precompiler's read failure (end of source, or an unopened handle).
    fn read_token(&mut self, handle: c_int) -> Option<String> {
        let mut token = pc_token_t {
            type_: 0,
            subtype: 0,
            intvalue: 0,
            floatvalue: 0.0,
            string: [0; MAX_TOKENLENGTH],
        };
        if PC_ReadTokenHandle(self.bot, handle, &mut token as *mut pc_token_t) == 0 {
            return None;
        }
        Some(pc_token_string(&token))
    }

    /// `trap_PC_LoadSource`'s equivalent: opens `path` on the precompiler and
    /// returns its handle, `None` for Raven's `0` (not found).
    fn load_source(&mut self, path: &str) -> Option<c_int> {
        // An interior NUL cannot reach the precompiler's `const char *`; a
        // path carrying one is treated as not found.
        let path_c = CString::new(path).ok()?;
        let handle = PC_LoadSourceHandle(self.bot, path_c.as_ptr());
        if handle == 0 {
            None
        } else {
            Some(handle)
        }
    }

    /// Consumes a balanced `{ ... }` block off `handle`, leaving the cursor
    /// just past its closing brace — what `Asset_Parse` would have consumed.
    fn skip_block(&mut self, handle: c_int) {
        let mut depth: i32 = 0;
        loop {
            let Some(token) = self.read_token(handle) else {
                return;
            };
            if token.starts_with('{') {
                depth += 1;
            } else if token.starts_with('}') {
                depth -= 1;
                if depth <= 0 {
                    return;
                }
            }
        }
    }

    /// Raven `UI_ParseMenu` — parses one menu file, returning how many
    /// `menudef` blocks it contributed.
    ///
    /// The one divergence from the oracle is `assetGlobalDef`: `Asset_Parse` is
    /// `ui_main.c`-owned (it writes `DisplayState::Assets` through a
    /// `UiContext`), so the block is consumed and logged instead of applied.
    /// The retail `ui/jampmenus.txt` set carries it only in `hud.menu`, which
    /// this harness does not open.
    ///
    /// Source: `oracle/codemp/ui/ui_main.c:1731-1776`
    fn parse_menu(&mut self, menus: &mut MenuSystem, ds: &mut DisplayState, menuFile: &str) -> u32 {
        let Some(handle) = self.load_source(menuFile) else {
            println!("ui_harness: load_menus — menu not found: {menuFile}");
            return 0;
        };

        let mut parsed = 0;
        loop {
            let Some(token) = self.read_token(handle) else {
                break;
            };

            if token.starts_with('}') {
                break;
            }

            if token.eq_ignore_ascii_case("assetGlobalDef") {
                self.stubs.hit("assetGlobalDef");
                self.skip_block(handle);
                continue;
            }

            if token.eq_ignore_ascii_case("menudef") {
                // start a new menu
                Menu_New(menus, ds, &mut *self, handle);
                parsed += 1;
            }
        }

        PC_FreeSourceHandle(self.bot, handle);
        parsed
    }

    /// Raven `UI_LoadMenus` + `Load_Menu` + `UI_ParseMenu`, replicated against
    /// the precompiler directly.
    ///
    /// The three oracle functions are `UiContext`-bound (each takes the module
    /// syscall transport to reach `trap_PC_*`), so their bodies are inlined
    /// here over `mp_engine_botlib`'s `PC_*` entry points — the same tokenizer
    /// those traps dispatch to in the live engine. Two oracle steps are
    /// deliberately absent: the `ui/jampmenus.txt` fallback + `Com_Error` pair
    /// (the harness reports and returns instead of aborting) and the `reset`
    /// parameter's `Menu_Reset` (the caller boots into a fresh `MenuSystem`).
    ///
    /// Source: `oracle/codemp/ui/ui_main.c:1805-1852` (`UI_LoadMenus`),
    /// `:1778-1803` (`Load_Menu`), `:1731-1776` (`UI_ParseMenu`)
    pub fn load_menus(&mut self, menus: &mut MenuSystem, ds: &mut DisplayState, menu_file: &str) {
        let defines = CString::new("ui/jamp/menudef.h").expect("literal has no interior NUL");
        PC_LoadGlobalDefines(self.bot, defines.as_ptr());

        let Some(handle) = self.load_source(menu_file) else {
            println!("ui_harness: load_menus — menu file not found: {menu_file} (no menus loaded)");
            PC_RemoveAllGlobalDefines(self.bot);
            return;
        };

        let mut parsed = 0;
        loop {
            let Some(token) = self.read_token(handle) else {
                break;
            };
            if token.is_empty() || token.starts_with('}') {
                break;
            }

            if !token.eq_ignore_ascii_case("loadmenu") {
                continue;
            }

            // `Load_Menu`: `{`, then one menu FILE NAME per token until `}`.
            let Some(brace) = self.read_token(handle) else {
                break;
            };
            if !brace.starts_with('{') {
                break;
            }
            let mut closed = false;
            loop {
                let Some(name) = self.read_token(handle) else {
                    break;
                };
                if name.is_empty() {
                    break;
                }
                if name.starts_with('}') {
                    closed = true;
                    break;
                }
                parsed += self.parse_menu(menus, ds, &name);
            }
            if !closed {
                // Raven's `Load_Menu` returning `qfalse` ends the outer walk.
                break;
            }
        }

        PC_FreeSourceHandle(self.bot, handle);
        PC_RemoveAllGlobalDefines(self.bot);

        println!("ui_harness: load_menus({menu_file}) — source opened, {parsed} menudefs parsed");
    }
}

/// Decodes a `pc_token_t.string` fixed buffer into an owned `String` — the
/// port's token buffer is bytes, not a Rust string (mirrors `ui_main.c`'s own
/// `pc_token_str` helper, reimplemented here because that one is private).
fn pc_token_string(token: &pc_token_t) -> String {
    buf_to_string(&token.string.iter().map(|&c| c as u8).collect::<Vec<u8>>())
}

// ============================================================================
// The trait itself. Every slot's doc comment states REAL (and what it enters)
// or STUB (and why) — see the module doc for the two stub families.
// ============================================================================

impl DisplayContext for HarnessDc<'_> {
    /// REAL — enters `mp_renderer`'s `RE_RegisterShaderNoMip`, which parses the
    /// shader off the retail pk3s and loads its images.
    fn registerShaderNoMip(&mut self, p: &str) -> qhandle_t {
        RE_RegisterShaderNoMip(
            p,
            self.qs,
            self.frame,
            self.assets,
            &mut self.view,
            &*self.cvars,
            self.sim,
            &*self.models,
            self.img_state,
            self.gpu,
            self.sky_view,
            self.sky,
        )
    }

    /// REAL — `UI_SetColor`'s `trap_R_SetColor`, as a frame event. Raven's
    /// `NULL` (reset) is white, matching `RE_SetColor`'s own default.
    /// Source: `oracle/codemp/ui/ui_atoms.c:469-471`
    fn setColor(&mut self, v: Option<vec4_t>) {
        self.frame_data
            .events
            .push(FrameEvent::SetColor(v.unwrap_or([1.0, 1.0, 1.0, 1.0])));
    }

    /// REAL — `UI_DrawHandlePic`'s body: a negative `w`/`h` flips the matching
    /// texture-coordinate pair and takes the absolute extent.
    /// Source: `oracle/codemp/ui/ui_atoms.c:400-427`
    fn drawHandlePic(&mut self, x: f32, y: f32, w: f32, h: f32, asset: qhandle_t) {
        let (w, s0, s1) = if w < 0.0 {
            // flip about vertical
            (-w, 1.0, 0.0)
        } else {
            (w, 0.0, 1.0)
        };
        let (h, t0, t1) = if h < 0.0 {
            // flip about horizontal
            (-h, 1.0, 0.0)
        } else {
            (h, 0.0, 1.0)
        };
        self.push_stretch_pic(x, y, w, h, s0, t0, s1, t1, asset);
    }

    /// REAL — a `DrawStretchPic` event. (Raven's own `ui` fills this slot and
    /// never calls it; the harness's other draw slots route through it, and it
    /// stays live for any host that does.)
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
    ) {
        self.push_stretch_pic(x, y, w, h, s1, t1, s2, t2, hShader);
    }

    /// REAL — `Text_Paint` (`adjust` is unused there too), into the real
    /// `RE_Font_DrawString`.
    /// Source: `oracle/codemp/ui/ui_main.c:1103-1130`
    fn drawText(
        &mut self,
        ds: &DisplayState,
        x: f32,
        y: f32,
        scale: f32,
        color: vec4_t,
        text: &str,
        _adjust: f32,
        limit: c_int,
        style: c_int,
        iMenuFont: c_int,
    ) {
        self.text_paint(ds, x, y, scale, color, text, limit, style, iMenuFont);
    }

    /// REAL — `Text_Width`: `MenuFontToHandle` then `RE_Font_StrLenPixels`.
    /// Source: `oracle/codemp/ui/ui_main.c:1089-1094`
    fn textWidth(&mut self, ds: &DisplayState, text: &str, scale: f32, iMenuFont: c_int) -> c_int {
        let iFontIndex = MenuFontToHandle(ds, iMenuFont);
        self.font_str_len_pixels(text, iFontIndex, scale)
    }

    /// REAL — `Text_Height`: `MenuFontToHandle` then `RE_Font_HeightPixels`
    /// (Raven ignores `text` here as well).
    /// Source: `oracle/codemp/ui/ui_main.c:1096-1101`
    fn textHeight(
        &mut self,
        ds: &DisplayState,
        _text: &str,
        scale: f32,
        iMenuFont: c_int,
    ) -> c_int {
        let iFontIndex = MenuFontToHandle(ds, iMenuFont);
        RE_Font_HeightPixels(
            self.qs,
            self.frame,
            self.assets,
            &mut self.view,
            &*self.cvars,
            self.sim,
            &*self.models,
            self.img_state,
            self.gpu,
            self.sky_view,
            self.sky,
            self.font,
            HARNESS_LANGUAGE,
            HARNESS_LANGUAGE_MODCOUNT,
            iFontIndex,
            scale,
        )
    }

    /// STUB — model registration belongs to the renderer's model pool, which
    /// this harness boots but does not paint (backend #1 has no scene path).
    fn registerModel(&mut self, _p: &str) -> qhandle_t {
        self.stubs.hit("registerModel");
        0
    }

    /// STUB — the twin of `registerModel`: no registered models, no bounds.
    fn modelBounds(&mut self, _model: qhandle_t) -> (vec3_t, vec3_t) {
        self.stubs.hit("modelBounds");
        ([0.0; 3], [0.0; 3])
    }

    /// REAL — `UI_FillRect`: colour, one white-shader quad, colour reset.
    /// Source: `oracle/codemp/ui/ui_atoms.c:436-442`
    fn fillRect(&mut self, ds: &DisplayState, x: f32, y: f32, w: f32, h: f32, color: vec4_t) {
        self.setColor(Some(color));
        let white = ds.whiteShader;
        self.push_stretch_pic(x, y, w, h, 0.0, 0.0, 0.0, 0.0, white);
        self.setColor(None);
    }

    /// REAL — `_UI_DrawRect`: colour, then the top/bottom and left/right edges,
    /// then the colour reset.
    /// Source: `oracle/codemp/ui/ui_main.c:1065-1072`
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
        self.setColor(Some(color));
        self.draw_top_bottom(ds, x, y, w, h, size);
        self.draw_sides(ds, x, y, w, h, size);
        self.setColor(None);
    }

    /// REAL — `_UI_DrawSides`.
    fn drawSides(&mut self, ds: &DisplayState, x: f32, y: f32, w: f32, h: f32, size: f32) {
        self.draw_sides(ds, x, y, w, h, size);
    }

    /// REAL — `_UI_DrawTopBottom`.
    fn drawTopBottom(&mut self, ds: &DisplayState, x: f32, y: f32, w: f32, h: f32, size: f32) {
        self.draw_top_bottom(ds, x, y, w, h, size);
    }

    /// STUB — the 3D scene path; `frame_exec` skips scene events, so recording
    /// them would only grow the stream.
    fn clearScene(&mut self) {
        self.stubs.hit("clearScene");
    }

    /// STUB — see `clearScene`.
    fn addRefEntityToScene(&mut self, _re: &refEntity_t) {
        self.stubs.hit("addRefEntityToScene");
    }

    /// STUB — see `clearScene`.
    fn renderScene(&mut self, _fd: &refdef_t) {
        self.stubs.hit("renderScene");
    }

    /// REAL — enters `mp_renderer`'s `RE_RegisterFont`, which loads the
    /// `.fontdat` and its glyph pages off the retail pk3s.
    fn RegisterFont(&mut self, fontName: &str) -> qhandle_t {
        RE_RegisterFont(
            self.qs,
            self.frame,
            self.assets,
            &mut self.view,
            &*self.cvars,
            self.sim,
            &*self.models,
            self.img_state,
            self.gpu,
            self.sky_view,
            self.sky,
            self.font,
            HARNESS_LANGUAGE,
            HARNESS_LANGUAGE_MODCOUNT,
            fontName,
        )
    }

    /// REAL — `RE_Font_StrLenPixels` against the already-resolved font index.
    fn Font_StrLenPixels(&mut self, text: &str, iFontIndex: c_int, scale: f32) -> c_int {
        self.font_str_len_pixels(text, iFontIndex, scale)
    }

    /// REAL — `RE_Font_StrLenChars`, the renderer's own character count (it
    /// walks the MBCS decode rather than counting bytes).
    fn Font_StrLenChars(&mut self, text: &str) -> c_int {
        let bytes = string_to_latin1(text);
        RE_Font_StrLenChars(self.font, HARNESS_LANGUAGE, &bytes)
    }

    /// REAL — `RE_Font_HeightPixels`.
    fn Font_HeightPixels(&mut self, iFontIndex: c_int, scale: f32) -> c_int {
        RE_Font_HeightPixels(
            self.qs,
            self.frame,
            self.assets,
            &mut self.view,
            &*self.cvars,
            self.sim,
            &*self.models,
            self.img_state,
            self.gpu,
            self.sky_view,
            self.sky,
            self.font,
            HARNESS_LANGUAGE,
            HARNESS_LANGUAGE_MODCOUNT,
            iFontIndex,
            scale,
        )
    }

    /// REAL — `RE_Font_DrawString`, straight into this frame's event stream.
    fn Font_DrawString(
        &mut self,
        ox: c_int,
        oy: c_int,
        text: &str,
        rgba: vec4_t,
        setIndex: c_int,
        iCharLimit: c_int,
        scale: f32,
    ) {
        self.font_draw_string(ox, oy, text, rgba, setIndex, iCharLimit, scale);
    }

    /// REAL — the harness runs the western language package only (see
    /// [`HARNESS_LANGUAGE`]).
    fn Language_IsAsian(&mut self) -> bool {
        false
    }

    /// REAL — the western package's answer.
    fn Language_UsesSpaces(&mut self) -> bool {
        true
    }

    /// REAL — the western branch of `AnyLanguage_ReadCharFromString`: one byte
    /// per character, never trailing punctuation. An empty string yields
    /// Raven's terminating NUL.
    fn AnyLanguage_ReadCharFromString(&mut self, psText: &[u8]) -> (u32, c_int, bool) {
        (psText.first().copied().unwrap_or(0) as u32, 1, false)
    }

    /// STUB — `UI_OwnerDraw` is `ui_main.c`-owned (it reads `UiWorld`'s server
    /// browser, player list, force config, …).
    fn ownerDrawItem(
        &mut self,
        _menus: &mut MenuSystem,
        _ds: &DisplayState,
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
        self.stubs.hit("ownerDrawItem");
    }

    /// STUB — `UI_GetValue` is `ui_main.c`-owned; zero is its "no such
    /// ownerdraw" answer.
    fn getValue(&mut self, _ownerDraw: c_int) -> f32 {
        self.stubs.hit("getValue");
        0.0
    }

    /// STUB — `UI_OwnerDrawVisible` is `ui_main.c`-owned. `true` keeps items
    /// on screen rather than blanking the menu (the visible degradation).
    fn ownerDrawVisible(&mut self, _ds: &DisplayState, _flags: c_int) -> bool {
        self.stubs.hit("ownerDrawVisible");
        true
    }

    /// STUB — `UI_RunMenuScript` is `ui_main.c`-owned. The whole remaining
    /// cursor is consumed so the caller's `String_Parse` loop terminates
    /// instead of re-offering the same token forever.
    fn runScript(&mut self, _menus: &mut MenuSystem, _ds: &DisplayState, p: &mut &str) {
        self.stubs.hit("runScript");
        *p = "";
    }

    /// STUB — `UI_DeferMenuScript` is `ui_main.c`-owned; `false` is its "not
    /// deferred" answer. Cursor consumed as in `runScript`.
    fn deferScript(&mut self, _menus: &mut MenuSystem, _ds: &DisplayState, p: &mut &str) -> bool {
        self.stubs.hit("deferScript");
        *p = "";
        false
    }

    /// STUB — `UI_GetTeamColor` is `ui_main.c`-owned (and empty in Raven, which
    /// leaves the caller's colour untouched); white is the neutral stand-in.
    fn getTeamColor(&mut self) -> vec4_t {
        self.stubs.hit("getTeamColor");
        [1.0, 1.0, 1.0, 1.0]
    }

    /// REAL — the engine's own cvar table, truncated to `bufsize` exactly as
    /// `Cvar_VariableStringBuffer`'s `Q_strncpyz` does.
    fn getCVarString(&mut self, cvar: &str, bufsize: usize) -> String {
        let bytes = string_to_latin1(Cvar_VariableString(self.view.common, cvar));
        let take = bufsize.saturating_sub(1).min(bytes.len());
        latin1_to_string(&bytes[..take])
    }

    /// REAL — the engine's own cvar table.
    fn getCVarValue(&mut self, cvar: &str) -> f32 {
        Cvar_VariableValue(self.view.common, cvar)
    }

    /// REAL — the engine's own `Cvar_Set`.
    fn setCVar(&mut self, cvar: &str, value: &str) {
        Cvar_Set(&mut self.view, cvar, value);
    }

    /// REAL — `Text_PaintWithCursor`: the string, then the cursor glyph
    /// measured to `cursorPos` and painted blinking.
    /// Source: `oracle/codemp/ui/ui_main.c:1133-1157`
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
        self.text_paint(ds, x, y, scale, color, text, limit, style, iFontIndex);

        // now print the cursor as well...
        let textLen = text.chars().count();
        let iCopyCount = if limit != 0 {
            textLen.min(limit as usize)
        } else {
            textLen
        };
        // §19: Raven's `min(iCopyCount, cursorPos)` fed a negative `cursorPos`
        // to `strncpy` as a huge size_t; clamping at 0 is the defined choice.
        let iCopyCount = iCopyCount.min(cursorPos.max(0) as usize).min(1024);

        // copy text into temp buffer for pixel measure...
        let sTemp: String = text.chars().take(iCopyCount).collect();
        let iMenuFontHandle = MenuFontToHandle(ds, iFontIndex);
        let iNextXpos = self.font_str_len_pixels(&sTemp, iMenuFontHandle, scale);

        let cursorStr = latin1_to_string(&[cursor]);
        self.text_paint(
            ds,
            x + iNextXpos as f32,
            y,
            scale,
            color,
            &cursorStr,
            limit,
            style | ITEM_TEXTSTYLE_BLINK,
            iFontIndex,
        );
    }

    /// REAL — the harness owns the overstrike flag the engine used to.
    fn setOverstrikeMode(&mut self, b: bool) {
        self.input.overstrike = b;
    }

    /// REAL — see `setOverstrikeMode`.
    fn getOverstrikeMode(&mut self) -> bool {
        self.input.overstrike
    }

    /// STUB — no sound system is booted.
    fn startLocalSound(&mut self, _sfx: sfxHandle_t, _channelNum: c_int) {
        self.stubs.hit("startLocalSound");
    }

    /// STUB — `UI_OwnerDrawHandleKey` is `ui_main.c`-owned; `false` is "key not
    /// consumed", which lets the framework's own handling continue.
    fn ownerDrawHandleKey(
        &mut self,
        _menus: &mut MenuSystem,
        _ds: &DisplayState,
        _ownerDraw: c_int,
        _flags: c_int,
        _special: &mut f32,
        _key: c_int,
    ) -> bool {
        self.stubs.hit("ownerDrawHandleKey");
        false
    }

    /// STUB — every feeder walks `UiWorld` lists (`ui_main.c`-owned); an empty
    /// feeder paints an empty list box.
    fn feederCount(
        &mut self,
        _menus: &mut MenuSystem,
        _ds: &DisplayState,
        _feederID: f32,
    ) -> c_int {
        self.stubs.hit("feederCount");
        0
    }

    /// STUB — see `feederCount`; `None` is Raven's `NULL` return.
    fn feederItemText(
        &mut self,
        _ds: &DisplayState,
        _feederID: f32,
        _index: c_int,
        _column: c_int,
    ) -> (Option<String>, qhandle_t, qhandle_t, qhandle_t) {
        self.stubs.hit("feederItemText");
        (None, 0, 0, 0)
    }

    /// STUB — see `feederCount`.
    fn feederItemImage(
        &mut self,
        _menus: &mut MenuSystem,
        _ds: &DisplayState,
        _feederID: f32,
        _index: c_int,
    ) -> qhandle_t {
        self.stubs.hit("feederItemImage");
        0
    }

    /// STUB — see `feederCount`; `false` is "selection not handled".
    fn feederSelection(
        &mut self,
        _menus: &mut MenuSystem,
        _ds: &DisplayState,
        _feederID: f32,
        _index: c_int,
        _item: Option<ItemId>,
    ) -> bool {
        self.stubs.hit("feederSelection");
        false
    }

    /// STUB — the engine's key-name table belongs to the unported client.
    fn keynumToStringBuf(&mut self, _keynum: c_int, _buflen: usize) -> String {
        self.stubs.hit("keynumToStringBuf");
        String::new()
    }

    /// STUB — the engine's binding table belongs to the unported client.
    fn getBindingBuf(&mut self, _keynum: c_int, _buflen: usize) -> String {
        self.stubs.hit("getBindingBuf");
        String::new()
    }

    /// STUB — see `getBindingBuf`.
    fn setBinding(&mut self, _keynum: c_int, _binding: &str) {
        self.stubs.hit("setBinding");
    }

    /// REAL — the engine's own command buffer.
    ///
    /// Every `exec_when` appends rather than dispatching: `Cbuf_ExecuteText`'s
    /// `EXEC_NOW` path runs `Cmd_ExecuteString`, which unwraps the
    /// `CL_GameCommand`/`SV_GameCommand` hooks this harness deliberately never
    /// installed (no client, no server). Appending keeps the text on the real
    /// buffer, where it is inspectable, and nothing in the harness pumps
    /// `Cbuf_Execute`.
    fn executeText(&mut self, _exec_when: c_int, text: &str) {
        self.stubs.hit("executeText (appended, never dispatched)");
        Cbuf_AddText(self.view.common, text);
    }

    /// REAL, degraded — reports and continues. Raven's `Com_Error` would tear
    /// the process down; the harness exists to show how far a menu gets, so an
    /// error is loud and survivable.
    fn Error(&mut self, level: c_int, error: &str) {
        eprintln!("ui_harness: DisplayContext::Error(level {level}) — {error}");
        self.stubs.hit("Error");
    }

    /// REAL — the engine's own `Com_Printf`.
    fn Print(&mut self, msg: &str) {
        com_printf(self.view.common, msg);
    }

    /// STUB — `UI_Pause` toggles the unported client's cgame pause.
    fn Pause(&mut self, _b: bool) {
        self.stubs.hit("Pause");
    }

    /// STUB — `UI_OwnerDrawWidth` is `ui_main.c`-owned; zero-width is the
    /// no-ownerdraw answer.
    fn ownerDrawWidth(
        &mut self,
        _menus: &mut MenuSystem,
        _ds: &DisplayState,
        _ownerDraw: c_int,
        _scale: f32,
    ) -> c_int {
        self.stubs.hit("ownerDrawWidth");
        0
    }

    /// STUB — no sound system is booted.
    fn registerSound(&mut self, _name: &str) -> sfxHandle_t {
        self.stubs.hit("registerSound");
        0
    }

    /// STUB — no sound system is booted.
    fn startBackgroundTrack(&mut self, _intro: &str, _loop_: &str, _bReturnWithoutStarting: bool) {
        self.stubs.hit("startBackgroundTrack");
    }

    /// STUB — no sound system is booted.
    fn stopBackgroundTrack(&mut self) {
        self.stubs.hit("stopBackgroundTrack");
    }

    /// STUB — the RoQ cinematic player belongs to the unported client; `-1` is
    /// Raven's invalid-handle return.
    fn playCinematic(&mut self, _name: &str, _x: f32, _y: f32, _w: f32, _h: f32) -> c_int {
        self.stubs.hit("playCinematic");
        -1
    }

    /// STUB — see `playCinematic`.
    fn stopCinematic(&mut self, _handle: c_int) {
        self.stubs.hit("stopCinematic");
    }

    /// STUB — see `playCinematic`.
    fn drawCinematic(&mut self, _handle: c_int, _x: f32, _y: f32, _w: f32, _h: f32) {
        self.stubs.hit("drawCinematic");
    }

    /// STUB — see `playCinematic`.
    fn runCinematicFrame(&mut self, _handle: c_int) {
        self.stubs.hit("runCinematicFrame");
    }

    /// REAL — milliseconds since boot, the harness's `Sys_Milliseconds`.
    fn Milliseconds(&mut self) -> c_int {
        self.millis
    }

    /// REAL — the engine's own `Cvar_SetValue`.
    fn setCVarValue(&mut self, cvar: &str, value: f32) {
        Cvar_SetValue(&mut self.view, cvar, value);
    }

    /// REAL — the harness's own key table (the engine's `keys[]`).
    fn Key_IsDown(&mut self, keynum: c_int) -> bool {
        self.input.down.get(&keynum).copied().unwrap_or(false)
    }

    /// REAL — the harness's own key catcher.
    fn Key_GetCatcher(&mut self) -> c_int {
        self.input.catcher
    }

    /// REAL — the harness's own key catcher.
    fn Key_SetCatcher(&mut self, catcher: c_int) {
        self.input.catcher = catcher;
    }

    /// REAL — clears the harness's key table, as `Key_ClearStates` clears
    /// `keys[]`.
    fn Key_ClearStates(&mut self) {
        self.input.down.clear();
    }

    /// REAL — the production `mp_engine_botlib` precompiler, the same tokenizer
    /// `trap_PC_ReadToken` dispatches to in the live engine.
    fn PC_ReadToken(&mut self, handle: c_int, pc_token: &mut pc_token_t) -> bool {
        PC_ReadTokenHandle(self.bot, handle, pc_token as *mut pc_token_t) != 0
    }

    /// REAL — the precompiler's own source/line report, for `PC_SourceError`'s
    /// menu-parse diagnostics.
    fn PC_SourceFileAndLine(&mut self, handle: c_int, buffer_len: usize) -> (c_int, String, c_int) {
        let mut buf = vec![0u8; buffer_len.max(1)];
        let mut line: c_int = 0;
        let status = PC_SourceFileAndLine(
            self.bot,
            handle,
            buf.as_mut_ptr() as *mut c_char,
            &mut line as *mut c_int,
        );
        // SAFETY: `PC_SourceFileAndLine` NUL-terminates `buf` on success, and
        // `buf` was zero-filled, so a failure leaves a valid empty C string.
        let filename =
            latin1_to_string(unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) }.to_bytes());
        (status, filename, line)
    }

    /// REAL — the engine's StringEd (`SE_GetString`, lazy package load via
    /// FS); `SE_Init` runs at boot. An unknown reference returns Raven's empty
    /// string, which the framework renders as-is.
    fn SP_GetStringTextString(&mut self, text: &str, _buffer_len: usize) -> Option<String> {
        Some(SE_GetString(&mut self.view, text))
    }

    /// REAL — enters `mp_renderer`'s `RE_RegisterSkin`.
    fn R_RegisterSkin(&mut self, name: &str) -> qhandle_t {
        RE_RegisterSkin(
            self.qs,
            self.frame,
            self.assets,
            &mut self.view,
            &*self.cvars,
            self.sim,
            &*self.models,
            self.img_state,
            self.gpu,
            self.sky_view,
            self.sky,
            name,
        )
    }

    /// STUB — the language enumeration lives in the unported StringEd/client
    /// pair; the harness is western-only (see [`HARNESS_LANGUAGE`]).
    fn GetLanguageName(&mut self, _languageIndex: c_int, _buffer_len: usize) -> String {
        self.stubs.hit("GetLanguageName");
        String::new()
    }

    /// STUB — the ghoul2 model preview needs the scene path backend #1 does not
    /// render. `ghoul2Ptr` is left as the caller had it (NULL), so no dangling
    /// sentinel escapes.
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
        self.stubs.hit("G2API_InitGhoul2Model");
        0
    }

    /// STUB — see `G2API_InitGhoul2Model`.
    fn G2API_SetSkin(
        &mut self,
        _ghoul2: *mut c_void,
        _modelIndex: c_int,
        _customSkin: qhandle_t,
        _renderSkin: qhandle_t,
    ) -> bool {
        self.stubs.hit("G2API_SetSkin");
        false
    }

    /// STUB — see `G2API_InitGhoul2Model`; nothing was ever allocated, so there
    /// is nothing to clean.
    fn G2API_CleanGhoul2Models(&mut self, _ghoul2Ptr: *mut *mut c_void) {
        self.stubs.hit("G2API_CleanGhoul2Models");
    }

    /// STUB — see `G2API_InitGhoul2Model`.
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
        self.stubs.hit("G2API_SetBoneAnim");
        false
    }

    /// STUB — see `G2API_InitGhoul2Model`; the empty name is what gates
    /// `ItemParse_asset_model_go`'s animation branch off.
    fn G2API_GetGLAName(
        &mut self,
        _ghoul2: *mut c_void,
        _modelIndex: c_int,
        _buffer_len: usize,
    ) -> String {
        self.stubs.hit("G2API_GetGLAName");
        String::new()
    }

    /// STUB — see `G2API_InitGhoul2Model`.
    fn G2_HaveWeGhoul2Models(&mut self, _ghoul2: *mut c_void) -> bool {
        self.stubs.hit("G2_HaveWeGhoul2Models");
        false
    }

    /// STUB — `UI_CacheSaberGlowGraphics`/`UI_SaberLoadParms` are
    /// `ui_main.c`/`ui_saber.c`-owned and write `UiWorld`'s saber state.
    fn UI_CacheSaberGlowGraphics(&mut self) {
        self.stubs.hit("UI_CacheSaberGlowGraphics");
    }

    /// STUB — see `UI_CacheSaberGlowGraphics`; the blades draw into the scene
    /// path backend #1 does not render.
    fn UI_SaberDrawBlades(
        &mut self,
        _ds: &DisplayState,
        _item: &ItemDef,
        _origin: vec3_t,
        _angles: vec3_t,
    ) {
        self.stubs.hit("UI_SaberDrawBlades");
    }

    /// STUB — the animation cache is `UiWorld`'s `BgState.bgAllAnims`
    /// (DEC-36 D5), reached only through `UiContext`. `None` is Raven's
    /// parse-failure return, which skips the `G2API_SetBoneAnim` call.
    fn UI_ParseAnimationFile(&mut self, _filename: &str, _g2anim: c_int) -> Option<animation_t> {
        self.stubs.hit("UI_ParseAnimationFile");
        None
    }

    /// STUB — the datapad-move state machine reads `UiWorld`'s
    /// `moveAnimTime`/`movesBaseAnim`; Raven's own guard makes it a no-op
    /// unless armed, which it never is here.
    fn UI_MovesDatapadAnimTick(
        &mut self,
        _ds: &DisplayState,
        _menus: &mut MenuSystem,
        _item: ItemId,
    ) {
        self.stubs.hit("UI_MovesDatapadAnimTick");
    }

    /// STUB — `uiInfo.playerSpecies` is `UiWorld`-only state; an empty list
    /// leaves the cycle item with no entries.
    fn UI_PlayerSpeciesCvarStrList(&mut self) -> Vec<(String, String)> {
        self.stubs.hit("UI_PlayerSpeciesCvarStrList");
        Vec::new()
    }

    /// STUB — `uiInfo.languageCount` is `UiWorld`-only state, and the harness
    /// enumerates no languages (see `GetLanguageName`).
    fn UI_LanguageCvarStrList(&mut self) -> Vec<(String, String)> {
        self.stubs.hit("UI_LanguageCvarStrList");
        Vec::new()
    }
}

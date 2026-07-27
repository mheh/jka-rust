//! Raven `tr_font.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_font.cpp`

// Raven-named items keep their original casing across this transcription,
// including its file-scope `#define`s/tables (`sFILENAME_THAI_*`,
// `g_SBCSOverrideLanguages`).
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use core::array::from_fn;
use core::mem::{offset_of, size_of};
use std::collections::HashMap;

use mp_engine_qcommon::common::common::com_printf;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::cvar_fns::Cvar_Set;
use mp_engine_qcommon::files_common::{FS_FCloseFile, FS_FOpenFileRead, FS_ReadFileVec};
use mp_engine_qcommon::qfiles::dfontdat_s::{dfontdat_t, GLYPH_COUNT};
use mp_engine_qcommon::qfiles::glyph_info_t::glyphInfo_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::q_string::COM_StripExtension;
use mp_qshared::shared::{fileHandle_t, MAX_QPATH};

/// Raven `sFILENAME_THAI_WIDTHS`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:75`
const sFILENAME_THAI_WIDTHS: &str = "fonts/tha_widths.dat";

/// Raven `sFILENAME_THAI_CODES`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:76`
const sFILENAME_THAI_CODES: &str = "fonts/tha_codes.dat";

/// Raven `GLYPH_MAX_KOREAN_SHADERS`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:170`
const GLYPH_MAX_KOREAN_SHADERS: i32 = 3;

/// Raven `GLYPH_MAX_TAIWANESE_SHADERS`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:171`
const GLYPH_MAX_TAIWANESE_SHADERS: i32 = 4;

/// Raven `GLYPH_MAX_JAPANESE_SHADERS`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:172`
const GLYPH_MAX_JAPANESE_SHADERS: i32 = 3;

/// Raven `GLYPH_MAX_CHINESE_SHADERS`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:173`
const GLYPH_MAX_CHINESE_SHADERS: i32 = 3;

/// Raven `GLYPH_MAX_THAI_SHADERS`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:174`
const GLYPH_MAX_THAI_SHADERS: i32 = 3;

/// Raven `GLYPH_MAX_ASIAN_SHADERS`.
///
/// Raven: this MUST equal the larger of the above defines.
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:175`
const GLYPH_MAX_ASIAN_SHADERS: usize = 4;

/// Raven `TIS_SARA_AM`.
///
/// Raven: special case letter, both a new letter and a trailing accent for
/// the prev one.
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:552`
const TIS_SARA_AM: u32 = 0xD3;

/// Raven `KSC5601_HANGUL_HIBYTE_START` — range is...
/// Source: `oracle/codemp/renderer/tr_font.cpp:256`
const KSC5601_HANGUL_HIBYTE_START: u8 = 0xB0;

/// Raven `KSC5601_HANGUL_HIBYTE_STOP` — ... inclusive.
/// Source: `oracle/codemp/renderer/tr_font.cpp:257`
const KSC5601_HANGUL_HIBYTE_STOP: u8 = 0xC8;

/// Raven `KSC5601_HANGUL_LOBYTE_LOBOUND` — range is...
/// Source: `oracle/codemp/renderer/tr_font.cpp:258`
const KSC5601_HANGUL_LOBYTE_LOBOUND: u8 = 0xA0;

/// Raven `KSC5601_HANGUL_LOBYTE_HIBOUND` — ...bounding (ie only valid in
/// between these points, but NULLs in charsets for these codes).
/// Source: `oracle/codemp/renderer/tr_font.cpp:259`
const KSC5601_HANGUL_LOBYTE_HIBOUND: u8 = 0xFF;

/// Raven `KSC5601_HANGUL_CODES_PER_ROW` — 2 more than the number of glyphs.
/// Source: `oracle/codemp/renderer/tr_font.cpp:260`
const KSC5601_HANGUL_CODES_PER_ROW: u32 = 96;

/// Raven `BIG5_HIBYTE_START0` — misc chars + level 1 hanzi (all Big5 ranges
/// inclusive).
/// Source: `oracle/codemp/renderer/tr_font.cpp:307`
const BIG5_HIBYTE_START0: u8 = 0xA1;

/// Raven `BIG5_HIBYTE_STOP0`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:308`
const BIG5_HIBYTE_STOP0: u8 = 0xC6;

/// Raven `BIG5_HIBYTE_START1` — level 2 hanzi.
/// Source: `oracle/codemp/renderer/tr_font.cpp:309`
const BIG5_HIBYTE_START1: u8 = 0xC9;

/// Raven `BIG5_HIBYTE_STOP1`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:310`
const BIG5_HIBYTE_STOP1: u8 = 0xF9;

/// Raven `BIG5_LOBYTE_LOBOUND0`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:311`
const BIG5_LOBYTE_LOBOUND0: u8 = 0x40;

/// Raven `BIG5_LOBYTE_HIBOUND0`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:312`
const BIG5_LOBYTE_HIBOUND0: u8 = 0x7E;

/// Raven `BIG5_LOBYTE_LOBOUND1`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:313`
const BIG5_LOBYTE_LOBOUND1: u8 = 0xA1;

/// Raven `BIG5_LOBYTE_HIBOUND1`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:314`
const BIG5_LOBYTE_HIBOUND1: u8 = 0xFE;

/// Raven `SHIFTJIS_HIBYTE_START0` (all Shift-JIS ranges inclusive).
/// Source: `oracle/codemp/renderer/tr_font.cpp:390`
const SHIFTJIS_HIBYTE_START0: u8 = 0x81;

/// Raven `SHIFTJIS_HIBYTE_STOP0`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:391`
const SHIFTJIS_HIBYTE_STOP0: u8 = 0x9F;

/// Raven `SHIFTJIS_HIBYTE_START1`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:392`
const SHIFTJIS_HIBYTE_START1: u8 = 0xE0;

/// Raven `SHIFTJIS_HIBYTE_STOP1`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:393`
const SHIFTJIS_HIBYTE_STOP1: u8 = 0xEF;

/// Raven `SHIFTJIS_LOBYTE_START0`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:395`
const SHIFTJIS_LOBYTE_START0: u8 = 0x40;

/// Raven `SHIFTJIS_LOBYTE_STOP0`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:396`
const SHIFTJIS_LOBYTE_STOP0: u8 = 0x7E;

/// Raven `SHIFTJIS_LOBYTE_START1`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:397`
const SHIFTJIS_LOBYTE_START1: u8 = 0x80;

/// Raven `SHIFTJIS_LOBYTE_STOP1`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:398`
const SHIFTJIS_LOBYTE_STOP1: u8 = 0xFC;

/// Raven `SHIFTJIS_CODES_PER_ROW`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:399`
const SHIFTJIS_CODES_PER_ROW: u32 =
    ((SHIFTJIS_LOBYTE_STOP0 as u32 - SHIFTJIS_LOBYTE_START0 as u32) + 1)
        + ((SHIFTJIS_LOBYTE_STOP1 as u32 - SHIFTJIS_LOBYTE_START1 as u32) + 1);

/// Raven `GB_HIBYTE_START` — range is...
/// Source: `oracle/codemp/renderer/tr_font.cpp:482`
const GB_HIBYTE_START: u8 = 0xA1;

/// Raven `GB_HIBYTE_STOP` — ... inclusive.
/// Source: `oracle/codemp/renderer/tr_font.cpp:483`
const GB_HIBYTE_STOP: u8 = 0xF7;

/// Raven `GB_LOBYTE_LOBOUND` — range is...
/// Source: `oracle/codemp/renderer/tr_font.cpp:484`
const GB_LOBYTE_LOBOUND: u8 = 0xA0;

/// Raven `GB_LOBYTE_HIBOUND` — ...bounding (ie only valid in between these
/// points, but NULLs in charsets for these codes).
/// Source: `oracle/codemp/renderer/tr_font.cpp:485`
const GB_LOBYTE_HIBOUND: u8 = 0xFF;

/// Raven `GB_CODES_PER_ROW` — 1 more than the number of glyphs.
/// Source: `oracle/codemp/renderer/tr_font.cpp:486`
const GB_CODES_PER_ROW: u32 = 95;

/// Raven `TIS_GLYPHS_START`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:551`
const TIS_GLYPHS_START: u32 = 160;

// DEFERRED: GetLanguageEnum — `se_language` (STATE HOMES row: "NOT renderer
// state — declared outside the renderer TU set... already homed by the
// engine port... confirm the exact receiver at port time") has no resolvable
// receiver from this packet's scope (engine crate not in the allowed read
// set), and its 7 callees (`Language_IsChinese`/`IsJapanese`/`IsKorean`/
// `IsPolish`/`IsRussian`/`IsTaiwanese`/`IsThai`) are inline header helpers
// with no ported equivalent and no body in this packet — porting either the
// engine-cvar-modificationCount cache (a kind-3 fn-scope-static escalation,
// R2 assigns it no carrier) or the language-string dispatch would be
// speculative behavior (porting-rules §A2). Left untranscribed; raised as a
// wave-planning gap. Its *result* is threaded in as a `Language_e` parameter
// by every method that needs it (`CFontInfo::{new, UpdateAsianIfNeeded,
// GetLetter, GetCollapsedAsianCode, GetLetterWidth, GetLetterHorizAdvance}`),
// per porting-rules §B4 — reached-for ambient language state is what stays
// unported, not the language itself.
// Source: `oracle/codemp/renderer/tr_font.cpp:31-53`

/// Raven `Language_e`.
///
/// Type definition source: `oracle/codemp/renderer/tr_font.cpp:17-27`
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Language_e {
    /// ( I only care about asian languages in here at the moment )
    #[default]
    eWestern,
    /// .. but now I need to care about this, since it uses a different TP
    eRussian,
    /// ditto
    ePolish,
    eKorean,
    /// 15x15 glyphs tucked against BR of 16x16 space
    eTaiwanese,
    /// 15x15 glyphs tucked against TL of 16x16 space
    eJapanese,
    /// 15x15 glyphs tucked against TL of 16x16 space
    eChinese,
    /// 16x16 cells with glyphs against left edge, special file
    /// (tha_widths.dat) for variable widths
    eThai,
}

/// Raven `SBCSOverrideLanguages_t`.
///
/// Type definition source: `oracle/codemp/renderer/tr_font.cpp:55-59`
pub struct SBCSOverrideLanguages_t {
    pub m_psName: &'static str,
    pub m_eLanguage: Language_e,
}

/// Raven `g_SBCSOverrideLanguages`.
///
/// Raven: so I can do some stuff with for-next loops when I add polish etc...
///
/// PORT-NOTE: Raven's trailing `{NULL, eWestern}` row is the C loop
/// terminator; a Rust slice carries its own length, so the sentinel is
/// dropped (porting-rules §C10).
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:63-68`
const g_SBCSOverrideLanguages: [SBCSOverrideLanguages_t; 2] = [
    SBCSOverrideLanguages_t {
        m_psName: "russian",
        m_eLanguage: Language_e::eRussian,
    },
    SBCSOverrideLanguages_t {
        m_psName: "polish",
        m_eLanguage: Language_e::ePolish,
    },
];

/// Raven `ThaiCodes_t` — the Thai-language valid-code table + per-glyph
/// widths.
///
/// Type definition source: `oracle/codemp/renderer/tr_font.cpp:77-167`
#[derive(Default)]
pub struct ThaiCodes_t {
    m_mapValidCodes: HashMap<i32, i32>,
    m_viGlyphWidths: Vec<i32>,
    /// If blank, never failed; else says don't bother re-trying.
    m_strInitFailureReason: String,
}

impl ThaiCodes_t {
    /// Raven `ThaiCodes_t::ThaiCodes_t` (constructor).
    /// Source: `oracle/codemp/renderer/tr_font.cpp:91-94`
    pub fn new() -> ThaiCodes_t {
        let mut codes = ThaiCodes_t::default();
        codes.Clear();
        codes
    }

    /// Raven `ThaiCodes_t::Clear`.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:84-89`
    pub fn Clear(&mut self) {
        self.m_mapValidCodes.clear();
        self.m_viGlyphWidths.clear();
        // if blank, never failed, else says don't bother re-trying
        self.m_strInitFailureReason.clear();
    }

    /// Raven `ThaiCodes_t::GetValidIndex`.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:98-107`
    pub fn GetValidIndex(&self, iCode: i32) -> i32 {
        match self.m_mapValidCodes.get(&iCode) {
            Some(&index) => index,
            None => -1,
        }
    }

    /// Raven `ThaiCodes_t::GetWidth`.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:109-118`
    pub fn GetWidth(&self, iGlyphIndex: i32) -> i32 {
        if let Ok(index) = usize::try_from(iGlyphIndex) {
            if let Some(&width) = self.m_viGlyphWidths.get(index) {
                return width;
            }
        }

        debug_assert!(false);
        0
    }

    /// Raven `ThaiCodes_t::Init`.
    ///
    /// Raven: return is error message to display, or NULL for success.
    ///
    /// PORT-NOTE: the two `FS_ReadFile`/`FS_FreeFile` pairs collapse into
    /// [`FS_ReadFileVec`]'s owned bytes (the engine's block is copied out and
    /// freed there), and the `int *piData` `[]`-access becomes explicit
    /// little-endian 4-byte reads over those bytes — no `int` view is ever
    /// cast over the file image. Raven's `const char *` return (a pointer
    /// into `m_strInitFailureReason`) becomes an owned `String`, empty for
    /// success, so the caller never holds a borrow of this object.
    ///
    /// Source: `oracle/codemp/renderer/tr_font.cpp:120-166`
    pub fn Init(&mut self, view: &mut EngineHostView) -> String {
        if self.m_mapValidCodes.is_empty() && self.m_viGlyphWidths.is_empty() {
            // never tried and failed already?
            if self.m_strInitFailureReason.is_empty() {
                // read the valid-codes table in...
                //
                let piData = FS_ReadFileVec(view, sFILENAME_THAI_CODES);
                let iBytesRead = match &piData {
                    Some(buf) => buf.len() as i32,
                    None => -1,
                };
                // valid length and multiple of 4 bytes long
                if iBytesRead > 0 && (iBytesRead & 3) == 0 {
                    let piData = piData.unwrap_or_default();
                    let iTableEntries = iBytesRead / size_of::<i32>() as i32;

                    for i in 0..iTableEntries {
                        // convert MBCS code to sequential index...
                        self.m_mapValidCodes
                            .insert(read_i32(&piData, i as usize * size_of::<i32>()), i);
                    }

                    // now read in the widths... (I'll keep these in a simple STL vector, so they'all disappear when the <map> entries do...
                    //
                    let piData = FS_ReadFileVec(view, sFILENAME_THAI_WIDTHS);
                    let iBytesRead = match &piData {
                        Some(buf) => buf.len() as i32,
                        None => -1,
                    };
                    if iBytesRead > 0 && (iBytesRead & 3) == 0 && (iBytesRead >> 2) == iTableEntries
                    {
                        let piData = piData.unwrap_or_default();
                        for i in 0..iTableEntries {
                            self.m_viGlyphWidths
                                .push(read_i32(&piData, i as usize * size_of::<i32>()));
                        }
                    } else {
                        self.m_strInitFailureReason = format!(
                            "Error with file \"{sFILENAME_THAI_WIDTHS}\", size = {iBytesRead}!\n"
                        );
                    }
                } else {
                    self.m_strInitFailureReason = format!(
                        "Error with file \"{sFILENAME_THAI_CODES}\", size = {iBytesRead}!\n"
                    );
                }
            }
        }

        self.m_strInitFailureReason.clone()
    }
}

/// Raven `CFontInfo` — a single loaded/scaled bitmap-font's metrics and
/// Asian-glyph shader set.
///
/// Type definition source: `oracle/codemp/renderer/tr_font.cpp:177-231`
///
/// PORT-NOTE: Raven's `~CFontInfo(void) {}` (`tr_font.cpp:213`) is an empty
/// destructor — Rust's default drop glue over owned fields is already the
/// identical no-op, so no `Drop` impl is written for it.
pub struct CFontInfo {
    // From the fontdat file
    /// Raven `glyphInfo_t mGlyphs[GLYPH_COUNT]`.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:181`
    mGlyphs: [glyphInfo_t; GLYPH_COUNT],
    // `int mAsianHack` (`:183`) is commented out in Raven — "unused junk from
    // John's fontdat file format" — so it has no field here.
    // end of fontdat data
    /// Raven: handle to the shader with the glyph.
    ///
    /// Raw shader index (Raven `qhandle_t`) — kept as a plain index per the
    /// interior-safety law, not wrapped in `Handle<ShaderAsset>`: slot 0
    /// there is a *valid* default-shader handle (A12), not this class's "no
    /// shader" sentinel, so the two zero conventions don't line up without a
    /// design decision this packet doesn't make.
    ///
    /// Source: `oracle/codemp/renderer/tr_font.cpp:186`
    mShader: i32,
    /// Raven: shaders for Korean glyphs where applicable.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:188`
    m_hAsianShaders: [i32; GLYPH_MAX_ASIAN_SHADERS],
    /// Raven: special glyph containing asian->western scaling info for all
    /// glyphs.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:189`
    m_AsianGlyph: glyphInfo_t,
    /// Raven: needed to dynamically calculate S,T coords.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:190`
    m_iAsianGlyphsAcross: i32,
    /// Source: `oracle/codemp/renderer/tr_font.cpp:191`
    m_iAsianPagesLoaded: i32,
    /// Source: `oracle/codemp/renderer/tr_font.cpp:192`
    m_bAsianLastPageHalfHeight: bool,
    /// Raven: doesn't matter what this is, so long as it's comparable as
    /// being changed.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:193`
    m_iLanguageModificationCount: i32,
    /// Raven `ThaiCodes_t *m_pThaiData` — never dereferenced anywhere in
    /// Raven: it only ever holds `NULL` or `&g_ThaiCodes` and is read as
    /// "have the Thai tables been hooked up yet" (`:987-992`), while every
    /// actual width lookup goes through `g_ThaiCodes` directly (`:1156`). A
    /// `bool` carries that per the interior-safety law (no raw pointers).
    /// Source: `oracle/codemp/renderer/tr_font.cpp:195`
    m_bThaiData: bool,

    /// Raven: eg "fonts/lcd" // needed for korean font-hint if we need >1
    /// hangul set.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:198`
    pub m_sFontName: String,
    /// Source: `oracle/codemp/renderer/tr_font.cpp:199`
    pub mPointSize: i32,
    /// Source: `oracle/codemp/renderer/tr_font.cpp:200`
    pub mHeight: i32,
    /// Source: `oracle/codemp/renderer/tr_font.cpp:201`
    pub mAscender: i32,
    /// Source: `oracle/codemp/renderer/tr_font.cpp:202`
    pub mDescender: i32,

    /// Raven: trying to make this !@#$%^ thing work with scaling.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:204`
    pub mbRoundCalcs: bool,
    /// Raven: handle to itself.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:205`
    pub m_iThisFont: i32,
    /// Raven: -1 == NULL // alternative single-byte font for languages like
    /// russian/polish etc that need to override high characters ?
    /// Source: `oracle/codemp/renderer/tr_font.cpp:206`
    pub m_iAltSBCSFont: i32,
    /// Source: `oracle/codemp/renderer/tr_font.cpp:207`
    pub m_iOriginalFontWhenSBCSOverriden: i32,
    /// Raven: -1, else amount to adjust returned values by to make them fit
    /// the master western font they're substituting for.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:208`
    pub m_fAltSBCSFontScaleFactor: f32,
    /// Raven: ... if true, don't process as MBCS or override as SBCS etc.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:209`
    pub m_bIsFakeAlienLanguage: bool,
}

impl CFontInfo {
    /// Raven `CFontInfo::CFontInfo` (constructor).
    ///
    /// Raven: name is (eg) "ergo" or "lcd", no extension.
    ///
    /// Raven: If path present, it's a special language hack for SBCS override
    /// languages, eg: "lcd/russian", which means just treat the file as
    /// "russian", but with the "lcd" part ensuring we don't find a different
    /// registered russian font.
    ///
    /// PORT-NOTE: Raven's constructor files `this` into `g_vFontArray` at
    /// `g_iCurrentFontIndex++` as its "finished..." step (`:882-884`); a Rust
    /// value cannot move itself into the arena it is being built for, so this
    /// fn performs that insert itself and returns the index it registered at
    /// — arena + id per porting-rules §B5, and the same number
    /// `RE_RegisterFont` reads back as `g_iCurrentFontIndex - 1` (`:1629`).
    /// The insert consequently runs *after* the `com_buildScript == 2` block
    /// (`:887-955`) instead of before it; nothing in that block reads
    /// `g_vFontArray`/`g_iCurrentFontIndex`, so the reorder is unobservable.
    ///
    /// PORT-NOTE: `GetLanguageEnum()` and `se_language->modificationCount`
    /// are unported (file-head DEFERRED, `:31-53`), so both are threaded in
    /// as parameters (porting-rules §B4).
    ///
    /// Source: `oracle/codemp/renderer/tr_font.cpp:815-956`
    pub fn new(
        view: &mut EngineHostView,
        font: &mut FontState,
        eLanguage: Language_e,
        iSE_Language_ModificationCount: i32,
        _fontName: &str,
    ) -> i32 {
        // remove any special hack name insertions...
        //
        // Raven's `COM_SkipPath` (`oracle/codemp/game/q_shared.c:80-92`)
        // returns the tail past the last '/'; the workspace's ported twin
        // takes a `*mut c_char`, which the interior-safety law bars here, so
        // the two-line scan is spelled out over `&str`.
        let psSkipPath = match _fontName.rsplit_once('/') {
            Some((_, tail)) => tail,
            None => _fontName,
        };
        // Raven's `sprintf` into `char fontName[MAX_QPATH]` overruns for a
        // long name (UB); the owned `String` cannot (porting-rules §19).
        let fontName = format!("fonts/{psSkipPath}.fontdat");

        let mut me = CFontInfo {
            // Raven's `new CFontInfo` leaves these indeterminate until the
            // fontdat read fills them, and the failure path below then reads
            // `GetPointSize()` (`:1630`) off that indeterminate memory (UB).
            // Zero is the one defined behavior chosen (porting-rules §19): a
            // failed load reads back point size 0 = "missing/invalid".
            mGlyphs: from_fn(|_| glyph_zero()),
            mShader: 0,
            m_hAsianShaders: [0; GLYPH_MAX_ASIAN_SHADERS],
            m_AsianGlyph: glyph_zero(),
            m_iAsianGlyphsAcross: 0,
            m_iAsianPagesLoaded: 0,
            m_bAsianLastPageHalfHeight: false,
            m_iLanguageModificationCount: 0,
            m_sFontName: String::new(),
            mPointSize: 0,
            mHeight: 0,
            mAscender: 0,
            mDescender: 0,
            mbRoundCalcs: false,
            // clear some general things...
            m_bThaiData: false,
            m_iAltSBCSFont: -1,
            m_iThisFont: -1,
            m_iOriginalFontWhenSBCSOverriden: -1,
            m_fAltSBCSFontScaleFactor: -1.0,
            // dont try and make SBCS or asian overrides for this
            m_bIsFakeAlienLanguage: _fontName == "aurabesh",
        };

        // Raven's `FS_ReadFile(fontName, NULL)` length probe plus its second
        // `FS_ReadFile(fontName, &buff)` collapse into one owned read; the
        // `FS_FreeFile(buff)` (`:867`) is that `Vec`'s drop.
        let buff = FS_ReadFileVec(view, &fontName);
        match &buff {
            Some(buff) if buff.len() == size_of::<dfontdat_t>() => {
                // The `.fontdat` image is parsed field by field, little-endian,
                // never cast over `dfontdat_t` (interior-safety law).
                for i in 0..GLYPH_COUNT {
                    me.mGlyphs[i] = read_glyph(buff, i * size_of::<glyphInfo_t>());
                }
                me.mPointSize = read_i16(buff, offset_of!(dfontdat_t, mPointSize)) as i32;
                me.mHeight = read_i16(buff, offset_of!(dfontdat_t, mHeight)) as i32;
                me.mAscender = read_i16(buff, offset_of!(dfontdat_t, mAscender)) as i32;
                me.mDescender = read_i16(buff, offset_of!(dfontdat_t, mDescender)) as i32;
                // mAsianHack = fontdat->mKoreanHack; // ignore this crap, it's some junk in the fontdat file that no-one uses
                me.mbRoundCalcs = fontName.contains("ergo");

                // cope with bad fontdat headers...
                //
                if me.mHeight == 0 {
                    me.mHeight = me.mPointSize;
                    // have to completely guess at the baseline... sigh.
                    me.mAscender = me.mPointSize - Round(me.mPointSize as f32 / 10.0f32 + 2.0f32);
                    me.mDescender = me.mHeight - me.mAscender;
                }
            }
            _ => {
                me.mHeight = 0;
                me.mShader = 0;
            }
        }

        // Q_strncpyz(m_sFontName, fontName, sizeof(m_sFontName))
        me.m_sFontName = fontName.clone();
        truncate_to_qpath(&mut me.m_sFontName);
        // so we get better error printing if failed to load shader (ie lose ".fontdat")
        me.m_sFontName = COM_StripExtension(&me.m_sFontName);
        // DEFERRED: RE_RegisterShaderNoMip — this crate has no ported shader
        // registration entry point yet (`tr_shader.rs` wave 0 landed the
        // parser, not the registry seam), so the handle stays 0, Raven's own
        // "no shader" value.
        // Source: `oracle/codemp/renderer/tr_font.cpp:877`
        me.mShader = 0;

        me.FlagNoAsianGlyphs();
        me.UpdateAsianIfNeeded(view, font, eLanguage, iSE_Language_ModificationCount, true);

        if view.common.com_buildScript.is_some()
            && view.common.cvar(view.common.com_buildScript).integer == 2
        {
            com_printf(
                view.common,
                "com_buildScript(2): Registering foreign fonts...\n",
            );
            // Do this once only (for speed)...
            if !font.bDone_ForeignFontsRegistered {
                font.bDone_ForeignFontsRegistered = true;

                // SBCS override languages...
                //
                let mut f: fileHandle_t = 0;
                for entry in g_SBCSOverrideLanguages.iter() {
                    let sTemp = format!("fonts/{}.tga", entry.m_psName);
                    FS_FOpenFileRead(view, &sTemp, &mut f as *mut fileHandle_t, false);
                    if f != 0 {
                        FS_FCloseFile(view.common, f);
                    }

                    let sTemp = format!("fonts/{}.fontdat", entry.m_psName);
                    FS_FOpenFileRead(view, &sTemp, &mut f as *mut fileHandle_t, false);
                    if f != 0 {
                        FS_FCloseFile(view.common, f);
                    }
                }

                // asian MBCS override languages...
                //
                for iLang in 0..5 {
                    let fields = match iLang {
                        0 => Korean_InitFields(font),
                        1 => Taiwanese_InitFields(font),
                        2 => Japanese_InitFields(font),
                        3 => Chinese_InitFields(font),
                        _ => {
                            let fields = Thai_InitFields(font);
                            // additional files needed for Thai language...
                            //
                            FS_FOpenFileRead(
                                view,
                                sFILENAME_THAI_WIDTHS,
                                &mut f as *mut fileHandle_t,
                                false,
                            );
                            if f != 0 {
                                FS_FCloseFile(view.common, f);
                            }

                            FS_FOpenFileRead(
                                view,
                                sFILENAME_THAI_CODES,
                                &mut f as *mut fileHandle_t,
                                false,
                            );
                            if f != 0 {
                                FS_FCloseFile(view.common, f);
                            }
                            fields
                        }
                    };
                    me.m_iAsianGlyphsAcross = fields.m_iAsianGlyphsAcross;

                    for i in 0..fields.iGlyphTPs {
                        let sTemp = format!(
                            "fonts/{}_{}_1024_{}.tga",
                            fields.psLang,
                            1024 / me.m_iAsianGlyphsAcross,
                            i
                        );

                        // RE_RegisterShaderNoMip( sTemp );	// don't actually need to load it, so...
                        FS_FOpenFileRead(view, &sTemp, &mut f as *mut fileHandle_t, false);
                        if f != 0 {
                            FS_FCloseFile(view.common, f);
                        }
                    }
                }
            }
        }

        // finished...
        let iThisFont = font.g_iCurrentFontIndex;
        font.g_vFontArray
            .resize_with((iThisFont + 1).max(0) as usize, || None);
        font.g_vFontArray[iThisFont as usize] = Some(Box::new(me));
        font.g_iCurrentFontIndex += 1;

        iThisFont
    }

    /// Raven `CFontInfo::GetPointSize`.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:215`
    pub fn GetPointSize(&self) -> i32 {
        self.mPointSize
    }

    /// Raven `CFontInfo::GetHeight`.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:216`
    pub fn GetHeight(&self) -> i32 {
        self.mHeight
    }

    /// Raven `CFontInfo::GetAscender`.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:217`
    pub fn GetAscender(&self) -> i32 {
        self.mAscender
    }

    /// Raven `CFontInfo::GetDescender`.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:218`
    pub fn GetDescender(&self) -> i32 {
        self.mDescender
    }

    /// Raven `CFontInfo::GetShader`.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:225`
    pub fn GetShader(&self) -> i32 {
        self.mShader
    }

    /// Raven `CFontInfo::FlagNoAsianGlyphs` — used during constructor.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:227`
    pub fn FlagNoAsianGlyphs(&mut self) {
        self.m_hAsianShaders[0] = 0;
        self.m_iLanguageModificationCount = -1;
    }

    /// Raven `CFontInfo::AsianGlyphsAvailable`.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:228`
    pub fn AsianGlyphsAvailable(&self) -> bool {
        self.m_hAsianShaders[0] != 0
    }

    /// Raven `CFontInfo::UpdateAsianIfNeeded`.
    ///
    /// PORT-NOTE: Raven's `GetLanguageEnum()` and `se_language->
    /// modificationCount` reads (`:963,970-972`) are threaded in as the
    /// `eLanguage`/`iSE_Language_ModificationCount` params — see the
    /// file-head DEFERRED (`:31-53`).
    ///
    /// Source: `oracle/codemp/renderer/tr_font.cpp:958-1069`
    pub fn UpdateAsianIfNeeded(
        &mut self,
        view: &mut EngineHostView,
        font: &mut FontState,
        eLanguage: Language_e,
        iSE_Language_ModificationCount: i32,
        mut bForceReEval: bool,
    ) {
        // if asian language, then provide an alternative glyph set and fill in relevant fields...
        //
        // western charset exists in first place, and isn't alien rubbish?
        if self.mHeight != 0 && !self.m_bIsFakeAlienLanguage {
            if matches!(
                eLanguage,
                Language_e::eKorean
                    | Language_e::eTaiwanese
                    | Language_e::eJapanese
                    | Language_e::eChinese
                    | Language_e::eThai
            ) {
                // arbitrary limit on small char sizes because Asian chars don't squash well
                let iCappedHeight = if self.mHeight < 16 { 16 } else { self.mHeight };

                if self.m_iLanguageModificationCount != iSE_Language_ModificationCount
                    || !self.AsianGlyphsAvailable()
                    || bForceReEval
                {
                    self.m_iLanguageModificationCount = iSE_Language_ModificationCount;

                    let fields = match eLanguage {
                        Language_e::eKorean => Korean_InitFields(font),
                        Language_e::eTaiwanese => Taiwanese_InitFields(font),
                        Language_e::eJapanese => Japanese_InitFields(font),
                        Language_e::eChinese => Chinese_InitFields(font),
                        Language_e::eThai => {
                            let fields = Thai_InitFields(font);

                            if !self.m_bThaiData {
                                let psFailureReason = font.g_ThaiCodes.Init(view);
                                if psFailureReason.is_empty() {
                                    self.m_bThaiData = true;
                                } else {
                                    // failed to load a needed file, reset to English...
                                    //
                                    Cvar_Set(view, "se_language", "english");
                                    com_error(errorParm_t::ERR_DROP, psFailureReason);
                                }
                            }

                            fields
                        }
                        // Raven's `switch` has no `default` arm; the enclosing
                        // `if` already restricts this to the five Asian
                        // languages, so this arm mirrors the untouched C
                        // locals (`iGlyphTPs = 0`, `psLang = NULL`) and leaves
                        // `m_iAsianGlyphsAcross` as it was.
                        _ => LanguageFontFields {
                            m_iAsianGlyphsAcross: self.m_iAsianGlyphsAcross,
                            iGlyphTPs: 0,
                            psLang: "",
                        },
                    };
                    self.m_iAsianGlyphsAcross = fields.m_iAsianGlyphsAcross;

                    // textures need loading...
                    //
                    if !self.m_sFontName.is_empty() {
                        // Use this sometime if we need to do logic to load alternate-height glyphs to better fit other fonts.
                        // (but for now, we just use the one glyph set)
                        //
                    }

                    for i in 0..fields.iGlyphTPs as usize {
                        // (Note!!  assumption for S,T calculations: all Asian glyph textures pages are square except for last one)
                        //
                        // DEFERRED: RE_RegisterShaderNoMip, together with the
                        // `Com_sprintf(sTemp, "fonts/%s_%d_1024_%d", psLang,
                        // 1024/m_iAsianGlyphsAcross, i)` name build that feeds
                        // it — no ported shader-registration entry point in
                        // this crate yet. Raven's own comment below states
                        // what the 0 left here then means.
                        // Source: `oracle/codemp/renderer/tr_font.cpp:1013-1017`
                        //
                        // returning 0 here will automatically inhibit Asian glyph calculations at runtime...
                        //
                        self.m_hAsianShaders[i] = 0;
                    }

                    // for now I'm hardwiring these, but if we ever have more than one glyph set per language then they'll be changed...
                    //
                    // not necessarily true, but will be safe, and show up obvious if something missing
                    self.m_iAsianPagesLoaded = fields.iGlyphTPs;
                    self.m_bAsianLastPageHalfHeight = true;

                    bForceReEval = true;
                }

                if bForceReEval {
                    // now init the Asian member glyph fields to make them come out the same size as the western ones
                    //	that they serve as an alternative for...
                    //
                    // square Asian chars same size as height of western set
                    self.m_AsianGlyph.width = iCappedHeight as i16;
                    // ""
                    self.m_AsianGlyph.height = iCappedHeight as i16;
                    match eLanguage {
                        // korean has a small amount of space at the edge of the glyph
                        Language_e::eKorean => {
                            self.m_AsianGlyph.horizAdvance = (iCappedHeight - 1) as i16
                        }
                        // need to force some spacing for these
                        Language_e::eTaiwanese | Language_e::eJapanese | Language_e::eChinese => {
                            self.m_AsianGlyph.horizAdvance = (iCappedHeight + 3) as i16
                        }
                        // eThai: this is done dynamically elsewhere, since Thai glyphs are variable width
                        _ => self.m_AsianGlyph.horizAdvance = iCappedHeight as i16,
                    }
                    // ""
                    self.m_AsianGlyph.horizOffset = 0;
                    self.m_AsianGlyph.baseline =
                        self.mAscender + ((iCappedHeight - self.mHeight) >> 1);
                }
            } else {
                // not using Asian...
                //
                self.FlagNoAsianGlyphs();
            }
        } else {
            // no western glyphs available, so don't attempt to match asian...
            //
            self.FlagNoAsianGlyphs();
        }
    }

    /// Raven `CFontInfo::GetLetter`.
    ///
    /// Raven: needed to add *piShader param because of multiple TPs, if not
    /// passed in, then I also skip S,T calculations for re-usable static
    /// asian glyphinfo struct...
    ///
    /// PORT-NOTE: Raven's optional `int *piShader` out-param becomes the
    /// `bWantShader` request flag plus the returned `Option<i32>` (a `None`
    /// return is Raven's `piShader == NULL`, which also skips the S,T work).
    /// The returned glyph is an owned copy of what Raven hands back a pointer
    /// to (`&mGlyphs[..]` or `&m_AsianGlyph`) — every caller only reads
    /// fields off it, and `glyphInfo_t` is plain scalars.
    ///
    /// Source: `oracle/codemp/renderer/tr_font.cpp:1089-1214`
    pub fn GetLetter(
        &mut self,
        font: &FontState,
        eLanguage: Language_e,
        uiLetter: u32,
        bWantShader: bool,
    ) -> (glyphInfo_t, Option<i32>) {
        if self.AsianGlyphsAvailable() {
            let mut iCollapsedAsianCode = self.GetCollapsedAsianCode(font, eLanguage, uiLetter);
            if iCollapsedAsianCode != 0 {
                let mut hShader = None;
                if bWantShader {
                    // (Note!!  assumption for S,T calculations: all asian glyph textures pages are square except for last one
                    //			which may or may not be half height) - but not for Thai
                    //
                    let mut iTexturePageIndex = iCollapsedAsianCode
                        / (self.m_iAsianGlyphsAcross * self.m_iAsianGlyphsAcross);

                    if iTexturePageIndex > self.m_iAsianPagesLoaded {
                        // should never happen
                        debug_assert!(false);
                        iTexturePageIndex = 0;
                    }

                    // need to back this up (if Thai) for later
                    let iOriginalCollapsedAsianCode = iCollapsedAsianCode;
                    iCollapsedAsianCode -=
                        iTexturePageIndex * (self.m_iAsianGlyphsAcross * self.m_iAsianGlyphsAcross);

                    let iColumn = iCollapsedAsianCode % self.m_iAsianGlyphsAcross;
                    let iRow = iCollapsedAsianCode / self.m_iAsianGlyphsAcross;
                    let bHalfT = iTexturePageIndex == (self.m_iAsianPagesLoaded - 1)
                        && self.m_bAsianLastPageHalfHeight;
                    let iAsianGlyphsDown = if bHalfT {
                        self.m_iAsianGlyphsAcross / 2
                    } else {
                        self.m_iAsianGlyphsAcross
                    };

                    match eLanguage {
                        Language_e::eTaiwanese => {
                            self.m_AsianGlyph.s = (((1024 / self.m_iAsianGlyphsAcross) * iColumn)
                                + 1) as f32
                                / 1024.0f32;
                            self.m_AsianGlyph.t =
                                (((1024 / iAsianGlyphsDown) * iRow) + 1) as f32 / 1024.0f32;
                            self.m_AsianGlyph.s2 =
                                ((1024 / self.m_iAsianGlyphsAcross) * (iColumn + 1)) as f32
                                    / 1024.0f32;
                            self.m_AsianGlyph.t2 =
                                ((1024 / iAsianGlyphsDown) * (iRow + 1)) as f32 / 1024.0f32;
                        }
                        Language_e::eJapanese | Language_e::eChinese => {
                            self.m_AsianGlyph.s =
                                ((1024 / self.m_iAsianGlyphsAcross) * iColumn) as f32 / 1024.0f32;
                            self.m_AsianGlyph.t =
                                ((1024 / iAsianGlyphsDown) * iRow) as f32 / 1024.0f32;
                            self.m_AsianGlyph.s2 =
                                (((1024 / self.m_iAsianGlyphsAcross) * (iColumn + 1)) - 1) as f32
                                    / 1024.0f32;
                            self.m_AsianGlyph.t2 =
                                (((1024 / iAsianGlyphsDown) * (iRow + 1)) - 1) as f32 / 1024.0f32;
                        }
                        Language_e::eThai => {
                            let mut iGlyphXpos = (1024 / self.m_iAsianGlyphsAcross) * iColumn;
                            let mut iGlyphWidth =
                                font.g_ThaiCodes.GetWidth(iOriginalCollapsedAsianCode);

                            // very thai-specific language-code...
                            //
                            if uiLetter == TIS_SARA_AM {
                                // these are pixel coords on the source TP, so don't affect scaled output
                                iGlyphXpos += 9;
                                iGlyphWidth = 20;
                            }
                            self.m_AsianGlyph.s = iGlyphXpos as f32 / 1024.0f32;
                            self.m_AsianGlyph.t =
                                ((1024 / iAsianGlyphsDown) * iRow) as f32 / 1024.0f32;
                            // technically this .s2 line should be modified to blit only the correct width, but since
                            //	all Thai glyphs are up against the left edge of their cells and have blank to the cell
                            //	boundary then it's better to keep these calculations simpler...
                            self.m_AsianGlyph.s2 = (iGlyphXpos + iGlyphWidth) as f32 / 1024.0f32;
                            self.m_AsianGlyph.t2 =
                                (((1024 / iAsianGlyphsDown) * (iRow + 1)) - 1) as f32 / 1024.0f32;

                            // special addition for Thai, need to bodge up the width and advance fields...
                            //
                            self.m_AsianGlyph.width = iGlyphWidth as i16;
                            self.m_AsianGlyph.horizAdvance = (iGlyphWidth + 1) as i16;
                        }
                        // eKorean and Raven's `default`
                        _ => {
                            self.m_AsianGlyph.s = iColumn as f32 / self.m_iAsianGlyphsAcross as f32;
                            self.m_AsianGlyph.t = iRow as f32 / iAsianGlyphsDown as f32;
                            self.m_AsianGlyph.s2 =
                                (iColumn + 1) as f32 / self.m_iAsianGlyphsAcross as f32;
                            self.m_AsianGlyph.t2 = (iRow + 1) as f32 / iAsianGlyphsDown as f32;
                        }
                    }
                    hShader = Some(self.m_hAsianShaders[iTexturePageIndex as usize]);
                }
                return (glyph_copy(&self.m_AsianGlyph), hShader);
            }
        }

        let mut hShader = None;
        if bWantShader {
            hShader = Some(self.GetShader());
        }

        let pGlyph = glyph_copy(&self.mGlyphs[(uiLetter & 0xff) as usize]);
        //
        // SBCS language substitution?...
        //
        if self.m_fAltSBCSFontScaleFactor != -1.0 {
            // sod it, use the asian glyph, that's fine...
            //
            // *before* changin pGlyph!
            self.m_AsianGlyph = glyph_copy(&pGlyph);

            let f = self.m_fAltSBCSFontScaleFactor;
            let bRound = self.mbRoundCalcs;
            self.m_AsianGlyph.baseline = assign_with_rounding(bRound, f, pGlyph.baseline as f32);
            self.m_AsianGlyph.height = assign_with_rounding(bRound, f, pGlyph.height as f32) as i16;
            self.m_AsianGlyph.horizAdvance =
                assign_with_rounding(bRound, f, pGlyph.horizAdvance as f32) as i16;
            // m_AsianGlyph.horizOffset = /*Round*/( m_fAltSBCSFontScaleFactor * pGlyph->horizOffset );
            self.m_AsianGlyph.width = assign_with_rounding(bRound, f, pGlyph.width as f32) as i16;

            return (glyph_copy(&self.m_AsianGlyph), hShader);
        }

        (pGlyph, hShader)
    }

    /// Raven `CFontInfo::GetCollapsedAsianCode`.
    ///
    /// Source: `oracle/codemp/renderer/tr_font.cpp:1217-1235`
    pub fn GetCollapsedAsianCode(
        &self,
        font: &FontState,
        eLanguage: Language_e,
        uiLetter: u32,
    ) -> i32 {
        let mut iCollapsedAsianCode = 0;

        if self.AsianGlyphsAvailable() {
            match eLanguage {
                // DEFERRED: Korean_CollapseKSC5601HangulCode (`:284-293`),
                // Taiwanese_CollapseBig5Code (`:362-375`),
                // Japanese_CollapseShiftJISCode (`:448-467`) and
                // Chinese_CollapseGBCode (`:527-537`) are untranscribed (tr_font
                // wave 0 landed the `*_Valid*`/`*_InitFields` half of each
                // language block only). These arms yield Raven's own "not a
                // valid Asian code" 0 until the collapse maths is transcribed.
                // Source: `oracle/codemp/renderer/tr_font.cpp:284-293,362-375,448-467,527-537`
                Language_e::eKorean
                | Language_e::eTaiwanese
                | Language_e::eJapanese
                | Language_e::eChinese => {}
                Language_e::eThai => {
                    iCollapsedAsianCode = Thai_CollapseTISCode(font, uiLetter);
                }
                // unhandled asian language
                _ => debug_assert!(false),
            }
        }

        iCollapsedAsianCode
    }

    /// Raven `CFontInfo::GetLetterWidth`.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:1237-1241`
    pub fn GetLetterWidth(
        &mut self,
        font: &FontState,
        eLanguage: Language_e,
        uiLetter: u32,
    ) -> i32 {
        let (pGlyph, _) = self.GetLetter(font, eLanguage, uiLetter, false);
        if pGlyph.width != 0 {
            pGlyph.width as i32
        } else {
            self.mGlyphs[b'.' as usize].width as i32
        }
    }

    /// Raven `CFontInfo::GetLetterHorizAdvance`.
    /// Source: `oracle/codemp/renderer/tr_font.cpp:1243-1247`
    pub fn GetLetterHorizAdvance(
        &mut self,
        font: &FontState,
        eLanguage: Language_e,
        uiLetter: u32,
    ) -> i32 {
        let (pGlyph, _) = self.GetLetter(font, eLanguage, uiLetter, false);
        if pGlyph.horizAdvance != 0 {
            pGlyph.horizAdvance as i32
        } else {
            self.mGlyphs[b'.' as usize].horizAdvance as i32
        }
    }
}

/// Raven `Round` (`oracle/codemp/qcommon/qcommon.h:1094-1097`), a `qcommon`
/// inline with no Rust home yet; kept private so the canonical home stays
/// qcommon's when it lands (DEC-32).
fn Round(value: f32) -> i32 {
    (value + 0.5f32).floor() as i32
}

/// Raven's `GetLetter`-local `ASSIGN_WITH_ROUNDING` macro.
/// Source: `oracle/codemp/renderer/tr_font.cpp:1203`
fn assign_with_rounding(mbRoundCalcs: bool, m_fAltSBCSFontScaleFactor: f32, src: f32) -> i32 {
    if mbRoundCalcs {
        Round(m_fAltSBCSFontScaleFactor * src)
    } else {
        (m_fAltSBCSFontScaleFactor * src) as i32
    }
}

/// Raven's `Q_strncpyz(dst, src, MAX_QPATH)` clamp over an owned `String`
/// (`m_sFontName`'s `char[MAX_QPATH]` is a `String` under the
/// interior-safety law).
/// Source: `oracle/codemp/renderer/tr_font.cpp:875`
fn truncate_to_qpath(s: &mut String) {
    let mut n = MAX_QPATH as usize - 1;
    if s.len() > n {
        while !s.is_char_boundary(n) {
            n -= 1;
        }
        s.truncate(n);
    }
}

/// A zeroed `glyphInfo_t`.
fn glyph_zero() -> glyphInfo_t {
    glyphInfo_t {
        width: 0,
        height: 0,
        horizAdvance: 0,
        horizOffset: 0,
        baseline: 0,
        s: 0.0,
        t: 0.0,
        s2: 0.0,
        t2: 0.0,
    }
}

/// A field-by-field copy of a `glyphInfo_t` (the seam type carries no
/// derives, and this crate does not own it).
fn glyph_copy(g: &glyphInfo_t) -> glyphInfo_t {
    glyphInfo_t {
        width: g.width,
        height: g.height,
        horizAdvance: g.horizAdvance,
        horizOffset: g.horizOffset,
        baseline: g.baseline,
        s: g.s,
        t: g.t,
        s2: g.s2,
        t2: g.t2,
    }
}

/// One little-endian `short` out of an on-disk image.
fn read_i16(buf: &[u8], off: usize) -> i16 {
    i16::from_le_bytes([buf[off], buf[off + 1]])
}

/// One little-endian `int` out of an on-disk image.
fn read_i32(buf: &[u8], off: usize) -> i32 {
    i32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
}

/// One little-endian `float` out of an on-disk image.
fn read_f32(buf: &[u8], off: usize) -> f32 {
    f32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
}

/// One `glyphInfo_t` record read explicitly out of a `.fontdat` image at
/// `base`, field by field — the file is never cast to a `#[repr(C)]` view.
/// Source: `oracle/codemp/qcommon/qfiles.h:574-585`
fn read_glyph(buf: &[u8], base: usize) -> glyphInfo_t {
    glyphInfo_t {
        width: read_i16(buf, base + offset_of!(glyphInfo_t, width)),
        height: read_i16(buf, base + offset_of!(glyphInfo_t, height)),
        horizAdvance: read_i16(buf, base + offset_of!(glyphInfo_t, horizAdvance)),
        horizOffset: read_i16(buf, base + offset_of!(glyphInfo_t, horizOffset)),
        baseline: read_i32(buf, base + offset_of!(glyphInfo_t, baseline)),
        s: read_f32(buf, base + offset_of!(glyphInfo_t, s)),
        t: read_f32(buf, base + offset_of!(glyphInfo_t, t)),
        s2: read_f32(buf, base + offset_of!(glyphInfo_t, s2)),
        t2: read_f32(buf, base + offset_of!(glyphInfo_t, t2)),
    }
}

/// Raven `RoundTenth`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:240-243`
pub fn RoundTenth(fValue: f32) -> f32 {
    (fValue * 10.0f32 + 0.5f32).floor() / 10.0f32
}

/// Raven `Korean_ValidKSC5601Hangul`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:264-271`
pub fn Korean_ValidKSC5601Hangul(_iHi: u8, _iLo: u8) -> bool {
    _iHi >= KSC5601_HANGUL_HIBYTE_START
        && _iHi <= KSC5601_HANGUL_HIBYTE_STOP
        && _iLo > KSC5601_HANGUL_LOBYTE_LOBOUND
        && _iLo < KSC5601_HANGUL_LOBYTE_HIBOUND
}

/// Raven `Korean_ValidKSC5601Hangul` (the single-`uiCode` overload).
///
/// PORT-NOTE: Raven overloads this name for both the `(hi, lo)` byte-pair
/// form above and this packed-code form; Rust has no overloading, so this
/// overload is disambiguated with the `_uiCode` suffix, taken from its own
/// oracle parameter name. The C narrowing conversion of `uiCode >> 8` into
/// the `byte _iHi` parameter is spelled out as an explicit `as u8` truncation.
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:273-276`
pub fn Korean_ValidKSC5601Hangul_uiCode(uiCode: u32) -> bool {
    Korean_ValidKSC5601Hangul((uiCode >> 8) as u8, (uiCode & 0xFF) as u8)
}

/// Raven `Korean_CollapseKSC5601HangulCode`.
///
/// Raven: sneaky maths on both bytes, reduce to 0x0000 onwards.
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:284-293`
pub fn Korean_CollapseKSC5601HangulCode(mut uiCode: u32) -> i32 {
    if Korean_ValidKSC5601Hangul_uiCode(uiCode) {
        uiCode -= (KSC5601_HANGUL_HIBYTE_START as u32 * 256) + KSC5601_HANGUL_LOBYTE_LOBOUND as u32;
        uiCode = ((uiCode >> 8) * KSC5601_HANGUL_CODES_PER_ROW) + (uiCode & 0xFF);
        return uiCode as i32;
    }
    0
}

/// Per-subsystem owned state for the Asian-glyph + font-registry code
/// (DEC-37 A13.3) — homes `g_iNonScaledCharRange` (write sites `:299,381`,
/// and this shard's `:476,544,646`), the font registry
/// `g_iCurrentFontIndex`/`g_mapFontIndexes`/`g_vFontArray`
/// (`:1616-1662`), and `g_ThaiCodes` (`:574-639,1661`). Named by this wave
/// (R3 wave 0, `tr_font.cpp`) per the packet's STATE HOMES rows.
///
/// Source: `oracle/codemp/renderer/tr_font.cpp` (file-scope globals,
/// declarations not in this packet's slice).
#[derive(Default)]
pub struct FontState {
    /// Raven `g_iNonScaledCharRange`.
    pub g_iNonScaledCharRange: i32,
    /// Raven `g_iCurrentFontIndex` — entry 0 reserved for "missing/invalid".
    pub g_iCurrentFontIndex: i32,
    /// Raven `g_mapFontIndexes` (`FontIndexMap_t`) — font name -> index.
    pub g_mapFontIndexes: HashMap<String, i32>,
    /// Raven `g_vFontArray` — index -> loaded `CFontInfo`; owned per the
    /// interior-safety law (Raven's shape was `std::vector<CFontInfo*>`).
    pub g_vFontArray: Vec<Option<Box<CFontInfo>>>,
    /// Raven `g_ThaiCodes` (`ThaiCodes_t`).
    pub g_ThaiCodes: ThaiCodes_t,
    /// Raven's fn-scope `static qboolean bDone` inside the constructor's
    /// `com_buildScript == 2` block (`:891`) — "Do this once only (for
    /// speed)". A fn-scope static is a hidden global (porting-rules §B3), so
    /// it is homed here with the file's other statics.
    pub bDone_ForeignFontsRegistered: bool,
}

/// Raven's `int &iGlyphTPs, LPCSTR &psLang` out-params plus the `int` return
/// value (`m_iAsianGlyphsAcross`) shared by `Korean_InitFields`/
/// `Taiwanese_InitFields` (and future per-language `*_InitFields` fns),
/// collapsed into one return per the out-params→return-values dictionary
/// rule.
pub struct LanguageFontFields {
    /// The `int` return value (`m_iAsianGlyphsAcross`).
    pub m_iAsianGlyphsAcross: i32,
    pub iGlyphTPs: i32,
    pub psLang: &'static str,
}

/// Raven `Korean_InitFields`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:295-301`
pub fn Korean_InitFields(font: &mut FontState) -> LanguageFontFields {
    font.g_iNonScaledCharRange = 255;
    LanguageFontFields {
        iGlyphTPs: GLYPH_MAX_KOREAN_SHADERS,
        psLang: "kor",
        m_iAsianGlyphsAcross: 32,
    }
}

/// Raven `Taiwanese_ValidBig5Code`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:319-337`
pub fn Taiwanese_ValidBig5Code(uiCode: u32) -> bool {
    let _iHi = ((uiCode >> 8) & 0xFF) as u8;
    if (_iHi >= BIG5_HIBYTE_START0 && _iHi <= BIG5_HIBYTE_STOP0)
        || (_iHi >= BIG5_HIBYTE_START1 && _iHi <= BIG5_HIBYTE_STOP1)
    {
        let _iLo = (uiCode & 0xFF) as u8;

        if (_iLo >= BIG5_LOBYTE_LOBOUND0 && _iLo <= BIG5_LOBYTE_HIBOUND0)
            || (_iLo >= BIG5_LOBYTE_LOBOUND1 && _iLo <= BIG5_LOBYTE_HIBOUND1)
        {
            return true;
        }
    }

    false
}

/// Raven `Taiwanese_IsTrailingPunctuation`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:342-354`
pub fn Taiwanese_IsTrailingPunctuation(uiCode: u32) -> bool {
    // so far I'm just counting the first 21 chars, those seem to be all the basic punctuation...
    //
    let hi = (BIG5_HIBYTE_START0 as u32) << 8;
    let lo = BIG5_LOBYTE_LOBOUND0 as u32;
    if uiCode >= (hi | lo) && uiCode < (hi | (lo + 20)) {
        return true;
    }

    false
}

/// Raven `Taiwanese_CollapseBig5Code`.
///
/// Raven: sneaky maths on both bytes, reduce to 0x0000 onwards.
///
/// PORT-NOTE: every constant this body needs except `BIG5_CODES_PER_ROW` is
/// already defined in this file; `BIG5_CODES_PER_ROW`'s value is not in this
/// packet's slice, so the final collapse step (the only line that needs it)
/// is left as a cited `todo!()` rather than guessed (wave-0 ruling: never
/// guess a `#define` value) — everything computable above it is transcribed
/// faithfully, including both bounds guards.
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:362-375`
pub fn Taiwanese_CollapseBig5Code(mut uiCode: u32) -> i32 {
    if Taiwanese_ValidBig5Code(uiCode) {
        uiCode -= (BIG5_HIBYTE_START0 as u32 * 256) + BIG5_LOBYTE_LOBOUND0 as u32;
        if (uiCode & 0xFF) >= (BIG5_LOBYTE_LOBOUND1 as u32 - 1) - BIG5_LOBYTE_LOBOUND0 as u32 {
            uiCode -= ((BIG5_LOBYTE_LOBOUND1 as u32 - 1) - (BIG5_LOBYTE_HIBOUND0 as u32 + 1)) - 1;
        }
        //TODO: Port BIG5_CODES_PER_ROW
        // Source: oracle/codemp/renderer/tr_font.cpp:371 (#define, value not in packet)
        let _ = uiCode;
        todo!("Port BIG5_CODES_PER_ROW — oracle/codemp/renderer/tr_font.cpp:371");
    }
    0
}

/// Raven `Taiwanese_InitFields`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:377-383`
pub fn Taiwanese_InitFields(font: &mut FontState) -> LanguageFontFields {
    font.g_iNonScaledCharRange = 255;
    LanguageFontFields {
        iGlyphTPs: GLYPH_MAX_TAIWANESE_SHADERS,
        psLang: "tai",
        m_iAsianGlyphsAcross: 64,
    }
}

/// Raven `Japanese_ValidShiftJISCode`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:404-419`
pub fn Japanese_ValidShiftJISCode(_iHi: u8, _iLo: u8) -> bool {
    if (_iHi >= SHIFTJIS_HIBYTE_START0 && _iHi <= SHIFTJIS_HIBYTE_STOP0)
        || (_iHi >= SHIFTJIS_HIBYTE_START1 && _iHi <= SHIFTJIS_HIBYTE_STOP1)
    {
        if (_iLo >= SHIFTJIS_LOBYTE_START0 && _iLo <= SHIFTJIS_LOBYTE_STOP0)
            || (_iLo >= SHIFTJIS_LOBYTE_START1 && _iLo <= SHIFTJIS_LOBYTE_STOP1)
        {
            return true;
        }
    }

    false
}

/// Raven `Japanese_ValidShiftJISCode` (the single-`uiCode` overload).
///
/// PORT-NOTE: same overload-disambiguation as
/// [`Korean_ValidKSC5601Hangul_uiCode`] — Rust has no overloading, so the
/// packed-code overload gets the `_uiCode` suffix.
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:421-424`
pub fn Japanese_ValidShiftJISCode_uiCode(uiCode: u32) -> bool {
    Japanese_ValidShiftJISCode((uiCode >> 8) as u8, (uiCode & 0xFF) as u8)
}

/// Raven `Japanese_IsTrailingPunctuation`.
///
/// Raven: so far I'm just counting the first 18 chars, those seem to be all
/// the basic punctuation...
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:429-441`
pub fn Japanese_IsTrailingPunctuation(uiCode: u32) -> bool {
    if uiCode >= ((SHIFTJIS_HIBYTE_START0 as u32) << 8 | SHIFTJIS_LOBYTE_START0 as u32)
        && uiCode < ((SHIFTJIS_HIBYTE_START0 as u32) << 8 | (SHIFTJIS_LOBYTE_START0 as u32 + 18))
    {
        return true;
    }

    false
}

/// Raven `Japanese_CollapseShiftJISCode`.
///
/// Raven: sneaky maths on both bytes, reduce to 0x0000 onwards.
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:448-469`
pub fn Japanese_CollapseShiftJISCode(mut uiCode: u32) -> i32 {
    if Japanese_ValidShiftJISCode_uiCode(uiCode) {
        uiCode -= ((SHIFTJIS_HIBYTE_START0 as u32) << 8) | SHIFTJIS_LOBYTE_START0 as u32;

        if (uiCode & 0xFF) >= (SHIFTJIS_LOBYTE_START1 as u32) - SHIFTJIS_LOBYTE_START0 as u32 {
            uiCode -= (SHIFTJIS_LOBYTE_START1 as u32 - SHIFTJIS_LOBYTE_STOP0 as u32) - 1;
        }

        if ((uiCode >> 8) & 0xFF) >= (SHIFTJIS_HIBYTE_START1 as u32) - SHIFTJIS_HIBYTE_START0 as u32
        {
            uiCode -= ((SHIFTJIS_HIBYTE_START1 as u32 - SHIFTJIS_HIBYTE_STOP0 as u32) - 1) << 8;
        }

        uiCode = ((uiCode >> 8) * SHIFTJIS_CODES_PER_ROW) + (uiCode & 0xFF);

        return uiCode as i32;
    }
    0
}

/// Raven `Japanese_InitFields`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:472-478`
pub fn Japanese_InitFields(font: &mut FontState) -> LanguageFontFields {
    font.g_iNonScaledCharRange = 255;
    LanguageFontFields {
        iGlyphTPs: GLYPH_MAX_JAPANESE_SHADERS,
        psLang: "jap",
        m_iAsianGlyphsAcross: 64,
    }
}

/// Raven `Chinese_ValidGBCode`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:490-497`
pub fn Chinese_ValidGBCode(_iHi: u8, _iLo: u8) -> bool {
    _iHi >= GB_HIBYTE_START
        && _iHi <= GB_HIBYTE_STOP
        && _iLo > GB_LOBYTE_LOBOUND
        && _iLo < GB_LOBYTE_HIBOUND
}

/// Raven `Chinese_ValidGBCode` (the single-`uiCode` overload).
///
/// PORT-NOTE: same overload-disambiguation as
/// [`Korean_ValidKSC5601Hangul_uiCode`] — Rust has no overloading, so the
/// packed-code overload gets the `_uiCode` suffix.
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:499-502`
pub fn Chinese_ValidGBCode_uiCode(uiCode: u32) -> bool {
    Chinese_ValidGBCode((uiCode >> 8) as u8, (uiCode & 0xFF) as u8)
}

/// Raven `Chinese_IsTrailingPunctuation`.
///
/// Raven: so far I'm just counting the first 13 chars, those seem to be all
/// the basic punctuation...
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:507-519`
pub fn Chinese_IsTrailingPunctuation(uiCode: u32) -> bool {
    if uiCode > ((GB_HIBYTE_START as u32) << 8 | GB_LOBYTE_LOBOUND as u32)
        && uiCode < ((GB_HIBYTE_START as u32) << 8 | (GB_LOBYTE_LOBOUND as u32 + 14))
    {
        return true;
    }

    false
}

/// Raven `Chinese_CollapseGBCode`.
///
/// Raven: sneaky maths on both bytes, reduce to 0x0000 onwards.
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:527-537`
pub fn Chinese_CollapseGBCode(mut uiCode: u32) -> i32 {
    if Chinese_ValidGBCode_uiCode(uiCode) {
        uiCode -= (GB_HIBYTE_START as u32 * 256) + GB_LOBYTE_LOBOUND as u32;
        uiCode = ((uiCode >> 8) * GB_CODES_PER_ROW) + (uiCode & 0xFF);
        return uiCode as i32;
    }

    0
}

/// Raven `Chinese_InitFields`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:539-545`
pub fn Chinese_InitFields(font: &mut FontState) -> LanguageFontFields {
    font.g_iNonScaledCharRange = 255;
    LanguageFontFields {
        iGlyphTPs: GLYPH_MAX_CHINESE_SHADERS,
        psLang: "chi",
        m_iAsianGlyphsAcross: 64,
    }
}

/// Raven `Thai_ValidTISCode`.
///
/// Raven: this code is heavily little-endian, so someone else will need to
/// port for Mac etc... (not my problem ;-)
///
/// PORT-NOTE: Raven's C union `CodeToTry_t` (`char sChars[4]` /
/// `unsigned int uiCode`) packs up to 3 bytes little-endian into a 32-bit
/// code; replicated here with `u32::from_le_bytes` over an explicit 4-byte
/// array. Raven's `for (int i=0; i<3; i++)` loop variable `i` is read right
/// after the loop closes (`iThaiBytes = i;`) — under the pre-C++11 MSVC
/// for-scope leak this codebase relies on elsewhere, its final value (3 on
/// full success, or the failing index on an early `break`) survives past the
/// loop; threaded out explicitly here as the returned byte count instead.
/// The out-param `int &iThaiBytes` becomes the second element of the
/// returned tuple.
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:574-612`
pub fn Thai_ValidTISCode(font: &FontState, psString: &[u8]) -> (u32, i32) {
    // so western letters drop through and use normal font
    if psString[0] < 160 {
        return (0, 0);
    }

    let mut code_chars = [0u8; 4]; // important that we clear all 4 bytes in sChars here
    let mut bytes_matched = 3;
    for i in 0..3usize {
        // §19: sibling of `AnyLanguage_ReadCharFromString`'s second-byte read —
        // C walks into the NUL terminator on a short string, so a missing byte
        // reads as 0 here rather than panicking.
        code_chars[i] = psString.get(i).copied().unwrap_or(0);

        let code = u32::from_le_bytes(code_chars);
        let iIndex = font.g_ThaiCodes.GetValidIndex(code as i32);
        if iIndex == -1 {
            // failed, so return previous-longest code...
            code_chars[i] = 0;
            bytes_matched = i;
            break;
        }
    }

    let code = u32::from_le_bytes(code_chars);
    // if 'bytes_matched' was 0, then this may be an error, trying to get a
    // thai accent as standalone char?
    debug_assert!(bytes_matched != 0);
    (code, bytes_matched as i32)
}

/// Raven `Thai_IsTrailingPunctuation`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:618-621`
pub fn Thai_IsTrailingPunctuation(uiCode: u32) -> bool {
    uiCode == '_' as u32
}

/// Raven `Thai_CollapseTISCode`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:627-639`
pub fn Thai_CollapseTISCode(font: &FontState, uiCode: u32) -> i32 {
    if uiCode >= TIS_GLYPHS_START {
        // so western letters drop through as invalid
        let iCollapsedIndex = font.g_ThaiCodes.GetValidIndex(uiCode as i32);
        if iCollapsedIndex != -1 {
            return iCollapsedIndex;
        }
    }

    0
}

/// Raven `Thai_InitFields`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:641-647`
pub fn Thai_InitFields(font: &mut FontState) -> LanguageFontFields {
    // in other words, don't scale any thai chars down
    font.g_iNonScaledCharRange = i32::MAX;
    LanguageFontFields {
        iGlyphTPs: GLYPH_MAX_THAI_SHADERS,
        psLang: "tha",
        m_iAsianGlyphsAcross: 32,
    }
}

/// Raven `AnyLanguage_ReadCharFromString`.
///
/// PORT-NOTE: `GetLanguageEnum()` is unported (file-head DEFERRED,
/// `:31-53`); threaded in as `eLanguage`, same as every other caller in this
/// file. Raven's `const byte *psString = (const byte *)psText` sign-promote
/// dodge needs no equivalent — `&[u8]` is already unsigned. The optional
/// `int *piAdvanceCount` out-param is not optional in Raven (always written),
/// so it is simply the tuple's second element; the optional
/// `qboolean *pbIsTrailingPunctuation` becomes the `bWantTrailingPunctuation`
/// request flag plus a `Option<bool>` third element, mirroring
/// [`CFontInfo::GetLetter`]'s `bWantShader`/`Option<i32>` pair.
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:659-779`
pub fn AnyLanguage_ReadCharFromString(
    font: &FontState,
    eLanguage: Language_e,
    psString: &[u8],
    bWantTrailingPunctuation: bool,
) -> (u32, i32, Option<bool>) {
    // §19: the double-byte arms read `psString[1]` on a one-character string,
    // where C reads the NUL terminator (0); `&[u8]` has no terminator, so the
    // second byte reads as 0 when absent instead of panicking.
    let second = psString.get(1).copied().unwrap_or(0);

    match eLanguage {
        Language_e::eKorean => {
            if Korean_ValidKSC5601Hangul(psString[0], second) {
                let uiLetter = (psString[0] as u32 * 256) + second as u32;
                let piAdvanceCount = 2;

                // not going to bother testing for korean punctuation here, since korean already
                //	uses spaces, and I don't have the punctuation glyphs defined, only the basic 2350 hanguls
                //
                let pbIsTrailingPunctuation = bWantTrailingPunctuation.then_some(false);

                return (uiLetter, piAdvanceCount, pbIsTrailingPunctuation);
            }
        }
        Language_e::eTaiwanese => {
            if Taiwanese_ValidBig5Code((psString[0] as u32 * 256) + second as u32) {
                let uiLetter = (psString[0] as u32 * 256) + second as u32;
                let piAdvanceCount = 2;

                // need to ask if this is a trailing (ie like a comma or full-stop) punctuation?...
                //
                let pbIsTrailingPunctuation =
                    bWantTrailingPunctuation.then(|| Taiwanese_IsTrailingPunctuation(uiLetter));

                return (uiLetter, piAdvanceCount, pbIsTrailingPunctuation);
            }
        }
        Language_e::eJapanese => {
            if Japanese_ValidShiftJISCode(psString[0], second) {
                let uiLetter = (psString[0] as u32 * 256) + second as u32;
                let piAdvanceCount = 2;

                // need to ask if this is a trailing (ie like a comma or full-stop) punctuation?...
                //
                let pbIsTrailingPunctuation =
                    bWantTrailingPunctuation.then(|| Japanese_IsTrailingPunctuation(uiLetter));

                return (uiLetter, piAdvanceCount, pbIsTrailingPunctuation);
            }
        }
        Language_e::eChinese => {
            if Chinese_ValidGBCode_uiCode((psString[0] as u32 * 256) + second as u32) {
                let uiLetter = (psString[0] as u32 * 256) + second as u32;
                let piAdvanceCount = 2;

                // need to ask if this is a trailing (ie like a comma or full-stop) punctuation?...
                //
                let pbIsTrailingPunctuation =
                    bWantTrailingPunctuation.then(|| Chinese_IsTrailingPunctuation(uiLetter));

                return (uiLetter, piAdvanceCount, pbIsTrailingPunctuation);
            }
        }
        Language_e::eThai => {
            let (uiLetter, iThaiBytes) = Thai_ValidTISCode(font, psString);
            if uiLetter != 0 {
                let piAdvanceCount = iThaiBytes;

                let pbIsTrailingPunctuation =
                    bWantTrailingPunctuation.then(|| Thai_IsTrailingPunctuation(uiLetter));

                return (uiLetter, piAdvanceCount, pbIsTrailingPunctuation);
            }
        }
        // Raven's `switch` has no `default` arm for the remaining (Western/
        // Russian/Polish) languages — they fall straight to the shared
        // single-byte tail below.
        _ => {}
    }

    // ... must not have been an MBCS code...
    //
    let uiLetter = psString[0] as u32;
    let piAdvanceCount = 1;

    let pbIsTrailingPunctuation = bWantTrailingPunctuation.then(|| {
        uiLetter == b'!' as u32
            || uiLetter == b'?' as u32
            || uiLetter == b',' as u32
            || uiLetter == b'.' as u32
            || uiLetter == b';' as u32
            || uiLetter == b':' as u32
    });

    (uiLetter, piAdvanceCount, pbIsTrailingPunctuation)
}

/// Raven `Language_IsAsian`.
///
/// PORT-NOTE: `GetLanguageEnum()` is unported (file-head DEFERRED,
/// `:31-53`); threaded in as the `eLanguage` parameter per that note's
/// established pattern (porting-rules §B4) — every other caller in this
/// file already does the same.
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:785-798`
pub fn Language_IsAsian(eLanguage: Language_e) -> bool {
    matches!(
        eLanguage,
        Language_e::eKorean
            | Language_e::eTaiwanese
            | Language_e::eJapanese
            | Language_e::eChinese
            // this is asian, but the query is normally used for scaling
            | Language_e::eThai
    )
}

/// Raven `Language_UsesSpaces`.
///
/// Raven: ( korean uses spaces ).
///
/// PORT-NOTE: `GetLanguageEnum()` is unported (file-head DEFERRED,
/// `:31-53`); threaded in as the `eLanguage` parameter, same as
/// [`Language_IsAsian`].
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:800-813`
pub fn Language_UsesSpaces(eLanguage: Language_e) -> bool {
    !matches!(
        eLanguage,
        Language_e::eTaiwanese | Language_e::eJapanese | Language_e::eChinese | Language_e::eThai
    )
}

/// Raven `GetFont_SBCSOverride`.
///
/// Raven: work out the scaling factor for this font's glyphs, then override
/// with the main properties of the original font.
///
/// PORT-NOTE: `GetLanguageEnum()` is unported (file-head DEFERRED,
/// `:31-53`); threaded in as the `eLanguage` parameter. `pFont`/the returned
/// `CFontInfo *` are `FontState::g_vFontArray` indices per the arena+id
/// pattern (porting-rules §B5) the rest of this file already uses.
///
/// PORT-NOTE: this packet's RESOLVED CALL SURFACE lists `RE_RegisterFont`
/// and `GetFont_Actual` as already ported in a lower wave; grepping this
/// crate shows neither exists yet (a wave-planning defect, not a genuine
/// call surface). Every guard this function can evaluate without them is
/// transcribed faithfully; only the two branches that actually need them are
/// left as cited `todo!()`s.
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:1251-1295`
pub fn GetFont_SBCSOverride(
    font: &mut FontState,
    eLanguage: Language_e,
    iFont: i32,
    eLanguageSBCS: Language_e,
    psLanguageNameSBCS: &str,
) -> Option<i32> {
    let (m_bIsFakeAlienLanguage, m_iAltSBCSFont) = match font
        .g_vFontArray
        .get(iFont as usize)
        .and_then(|f| f.as_deref())
    {
        Some(f) => (f.m_bIsFakeAlienLanguage, f.m_iAltSBCSFont),
        None => return None,
    };

    if !m_bIsFakeAlienLanguage && eLanguage == eLanguageSBCS {
        if m_iAltSBCSFont == -1 {
            // no reg attempted yet?
            // need to register this alternative SBCS font...
            //
            //TODO: Port RE_RegisterFont
            // Source: oracle/codemp/renderer/tr_font.cpp:1261
            //TODO: Port GetFont_Actual
            // Source: oracle/codemp/renderer/tr_font.cpp:1262
            let _ = psLanguageNameSBCS;
            todo!(
                "Port GetFont_SBCSOverride's alt-font registration — depends on unported RE_RegisterFont/GetFont_Actual — oracle/codemp/renderer/tr_font.cpp:1259-1284"
            );
        }

        if m_iAltSBCSFont > 0 {
            //TODO: Port GetFont_Actual
            // Source: oracle/codemp/renderer/tr_font.cpp:1289
            todo!(
                "Port GetFont_SBCSOverride's alt-font lookup — depends on unported GetFont_Actual — oracle/codemp/renderer/tr_font.cpp:1287-1290"
            );
        }
    }

    None
}

/// Raven `GetFont`.
///
/// PORT-NOTE: `GetLanguageEnum()` is unported (file-head DEFERRED,
/// `:31-53`); threaded in as `eLanguage`, forwarded to
/// [`GetFont_SBCSOverride`] exactly as its own PORT-NOTE requires.
///
/// PORT-NOTE: this packet's RESOLVED CALL SURFACE lists `GetFont_Actual` as
/// already ported in a lower wave; grepping this crate shows it does not
/// exist yet — the same wave-planning defect [`GetFont_SBCSOverride`]'s
/// PORT-NOTE already raised for the identical fn. `pFont`/the returned
/// `CFontInfo *` are `FontState::g_vFontArray` indices per the arena+id
/// pattern this file already uses; every step this fn can perform without
/// `GetFont_Actual` is transcribed faithfully, only the initial lookup is
/// left as a cited `todo!()`.
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:1299-1318`
#[allow(unreachable_code, unused_variables)]
pub fn GetFont(font: &mut FontState, eLanguage: Language_e, index: i32) -> Option<i32> {
    let _ = index;
    //TODO: Port GetFont_Actual
    // Source: oracle/codemp/renderer/tr_font.cpp:1301
    let pFont: Option<i32> =
        todo!("Port GetFont's GetFont_Actual call — oracle/codemp/renderer/tr_font.cpp:1301");

    if let Some(iFont) = pFont {
        // any SBCS overrides? (this has to be pretty quick, and is (sort of))...
        //
        for entry in g_SBCSOverrideLanguages.iter() {
            let pAltFont =
                GetFont_SBCSOverride(font, eLanguage, iFont, entry.m_eLanguage, entry.m_psName);
            if pAltFont.is_some() {
                return pAltFont;
            }
        }
    }

    pFont
}

/// Raven `R_InitFonts`.
/// Source: `oracle/codemp/renderer/tr_font.cpp:1645-1649`
pub fn R_InitFonts(font: &mut FontState) {
    // entry 0 is reserved for "missing/invalid"
    font.g_iCurrentFontIndex = 1;
    // default all chars to have no special scaling (other than user supplied)
    font.g_iNonScaledCharRange = i32::MAX;
}

/// Raven `R_ShutdownFonts`.
///
/// PORT-NOTE: Raven's `for` loop explicitly `delete`s each `g_vFontArray`
/// entry before `g_mapFontIndexes.clear()`/`g_vFontArray.clear()`; owned
/// `Vec<Option<Box<CFontInfo>>>` drop glue frees every entry when the vector
/// is cleared, so the explicit per-entry loop is redundant here and dropped
/// (porting-rules §10 — control-flow behavior preserved, shape is not).
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:1651-1662`
pub fn R_ShutdownFonts(font: &mut FontState) {
    font.g_mapFontIndexes.clear();
    font.g_vFontArray.clear();
    // entry 0 is reserved for "missing/invalid"
    font.g_iCurrentFontIndex = 1;

    font.g_ThaiCodes.Clear();
}

/// Raven `R_ReloadFonts_f`.
///
/// PORT-NOTE: Raven's inner `for` loop leaks its `it` iterator past the loop
/// (the pre-C++11 MSVC for-scope-leak idiom this codebase relies on
/// elsewhere) to learn afterward whether the search broke early or ran to
/// completion; a `found` flag threaded out of the loop carries the same
/// information explicitly. `g_mapFontIndexes`' Rust `HashMap` has no
/// iteration order, but the search is a reverse (index -> name) lookup by
/// value, not an order-dependent scan, so this is behavior-preserving
/// (porting-rules §10 — control flow preserved, shape is not).
///
/// PORT-NOTE: this packet's RESOLVED CALL SURFACE lists `RE_RegisterFont` as
/// already ported in a lower wave; grepping this crate shows it does not
/// exist yet (a wave-planning defect). Every step that does not need it —
/// the ordered-name collection, the shutdown/restart, both `Com_Printf`
/// paths — is transcribed faithfully; only the re-registration loop's actual
/// `RE_RegisterFont` call is left as a cited `todo!()`. The `#ifdef _DEBUG`
/// `assert(iNewFontHandle == iFont+1)` twin of the release-build call is the
/// same blocked call either way, so both arms collapse to the one `todo!()`.
///
/// Source: `oracle/codemp/renderer/tr_font.cpp:1666-1711`
pub fn R_ReloadFonts_f(view: &mut EngineHostView, font: &mut FontState) {
    // first, grab all the currently-registered fonts IN THE ORDER THEY WERE
    // REGISTERED...
    //
    let mut vstrFonts: Vec<String> = Vec::new();
    let mut found_all = true;

    for iFontToFind in 1..font.g_iCurrentFontIndex {
        let found = font
            .g_mapFontIndexes
            .iter()
            .find(|&(_, &idx)| idx == iFontToFind)
            .map(|(name, _)| name.clone());
        match found {
            Some(name) => vstrFonts.push(name),
            None => {
                // couldn't find this font
                found_all = false;
                break;
            }
        }
    }

    if found_all {
        // found all of them? now restart the font system...
        //
        R_ShutdownFonts(font);
        R_InitFonts(font);
        //
        // and re-register our fonts in the same order as before (note that
        // some menu items etc cache the string lengths so really a
        // vid_restart is better, but this is just for my testing)
        //
        for _name in &vstrFonts {
            //TODO: Port RE_RegisterFont
            // Source: oracle/codemp/renderer/tr_font.cpp:1699,1702
            todo!("Port RE_RegisterFont — oracle/codemp/renderer/tr_font.cpp:1699,1702");
        }
        com_printf(view.common, "Done.\n");
    } else {
        // poo. Oh well, forget it.
        com_printf(
            view.common,
            "Problem encountered finding current fonts, ignoring.\n",
        );
    }
}

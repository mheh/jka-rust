//! `ServerSkins` — the dedicated-live `.skin` registration family of
//! `tr_image.cpp` (§F idiomatic reimplementation), transcribed as
//! `impl RenderModels` methods per user ruling 2026-07-12 (server skins
//! name-pool), which amends the FROZEN `tr-model.md` to home `tr.skins`/
//! `tr.numSkins` on `RenderModels` and flatten the server shader objects to a
//! name pool (`RenderModels.server_shaders`).
//!
//! Server skins exist so ghoul2 server-side surface state works: a `.skin`
//! file's `"*off"` shader rows mark surfaces off
//! (`G2_SetSurfaceOnOffFromSkin`, `G2_surfaces.cpp:201-226`), which affects
//! server collision. On this slice `gServerSkinHack` (`tr_image.cpp:2967`) is
//! const-true: `RE_RegisterServerSkin`'s `com_cl_running` fast-path and
//! `RE_RegisterIndividualSkin`'s `R_FindShader` arm both need the §20-dropped
//! client shader table (`tr_shader.cpp`), so the `R_FindServerShader` arm is
//! the only live shader resolver — flattened to [`find_server_shader`]'s name
//! pool, since the name is the sole shader field the dedicated path reads
//! (`G2_surfaces.cpp:212`).
//!
//! Source: `oracle/codemp/renderer/tr_image.cpp:2956-3346`
//!
//! [`find_server_shader`]: RenderModels::find_server_shader

use mp_host_interface::EngineHost;
use mp_qshared::shared::limits::MAX_TOKEN_CHARS;
use mp_qshared::shared::q_string::COM_StripExtension;
use mp_qshared::shared::{qhandle_t, MAX_QPATH};

use crate::tr_local::tr_globals_t::MAX_SKINS;

use super::render_models::RenderModels;
use super::server_skin::ServerSkin;
use super::server_skin_surface::ServerSkinSurface;

/// Raven `skin_t.surfaces[128]` — the per-skin surface cap
/// (`sizeof(skin->surfaces) / sizeof(skin->surfaces[0])`).
///
/// Source: `oracle/codemp/renderer/tr_local.h:612`
const MAX_SKIN_SURFACES: usize = 128;

/// The seeded name of pool slot 0 — Raven `tr.defaultShader`. On the oracle
/// DEDICATED build `tr.defaultShader` is never created (`CreateInternalShaders`
/// is client-only) and stays NULL; a read through it is a null deref, so the
/// pool seeds a real named entry instead (§19 — one defined behavior).
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp:4180` (`"<default>"`).
const DEFAULT_SHADER_NAME: &str = "<default>";

impl RenderModels {
    /// Raven `R_InitSkins` — reset the skin registry to the single default
    /// skin whose one surface wears `tr.defaultShader` (`Hunk_Alloc`'s
    /// zero-fill leaves that surface's name empty). The server-shader name
    /// pool resets alongside: `SV_SpawnServer` runs `Hunk_Clear` →
    /// `R_InitSkins` → `R_InitShaders` back-to-back (`sv_init.cpp`), and the
    /// pool is this slice's flattened shader hash table, referenced only by
    /// the skins just cleared.
    ///
    /// Source: `oracle/codemp/renderer/tr_image.cpp:3324-3334`
    pub fn init_skins(&mut self) {
        self.server_shaders.clear();
        self.server_shaders.push(DEFAULT_SHADER_NAME.to_owned());

        // tr.numSkins = 1;
        self.skins.clear();
        // make the default skin have all default shaders
        self.skins.push(ServerSkin {
            name: "<default skin>".to_owned(),
            surfaces: vec![ServerSkinSurface {
                name: String::new(),
                shader: 0,
            }],
        });
    }

    /// Raven `RE_RegisterServerSkin` — "Mangled version of the above function
    /// to load .skin files on the server." The `com_cl_running &&
    /// Com_TheHunkMarkHasBeenMade() && ShaderHashTableExists()` fast-path
    /// re-enters `RE_RegisterSkin` against the client shader table —
    /// §20-dropped here, so the `gServerSkinHack = true` arm is the only live
    /// one (both arms record the same shader NAME, the sole field the
    /// dedicated consumer reads, so the `"*off"` sentinel is arm-invariant).
    ///
    /// Source: `oracle/codemp/renderer/tr_image.cpp:3301-3318`
    pub fn register_server_skin(&mut self, host: &mut impl EngineHost, name: &str) -> qhandle_t {
        // gServerSkinHack = true; r = RE_RegisterSkin(name); gServerSkinHack
        // = false — const-true on this slice (module doc-comment).
        self.register_skin(host, name)
    }

    /// Raven `RE_RegisterSkin` — dedup an already-registered skin by
    /// case-insensitive name (`Q_stricmp`; a known zero-surface skin reads as
    /// the default skin `0`), allocate the pool entry, then parse either the
    /// three-part `|`-macro form ([`re_split_skins`]) or the single `.skin`
    /// file into it. The parse-failed skin stays pooled with zero surfaces,
    /// exactly as Raven leaves the `Hunk_Alloc`'d entry behind.
    ///
    /// Source: `oracle/codemp/renderer/tr_image.cpp:3113-3181`
    fn register_skin(&mut self, host: &mut impl EngineHost, name: &str) -> qhandle_t {
        if name.is_empty() {
            host.print("Empty name passed to RE_RegisterSkin\n");
            return 0;
        }

        if name.len() >= MAX_QPATH {
            host.print("Skin name exceeds MAX_QPATH\n");
            return 0;
        }

        // see if the skin is already loaded
        for h_skin in 1..self.skins.len() {
            let skin = &self.skins[h_skin];
            if skin.name.eq_ignore_ascii_case(name) {
                if skin.surfaces.is_empty() {
                    return 0; // default skin
                }
                return h_skin as qhandle_t;
            }
        }

        // allocate a new skin
        if self.skins.len() == MAX_SKINS {
            host.print(&format!(
                "WARNING: RE_RegisterSkin( '{name}' ) MAX_SKINS hit\n"
            ));
            return 0;
        }
        // `tr.numSkins++; tr.skins[hSkin] = Hunk_Alloc(...)` — `hSkin` is the
        // old high-water mark (the pool is always `init_skins`-seeded first).
        let h_skin = self.skins.len() as qhandle_t;
        self.skins.push(ServerSkin {
            name: name.to_owned(),
            surfaces: Vec::new(),
        });

        // `R_SyncRenderThread()` — client render-thread sync, dead on the
        // dedicated slice (§C10). Raven's "If not a .skin file, load as a
        // single shader" branch is commented out in the oracle — not ported.

        if let Some((skinhead, skintorso, skinlower)) = re_split_skins(name) {
            // three part
            let mut h_skin = self.register_individual_skin(host, &skinhead, h_skin);
            if h_skin != 0 {
                h_skin = self.register_individual_skin(host, &skintorso, h_skin);
                if h_skin != 0 {
                    h_skin = self.register_individual_skin(host, &skinlower, h_skin);
                }
            }
            h_skin
        } else {
            // single skin
            self.register_individual_skin(host, name, h_skin)
        }
    }

    /// Raven `RE_RegisterIndividualSkin` — "given a name, go get the skin we
    /// want": load and parse one `.skin` file, appending each
    /// `surface,shader` row onto `tr.skins[hSkin]` (an append, since the
    /// three-part form calls this thrice on one skin). Surface names are
    /// `MAX_QPATH`-truncated and lowercased; `tag_` rows and redundant
    /// `_off`/`*off` doubles are skipped; a skin left with zero surfaces
    /// reads as the default skin (`0`). The `#ifndef FINAL_BUILD` load-failure
    /// warning is live (`FINAL_BUILD` undefined, porting-rules precedent);
    /// `assert(tr.skins[hSkin])` is debug-only, dropped.
    ///
    /// Source: `oracle/codemp/renderer/tr_image.cpp:3030-3111`
    fn register_individual_skin(
        &mut self,
        host: &mut impl EngineHost,
        name: &str,
        h_skin: qhandle_t,
    ) -> qhandle_t {
        // load and parse the skin file
        let Some(text) = host.fs_read_file(name) else {
            host.print(&format!(
                "WARNING: RE_RegisterSkin( '{name}' ) failed to load!\n"
            ));
            return 0;
        };

        let mut pos = 0usize;
        while pos < text.len() && text[pos] != 0 {
            // get surface name
            let token = comma_parse(&text, &mut pos);
            // Q_strncpyz( surfName, token, sizeof( surfName ) )
            let mut surf_name: Vec<u8> = token.iter().copied().take(MAX_QPATH - 1).collect();

            if token.is_empty() {
                break;
            }
            // lowercase the surface name so skin compares are faster
            surf_name.make_ascii_lowercase();

            if pos < text.len() && text[pos] == b',' {
                pos += 1;
            }

            // these aren't in there, but just in case you load an id style one...
            if token.starts_with(b"tag_") {
                continue;
            }

            // parse the shader name
            let token = comma_parse(&text, &mut pos);

            // Raven indexes `&surfName[strlen(surfName)-4]` unchecked — a
            // <4-byte name underruns the buffer (UB); guarded to len >= 4 (§19).
            if surf_name.len() >= 4 && surf_name.ends_with(b"_off") {
                if token == b"*off" {
                    continue; // don't need these double offs
                }
                let stripped_len = surf_name.len() - 4;
                surf_name.truncate(stripped_len); // remove the "_off"
            }

            if self.skins[h_skin as usize].surfaces.len() >= MAX_SKIN_SURFACES {
                host.print(&format!(
                    "WARNING: RE_RegisterSkin( '{name}' ) more than {MAX_SKIN_SURFACES} surfaces!\n"
                ));
                break;
            }

            // `surf->shader = R_FindServerShader(token, lightmapsNone,
            // stylesDefault, qtrue)` — the live `gServerSkinHack` arm (module
            // doc-comment).
            let shader = self.find_server_shader(&String::from_utf8_lossy(&token));
            self.skins[h_skin as usize]
                .surfaces
                .push(ServerSkinSurface {
                    name: String::from_utf8_lossy(&surf_name).into_owned(),
                    shader,
                });
        }

        host.fs_free_file(text);

        // never let a skin have 0 shaders
        if self.skins[h_skin as usize].surfaces.is_empty() {
            return 0; // use default skin
        }

        h_skin
    }

    /// Raven `R_FindServerShader`, flattened to the name pool (user ruling
    /// 2026-07-12 (server skins name-pool)): pool entries carry ONLY the
    /// shader name — the sole field the dedicated path ever reads
    /// (`G2_surfaces.cpp:212`'s `"*off"` compare) — so `ClearGlobalShader` +
    /// `FinishShader` collapse into the push. Dedup follows `IsShader`
    /// (`tr_shader.cpp:3373-3396`): every server shader is `defaultShader =
    /// true` (`:3594`), for which `IsShader` matches on the case-insensitive
    /// stripped name alone (`lightmapIndex`/`styles` skipped — the skin path
    /// always passes `lightmapsNone`/`stylesDefault` anyway). Empty name →
    /// `tr.defaultShader`, pool slot 0.
    ///
    /// Source: `oracle/codemp/renderer/tr_shader.cpp:3560-3596`
    fn find_server_shader(&mut self, name: &str) -> usize {
        if name.is_empty() {
            return 0; // tr.defaultShader
        }

        let stripped_name = COM_StripExtension(name);

        // see if the shader is already loaded
        if let Some(index) = self
            .server_shaders
            .iter()
            .position(|s| s.eq_ignore_ascii_case(&stripped_name))
        {
            return index;
        }

        self.server_shaders.push(stripped_name);
        self.server_shaders.len() - 1
    }

    /// Raven `R_GetSkinByHandle`, flattened to the one read the dedicated
    /// consumer makes (`G2_SetSurfaceOnOffFromSkin`, `G2_surfaces.cpp:204-226`):
    /// the resolved skin's per-surface `(surface-name, shader-name)` rows.
    /// Out-of-range handles resolve to the default skin `tr.skins[0]`; an
    /// empty registry (`R_InitSkins` never run — Raven derefs a NULL
    /// `tr.skins[0]`) reads as no surfaces (§19).
    ///
    /// Source: `oracle/codemp/renderer/tr_image.cpp:3342-3347`
    pub fn skin_surfaces(&self, h_skin: qhandle_t) -> Vec<(String, String)> {
        let index = if h_skin < 1 || h_skin as usize >= self.skins.len() {
            0
        } else {
            h_skin as usize
        };
        let Some(skin) = self.skins.get(index) else {
            return Vec::new();
        };
        skin.surfaces
            .iter()
            .map(|surf| {
                (
                    surf.name.clone(),
                    self.server_shaders
                        .get(surf.shader)
                        .cloned()
                        .unwrap_or_default(),
                )
            })
            .collect()
    }
}

/// Raven `RE_SplitSkins` — "input = skinname, possibly being a macro for three
/// skins; return = true if three part skins found; output = qualified names to
/// three skins": `"models/players/jedi_tf/|head01|torso01|lower01"` splits
/// into `(head, torso, lower)` `.skin` paths. `None` covers Raven's
/// missing-separator `false` paths (the `assert`s are debug-only, dropped).
/// (Raven `strcat`s into `MAX_QPATH` stack buffers — over-long names overrun,
/// UB; owned `String`s keep the full text, §19.)
///
/// Source: `oracle/codemp/renderer/tr_image.cpp:2980-3024`
fn re_split_skins(name: &str) -> Option<(String, String, String)> {
    // INname = "models/players/jedi_tf/|head01_skin1|torso01|lower01";
    let p = name.find('|')?;
    // fill in the base path
    let base = &name[..p];
    let rest = &name[p + 1..];

    // now get the the individual files

    // advance to second
    let p2 = rest.find('|')?;
    let head = &rest[..p2];
    let rest = &rest[p2 + 1..];

    // advance to third
    let p3 = rest.find('|')?;
    let torso = &rest[..p3];
    let lower = &rest[p3 + 1..];

    Some((
        format!("{base}{head}.skin"),
        format!("{base}{torso}.skin"),
        format!("{base}{lower}.skin"),
    ))
}

/// Raven `CommaParse` — "This is unfortunate, but the skin files aren't
/// compatable with our normal parsing rules": a comma/whitespace tokenizer
/// with `//` and `/* */` comment skipping and quoted strings, over the raw
/// file bytes with a cursor. Reads at/past the buffer end behave as Raven's
/// NUL terminator; a token that fills all `MAX_TOKEN_CHARS` is discarded
/// (Raven's `len = 0`), exactly as the oracle's final length check does.
///
/// Source: `oracle/codemp/renderer/tr_image.cpp:3193-3292`
fn comma_parse(data: &[u8], pos: &mut usize) -> Vec<u8> {
    let at = |i: usize| -> u8 {
        if i < data.len() {
            data[i]
        } else {
            0
        }
    };

    let mut token: Vec<u8> = Vec::new();
    let mut c;

    loop {
        // skip whitespace
        loop {
            c = at(*pos);
            if c > b' ' || c == 0 {
                break;
            }
            *pos += 1;
        }

        c = at(*pos);

        // skip double slash comments
        if c == b'/' && at(*pos + 1) == b'/' {
            while at(*pos) != 0 && at(*pos) != b'\n' {
                *pos += 1;
            }
        }
        // skip /* */ comments
        else if c == b'/' && at(*pos + 1) == b'*' {
            while at(*pos) != 0 && (at(*pos) != b'*' || at(*pos + 1) != b'/') {
                *pos += 1;
            }
            if at(*pos) != 0 {
                *pos += 2;
            }
        } else {
            break;
        }
    }

    if c == 0 {
        return Vec::new();
    }

    // handle quoted strings
    if c == b'"' {
        *pos += 1;
        loop {
            c = at(*pos);
            *pos += 1;
            if c == b'"' || c == 0 {
                return token;
            }
            if token.len() < MAX_TOKEN_CHARS {
                token.push(c);
            }
        }
    }

    // parse a regular word
    loop {
        if token.len() < MAX_TOKEN_CHARS {
            token.push(c);
        }
        *pos += 1;
        c = at(*pos);
        if c <= 32 || c == b',' {
            break;
        }
    }

    if token.len() == MAX_TOKEN_CHARS {
        token.clear();
    }
    token
}

#[cfg(test)]
mod tests {
    use mp_host_interface::mock::MockHost;

    use super::*;

    fn seeded() -> RenderModels {
        let mut rm = RenderModels::default();
        rm.init_skins();
        rm
    }

    #[test]
    fn init_skins_seeds_default_skin_and_shader() {
        let rm = seeded();
        assert_eq!(rm.skins.len(), 1);
        assert_eq!(rm.skins[0].name, "<default skin>");
        assert_eq!(rm.skins[0].surfaces.len(), 1);
        assert_eq!(rm.server_shaders[0], DEFAULT_SHADER_NAME);
    }

    #[test]
    fn register_server_skin_parses_off_sentinels() {
        let mut rm = seeded();
        let mut host = MockHost::new().with_file(
            "models/players/test/model_default.skin",
            b"hips,models/players/test/hips.tga\n\
              torso_armor_off,*off\n\
              head,*off\n\
              tag_torso,\n"
                .to_vec(),
        );
        let h = rm.register_server_skin(&mut host, "models/players/test/model_default.skin");
        assert_eq!(h, 1);
        let rows = rm.skin_surfaces(h);
        // torso_armor_off + *off is a "double off" — skipped; tag_ rows skipped.
        assert_eq!(
            rows,
            vec![
                (
                    "hips".to_owned(),
                    "models/players/test/hips".to_owned() // extension stripped
                ),
                ("head".to_owned(), "*off".to_owned()),
            ]
        );
    }

    #[test]
    fn register_server_skin_dedups_by_name_and_reports_empty_as_default() {
        let mut rm = seeded();
        let mut host = MockHost::new().with_file("a.skin", b"face,gfx/face\n".to_vec());
        let h1 = rm.register_server_skin(&mut host, "a.skin");
        let h2 = rm.register_server_skin(&mut host, "A.SKIN"); // Q_stricmp dedup
        assert_eq!(h1, h2);
        // missing file: pooled with zero surfaces, handle 0 returned, and a
        // re-register still reads it as the default skin.
        assert_eq!(rm.register_server_skin(&mut host, "missing.skin"), 0);
        assert_eq!(rm.register_server_skin(&mut host, "missing.skin"), 0);
    }

    #[test]
    fn three_part_skins_split_and_append() {
        assert_eq!(
            re_split_skins("models/players/jedi_tf/|head01|torso01|lower01"),
            Some((
                "models/players/jedi_tf/head01.skin".to_owned(),
                "models/players/jedi_tf/torso01.skin".to_owned(),
                "models/players/jedi_tf/lower01.skin".to_owned(),
            ))
        );
        assert_eq!(
            re_split_skins("models/players/jedi_tf/|head01|torso01"),
            None
        );
        assert_eq!(re_split_skins("models/players/jedi_tf/model.skin"), None);

        let mut rm = seeded();
        let mut host = MockHost::new()
            .with_file("m/h.skin", b"head,*off\n".to_vec())
            .with_file("m/t.skin", b"torso,gfx/torso\n".to_vec())
            .with_file("m/l.skin", b"hips,gfx/hips\n".to_vec());
        let h = rm.register_server_skin(&mut host, "m/|h|t|l");
        assert_eq!(rm.skin_surfaces(h).len(), 3);
    }

    #[test]
    fn skin_surfaces_out_of_range_reads_default_skin() {
        let rm = seeded();
        assert_eq!(rm.skin_surfaces(99), rm.skin_surfaces(0));
        assert_eq!(rm.skin_surfaces(99).len(), 1);
        // pre-init registry reads as no surfaces (§19 divergence)
        assert!(RenderModels::default().skin_surfaces(1).is_empty());
    }

    #[test]
    fn comma_parse_handles_comments_quotes_and_commas() {
        let text = b"// comment\nhips,\"quoted token\" /* block */ next\n";
        let mut pos = 0;
        // word tokens stop before the ',' without consuming it (the caller's
        // `if (*text_p == ',') text_p++` step)
        assert_eq!(comma_parse(text, &mut pos), b"hips");
        assert_eq!(text[pos], b',');
        pos += 1;
        assert_eq!(comma_parse(text, &mut pos), b"quoted token");
        assert_eq!(comma_parse(text, &mut pos), b"next");
        assert_eq!(comma_parse(text, &mut pos), b"");
    }
}

// stubs.c — argless K&R-style definitions for every extern ui_shared.c
// REFERENCES but this harness's fixtures never actually CALL (paint, key
// handling, saber-glow caching, ...). Compiled WITHOUT any oracle headers on
// purpose: the C linker binds by symbol name alone, so an untyped definition
// here satisfies any caller's prototype as long as the body never touches
// its (unnamed) arguments. Every stub aborts loudly if actually invoked —
// see run.sh's header comment.
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>

static void loud(const char *name) {
	fprintf(stderr, "ui-oracle: unexpected call to %s (not on the tested parse path)\n", name);
	abort();
}

/* Com_Printf: real (not abort) — PC_SourceError/PC_SourceWarning route
   through it and DO run on the tested path (unknown-keyword recovery,
   truncated-source edge cases). Output goes to stderr, never stdout, so it
   cannot pollute the canonical dump. */
void Com_Printf(const char *fmt, ...) {
	va_list ap;
	va_start(ap, fmt);
	vfprintf(stderr, fmt, ap);
	va_end(ap);
}

/* Paint-only / saber-glow-cache / character-model surface: no fixture's
   Menu_New call reaches any of these (see main.cpp's header comment for
   which itemDef keywords were deliberately kept off these branches:
   isSaber/isSaber2 use value 0, asset_model's GLA-name branch is always
   empty). */
void AnglesToAxis() { loud("AnglesToAxis"); }
void AxisClear() { loud("AxisClear"); }
void UI_CacheSaberGlowGraphics() { loud("UI_CacheSaberGlowGraphics"); }
void UI_SaberLoadParms() { loud("UI_SaberLoadParms"); }
void UI_Cvar_VariableString() { loud("UI_Cvar_VariableString"); }
void UI_ParseAnimationFile() { loud("UI_ParseAnimationFile"); }
void UI_SaberAttachToChar() { loud("UI_SaberAttachToChar"); }
void UI_SaberDrawBlades() { loud("UI_SaberDrawBlades"); }
void UI_UpdateCharacterSkin() { loud("UI_UpdateCharacterSkin"); }

/* Data placeholders for globals ui_shared.c references only from dead
   branches on every fixture's tested path (paint functions; the
   isSaber/isSaber2 `if (i)` arm, which every fixture keeps false; the
   asset_model GLA-name branch, which trap_G2API_GetGLAName always empties
   out; ItemParse_cvarStrList's FEEDER_PLAYER_SPECIES/FEEDER_LANGUAGES
   shortcuts, which no fixture's cvarStrList uses). Never dereferenced, so an
   oversized untyped blob (not the real typed struct — this file has no
   headers on purpose, see the file header comment) is a safe placeholder;
   `animTable`, which genuinely IS read, gets a real typed definition in
   main.cpp instead. */
char uiInfo[65536];
char se_language[4096];
char ui_char_color_red[4096];
char ui_char_color_green[4096];
char ui_char_color_blue[4096];
int ui_saber_parms_parsed;
char bgAllAnims[65536];

/* strupr is an MSVC CRT extension; only referenced from
   ItemParse_cvarStrList's FEEDER_PLAYER_SPECIES branch (dead in every
   fixture), but a genuine prototype-satisfying definition is cheap and
   correct, so it gets one instead of an abort. */
char *strupr(char *s) {
	char *p = s;
	while (*p) {
		if (*p >= 'a' && *p <= 'z') *p -= 32;
		p++;
	}
	return s;
}

/* populated by iterating `run.sh`'s link step against undefined symbols */

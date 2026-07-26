// ui-oracle dumper — drives the UNMODIFIED Menu_New -> Menu_Parse ->
// dispatch_menu_keyword -> MenuParse_* -> MenuParse_itemDef -> Item_Parse ->
// dispatch_item_keyword -> ItemParse_* pipeline (oracle/codemp/ui/ui_shared.c)
// over a fixture and prints a canonical text dump of the resulting
// menuDef_t/itemDef_t state. See run.sh's header comment for the PC_*
// linking strategy and the stub-symbol rationale.
//
// Deterministic stand-ins (documented so the Rust test's TestDisplayContext
// can reproduce the SAME values — porting-rules golden parity requires the
// two sides' handle-valued fields to agree, not just be internally
// consistent):
//   - DC->registerShaderNoMip / registerModel / registerSound and
//     trap_R_RegisterSkin each hand out a monotonically increasing counter
//     from their own base (1000/2000/3000/4000), one call = one increment,
//     regardless of the name argument.
//   - DC->RegisterFont hands out a counter from base 5000.
//   - DC->getCVarString and trap_Cvar_VariableStringBuffer always return ""
//     (no cvar system is live here — every cvar reads as unset).
//   - DC->textWidth(text, scale, font) = strlen(text) * 8 * scale (used only
//     by the TEXTSCROLL line-breaker reached via Menu_PostParse).
//   - trap_AnyLanguage_ReadCharFromString decodes one raw byte per call
//     (advance = 1, not-trailing-punctuation), matching the Latin-1/no-Asian
//     fixture content.
//   - trap_Language_UsesSpaces always returns true.
//   - trap_G2API_InitGhoul2Model always "succeeds" (returns 0) and hands back
//     a fixed non-NULL sentinel pointer (never dereferenced or dumped — only
//     the resulting ITF_G2VALID flag bit is observable); trap_G2API_SetSkin/
//     SetBoneAnim return qtrue; trap_G2API_GetGLAName always returns "" (so
//     the animation-index branch that depends on it, and the large
//     `bgAllAnims`/animTable dependency behind it, is never reached — out of
//     scope for this harness); trap_G2API_CleanGhoul2Models nulls the
//     pointer; trap_G2_HaveWeGhoul2Models returns qfalse.
//   - Every other DC-> vtable entry and every other trap_* this TU never
//     calls on the fixtures' executed path aborts loudly if it IS ever
//     reached (a hard signal that a fixture strayed onto unmodeled surface,
//     not a silent wrong answer).
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "ui_local.h"
#include "botlib.h"
#include "pc_bridge.h"

// ---- globals ui_shared.c defines but no header declares ------------------
extern menuDef_t Menus[MAX_MENUS];
extern int menuCount;
extern displayContextDef_t *DC;

// ---- real, typed definitions for externs the tested path genuinely
// touches (data, not function surface — must have the correct layout,
// unlike stubs.c's untyped placeholders for symbols nothing ever
// dereferences) ----

// `void *BG_Alloc(int size)` (oracle/codemp/game/bg_public.h:1643) — a real
// forwarding malloc: ItemParse_asset_model_go's ITF_G2VALID-success branch
// calls UI_InsertG2Pointer, which allocates a `uiG2PtrTracker_t` through it.
void *BG_Alloc(int size) { return malloc((size_t)size); }

// Com_Memset/Com_Memcpy/Com_Error (q_shared.h:512-513,1767) are declared but
// never DEFINED in q_shared.c (retail links them from the engine's
// common.c, outside this harness's reach) — plain forwards, real (not
// abort) since q_shared.c's own string helpers are genuinely on the tested
// path (String_Alloc/Q_strncpyz/... run for every token).
void Com_Memset(void *dest, const int val, const size_t count) { memset(dest, val, count); }
void Com_Memcpy(void *dest, const void *src, const size_t count) { memcpy(dest, src, count); }
void Com_Error(int level, const char *error, ...) {
	(void)level;
	va_list ap;
	va_start(ap, error);
	vfprintf(stderr, error, ap);
	va_end(ap);
	fputc('\n', stderr);
	exit(1);
}

// `extern stringID_table_t animTable[MAX_ANIMATIONS+1]` (ui_shared.c:15) —
// ItemParse_model_g2anim/ItemParse_model_g2anim_go linearly scan this for a
// name match. Every fixture uses a deliberately-absent animation name (see
// fixture comments), so every entry just needs a non-NULL `.name` to keep
// Q_stricmp from dereferencing NULL — content is never meant to match.
stringID_table_t animTable[MAX_ANIMATIONS + 1];
static void InitAnimTable(void) {
	static const char empty[] = "";
	for (int i = 0; i <= MAX_ANIMATIONS; i++) {
		animTable[i].name = (char *)empty;
		animTable[i].id = 0;
	}
}

// ============================================================================
// botlib_import_t — LoadSourceMemory/PC_ReadTokenHandle never touch the
// filesystem (fixtures are loaded as memory buffers), so only Print and the
// Zone-memory hooks GetMemory/l_memory.cpp's allocator chain actually need
// are wired; FS_* stay NULL and are never called.
// ============================================================================
botlib_import_t botimport;

static void QDECL botimport_Print(int type, char *fmt, ...) {
	char buf[4096];
	va_list ap;
	va_start(ap, fmt);
	vsnprintf(buf, sizeof(buf), fmt, ap);
	va_end(ap);
	fputs(buf, stderr);
}
static void *botimport_GetMemory(int size) { return malloc((size_t)size); }
static void botimport_FreeMemory(void *ptr) { free(ptr); }
static int botimport_AvailableMemory(void) { return 0x7fffffff; }

static void InitBotImport(void) {
	memset(&botimport, 0, sizeof(botimport));
	botimport.Print = botimport_Print;
	botimport.GetMemory = botimport_GetMemory;
	botimport.FreeMemory = botimport_FreeMemory;
	botimport.AvailableMemory = botimport_AvailableMemory;
}

// ============================================================================
// trap_PC_ReadToken / trap_PC_SourceFileAndLine — the two REAL forwards onto
// botlib's unmodified preprocessor (l_precomp.cpp), via pc_bridge.cpp (the
// one C++-compiled TU — see pc_bridge.h). Every other trap_* below is a
// deterministic stand-in or a loud abort.
// ============================================================================
int trap_PC_ReadToken(int handle, pc_token_t *pc_token) { return ui_oracle_PC_ReadTokenHandle(handle, pc_token); }
int trap_PC_SourceFileAndLine(int handle, char *filename, int *line) { return ui_oracle_PC_SourceFileAndLine(handle, filename, line); }

static void aborting(const char *what) {
	fprintf(stderr, "ui-oracle: unexpected call to %s (not on the tested parse path)\n", what);
	abort();
}

// ---- deterministic counters ----
static int g_shaderCounter = 1000;
static int g_modelCounter = 2000;
static int g_soundCounter = 3000;
static int g_skinCounter = 4000;
static int g_fontCounter = 5000;
static char g_g2Sentinel; // never dereferenced; only its non-NULL-ness is observable via ITF_G2VALID

// ---- meaningful trap_* stand-ins ----
qhandle_t trap_R_RegisterSkin(const char *name) { (void)name; return g_skinCounter++; }

int trap_G2API_InitGhoul2Model(void **ghoul2Ptr, const char *fileName, int modelIndex, qhandle_t customSkin,
								qhandle_t customShader, int modelFlags, int lodBias) {
	(void)fileName; (void)modelIndex; (void)customSkin; (void)customShader; (void)modelFlags; (void)lodBias;
	*ghoul2Ptr = &g_g2Sentinel;
	return 0;
}
qboolean trap_G2API_SetSkin(void *ghoul2, int modelIndex, qhandle_t customSkin, qhandle_t renderSkin) {
	(void)ghoul2; (void)modelIndex; (void)customSkin; (void)renderSkin;
	return qtrue;
}
void trap_G2API_CleanGhoul2Models(void **ghoul2Ptr) { if (ghoul2Ptr) *ghoul2Ptr = NULL; }
qboolean trap_G2API_SetBoneAnim(void *ghoul2, const int modelIndex, const char *boneName, const int startFrame,
								 const int endFrame, const int flags, const float animSpeed, const int currentTime,
								 const float setFrame, const int blendTime) {
	(void)ghoul2; (void)modelIndex; (void)boneName; (void)startFrame; (void)endFrame; (void)flags;
	(void)animSpeed; (void)currentTime; (void)setFrame; (void)blendTime;
	return qtrue;
}
void trap_G2API_GetGLAName(void *ghoul2, int modelIndex, char *fillBuf) { (void)ghoul2; (void)modelIndex; fillBuf[0] = 0; }
qboolean trap_G2_HaveWeGhoul2Models(void *ghoul2) { (void)ghoul2; return qfalse; }

void trap_Cvar_VariableStringBuffer(const char *var_name, char *buffer, int bufsize) {
	(void)var_name;
	if (bufsize > 0) buffer[0] = 0;
}
qboolean trap_Language_UsesSpaces(void) { return qtrue; }
unsigned int trap_AnyLanguage_ReadCharFromString(const char *psText, int *piAdvanceCount, qboolean *pbIsTrailingPunctuation) {
	*piAdvanceCount = 1;
	if (pbIsTrailingPunctuation) *pbIsTrailingPunctuation = qfalse;
	return (unsigned char)psText[0];
}
int trap_SP_GetStringTextString(const char *text, char *buffer, int bufferLength) {
	(void)text;
	if (bufferLength > 0) buffer[0] = 0;
	return 0;
}
void trap_Print(const char *string) { fputs(string, stderr); }

// ---- never exercised by our fixtures on purpose (see fixture comments); loud if wrong ----
void trap_Cvar_Set(const char *var_name, const char *value) { (void)var_name; (void)value; aborting("trap_Cvar_Set"); }
void trap_Cvar_SetValue(const char *var_name, float value) { (void)var_name; (void)value; aborting("trap_Cvar_SetValue"); }
float trap_Cvar_VariableValue(const char *var_name) { (void)var_name; aborting("trap_Cvar_VariableValue"); return 0; }
void trap_GetLanguageName(const int languageIndex, char *buffer) { (void)languageIndex; (void)buffer; aborting("trap_GetLanguageName"); }
qboolean trap_Key_IsDown(int keynum) { (void)keynum; aborting("trap_Key_IsDown"); return qfalse; }
int trap_Key_GetCatcher(void) { aborting("trap_Key_GetCatcher"); return 0; }
void trap_Key_SetCatcher(int catcher) { (void)catcher; aborting("trap_Key_SetCatcher"); }
void trap_Key_ClearStates(void) { aborting("trap_Key_ClearStates"); }
int trap_Milliseconds(void) { aborting("trap_Milliseconds"); return 0; }
void trap_S_StartLocalSound(sfxHandle_t sfx, int channelNum) { (void)sfx; (void)channelNum; aborting("trap_S_StartLocalSound"); }

// ============================================================================
// displayContextDef_t — one static instance, installed as the file-scope DC.
// registerShaderNoMip/registerModel/registerSound/RegisterFont/getCVarString/
// textWidth are meaningful (see file header); everything else aborts loudly.
// ============================================================================
static qhandle_t dc_registerShaderNoMip(const char *p) { (void)p; return g_shaderCounter++; }
static void dc_setColor(const vec4_t v) { (void)v; aborting("DC->setColor"); }
static void dc_drawHandlePic(float x, float y, float w, float h, qhandle_t asset) { (void)x; (void)y; (void)w; (void)h; (void)asset; aborting("DC->drawHandlePic"); }
static void dc_drawStretchPic(float x, float y, float w, float h, float s1, float t1, float s2, float t2, qhandle_t hShader) {
	(void)x; (void)y; (void)w; (void)h; (void)s1; (void)t1; (void)s2; (void)t2; (void)hShader;
	aborting("DC->drawStretchPic");
}
static void dc_drawText(float x, float y, float scale, vec4_t color, const char *text, float adjust, int limit, int style, int iMenuFont) {
	(void)x; (void)y; (void)scale; (void)color; (void)text; (void)adjust; (void)limit; (void)style; (void)iMenuFont;
	aborting("DC->drawText");
}
static int dc_textWidth(const char *text, float scale, int iMenuFont) { (void)iMenuFont; return (int)(strlen(text) * 8.0f * scale); }
static int dc_textHeight(const char *text, float scale, int iMenuFont) { (void)text; (void)scale; (void)iMenuFont; aborting("DC->textHeight"); return 0; }
static qhandle_t dc_registerModel(const char *p) { (void)p; return g_modelCounter++; }
static void dc_modelBounds(qhandle_t model, vec3_t min, vec3_t max) { (void)model; (void)min; (void)max; aborting("DC->modelBounds"); }
static void dc_fillRect(float x, float y, float w, float h, const vec4_t color) { (void)x; (void)y; (void)w; (void)h; (void)color; aborting("DC->fillRect"); }
static void dc_drawRect(float x, float y, float w, float h, float size, const vec4_t color) { (void)x; (void)y; (void)w; (void)h; (void)size; (void)color; aborting("DC->drawRect"); }
static void dc_drawSides(float x, float y, float w, float h, float size) { (void)x; (void)y; (void)w; (void)h; (void)size; aborting("DC->drawSides"); }
static void dc_drawTopBottom(float x, float y, float w, float h, float size) { (void)x; (void)y; (void)w; (void)h; (void)size; aborting("DC->drawTopBottom"); }
static void dc_clearScene(void) { aborting("DC->clearScene"); }
static void dc_addRefEntityToScene(const refEntity_t *re) { (void)re; aborting("DC->addRefEntityToScene"); }
static void dc_renderScene(const refdef_t *fd) { (void)fd; aborting("DC->renderScene"); }
static qhandle_t dc_RegisterFont(const char *fontName) { (void)fontName; return g_fontCounter++; }
static int dc_Font_StrLenPixels(const char *text, const int iFontIndex, const float scale) { (void)text; (void)iFontIndex; (void)scale; aborting("DC->Font_StrLenPixels"); return 0; }
static int dc_Font_StrLenChars(const char *text) { (void)text; aborting("DC->Font_StrLenChars"); return 0; }
static int dc_Font_HeightPixels(const int iFontIndex, const float scale) { (void)iFontIndex; (void)scale; aborting("DC->Font_HeightPixels"); return 0; }
static void dc_Font_DrawString(int ox, int oy, const char *text, const float *rgba, const int setIndex, int iCharLimit, const float scale) {
	(void)ox; (void)oy; (void)text; (void)rgba; (void)setIndex; (void)iCharLimit; (void)scale;
	aborting("DC->Font_DrawString");
}
static qboolean dc_Language_IsAsian(void) { aborting("DC->Language_IsAsian"); return qfalse; }
static qboolean dc_Language_UsesSpaces(void) { aborting("DC->Language_UsesSpaces"); return qfalse; }
static unsigned int dc_AnyLanguage_ReadCharFromString(const char *psText, int *piAdvanceCount, qboolean *pbIsTrailingPunctuation) {
	(void)psText; (void)piAdvanceCount; (void)pbIsTrailingPunctuation;
	aborting("DC->AnyLanguage_ReadCharFromString");
	return 0;
}
static void dc_ownerDrawItem(float x, float y, float w, float h, float text_x, float text_y, int ownerDraw, int ownerDrawFlags,
							  int align, float special, float scale, vec4_t color, qhandle_t shader, int textStyle, int iMenuFont) {
	(void)x; (void)y; (void)w; (void)h; (void)text_x; (void)text_y; (void)ownerDraw; (void)ownerDrawFlags;
	(void)align; (void)special; (void)scale; (void)color; (void)shader; (void)textStyle; (void)iMenuFont;
	aborting("DC->ownerDrawItem");
}
static float dc_getValue(int ownerDraw) { (void)ownerDraw; aborting("DC->getValue"); return 0; }
static qboolean dc_ownerDrawVisible(int flags) { (void)flags; aborting("DC->ownerDrawVisible"); return qfalse; }
static void dc_runScript(char **p) { (void)p; aborting("DC->runScript"); }
static qboolean dc_deferScript(char **p) { (void)p; aborting("DC->deferScript"); return qfalse; }
static void dc_getTeamColor(vec4_t *color) { (void)color; aborting("DC->getTeamColor"); }
static void dc_getCVarString(const char *cvar, char *buffer, int bufsize) { (void)cvar; if (bufsize > 0) buffer[0] = 0; }
static float dc_getCVarValue(const char *cvar) { (void)cvar; aborting("DC->getCVarValue"); return 0; }
static void dc_setCVar(const char *cvar, const char *value) { (void)cvar; (void)value; aborting("DC->setCVar"); }
static void dc_drawTextWithCursor(float x, float y, float scale, vec4_t color, const char *text, int cursorPos, char cursor, int limit, int style, int iFontIndex) {
	(void)x; (void)y; (void)scale; (void)color; (void)text; (void)cursorPos; (void)cursor; (void)limit; (void)style; (void)iFontIndex;
	aborting("DC->drawTextWithCursor");
}
static void dc_setOverstrikeMode(qboolean b) { (void)b; aborting("DC->setOverstrikeMode"); }
static qboolean dc_getOverstrikeMode(void) { aborting("DC->getOverstrikeMode"); return qfalse; }
static void dc_startLocalSound(sfxHandle_t sfx, int channelNum) { (void)sfx; (void)channelNum; aborting("DC->startLocalSound"); }
static qboolean dc_ownerDrawHandleKey(int ownerDraw, int flags, float *special, int key) {
	(void)ownerDraw; (void)flags; (void)special; (void)key;
	aborting("DC->ownerDrawHandleKey");
	return qfalse;
}
static int dc_feederCount(float feederID) { (void)feederID; aborting("DC->feederCount"); return 0; }
static const char *dc_feederItemText(float feederID, int index, int column, qhandle_t *handle1, qhandle_t *handle2, qhandle_t *handle3) {
	(void)feederID; (void)index; (void)column; (void)handle1; (void)handle2; (void)handle3;
	aborting("DC->feederItemText");
	return NULL;
}
static qhandle_t dc_feederItemImage(float feederID, int index) { (void)feederID; (void)index; aborting("DC->feederItemImage"); return 0; }
static qboolean dc_feederSelection(float feederID, int index, itemDef_t *item) { (void)feederID; (void)index; (void)item; aborting("DC->feederSelection"); return qfalse; }
static void dc_keynumToStringBuf(int keynum, char *buf, int buflen) { (void)keynum; (void)buf; (void)buflen; aborting("DC->keynumToStringBuf"); }
static void dc_getBindingBuf(int keynum, char *buf, int buflen) { (void)keynum; (void)buf; (void)buflen; aborting("DC->getBindingBuf"); }
static void dc_setBinding(int keynum, const char *binding) { (void)keynum; (void)binding; aborting("DC->setBinding"); }
static void dc_executeText(int exec_when, const char *text) { (void)exec_when; (void)text; aborting("DC->executeText"); }
static void dc_Error(int level, const char *error, ...) { (void)level; (void)error; aborting("DC->Error"); }
static void dc_Print(const char *msg, ...) { (void)msg; aborting("DC->Print"); }
static void dc_Pause(qboolean b) { (void)b; aborting("DC->Pause"); }
static int dc_ownerDrawWidth(int ownerDraw, float scale) { (void)ownerDraw; (void)scale; aborting("DC->ownerDrawWidth"); return 0; }
static sfxHandle_t dc_registerSound(const char *name) { (void)name; return g_soundCounter++; }
static void dc_startBackgroundTrack(const char *intro, const char *loop, qboolean bReturnWithoutStarting) {
	(void)intro; (void)loop; (void)bReturnWithoutStarting;
	aborting("DC->startBackgroundTrack");
}
static void dc_stopBackgroundTrack(void) { aborting("DC->stopBackgroundTrack"); }
static int dc_playCinematic(const char *name, float x, float y, float w, float h) { (void)name; (void)x; (void)y; (void)w; (void)h; aborting("DC->playCinematic"); return 0; }
static void dc_stopCinematic(int handle) { (void)handle; aborting("DC->stopCinematic"); }
static void dc_drawCinematic(int handle, float x, float y, float w, float h) { (void)handle; (void)x; (void)y; (void)w; (void)h; aborting("DC->drawCinematic"); }
static void dc_runCinematicFrame(int handle) { (void)handle; aborting("DC->runCinematicFrame"); }

static displayContextDef_t g_DC;

static void InitDC(void) {
	memset(&g_DC, 0, sizeof(g_DC));
	g_DC.registerShaderNoMip = dc_registerShaderNoMip;
	g_DC.setColor = dc_setColor;
	g_DC.drawHandlePic = dc_drawHandlePic;
	g_DC.drawStretchPic = dc_drawStretchPic;
	g_DC.drawText = dc_drawText;
	g_DC.textWidth = dc_textWidth;
	g_DC.textHeight = dc_textHeight;
	g_DC.registerModel = dc_registerModel;
	g_DC.modelBounds = dc_modelBounds;
	g_DC.fillRect = dc_fillRect;
	g_DC.drawRect = dc_drawRect;
	g_DC.drawSides = dc_drawSides;
	g_DC.drawTopBottom = dc_drawTopBottom;
	g_DC.clearScene = dc_clearScene;
	g_DC.addRefEntityToScene = dc_addRefEntityToScene;
	g_DC.renderScene = dc_renderScene;
	g_DC.RegisterFont = dc_RegisterFont;
	g_DC.Font_StrLenPixels = dc_Font_StrLenPixels;
	g_DC.Font_StrLenChars = dc_Font_StrLenChars;
	g_DC.Font_HeightPixels = dc_Font_HeightPixels;
	g_DC.Font_DrawString = dc_Font_DrawString;
	g_DC.Language_IsAsian = dc_Language_IsAsian;
	g_DC.Language_UsesSpaces = dc_Language_UsesSpaces;
	g_DC.AnyLanguage_ReadCharFromString = dc_AnyLanguage_ReadCharFromString;
	g_DC.ownerDrawItem = dc_ownerDrawItem;
	g_DC.getValue = dc_getValue;
	g_DC.ownerDrawVisible = dc_ownerDrawVisible;
	g_DC.runScript = dc_runScript;
	g_DC.deferScript = dc_deferScript;
	g_DC.getTeamColor = dc_getTeamColor;
	g_DC.getCVarString = dc_getCVarString;
	g_DC.getCVarValue = dc_getCVarValue;
	g_DC.setCVar = dc_setCVar;
	g_DC.drawTextWithCursor = dc_drawTextWithCursor;
	g_DC.setOverstrikeMode = dc_setOverstrikeMode;
	g_DC.getOverstrikeMode = dc_getOverstrikeMode;
	g_DC.startLocalSound = dc_startLocalSound;
	g_DC.ownerDrawHandleKey = dc_ownerDrawHandleKey;
	g_DC.feederCount = dc_feederCount;
	g_DC.feederItemText = dc_feederItemText;
	g_DC.feederItemImage = dc_feederItemImage;
	g_DC.feederSelection = dc_feederSelection;
	g_DC.keynumToStringBuf = dc_keynumToStringBuf;
	g_DC.getBindingBuf = dc_getBindingBuf;
	g_DC.setBinding = dc_setBinding;
	g_DC.executeText = dc_executeText;
	g_DC.Error = dc_Error;
	g_DC.Print = dc_Print;
	g_DC.Pause = dc_Pause;
	g_DC.ownerDrawWidth = dc_ownerDrawWidth;
	g_DC.registerSound = dc_registerSound;
	g_DC.startBackgroundTrack = dc_startBackgroundTrack;
	g_DC.stopBackgroundTrack = dc_stopBackgroundTrack;
	g_DC.playCinematic = dc_playCinematic;
	g_DC.stopCinematic = dc_stopCinematic;
	g_DC.drawCinematic = dc_drawCinematic;
	g_DC.runCinematicFrame = dc_runCinematicFrame;
	g_DC.realTime = 0;
	g_DC.frameTime = 0;
	g_DC.cursorx = 0;
	g_DC.cursory = 0;
	g_DC.debug = qfalse;
	// g_DC.Assets is left zeroed: fadeAmount/fadeClamp/fadeCycle default to 0
	// (Menu_Init reads them), fontRegistered defaults to qfalse (MenuParse_font
	// registers exactly once per process, matching one fixture per process).
	DC = &g_DC;
}

// ============================================================================
// Canonical dump — see tools/ui-oracle/README.md for the format. Field order
// mirrors the Rust ItemDef/MenuDef/WindowDef struct declarations exactly
// (crates/mp/uishared/src/shared/{item_def_s,menu_def_t,window_def_t}.rs) so
// the two dumpers are trivially eyeballable side by side.
//
// String fields: Raven's NULL default is either preserved as `<NULL>` (the
// Rust struct field is `Option<String>`) or normalized to the empty string
// (the Rust field is a plain `String` initialized to "") -- pstr_opt vs
// pstr_def picks the one matching the corresponding Rust field's type.
// ============================================================================
static void pesc(FILE *out, const char *s) {
	fputc('|', out);
	if (s) {
		for (const unsigned char *p = (const unsigned char *)s; *p; p++) {
			unsigned char c = *p;
			if (c == '|' || c == '\\' || c < 0x20 || c >= 0x7f) {
				fprintf(out, "\\x%02x", c);
			} else {
				fputc(c, out);
			}
		}
	}
	fputc('|', out);
}
static void pstr_opt(FILE *out, const char *label, const char *s) {
	fprintf(out, "%s: ", label);
	if (!s) {
		fprintf(out, "<NULL>\n");
	} else {
		pesc(out, s);
		fputc('\n', out);
	}
}
static void pstr_def(FILE *out, const char *label, const char *s) { pstr_opt(out, label, s ? s : ""); }
static void pint(FILE *out, const char *label, int v) { fprintf(out, "%s: %d\n", label, v); }
static void pfloat(FILE *out, const char *label, float v) { fprintf(out, "%s: %.6f\n", label, v); }
static void pbool(FILE *out, const char *label, qboolean v) { fprintf(out, "%s: %s\n", label, v ? "true" : "false"); }
static void prect(FILE *out, const char *label, const rectDef_t *r) {
	fprintf(out, "%s: (%.6f, %.6f, %.6f, %.6f)\n", label, r->x, r->y, r->w, r->h);
}
static void pvec4(FILE *out, const char *label, const float *v) {
	fprintf(out, "%s: (%.6f, %.6f, %.6f, %.6f)\n", label, v[0], v[1], v[2], v[3]);
}
static void pvec3(FILE *out, const char *label, const float *v) {
	fprintf(out, "%s: (%.6f, %.6f, %.6f)\n", label, v[0], v[1], v[2]);
}

static void dumpWindow(FILE *out, const windowDef_t *w) {
	prect(out, "  window.rect", &w->rect);
	prect(out, "  window.rectClient", &w->rectClient);
	pstr_opt(out, "  window.name", w->name);
	pstr_opt(out, "  window.group", w->group);
	pstr_def(out, "  window.cinematicName", w->cinematicName);
	pint(out, "  window.cinematic", w->cinematic);
	pint(out, "  window.style", w->style);
	pint(out, "  window.border", w->border);
	pint(out, "  window.ownerDraw", w->ownerDraw);
	pint(out, "  window.ownerDrawFlags", w->ownerDrawFlags);
	pfloat(out, "  window.borderSize", w->borderSize);
	pint(out, "  window.flags", w->flags);
	prect(out, "  window.rectEffects", &w->rectEffects);
	prect(out, "  window.rectEffects2", &w->rectEffects2);
	pint(out, "  window.offsetTime", w->offsetTime);
	pint(out, "  window.nextTime", w->nextTime);
	pvec4(out, "  window.foreColor", w->foreColor);
	pvec4(out, "  window.backColor", w->backColor);
	pvec4(out, "  window.borderColor", w->borderColor);
	pvec4(out, "  window.outlineColor", w->outlineColor);
	pint(out, "  window.background", w->background);
}

static void dumpTypeData(FILE *out, const itemDef_t *item) {
	if (!item->typeData) {
		fprintf(out, "  typeData: none\n");
		return;
	}
	if (item->type == ITEM_TYPE_LISTBOX) {
		const listBoxDef_t *l = (const listBoxDef_t *)item->typeData;
		fprintf(out, "  typeData: listbox\n");
		pint(out, "    startPos", l->startPos);
		pint(out, "    endPos", l->endPos);
		pint(out, "    drawPadding", l->drawPadding);
		pint(out, "    cursorPos", l->cursorPos);
		pfloat(out, "    elementWidth", l->elementWidth);
		pfloat(out, "    elementHeight", l->elementHeight);
		pint(out, "    elementStyle", l->elementStyle);
		pint(out, "    numColumns", l->numColumns);
		for (int i = 0; i < l->numColumns && i < MAX_LB_COLUMNS; i++) {
			fprintf(out, "    columnInfo[%d]: pos=%d width=%d maxChars=%d\n",
					i, l->columnInfo[i].pos, l->columnInfo[i].width, l->columnInfo[i].maxChars);
		}
		pstr_def(out, "    doubleClick", l->doubleClick);
		pbool(out, "    notselectable", l->notselectable);
		pbool(out, "    scrollhidden", l->scrollhidden);
	} else if (item->type == ITEM_TYPE_EDITFIELD || item->type == ITEM_TYPE_NUMERICFIELD ||
			   item->type == ITEM_TYPE_YESNO || item->type == ITEM_TYPE_BIND ||
			   item->type == ITEM_TYPE_SLIDER || item->type == ITEM_TYPE_TEXT) {
		const editFieldDef_t *e = (const editFieldDef_t *)item->typeData;
		fprintf(out, "  typeData: editfield\n");
		pfloat(out, "    minVal", e->minVal);
		pfloat(out, "    maxVal", e->maxVal);
		pfloat(out, "    defVal", e->defVal);
		pfloat(out, "    range", e->range);
		pint(out, "    maxChars", e->maxChars);
		pint(out, "    maxPaintChars", e->maxPaintChars);
		pint(out, "    paintOffset", e->paintOffset);
	} else if (item->type == ITEM_TYPE_MULTI) {
		const multiDef_t *m = (const multiDef_t *)item->typeData;
		fprintf(out, "  typeData: multi\n");
		pbool(out, "    strDef", m->strDef);
		pint(out, "    count", m->count);
		for (int i = 0; i < m->count && i < MAX_MULTI_CVARS; i++) {
			fprintf(out, "    cvarList[%d]: ", i);
			pesc(out, m->cvarList[i]);
			fputc('\n', out);
			if (m->strDef) {
				fprintf(out, "    cvarStr[%d]: ", i);
				pesc(out, m->cvarStr[i]);
				fputc('\n', out);
			} else {
				fprintf(out, "    cvarValue[%d]: %.6f\n", i, m->cvarValue[i]);
			}
		}
	} else if (item->type == ITEM_TYPE_MODEL) {
		const modelDef_t *md = (const modelDef_t *)item->typeData;
		fprintf(out, "  typeData: model\n");
		pint(out, "    angle", md->angle);
		pvec3(out, "    origin", md->origin);
		pfloat(out, "    fov_x", md->fov_x);
		pfloat(out, "    fov_y", md->fov_y);
		pint(out, "    rotationSpeed", md->rotationSpeed);
		pvec3(out, "    g2mins", md->g2mins);
		pvec3(out, "    g2maxs", md->g2maxs);
		pvec3(out, "    g2scale", md->g2scale);
		pint(out, "    g2skin", md->g2skin);
		pint(out, "    g2anim", md->g2anim);
		pvec3(out, "    g2mins2", md->g2mins2);
		pvec3(out, "    g2maxs2", md->g2maxs2);
		pvec3(out, "    g2minsEffect", md->g2minsEffect);
		pvec3(out, "    g2maxsEffect", md->g2maxsEffect);
		pfloat(out, "    fov_x2", md->fov_x2);
		pfloat(out, "    fov_y2", md->fov_y2);
		pfloat(out, "    fov_Effectx", md->fov_Effectx);
		pfloat(out, "    fov_Effecty", md->fov_Effecty);
	} else if (item->type == ITEM_TYPE_TEXTSCROLL) {
		const textScrollDef_t *t = (const textScrollDef_t *)item->typeData;
		fprintf(out, "  typeData: textscroll\n");
		pint(out, "    startPos", t->startPos);
		pint(out, "    endPos", t->endPos);
		pfloat(out, "    lineHeight", t->lineHeight);
		pint(out, "    maxLineChars", t->maxLineChars);
		pint(out, "    drawPadding", t->drawPadding);
		pint(out, "    iLineCount", t->iLineCount);
		for (int i = 0; i < t->iLineCount && i < MAX_TEXTSCROLL_LINES; i++) {
			if (!t->pLines[i]) continue;
			fprintf(out, "    pLines[%d]: ", i);
			pesc(out, t->pLines[i]);
			fputc('\n', out);
		}
	} else {
		fprintf(out, "  typeData: unrecognized-type-%d (typeData non-NULL but no known payload shape)\n", item->type);
	}
}

static void dumpItem(FILE *out, int idx, const itemDef_t *item) {
	fprintf(out, " == item %d ==\n", idx);
	dumpWindow(out, &item->window);
	prect(out, "  textRect", &item->textRect);
	pint(out, "  type", item->type);
	pint(out, "  alignment", item->alignment);
	pint(out, "  textalignment", item->textalignment);
	pfloat(out, "  textalignx", item->textalignx);
	pfloat(out, "  textaligny", item->textaligny);
	pfloat(out, "  textscale", item->textscale);
	pint(out, "  textStyle", item->textStyle);
	pstr_opt(out, "  text", item->text);
	pstr_def(out, "  text2", item->text2);
	pfloat(out, "  text2alignx", item->text2alignx);
	pfloat(out, "  text2aligny", item->text2aligny);
	pint(out, "  asset", item->asset);
	pint(out, "  flags", item->flags);
	pstr_def(out, "  mouseEnterText", item->mouseEnterText);
	pstr_def(out, "  mouseExitText", item->mouseExitText);
	pstr_def(out, "  mouseEnter", item->mouseEnter);
	pstr_def(out, "  mouseExit", item->mouseExit);
	pstr_def(out, "  action", item->action);
	pstr_def(out, "  accept", item->accept);
	pstr_def(out, "  selectionNext", item->selectionNext);
	pstr_def(out, "  selectionPrev", item->selectionPrev);
	pstr_def(out, "  onFocus", item->onFocus);
	pstr_def(out, "  leaveFocus", item->leaveFocus);
	pstr_opt(out, "  cvar", item->cvar);
	pstr_def(out, "  cvarTest", item->cvarTest);
	pstr_def(out, "  enableCvar", item->enableCvar);
	pint(out, "  cvarFlags", item->cvarFlags);
	pint(out, "  focusSound", item->focusSound);
	pint(out, "  numColors", item->numColors);
	for (int i = 0; i < item->numColors && i < MAX_COLOR_RANGES; i++) {
		fprintf(out, "  colorRanges[%d]: low=%.6f high=%.6f color=(%.6f, %.6f, %.6f, %.6f)\n",
				i, item->colorRanges[i].low, item->colorRanges[i].high,
				item->colorRanges[i].color[0], item->colorRanges[i].color[1],
				item->colorRanges[i].color[2], item->colorRanges[i].color[3]);
	}
	pfloat(out, "  special", item->special);
	pint(out, "  cursorPos", item->cursorPos);
	dumpTypeData(out, item);
	pstr_def(out, "  descText", item->descText);
	pint(out, "  appearanceSlot", item->appearanceSlot);
	pint(out, "  iMenuFont", item->iMenuFont);
	pbool(out, "  disabled", item->disabled);
	pint(out, "  invertYesNo", item->invertYesNo);
	pint(out, "  xoffset", item->xoffset);
}

static void dumpMenu(FILE *out, int idx, const menuDef_t *menu) {
	printf("== menu %d ==\n", idx);
	dumpWindow(out, &menu->window);
	pstr_def(out, "font", menu->font);
	pbool(out, "fullScreen", menu->fullScreen);
	pint(out, "fontIndex", menu->fontIndex);
	pint(out, "cursorItem", menu->cursorItem);
	pint(out, "fadeCycle", menu->fadeCycle);
	pfloat(out, "fadeClamp", menu->fadeClamp);
	pfloat(out, "fadeAmount", menu->fadeAmount);
	pstr_def(out, "onOpen", menu->onOpen);
	pstr_def(out, "onClose", menu->onClose);
	pstr_def(out, "onAccept", menu->onAccept);
	pstr_def(out, "onESC", menu->onESC);
	pstr_def(out, "soundName", menu->soundName);
	pvec4(out, "focusColor", menu->focusColor);
	pvec4(out, "disableColor", menu->disableColor);
	pint(out, "itemCount", menu->itemCount);
	pint(out, "descX", menu->descX);
	pint(out, "descY", menu->descY);
	pvec4(out, "descColor", menu->descColor);
	pint(out, "descAlignment", menu->descAlignment);
	pfloat(out, "descScale", menu->descScale);
	pfloat(out, "appearanceTime", menu->appearanceTime);
	pint(out, "appearanceCnt", menu->appearanceCnt);
	pfloat(out, "appearanceIncrement", menu->appearanceIncrement);
	for (int i = 0; i < menu->itemCount && i < MAX_MENUITEMS; i++) {
		dumpItem(out, i, menu->items[i]);
	}
}

int main(int argc, char **argv) {
	if (argc != 3) {
		fprintf(stderr, "usage: %s <fixture.menu> <menu_new_attempts>\n", argv[0]);
		return 2;
	}
	const char *fixturePath = argv[1];
	int attempts = atoi(argv[2]);

	FILE *f = fopen(fixturePath, "rb");
	if (!f) {
		fprintf(stderr, "cannot open %s\n", fixturePath);
		return 2;
	}
	fseek(f, 0, SEEK_END);
	long size = ftell(f);
	fseek(f, 0, SEEK_SET);
	char *buf = (char *)malloc((size_t)size + 1);
	fread(buf, 1, (size_t)size, f);
	buf[size] = 0;
	fclose(f);

	InitBotImport();
	InitAnimTable();
	// String_Init (ui_shared.c:363-378) builds the menu/item keyword hash
	// tables (Item_SetupKeywordHash/Menu_SetupKeywordHash) that
	// dispatch_menu_keyword/dispatch_item_keyword's KeywordHash_Find need —
	// without it every keyword looks unrecognized. Called BEFORE InitDC so
	// its `if (DC && DC->getBindingBuf) Controls_GetConfig()` guard sees a
	// still-NULL DC (matching the real engine's boot order, where String_Init
	// runs before display init) and skips a control-config path this harness
	// never wires up.
	String_Init();
	InitDC();

	// One shared source handle at index 1, installed directly into
	// l_precomp.cpp's own handle table (bypassing trap_PC_LoadSource's
	// filesystem path entirely — see run.sh's header comment).
	ui_oracle_install_source(1, buf, (int)size, fixturePath);

	for (int i = 0; i < attempts; i++) {
		int before = menuCount;
		Menu_New(1);
		int ok = (menuCount > before);
		printf("== attempt %d: %s (menuCount now %d) ==\n", i, ok ? "ok" : "error", menuCount);
	}

	printf("== menuCount %d ==\n", menuCount);
	for (int i = 0; i < menuCount && i < MAX_MENUS; i++) {
		dumpMenu(stdout, i, &Menus[i]);
	}
	printf("== end ==\n");

	return 0;
}

// bg_saberLoad differential-oracle dumper. Compiled against the UNMODIFIED
// Raven bg_saberLoad.c + q_shared.c (copied into build/ by run.sh) with
// QAGAME defined (jampgame == Raven's QAGAME build). The TU extern-declares
// its engine traps; this file provides deterministic STUB DEFINITIONS backed
// by the fixtures/ directory, plus the two lookup tables the linker demands
// (FPTable here; animTable via animtable_def.c). It drives the real oracle
// load path — WP_SaberLoadParms() to fill the SaberParms buffer from
// fixtures/sabers/*.sab exactly as the game does, then WP_SaberParseParms()
// per saber name — and prints every saberInfo_t field in declaration order
// (floats as IEEE-754 bit-hex, strings quoted, arrays indexed) plus the
// sound/skin registration logs. The Rust parity test reproduces this dump by
// driving the ported crate::bg_saberLoad over the same fixtures.
//
// Registration observability: trap_R_RegisterSkin is a name-logging counter
// (skins genuinely cross the observable BgTraps seam, so both sides return a
// deterministic per-saber counter). G_SoundIndex (behind BG_SoundIndex) is a
// name-logging observer that returns 0 — matching the port, whose
// G_SoundIndex is still a documented placeholder returning 0 (configstring
// architecture unwired). See README.md.
#include "q_shared.h"
#include "bg_public.h"
#include "dumpcommon.h"

#include <dirent.h>
#include <stdarg.h>

// Forward decls for the two bg_saberLoad.c entry points we drive (their real
// prototypes live in w_saber.h, which we don't include here).
qboolean WP_SaberParseParms(const char *SaberName, saberInfo_t *saber);
void     WP_SaberLoadParms(void);

// ---- Raven externs the TU needs but that live in other TUs / the engine ----

// FPTable — force-power name/id table (oracle bg_saga.c:100-121). Built with
// the same ENUM2STRING macro Raven uses; forcePowers_t is in scope via
// bg_public.h. Matches the port's FPTable (crates/mp/game/src/bg_saga.rs).
stringID_table_t FPTable[] =
{
	ENUM2STRING(FP_HEAL),
	ENUM2STRING(FP_LEVITATION),
	ENUM2STRING(FP_SPEED),
	ENUM2STRING(FP_PUSH),
	ENUM2STRING(FP_PULL),
	ENUM2STRING(FP_TELEPATHY),
	ENUM2STRING(FP_GRIP),
	ENUM2STRING(FP_LIGHTNING),
	ENUM2STRING(FP_RAGE),
	ENUM2STRING(FP_PROTECT),
	ENUM2STRING(FP_ABSORB),
	ENUM2STRING(FP_TEAM_HEAL),
	ENUM2STRING(FP_TEAM_FORCE),
	ENUM2STRING(FP_DRAIN),
	ENUM2STRING(FP_SEE),
	ENUM2STRING(FP_SABER_OFFENSE),
	ENUM2STRING(FP_SABER_DEFENSE),
	ENUM2STRING(FP_SABERTHROW),
	"",	-1
};

// Com_Printf/Com_Error: parser diagnostics. Routed to STDERR so they never
// pollute the golden (stdout). The port's Com_Printf likewise prints to
// stderr; its Com_Error panics — so fixtures must never trigger Com_Error
// (illegal numBlades / too-large buffer). This stub aborts loudly for the
// same reason (a triggered Com_Error is a fixture bug, not observable data).
void Com_Printf(const char *fmt, ...) {
	va_list ap; va_start(ap, fmt); vfprintf(stderr, fmt, ap); va_end(ap);
}
void Com_Error(int level, const char *fmt, ...) {
	va_list ap; va_start(ap, fmt);
	fprintf(stderr, "Com_Error(%d): ", level); vfprintf(stderr, fmt, ap);
	fprintf(stderr, "\n"); va_end(ap);
	exit(3);
}

// Q_irand — used ONLY by TranslateSaberColor's "random" color. Fixtures never
// use "random" (it would need seed-matched RNG on both sides), so this is a
// link-satisfying stub that is never called. Deterministic just in case.
int Q_irand(int min, int max) { (void)max; return min; }

// ------------------------- fixtures-backed FS traps -------------------------

static const char *g_fixdir; // argv[1]; sabers live in <g_fixdir>/sabers

// Map a Raven vpath ("ext_data/sabers[/name]") onto the fixtures tree by
// stripping the leading "ext_data/" and prefixing g_fixdir.
static void mappath(const char *vpath, char *out, size_t outsz) {
	const char *rel = vpath;
	const char *pfx = "ext_data/";
	if (strncmp(vpath, pfx, strlen(pfx)) == 0) rel = vpath + strlen(pfx);
	snprintf(out, outsz, "%s/%s", g_fixdir, rel);
}

#define MAXH 64
static FILE *g_handles[MAXH];

static int cmpstr(const void *a, const void *b) {
	return strcmp(*(const char *const *)a, *(const char *const *)b);
}

int trap_FS_GetFileList(const char *path, const char *extension, char *listbuf, int bufsize) {
	char dir[1024];
	mappath(path, dir, sizeof(dir));
	DIR *d = opendir(dir);
	if (!d) return 0;
	static char names[MAXH][256];
	char *ptrs[MAXH];
	int n = 0;
	struct dirent *e;
	size_t extlen = strlen(extension);
	while ((e = readdir(d)) && n < MAXH) {
		size_t l = strlen(e->d_name);
		if (l < extlen) continue;
		if (strcmp(e->d_name + (l - extlen), extension) != 0) continue;
		strncpy(names[n], e->d_name, sizeof(names[n]) - 1);
		names[n][sizeof(names[n]) - 1] = 0;
		ptrs[n] = names[n];
		n++;
	}
	closedir(d);
	// Deterministic order (readdir is unordered); the Rust side sorts too.
	qsort(ptrs, n, sizeof(ptrs[0]), cmpstr);
	int off = 0;
	for (int i = 0; i < n; i++) {
		int len = (int)strlen(ptrs[i]);
		if (off + len + 1 > bufsize) break;
		memcpy(listbuf + off, ptrs[i], len);
		listbuf[off + len] = 0;
		off += len + 1;
	}
	return n;
}

int trap_FS_FOpenFile(const char *qpath, fileHandle_t *f, fsMode_t mode) {
	(void)mode;
	char real[1024];
	mappath(qpath, real, sizeof(real));
	FILE *fp = fopen(real, "rb");
	if (!fp) { if (f) *f = 0; return -1; }
	fseek(fp, 0, SEEK_END);
	long len = ftell(fp);
	fseek(fp, 0, SEEK_SET);
	int h = 0;
	for (int i = 1; i < MAXH; i++) { if (!g_handles[i]) { h = i; break; } }
	if (!h) { fclose(fp); if (f) *f = 0; return -1; }
	g_handles[h] = fp;
	if (f) *f = h;
	return (int)len;
}

void trap_FS_Read(void *buffer, int len, fileHandle_t f) {
	if (f <= 0 || f >= MAXH || !g_handles[f]) return;
	fread(buffer, 1, len, g_handles[f]);
}

void trap_FS_Write(const void *buffer, int len, fileHandle_t f) {
	(void)buffer; (void)len; (void)f; // never exercised by the load path
}

void trap_FS_FCloseFile(fileHandle_t f) {
	if (f <= 0 || f >= MAXH || !g_handles[f]) return;
	fclose(g_handles[f]);
	g_handles[f] = NULL;
}

// ------------------- registration observers (per-saber) ---------------------

static char g_snd[512][256]; static int g_sndn;                 // sound names
static char g_skn[64][256];  static int g_sknid[64]; static int g_sknn, g_sknctr;

static void reg_reset(void) { g_sndn = 0; g_sknn = 0; g_sknctr = 0; }

// G_SoundIndex — QAGAME target of BG_SoundIndex. Observer: logs the name,
// returns 0 (matches the port's placeholder G_SoundIndex).
int G_SoundIndex(const char *name) {
	if (g_sndn < 512) { strncpy(g_snd[g_sndn], name ? name : "", 255); g_snd[g_sndn][255] = 0; g_sndn++; }
	return 0;
}

// trap_R_RegisterSkin — observable BgTraps seam. Per-saber counter (skins
// genuinely return renderer handles; both sides mint the same deterministic
// id) plus a name log.
qhandle_t trap_R_RegisterSkin(const char *name) {
	int id = ++g_sknctr;
	if (g_sknn < 64) { strncpy(g_skn[g_sknn], name ? name : "", 255); g_skn[g_sknn][255] = 0; g_sknid[g_sknn] = id; g_sknn++; }
	return id;
}

// ------------------------------- the dump -----------------------------------

static void qstr(const char *tag, const char *s) { printf("%s \"%s\"\n", tag, s); }
static void pi(const char *tag, int v) { printf("%s %d\n", tag, v); }
static void pfh(const char *tag, float v) { printf("%s %08x\n", tag, f2b(v)); }

static void dump_saber(const char *reqname) {
	reg_reset();
	saberInfo_t saber;
	memset(&saber, 0, sizeof(saber));
	qboolean ret = WP_SaberParseParms(reqname, &saber);

	printf("saber \"%s\"\n", reqname);
	pi("ret", ret ? 1 : 0);
	qstr("name", saber.name);
	qstr("fullName", saber.fullName);
	pi("type", (int)saber.type);
	qstr("model", saber.model);
	pi("skin", saber.skin);
	pi("soundOn", saber.soundOn);
	pi("soundLoop", saber.soundLoop);
	pi("soundOff", saber.soundOff);
	pi("numBlades", saber.numBlades);
	for (int i = 0; i < MAX_BLADES; i++) {
		printf("blade%d color %d radius %08x lengthMax %08x\n",
			i, (int)saber.blade[i].color, f2b(saber.blade[i].radius), f2b(saber.blade[i].lengthMax));
	}
	pi("stylesLearned", saber.stylesLearned);
	pi("stylesForbidden", saber.stylesForbidden);
	pi("maxChain", saber.maxChain);
	pi("forceRestrictions", saber.forceRestrictions);
	pi("lockBonus", saber.lockBonus);
	pi("parryBonus", saber.parryBonus);
	pi("breakParryBonus", saber.breakParryBonus);
	pi("breakParryBonus2", saber.breakParryBonus2);
	pi("disarmBonus", saber.disarmBonus);
	pi("disarmBonus2", saber.disarmBonus2);
	pi("singleBladeStyle", (int)saber.singleBladeStyle);
	pi("saberFlags", saber.saberFlags);
	pi("saberFlags2", saber.saberFlags2);
	pi("spinSound", saber.spinSound);
	printf("swingSound %d %d %d\n", saber.swingSound[0], saber.swingSound[1], saber.swingSound[2]);
	pfh("moveSpeedScale", saber.moveSpeedScale);
	pfh("animSpeedScale", saber.animSpeedScale);
	pi("kataMove", saber.kataMove);
	pi("lungeAtkMove", saber.lungeAtkMove);
	pi("jumpAtkUpMove", saber.jumpAtkUpMove);
	pi("jumpAtkFwdMove", saber.jumpAtkFwdMove);
	pi("jumpAtkBackMove", saber.jumpAtkBackMove);
	pi("jumpAtkRightMove", saber.jumpAtkRightMove);
	pi("jumpAtkLeftMove", saber.jumpAtkLeftMove);
	pi("readyAnim", saber.readyAnim);
	pi("drawAnim", saber.drawAnim);
	pi("putawayAnim", saber.putawayAnim);
	pi("tauntAnim", saber.tauntAnim);
	pi("bowAnim", saber.bowAnim);
	pi("meditateAnim", saber.meditateAnim);
	pi("flourishAnim", saber.flourishAnim);
	pi("gloatAnim", saber.gloatAnim);
	pi("bladeStyle2Start", saber.bladeStyle2Start);
	pi("trailStyle", saber.trailStyle);
	pi("g2MarksShader", saber.g2MarksShader);
	pi("g2WeaponMarkShader", saber.g2WeaponMarkShader);
	printf("hitSound %d %d %d\n", saber.hitSound[0], saber.hitSound[1], saber.hitSound[2]);
	printf("blockSound %d %d %d\n", saber.blockSound[0], saber.blockSound[1], saber.blockSound[2]);
	printf("bounceSound %d %d %d\n", saber.bounceSound[0], saber.bounceSound[1], saber.bounceSound[2]);
	pi("blockEffect", saber.blockEffect);
	pi("hitPersonEffect", saber.hitPersonEffect);
	pi("hitOtherEffect", saber.hitOtherEffect);
	pi("bladeEffect", saber.bladeEffect);
	pfh("knockbackScale", saber.knockbackScale);
	pfh("damageScale", saber.damageScale);
	pfh("splashRadius", saber.splashRadius);
	pi("splashDamage", saber.splashDamage);
	pfh("splashKnockback", saber.splashKnockback);
	pi("trailStyle2", saber.trailStyle2);
	pi("g2MarksShader2", saber.g2MarksShader2);
	pi("g2WeaponMarkShader2", saber.g2WeaponMarkShader2);
	printf("hit2Sound %d %d %d\n", saber.hit2Sound[0], saber.hit2Sound[1], saber.hit2Sound[2]);
	printf("block2Sound %d %d %d\n", saber.block2Sound[0], saber.block2Sound[1], saber.block2Sound[2]);
	printf("bounce2Sound %d %d %d\n", saber.bounce2Sound[0], saber.bounce2Sound[1], saber.bounce2Sound[2]);
	pi("blockEffect2", saber.blockEffect2);
	pi("hitPersonEffect2", saber.hitPersonEffect2);
	pi("hitOtherEffect2", saber.hitOtherEffect2);
	pi("bladeEffect2", saber.bladeEffect2);
	pfh("knockbackScale2", saber.knockbackScale2);
	pfh("damageScale2", saber.damageScale2);
	pfh("splashRadius2", saber.splashRadius2);
	pi("splashDamage2", saber.splashDamage2);
	pfh("splashKnockback2", saber.splashKnockback2);

	for (int i = 0; i < g_sknn; i++) printf("regskin %d \"%s\"\n", g_sknid[i], g_skn[i]);
	for (int i = 0; i < g_sndn; i++) printf("regsound \"%s\"\n", g_snd[i]);
	printf("--\n");
}

// The saber names parsed, in order. Mirrors tests/jampgame_parity.rs.
static const char *g_names[] = {
	"Kyle",             // found — realistic single
	"staff_saber",      // found — dual/staff, secondary-blade fields
	"edge_saber",       // found — clamps, unknown tokens, anims, customSkin
	"broken_saber",     // truncated block -> unexpected-EOF -> qfalse
	"nonexistent_xyz",  // not found -> DEFAULT_SABER fallback to "Kyle"
	"",                 // empty name -> DEFAULT_SABER ("Kyle") immediately
};

int main(int argc, char **argv) {
	if (argc != 2) { fprintf(stderr, "usage: %s <fixture-dir>\n", argv[0]); return 2; }
	g_fixdir = argv[1];
	WP_SaberLoadParms();
	printf("== saberload ==\n");
	for (unsigned i = 0; i < sizeof(g_names)/sizeof(g_names[0]); i++) {
		dump_saber(g_names[i]);
	}
	printf("== end ==\n");
	return 0;
}

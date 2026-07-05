// main_pmove.c -- pmove single-step differential dumper. Compiled with -DQAGAME
// (jampgame == Raven's QAGAME) it links the UNMODIFIED oracle pmove closure
// (bg_pmove.c, bg_slidemove.c, bg_panimate.c, bg_saber.c, bg_saberLoad.c,
// bg_misc.c, bg_weapons.c, q_shared.c, q_math.c -- see run.sh) and drives the
// real Pmove() over fixtures, dumping the playerState_t / pmove_t fields that
// change during basic on-foot movement as IEEE-754 bit-hex. The Rust parity
// test reproduces this by driving crate::bg_pmove::Pmove over the same
// fixtures + the same synthetic animation.cfg.
//
// This TU provides everything the closure extern-references that lives in other
// (unlinked) game TUs or the engine:
//   * zeroed g_entities[] / level / g_gametype / bg_fighterAltControl /
//     g_vehWeaponInfo -- the data globals the closure reads.
//   * FPTable (force-power name table) -- byte-identical to bg_saga's.
//   * the world trace / pointcontents / snap_vector, from pmworld.h, wired into
//     pmove_t.trace / .pointcontents / trap_SnapVector.
//   * fixtures-backed trap_FS_* (only the animation.cfg load path uses them).
//   * a 32-bit Q_irand mirroring Raven's holdrand LCG + a draw counter, so the
//     `rng` tripwire is observable (q_math.c's own Q_irand is renamed o_Q_irand
//     by run.sh to avoid a duplicate symbol -- see README "RNG tripwire").
//   * abort()ing stubs for EVERY other extern (G_*/NPC_*/Q3_*/vehicle/ghoul2/
//     trap_*). A firing stub == a fixture that leaked off the basic on-foot
//     path; the abort makes it loud and greppable.
//
// Anim mirror rule: bg_panimate's QAGAME restart-check reads
// g_entities[clientNum].s.legsAnim/torsoAnim (what BG_PlayerStateToEntityState
// writes live at the end of each server frame). We reproduce that by copying
// ps.legsAnim/torsoAnim into g_entities[0].s AFTER every Pmove, so the next
// step's restart-check sees the previous frame's value. The Rust
// TestCallbacks::entity_legs/torso_anim returns the same mirror.
#include "q_shared.h"
#include "bg_public.h"
#include "bg_local.h"
#include "g_local.h"
#include "pmworld.h"
#include "dumpcommon.h"

#include <ctype.h>
#include <stdarg.h>
#include <stddef.h>

// ======================= data globals the closure needs =======================

gentity_t        g_entities[MAX_GENTITIES];
level_locals_t   level;
vmCvar_t         g_gametype;
vmCvar_t         bg_fighterAltControl;
vehWeaponInfo_t  g_vehWeaponInfo[MAX_VEH_WEAPONS];

// FPTable -- force-power name/id table (oracle bg_saga.c:100-121), same
// ENUM2STRING form the port's bg_saga::FPTable uses.
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

// ============================ diagnostics / RNG ==============================

void Com_Printf(const char *fmt, ...) {
	va_list ap; va_start(ap, fmt); vfprintf(stderr, fmt, ap); va_end(ap);
}
void Com_Error(int level_, const char *fmt, ...) {
	va_list ap; va_start(ap, fmt);
	fprintf(stderr, "Com_Error(%d): ", level_); vfprintf(stderr, fmt, ap);
	fprintf(stderr, "\n"); va_end(ap);
	exit(3);
}

// RNG tripwire: Raven's holdrand LCG normalized to 32-bit (the port's u32
// model). Never drawn on the basic path -> holdrand stays 0x89abcdef and
// g_rngDraws stays 0. If a draw ever happens, both move and the diff catches it.
static unsigned int g_holdrand = 0x89abcdef;
static long         g_rngDraws = 0;

void Rand_Init(int seed) { g_holdrand = (unsigned int)seed; }
float flrand(float min, float max) {
	float result;
	g_rngDraws++;
	g_holdrand = (g_holdrand * 214013u) + 2531011u;
	result = (float)(g_holdrand >> 17);
	result = ((result * (max - min)) / 32768.0f) + min;
	return result;
}
float Q_flrand(float min, float max) { return flrand(min, max); }
int irand(int min, int max) {
	int result;
	g_rngDraws++;
	max++;
	g_holdrand = (g_holdrand * 214013u) + 2531011u;
	result = (int)(g_holdrand >> 17);
	result = ((result * (max - min)) >> 15) + min;
	return result;
}
int Q_irand(int value1, int value2) { return irand(value1, value2); }

// ========================= fixtures-backed FS traps ==========================
// Only the animation.cfg load path exercises these; everything is served out of
// the fixture directory (argv[2]). A missing file returns -1 so optional loads
// (animevents.cfg) are skipped gracefully.

static const char *g_fixdir;

static void map_anim_path(const char *vpath, char *out, size_t outsz) {
	// The only file we serve is animation.cfg; map any request to
	// <fixdir>/<basename>. This keeps the synthetic cfg self-contained.
	const char *base = strrchr(vpath, '/');
	base = base ? base + 1 : vpath;
	snprintf(out, outsz, "%s/%s", g_fixdir, base);
}

#define MAXH 8
static FILE *g_handles[MAXH];

int trap_FS_FOpenFile(const char *qpath, fileHandle_t *fh, fsMode_t mode) {
	char real[1024];
	FILE *fp;
	long len;
	int h;
	if (mode != FS_READ) { if (fh) *fh = 0; return -1; }
	map_anim_path(qpath, real, sizeof(real));
	fp = fopen(real, "rb");
	if (!fp) { if (fh) *fh = 0; return -1; }
	fseek(fp, 0, SEEK_END); len = ftell(fp); fseek(fp, 0, SEEK_SET);
	h = 0;
	for (int i = 1; i < MAXH; i++) { if (!g_handles[i]) { h = i; break; } }
	if (!h) { fclose(fp); if (fh) *fh = 0; return -1; }
	g_handles[h] = fp;
	if (fh) *fh = h;
	return (int)len;
}
void trap_FS_Read(void *buffer, int len, fileHandle_t f) {
	if (f <= 0 || f >= MAXH || !g_handles[f]) return;
	fread(buffer, 1, len, g_handles[f]);
}
void trap_FS_Write(const void *buffer, int len, fileHandle_t f) {
	(void)buffer; (void)len; (void)f;
}
void trap_FS_FCloseFile(fileHandle_t f) {
	if (f <= 0 || f >= MAXH || !g_handles[f]) return;
	fclose(g_handles[f]); g_handles[f] = NULL;
}
int trap_FS_GetFileList(const char *path, const char *ext, char *listbuf, int bufsize) {
	(void)path; (void)ext; (void)listbuf; (void)bufsize; return 0;
}

// ================================ SnapVector =================================

void trap_SnapVector(float *v) { pmw_snapvector(v); }

// ============================== abort() stubs ================================
// Reaching any of these means a fixture left the basic on-foot path.

#define STUB(sig, name) sig { fprintf(stderr, "pmove-oracle: STUB " #name " reached -- fixture off the basic path\n"); abort(); }

STUB(void Client_CheckImpactBBrush(gentity_t *self, gentity_t *other), Client_CheckImpactBBrush)
STUB(void G_CheapWeaponFire(int entNum, int ev), G_CheapWeaponFire)
STUB(gentity_t *G_PlayEffect(int fxID, vec3_t org, vec3_t ang), G_PlayEffect)
STUB(gentity_t *G_PlayEffectID(const int fxID, vec3_t org, vec3_t ang), G_PlayEffectID)
STUB(void G_AddEvent(gentity_t *ent, int event, int eventParm), G_AddEvent)
STUB(void G_Damage(gentity_t *targ, gentity_t *inflictor, gentity_t *attacker, vec3_t dir, vec3_t point, int damage, int dflags, int mod), G_Damage)
STUB(void G_DamageFromKiller(gentity_t *pEnt, gentity_t *pVehEnt, gentity_t *attacker, vec3_t org, int damage, int dflags, int mod), G_DamageFromKiller)
STUB(qboolean G_CanBeEnemy(gentity_t *self, gentity_t *enemy), G_CanBeEnemy)
STUB(void G_FlyVehicleSurfaceDestruction(gentity_t *veh, trace_t *trace, int magnitude, qboolean force), G_FlyVehicleSurfaceDestruction)
STUB(char *G_NewString(const char *string), G_NewString)
STUB(int G_SoundIndex(const char *name), G_SoundIndex)
STUB(void NPC_SetAnim(gentity_t *ent, int type, int anim, int priority), NPC_SetAnim)
STUB(void Q3_SetParm(int entID, int parmNum, const char *parmValue), Q3_SetParm)
STUB(qboolean TryGrapple(gentity_t *ent), TryGrapple)
STUB(void WP_GetVehicleCamPos(gentity_t *ent, gentity_t *pilot, vec3_t camPos), WP_GetVehicleCamPos)
STUB(qboolean FighterIsLanded(Vehicle_t *pVeh, playerState_t *parentPS), FighterIsLanded)
STUB(qhandle_t trap_R_RegisterSkin(const char *name), trap_R_RegisterSkin)
STUB(void trap_Trace(trace_t *results, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end, int passEntityNum, int contentmask), trap_Trace)
STUB(void trap_FX_PlayEffect(const char *file, vec3_t org, vec3_t fwd, int vol, int rad), trap_FX_PlayEffect)

// ghoul2 seam -- never touched on the on-foot path (bg_saber weapon-swap code)
STUB(int trap_G2API_InitGhoul2Model(void **ghoul2Ptr, const char *fileName, int modelIndex, qhandle_t customSkin, qhandle_t customShader, int modelFlags, int lodBias), trap_G2API_InitGhoul2Model)
STUB(void trap_G2API_CleanGhoul2Models(void **ghoul2Ptr), trap_G2API_CleanGhoul2Models)
STUB(qboolean strap_G2API_GetBoltMatrix(void *ghoul2, const int modelIndex, const int boltIndex, mdxaBone_t *matrix, const vec3_t angles, const vec3_t position, const int frameNum, qhandle_t *modelList, vec3_t scale), strap_G2API_GetBoltMatrix)
STUB(qboolean strap_G2API_GetBoltMatrix_NoRecNoRot(void *ghoul2, const int modelIndex, const int boltIndex, mdxaBone_t *matrix, const vec3_t angles, const vec3_t position, const int frameNum, qhandle_t *modelList, vec3_t scale), strap_G2API_GetBoltMatrix_NoRecNoRot)
STUB(qboolean strap_G2API_SetBoneAngles(void *ghoul2, int modelIndex, const char *boneName, const vec3_t angles, const int flags, const int up, const int right, const int forward, qhandle_t *modelList, int blendTime, int currentTime), strap_G2API_SetBoneAngles)
STUB(qboolean strap_G2API_SetBoneAnim(void *ghoul2, const int modelIndex, const char *boneName, const int startFrame, const int endFrame, const int flags, const float animSpeed, const int currentTime, const float setFrame, const int blendTime), strap_G2API_SetBoneAnim)
STUB(qboolean strap_G2API_GetBoneAnim(void *ghoul2, const char *boneName, const int currentTime, float *currentFrame, int *startFrame, int *endFrame, int *flags, float *animSpeed, int *modelList, const int modelIndex), strap_G2API_GetBoneAnim)
STUB(void strap_G2API_AnimateG2Models(void *ghoul2, int time, sharedRagDollUpdateParams_t *params), strap_G2API_AnimateG2Models)
STUB(qboolean strap_G2API_SetBoneIKState(void *ghoul2, int time, const char *boneName, int ikState, sharedSetBoneIKStateParams_t *params), strap_G2API_SetBoneIKState)
STUB(qboolean strap_G2API_IKMove(void *ghoul2, int time, sharedIKMoveParams_t *params), strap_G2API_IKMove)

// ============================ fixture parsing ================================

static float parse_float(const char *tok) {
	if (tok[0] == '0' && (tok[1] == 'x' || tok[1] == 'X')) {
		union { float f; unsigned u; } u;
		u.u = (unsigned)strtoul(tok, NULL, 16);
		return u.f;
	}
	return (float)atol(tok);
}
static int parse_int(const char *tok) { return (int)strtol(tok, NULL, 0); }

static int tokenize(char *buf, char *tok[], int maxtok) {
	int n = 0; char *p = buf;
	while (*p && n < maxtok) {
		while (*p && isspace((unsigned char)*p)) p++;
		if (!*p) break;
		tok[n++] = p;
		while (*p && !isspace((unsigned char)*p)) p++;
		if (*p) *p++ = 0;
	}
	return n;
}

// playerState_t override table -----------------------------------------------
enum { K_INT, K_FLOAT, K_VEC3F, K_VEC3I };
typedef struct { const char *name; size_t off; int kind; } psfield_t;

static playerState_t g_ps;   // the one playerState_t we drive

static const psfield_t g_psfields[] = {
	{ "origin",          offsetof(playerState_t, origin),          K_VEC3F },
	{ "velocity",        offsetof(playerState_t, velocity),        K_VEC3F },
	{ "viewangles",      offsetof(playerState_t, viewangles),      K_VEC3F },
	{ "delta_angles",    offsetof(playerState_t, delta_angles),    K_VEC3I },
	{ "groundEntityNum", offsetof(playerState_t, groundEntityNum), K_INT   },
	{ "pm_flags",        offsetof(playerState_t, pm_flags),        K_INT   },
	{ "pm_type",         offsetof(playerState_t, pm_type),         K_INT   },
	{ "legsAnim",        offsetof(playerState_t, legsAnim),        K_INT   },
	{ "torsoAnim",       offsetof(playerState_t, torsoAnim),       K_INT   },
	{ "weapon",          offsetof(playerState_t, weapon),          K_INT   },
	{ "gravity",         offsetof(playerState_t, gravity),         K_INT   },
	{ "speed",           offsetof(playerState_t, speed),           K_FLOAT },
	{ "basespeed",       offsetof(playerState_t, basespeed),       K_INT   },
	{ "fallingToDeath",  offsetof(playerState_t, fallingToDeath),  K_INT   },
	{ "clientNum",       offsetof(playerState_t, clientNum),       K_INT   },
};

static void apply_ps_override(char *tok[], int n) {
	for (unsigned i = 0; i < sizeof(g_psfields)/sizeof(g_psfields[0]); i++) {
		if (strcmp(tok[1], g_psfields[i].name)) continue;
		char *base = (char *)&g_ps + g_psfields[i].off;
		switch (g_psfields[i].kind) {
			case K_INT:   *(int *)base = parse_int(tok[2]); break;
			case K_FLOAT: *(float *)base = parse_float(tok[2]); break;
			case K_VEC3F: ((float *)base)[0]=parse_float(tok[2]); ((float *)base)[1]=parse_float(tok[3]); ((float *)base)[2]=parse_float(tok[4]); break;
			case K_VEC3I: ((int *)base)[0]=parse_int(tok[2]); ((int *)base)[1]=parse_int(tok[3]); ((int *)base)[2]=parse_int(tok[4]); break;
		}
		return;
	}
	fprintf(stderr, "pmove-oracle: unknown ps field '%s'\n", tok[1]);
	exit(2);
}

// ================================ baseline ==================================
// The zero+pins baseline every fixture starts from (spec section 2). Overrides
// are applied on top.

static void ps_baseline(void) {
	memset(&g_ps, 0, sizeof(g_ps));
	g_ps.pm_type          = PM_NORMAL;
	g_ps.weapon           = WP_MELEE;
	g_ps.weaponstate      = WEAPON_READY;
	g_ps.stats[STAT_HEALTH] = 100;
	g_ps.gravity          = 800;
	g_ps.speed            = 250.0f;
	g_ps.basespeed        = 250;
	g_ps.standheight      = DEFAULT_MAXS_2;   // 40
	g_ps.crouchheight     = CROUCH_MAXS_2;    // 16
	g_ps.viewheight       = DEFAULT_VIEWHEIGHT;
	g_ps.groundEntityNum  = ENTITYNUM_NONE;
	g_ps.clientNum        = 0;
	g_ps.m_iVehicleNum    = 0;
	g_ps.commandTime      = 0;
	g_ps.legsAnim         = 0;
	g_ps.torsoAnim        = 0;
}

// ================================== dump ====================================

static void dump_step(int stepNo, const pmove_t *pm) {
	const playerState_t *ps = pm->ps;
	printf("s=%d t=%d org=%08x,%08x,%08x vel=%08x,%08x,%08x va=%08x,%08x,%08x "
	       "da=%d,%d,%d gnd=%d pmf=%x pmt=%d la=%d:%d ta=%d:%d fl=%d%d bob=%d "
	       "vh=%d ef=%x seq=%d ev=%d:%d,%d:%d wt=%d ws=%d spd=%08x wl=%d wtp=%d "
	       "nt=%d mn=%08x mx=%08x xy=%08x air=%d f2d=%d fjz=%08x ntr=%ld rng=%08x\n",
	       stepNo, ps->commandTime,
	       f2b(ps->origin[0]), f2b(ps->origin[1]), f2b(ps->origin[2]),
	       f2b(ps->velocity[0]), f2b(ps->velocity[1]), f2b(ps->velocity[2]),
	       f2b(ps->viewangles[0]), f2b(ps->viewangles[1]), f2b(ps->viewangles[2]),
	       ps->delta_angles[0], ps->delta_angles[1], ps->delta_angles[2],
	       ps->groundEntityNum, ps->pm_flags, ps->pm_time,
	       ps->legsAnim, ps->legsTimer, ps->torsoAnim, ps->torsoTimer,
	       ps->legsFlip ? 1 : 0, ps->torsoFlip ? 1 : 0,
	       ps->bobCycle, ps->viewheight, ps->eFlags, ps->eventSequence,
	       ps->events[0], ps->eventParms[0], ps->events[1], ps->eventParms[1],
	       ps->weaponTime, ps->weaponstate, f2b(ps->speed),
	       pm->waterlevel, pm->watertype,
	       pm->numtouch, f2b(pm->mins[2]), f2b(pm->maxs[2]), f2b(pm->xyspeed),
	       ps->inAirAnim ? 1 : 0, ps->fallingToDeath, f2b(ps->fd.forceJumpZStart),
	       g_pmw_traceCount, g_holdrand);
}

// ================================== main ====================================

extern animation_t bgHumanoidAnimations[MAX_TOTALANIMATIONS];

int main(int argc, char **argv) {
	FILE *f;
	char line[2048];
	pmove_t pm;
	int prevServerTime = 0;
	int stepNo = 0;

	if (argc != 3) { fprintf(stderr, "usage: %s <fixture-file> <fixture-dir>\n", argv[0]); return 2; }
	g_fixdir = argv[2];

	// Load the synthetic humanoid animation set (both sides parse the same file).
	if (BG_ParseAnimationFile("models/players/_humanoid/animation.cfg", bgHumanoidAnimations, qtrue) == -1) {
		fprintf(stderr, "pmove-oracle: failed to load animation.cfg from %s\n", g_fixdir);
		return 2;
	}

	// Baseline pins, then apply the fixture's world + ps overrides.
	pmw_reset_world();
	ps_baseline();

	f = fopen(argv[1], "rb");
	if (!f) { fprintf(stderr, "cannot open %s\n", argv[1]); return 2; }

	// Set up the pmove_t skeleton (cmd rows patch cmd fields per step).
	memset(&pm, 0, sizeof(pm));
	pm.ps            = &g_ps;
	pm.trace         = pm_trace;
	pm.pointcontents = pm_pointcontents;
	pm.tracemask     = MASK_PLAYERSOLID;
	pm.animations    = bgHumanoidAnimations;
	pm.baseEnt       = (bgEntity_t *)g_entities;
	pm.entSize       = sizeof(gentity_t);
	pm.gametype      = 0;

	printf("== pmove ==\n");

	while (fgets(line, sizeof(line), f)) {
		char *tok[32];
		int n;
		char *hash = strchr(line, '#');
		if (hash) *hash = 0;
		n = tokenize(line, tok, 32);
		if (n == 0) continue;

		if (!strcmp(tok[0], "brush") && n >= 7) {
			int surf = 0;
			if (n >= 8) { const char *eq = strchr(tok[7], '='); if (eq) surf = (int)strtol(eq+1, NULL, 0); }
			pmw_add_brush(parse_float(tok[1]), parse_float(tok[2]), parse_float(tok[3]),
			              parse_float(tok[4]), parse_float(tok[5]), parse_float(tok[6]), surf);
		} else if (!strcmp(tok[0], "ps") && n >= 3) {
			apply_ps_override(tok, n);
		} else if (!strcmp(tok[0], "start")) {
			// Freeze the entity anim mirror to the initial ps values and emit
			// step 0 (the pre-move state) so the golden shows the baseline.
			g_entities[0].s.legsAnim  = g_ps.legsAnim;
			g_entities[0].s.torsoAnim = g_ps.torsoAnim;
			g_entities[0].s.number    = 0;
			g_entities[0].s.eType     = ET_GENERAL;
			g_pmw_traceCount = 0;
			dump_step(stepNo++, &pm);
		} else if (!strcmp(tok[0], "cmd") && n >= 9) {
			int dt      = parse_int(tok[1]);
			int fwd     = parse_int(tok[2]);
			int right   = parse_int(tok[3]);
			int up      = parse_int(tok[4]);
			int buttons = parse_int(tok[5]);
			int yaw     = parse_int(tok[6]);
			int pitch   = parse_int(tok[7]);
			int roll    = parse_int(tok[8]);
			int reps    = 1, yawinc = 0;
			if (n >= 10 && tok[9][0] == 'x') reps = atoi(tok[9] + 1);
			if (n >= 11) yawinc = parse_int(tok[10]);

			for (int r = 0; r < reps; r++) {
				pm.cmd.forwardmove = (signed char)fwd;
				pm.cmd.rightmove   = (signed char)right;
				pm.cmd.upmove      = (signed char)up;
				pm.cmd.buttons     = buttons;
				pm.cmd.weapon      = WP_MELEE;
				pm.cmd.angles[PITCH] = (short)pitch;
				pm.cmd.angles[YAW]   = (short)(yaw + r * yawinc);
				pm.cmd.angles[ROLL]  = (short)roll;
				prevServerTime    += dt;
				pm.cmd.serverTime  = prevServerTime;

				g_pmw_traceCount = 0;
				Pmove(&pm);

				// mirror ps anims into the stub entity for the next step's
				// restart-check (BG_PlayerStateToEntityState equivalent).
				g_entities[0].s.legsAnim  = g_ps.legsAnim;
				g_entities[0].s.torsoAnim = g_ps.torsoAnim;

				dump_step(stepNo++, &pm);
			}
		} else {
			fprintf(stderr, "pmove-oracle: bad line: %s\n", tok[0]);
			return 2;
		}
	}
	fclose(f);
	printf("== end ==\n");
	return 0;
}

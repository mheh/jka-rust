// w_saber differential-oracle dumper. Compiled against the UNMODIFIED Raven
// w_saber.c (copied into build-wsaber/ by run_wsaber.sh) linked with the
// UNMODIFIED q_shared.c + q_math.c, with QAGAME defined. w_saber.c is a large
// TU that extern-references ~130 game/engine symbols across functions this
// dumper never calls; the data globals it reads are defined here (zeroed) and
// every unreachable *function* is an abort()ing stub in stubs_wsaber.c (a
// header-free TU so its argless stub definitions never clash with the real
// prototypes). It drives TWO pure integer leaf functions over committed sweep
// fixtures and prints their return values; the Rust parity test reproduces the
// dump by driving the ported crate over the same sweeps.
//
// Sections (each a "== <name> ==" banner):
//   lockanim   G_SaberLockAnim over an exhaustive 5-axis integer sweep read
//              from fixtures/wsaber/lockanim.txt (saber-lock/break anim table)
//   knockaway  G_KnockawayForParry over an integer move sweep from
//              fixtures/wsaber/knockaway.txt (parry -> knockaway-anim switch)
#include "g_local.h"
#include "dumpcommon.h"

#include <stdarg.h>

// Neither function is prototyped in a game header (extern-declared at each call
// site), so declare them here.
int G_SaberLockAnim(int attackerSaberStyle, int defenderSaberStyle, int topOrSide, int lockOrBreakOrSuperBreak, int winOrLose);
int G_KnockawayForParry(int move);

// ------------------------- data globals w_saber.c reads ---------------------
// Zeroed definitions so the TU links; none is touched on the two tested paths
// (both functions are pure integer switch/table logic over their arguments).
gentity_t        g_entities[MAX_GENTITIES];
level_locals_t   level;
void            *g2SaberInstance;
stringID_table_t animTable[MAX_ANIMATIONS + 1];
bgLoadedAnim_t   bgAllAnims[MAX_ANIM_FILES];
saberMoveData_t  saberMoveData[LS_MOVE_MAX];
// siegeClass_t / MAX_SIEGE_CLASSES live outside g_local.h's closure; this table
// is never touched on the tested paths, so a raw byte symbol suffices.
char             bgSiegeClasses[1];
vmCvar_t d_saberAlwaysBoxTrace, d_saberBoxTraceSize, d_saberGhoul2Collision,
	d_saberInterpolate, d_saberKickTweak, d_saberSPStyleDamage, g_debugServerSkel,
	g_debugSaberLocks, g_duel_fraglimit, g_duelWeaponDisable, g_friendlyFire,
	g_friendlySaber, g_g2TraceLod, g_gametype, g_optvehtrace, g_saberBladeFaces,
	g_saberDamageScale, g_saberDebugBox, g_saberDebugPrint, g_saberDmgDelay_Idle,
	g_saberDmgDelay_Wound, g_saberLockFactor, g_saberLocking,
	g_saberRealisticCombat, g_saberTraceSaberFirst, g_saberWallDamageScale,
	g_svfps, g_weaponDisable;

// Com_Printf / Com_Error routed to stderr so diagnostics never enter the golden.
void Com_Printf(const char *fmt, ...) {
	va_list ap; va_start(ap, fmt); vfprintf(stderr, fmt, ap); va_end(ap);
}
void Com_Error(int lvl, const char *fmt, ...) {
	va_list ap; va_start(ap, fmt);
	fprintf(stderr, "Com_Error(%d): ", lvl); vfprintf(stderr, fmt, ap);
	fprintf(stderr, "\n"); va_end(ap);
	abort();
}

// -------------------------- value/token parsing ----------------------------
static int pi(const char *t) {
	if (t[0] == '0' && (t[1] == 'x' || t[1] == 'X')) return (int)strtoul(t, 0, 16);
	return (int)atol(t);
}
#define MAXTOK 32
static int tokenize(char *line, char *tok[]) {
	int n = 0;
	char *p = strtok(line, " \t\r\n");
	while (p && n < MAXTOK) { tok[n++] = p; p = strtok(0, " \t\r\n"); }
	return n;
}

// Read the single `sweep ...` line (skipping blanks/comments) into tok[]/n.
static int read_sweep(const char *dir, const char *fname, char *store, char *tok[]) {
	char path[1024]; snprintf(path, sizeof(path), "%s/%s", dir, fname);
	char *buf = slurp(path, 0);
	int n = 0;
	char *save; char *line = strtok_r(buf, "\n", &save);
	for (; line; line = strtok_r(0, "\n", &save)) {
		strncpy(store, line, 511); store[511] = 0;
		char tmp[512]; strncpy(tmp, store, sizeof(tmp)-1); tmp[sizeof(tmp)-1]=0;
		char *first = strtok(tmp, " \t\r\n");
		if (!first || first[0] == '#' || strcmp(first, "sweep")) continue;
		n = tokenize(store, tok);
		break;
	}
	free(buf);
	return n;
}

// ------------------------------ lockanim -----------------------------------
// G_SaberLockAnim is a pure 5-int switch/arithmetic table. The sweep bounds
// (inclusive) come from the fixture: sweep <sLo> <sHi> <tLo> <tHi> <lLo> <lHi>
// <wLo> <wHi> — attacker and defender both range over [sLo,sHi].
static void sec_lockanim(const char *dir) {
	char store[512]; char *tok[MAXTOK];
	int n = read_sweep(dir, "lockanim.txt", store, tok);
	printf("== lockanim ==\n");
	if (n < 9) { fprintf(stderr, "lockanim: bad sweep line\n"); exit(2); }
	int sLo = pi(tok[1]), sHi = pi(tok[2]);
	int tLo = pi(tok[3]), tHi = pi(tok[4]);
	int lLo = pi(tok[5]), lHi = pi(tok[6]);
	int wLo = pi(tok[7]), wHi = pi(tok[8]);
	for (int a = sLo; a <= sHi; a++)
	for (int d = sLo; d <= sHi; d++)
	for (int t = tLo; t <= tHi; t++)
	for (int l = lLo; l <= lHi; l++)
	for (int w = wLo; w <= wHi; w++) {
		int r = G_SaberLockAnim(a, d, t, l, w);
		printf("la %d %d %d %d %d %d\n", a, d, t, l, w, r);
	}
}

// ------------------------------ knockaway ----------------------------------
// G_KnockawayForParry maps a saber move to a knockaway anim (switch + default).
// sweep <moveLo> <moveHi> (inclusive).
static void sec_knockaway(const char *dir) {
	char store[512]; char *tok[MAXTOK];
	int n = read_sweep(dir, "knockaway.txt", store, tok);
	printf("== knockaway ==\n");
	if (n < 3) { fprintf(stderr, "knockaway: bad sweep line\n"); exit(2); }
	int lo = pi(tok[1]), hi = pi(tok[2]);
	for (int m = lo; m <= hi; m++) {
		int r = G_KnockawayForParry(m);
		printf("kp %d %d\n", m, r);
	}
}

int main(int argc, char **argv) {
	if (argc != 2) { fprintf(stderr, "usage: %s <fixture-dir>\n", argv[0]); return 2; }
	const char *dir = argv[1];
	printf("== wsaber ==\n");
	sec_lockanim(dir);
	sec_knockaway(dir);
	printf("== end ==\n");
	return 0;
}

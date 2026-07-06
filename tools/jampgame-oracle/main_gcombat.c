// g_combat differential-oracle dumper. Compiled against the UNMODIFIED Raven
// g_combat.c (copied into build-gcombat/ by run_gcombat.sh) linked with the
// UNMODIFIED q_shared.c + q_math.c, with QAGAME defined (jampgame == Raven's
// QAGAME build). g_combat.c is a large TU that extern-references ~120 game /
// engine symbols across functions this dumper never calls; the data globals it
// reads are defined here (zeroed) and every unreachable *function* is an
// abort()ing stub in stubs_gcombat.c (a header-free TU so its argless stub
// definitions never clash with the real prototypes). It drives THREE pure-ish
// leaf functions over committed fixtures and prints every observable output as
// IEEE-754 bit-hex (floats) / decimal (ints); the Rust parity test reproduces
// this dump by driving the ported crate over the same fixtures.
//
// Sections (each a "== <name> ==" banner):
//   raysphere  RaySphereIntersections over fixtures/gcombat/raysphere.txt
//              (ray/sphere hit count + normalized dir + both intersection pts)
//   hitloc     G_GetHitLocation over fixtures/gcombat/hitloc.txt (locational
//              hit-zone classification from an impact point + target box/angles)
//   armor      CheckArmor over fixtures/gcombat/armor.txt (shield absorption:
//              return value + the mutated STAT_ARMOR)
#include "g_local.h"
#include "dumpcommon.h"

#include <stdarg.h>
#include <math.h>

// These three g_combat.c functions are not prototyped in any game header
// (they're extern-declared at each call site), so declare them here.
int CheckArmor(gentity_t *ent, int damage, int dflags);
int G_GetHitLocation(gentity_t *target, vec3_t ppoint);
int RaySphereIntersections(vec3_t origin, float radius, vec3_t point, vec3_t dir, vec3_t intersections[2]);

// ------------------------- data globals g_combat.c reads --------------------
// Zeroed definitions so the TU links; none is touched on the three tested
// paths (CheckArmor reads only `level.time`; the others read their arguments).
gentity_t        g_entities[MAX_GENTITIES];
level_locals_t   level;
gitem_t          bg_itemlist[1];
weaponData_t     weaponData[WP_NUM_WEAPONS];
animation_t      bgHumanoidAnimations[MAX_TOTALANIMATIONS];
bgLoadedAnim_t   bgAllAnims[MAX_ANIM_FILES];
// siegeClass_t / MAX_SIEGE_CLASSES live in a cgame header outside g_local.h's
// closure; this table is never touched on the tested paths, so a raw byte
// symbol is enough for the linker.
char             bgSiegeClasses[1];
vehicleInfo_t    g_vehicleInfo[MAX_VEHICLES];
stringID_table_t animTable[MAX_ANIMATIONS + 1];
vmCvar_t g_armBreakage, g_austrian, g_debugDamage, g_debugMelee, g_dismember,
	g_ff_objectives, g_friendlyFire, g_gametype, g_gravity, g_knockback,
	g_locationBasedDamage, g_saberDmgVelocityScale, g_slowmoDuelEnd, g_trueJedi,
	d_projectileGhoul2Collision, d_saberGhoul2Collision;
qboolean g_dontFrickinCheck, g_dontPenalizeTeam, g_endPDuel, g_noPDuelCheck,
	gSiegeRoundBegun, gDoSlowMoDuel;
int gSlowMoDuelTime;

// Com_Printf / Com_Error routed to stderr so parser/link diagnostics never
// enter the golden (stdout); Com_Error additionally aborts.
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
// A float token is either a plain (possibly negative) integer parsed as
// (float)atol, or an 0xXXXXXXXX f32 bit pattern — never a decimal point (which
// would double-round differently on the two sides). Int tokens: decimal, or 0x.
static float pf(const char *t) {
	if (t[0] == '0' && (t[1] == 'x' || t[1] == 'X')) {
		union { unsigned u; float f; } c;
		c.u = (unsigned)strtoul(t, 0, 16);
		return c.f;
	}
	return (float)atol(t);
}
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

// ----------------------------- raysphere -----------------------------------
// RaySphereIntersections normalizes `dir` in place, so the dump includes the
// normalized dir plus both intersection slots (zero-initialized before the
// call, so slots the function does not write read as 0 deterministically).
static void sec_raysphere(const char *dir_) {
	char path[1024]; snprintf(path, sizeof(path), "%s/raysphere.txt", dir_);
	char *buf = slurp(path, 0);
	printf("== raysphere ==\n");
	int idx = 0;
	char *save; char *line = strtok_r(buf, "\n", &save);
	for (; line; line = strtok_r(0, "\n", &save)) {
		char copy[512]; strncpy(copy, line, sizeof(copy) - 1); copy[sizeof(copy)-1]=0;
		char *tok[MAXTOK]; int n = tokenize(copy, tok);
		if (n == 0 || tok[0][0] == '#' || strcmp(tok[0], "ray")) continue;
		vec3_t origin = { pf(tok[1]), pf(tok[2]), pf(tok[3]) };
		float  radius = pf(tok[4]);
		vec3_t point  = { pf(tok[5]), pf(tok[6]), pf(tok[7]) };
		vec3_t dir    = { pf(tok[8]), pf(tok[9]), pf(tok[10]) };
		vec3_t inter[2]; memset(inter, 0, sizeof(inter));
		int r = RaySphereIntersections(origin, radius, point, dir, inter);
		printf("ray %d n %d dir %08x %08x %08x i0 %08x %08x %08x i1 %08x %08x %08x\n",
			idx, r,
			f2b(dir[0]), f2b(dir[1]), f2b(dir[2]),
			f2b(inter[0][0]), f2b(inter[0][1]), f2b(inter[0][2]),
			f2b(inter[1][0]), f2b(inter[1][1]), f2b(inter[1][2]));
		idx++;
	}
	free(buf);
}

// ------------------------------ hitloc -------------------------------------
// G_GetHitLocation reads target->client (only to decide whether to zero
// pitch/roll — always set here, since a NULL client leaves `tangles`
// uninitialized in Raven, which is UB and excluded per porting-rules §19),
// target->r.currentAngles[YAW], r.absmin/absmax (box center), r.maxs/mins
// (radius, computed-but-unused), and the impact point argument.
static void sec_hitloc(const char *dir_) {
	char path[1024]; snprintf(path, sizeof(path), "%s/hitloc.txt", dir_);
	char *buf = slurp(path, 0);
	printf("== hitloc ==\n");
	static gclient_t hlClient; // non-NULL client pointer; contents unused here
	int idx = 0;
	char *save; char *line = strtok_r(buf, "\n", &save);
	for (; line; line = strtok_r(0, "\n", &save)) {
		char copy[512]; strncpy(copy, line, sizeof(copy) - 1); copy[sizeof(copy)-1]=0;
		char *tok[MAXTOK]; int n = tokenize(copy, tok);
		if (n == 0 || tok[0][0] == '#' || strcmp(tok[0], "h")) continue;
		gentity_t ent; memset(&ent, 0, sizeof(ent));
		ent.client = &hlClient;
		ent.r.currentAngles[YAW] = pf(tok[1]);
		ent.r.absmin[0] = pf(tok[2]); ent.r.absmin[1] = pf(tok[3]); ent.r.absmin[2] = pf(tok[4]);
		ent.r.absmax[0] = pf(tok[5]); ent.r.absmax[1] = pf(tok[6]); ent.r.absmax[2] = pf(tok[7]);
		ent.r.mins[0] = pf(tok[8]); ent.r.mins[1] = pf(tok[9]);
		ent.r.maxs[0] = pf(tok[10]); ent.r.maxs[1] = pf(tok[11]);
		vec3_t ppoint = { pf(tok[12]), pf(tok[13]), pf(tok[14]) };
		int hl = G_GetHitLocation(&ent, ppoint);
		printf("h %d hl %d\n", idx, hl);
		idx++;
	}
	free(buf);
}

// ------------------------------- armor -------------------------------------
// CheckArmor reads ent->client (STAT_ARMOR, NPC_class, ps.electrifyTime),
// ent->m_pVehicle, and level.time; it mutates STAT_ARMOR. Fixture columns:
//   a <armor> <isVehicle 0|1> <electrifyTime> <hasVehicle> <levelTime> <damage> <dflags>
// (isVehicle maps 1 -> CLASS_VEHICLE, 0 -> CLASS_NONE on both sides, so the
// vehicle-shield branch is exercised without pinning a fragile enum ordinal.)
static void sec_armor(const char *dir_) {
	char path[1024]; snprintf(path, sizeof(path), "%s/armor.txt", dir_);
	char *buf = slurp(path, 0);
	printf("== armor ==\n");
	static Vehicle_t dummyVeh; // any non-NULL target for ent.m_pVehicle
	int idx = 0;
	char *save; char *line = strtok_r(buf, "\n", &save);
	for (; line; line = strtok_r(0, "\n", &save)) {
		char copy[512]; strncpy(copy, line, sizeof(copy) - 1); copy[sizeof(copy)-1]=0;
		char *tok[MAXTOK]; int n = tokenize(copy, tok);
		if (n == 0 || tok[0][0] == '#' || strcmp(tok[0], "a")) continue;
		gentity_t ent; memset(&ent, 0, sizeof(ent));
		gclient_t client; memset(&client, 0, sizeof(client));
		ent.client = &client;
		client.ps.stats[STAT_ARMOR] = pi(tok[1]);
		client.NPC_class = pi(tok[2]) ? CLASS_VEHICLE : CLASS_NONE;
		client.ps.electrifyTime = pi(tok[3]);
		ent.m_pVehicle = pi(tok[4]) ? &dummyVeh : NULL;
		level.time = pi(tok[5]);
		int damage = pi(tok[6]);
		int dflags = pi(tok[7]);
		int r = CheckArmor(&ent, damage, dflags);
		printf("a %d ret %d armor %d\n", idx, r, client.ps.stats[STAT_ARMOR]);
		idx++;
	}
	free(buf);
}

int main(int argc, char **argv) {
	if (argc != 2) { fprintf(stderr, "usage: %s <fixture-dir>\n", argv[0]); return 2; }
	const char *dir = argv[1];
	printf("== gcombat ==\n");
	sec_raysphere(dir);
	sec_hitloc(dir);
	sec_armor(dir);
	printf("== end ==\n");
	return 0;
}

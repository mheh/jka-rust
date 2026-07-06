// bg_misc / bg_weapons differential-oracle dumper. Compiled against the
// UNMODIFIED Raven bg_misc.c + bg_weapons.c (copied into build-bgmisc/ by
// run_bgmisc.sh) linked with the UNMODIFIED q_shared.c + q_math.c, with
// QAGAME defined (jampgame == Raven's QAGAME build). The TUs extern-reference
// a small handful of game/engine symbols; this file provides them as loud
// abort()ing stubs (Com_Printf/Com_Error routed to stderr) — NONE of them is
// on any tested path (verified: a firing stub means a fixture leaked off
// path). It drives the real oracle functions over committed fixtures and
// prints every observable output as IEEE-754 bit-hex (floats) / decimal
// (ints). The Rust parity test reproduces this dump by driving the ported
// crate over the same fixtures.
//
// Sections (each a "== <name> ==" banner):
//   trajectory  BG_EvaluateTrajectory / BG_EvaluateTrajectoryDelta over
//               fixtures/bgmisc/trajectory.txt (every trType_t x edge atTimes)
//   itemlist    every field of every bg_itemlist[] entry, declaration order
//   findid      BG_FindItem / ForWeapon / ForPowerup / ForHoldable / ForAmmo
//   weapondata  the full weaponData[] table
//   ammodata    the full ammoData[] table
//   grab        BG_CanItemBeGrabbed over fixtures/bgmisc/grab.txt
//   ps2es       BG_PlayerStateToEntityState over fixtures/bgmisc/ps.txt
#include "q_shared.h"
#include "bg_public.h"
#include "bg_weapons.h"
#include "dumpcommon.h"

#include <stdarg.h>

// BG_FindItemForAmmo is defined in bg_misc.c but not prototyped in bg_public.h.
gitem_t *BG_FindItemForAmmo( ammo_t ammo );

// ------------------------- link-satisfying stubs ---------------------------
// bg_misc.c references these from functions we never call. Com_Printf/Error go
// to stderr (never stdout, so the golden is clean); everything else abort()s.
void Com_Printf(const char *fmt, ...) {
	va_list ap; va_start(ap, fmt); vfprintf(stderr, fmt, ap); va_end(ap);
}
void Com_Error(int level, const char *fmt, ...) {
	va_list ap; va_start(ap, fmt);
	fprintf(stderr, "Com_Error(%d): ", level); vfprintf(stderr, fmt, ap);
	fprintf(stderr, "\n"); va_end(ap);
	abort();
}
#define STUB0(sig, name) sig { fprintf(stderr, "STUB " #name " reached\n"); abort(); }
STUB0(char *G_NewString(const char *s), G_NewString)
STUB0(void Q3_SetParm(int a, int b, const char *c), Q3_SetParm)
STUB0(void trap_FS_FCloseFile(fileHandle_t f), trap_FS_FCloseFile)
STUB0(int trap_FS_FOpenFile(const char *a, fileHandle_t *b, fsMode_t c), trap_FS_FOpenFile)
STUB0(void trap_G2API_CleanGhoul2Models(void **a), trap_G2API_CleanGhoul2Models)
STUB0(int trap_G2API_InitGhoul2Model(void **a, const char *b, int c, qhandle_t d, qhandle_t e, int f, int g), trap_G2API_InitGhoul2Model)
STUB0(qhandle_t trap_R_RegisterSkin(const char *a), trap_R_RegisterSkin)

// -------------------------- value/token parsing ----------------------------
// A float token is either a plain (possibly negative) integer parsed as
// (float)atol, or an 0xXXXXXXXX f32 bit pattern — never a decimal point (which
// would double-round differently on the two sides). Int tokens: decimal, or
// 0x hex.
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

#define MAXTOK 24
static int tokenize(char *line, char *tok[]) {
	int n = 0;
	char *p = strtok(line, " \t\r\n");
	while (p && n < MAXTOK) { tok[n++] = p; p = strtok(0, " \t\r\n"); }
	return n;
}

// -------------------------- ps / es field setters --------------------------
// Set one playerState_t field named by tok[0] from tok[1..]. Abort on an
// unknown field so a fixture typo can never silently read as zero. The Rust
// TestState mirrors this dispatch exactly (single source of truth = fixture).
static void ps_set(playerState_t *ps, int n, char *tok[]) {
	const char *f = tok[0];
	#define S1(name, expr)  if (!strcmp(f, name)) { (expr) = pi(tok[1]); return; }
	#define SF(name, expr)  if (!strcmp(f, name)) { (expr) = pf(tok[1]); return; }
	#define SV(name, expr)  if (!strcmp(f, name)) { (expr)[0]=pf(tok[1]); (expr)[1]=pf(tok[2]); (expr)[2]=pf(tok[3]); return; }
	#define SI(name, arr)   if (!strcmp(f, name)) { (arr)[pi(tok[1])] = pi(tok[2]); return; }
	(void)n;
	S1("pm_type", ps->pm_type)
	S1("clientNum", ps->clientNum)
	S1("weapon", ps->weapon)
	S1("weaponstate", ps->weaponstate)
	S1("weaponChargeTime", ps->weaponChargeTime)
	S1("groundEntityNum", ps->groundEntityNum)
	S1("movementDir", ps->movementDir)
	S1("eFlags", ps->eFlags)
	S1("eFlags2", ps->eFlags2)
	S1("externalEvent", ps->externalEvent)
	S1("externalEventParm", ps->externalEventParm)
	S1("eventSequence", ps->eventSequence)
	S1("entityEventSequence", ps->entityEventSequence)
	S1("duelInProgress", ps->duelInProgress)
	S1("genericEnemyIndex", ps->genericEnemyIndex)
	S1("isJediMaster", ps->isJediMaster)
	S1("trueJedi", ps->trueJedi)
	S1("trueNonJedi", ps->trueNonJedi)
	S1("legsAnim", ps->legsAnim)
	S1("torsoAnim", ps->torsoAnim)
	S1("legsFlip", ps->legsFlip)
	S1("torsoFlip", ps->torsoFlip)
	S1("saberInFlight", ps->saberInFlight)
	S1("saberEntityNum", ps->saberEntityNum)
	S1("saberMove", ps->saberMove)
	S1("saberHolstered", ps->saberHolstered)
	S1("saberLockFrame", ps->saberLockFrame)
	S1("electrifyTime", ps->electrifyTime)
	S1("activeForcePass", ps->activeForcePass)
	S1("emplacedIndex", ps->emplacedIndex)
	S1("holocronBits", ps->holocronBits)
	S1("heldByClient", ps->heldByClient)
	S1("ragAttach", ps->ragAttach)
	S1("iModelScale", ps->iModelScale)
	S1("brokenLimbs", ps->brokenLimbs)
	S1("hasLookTarget", ps->hasLookTarget)
	S1("lookTarget", ps->lookTarget)
	S1("m_iVehicleNum", ps->m_iVehicleNum)
	S1("loopSound", ps->loopSound)
	S1("generic1", ps->generic1)
	SF("speed", ps->speed)
	SV("origin", ps->origin)
	SV("velocity", ps->velocity)
	SV("viewangles", ps->viewangles)
	SV("lastHitLoc", ps->lastHitLoc)
	SI("events", ps->events)
	SI("eventParms", ps->eventParms)
	SI("powerups", ps->powerups)
	SI("customRGBA", ps->customRGBA)
	SI("stats", ps->stats)
	SI("ammo", ps->ammo)
	SI("persistant", ps->persistant)
	// convenience stat aliases (identical index on both sides)
	S1("health", ps->stats[STAT_HEALTH])
	S1("maxhealth", ps->stats[STAT_MAX_HEALTH])
	S1("armor", ps->stats[STAT_ARMOR])
	S1("statweapons", ps->stats[STAT_WEAPONS])
	S1("holdables", ps->stats[STAT_HOLDABLE_ITEMS])
	// named powerups / persistant (hit the exact enum branch)
	S1("powerups_ysalamiri", ps->powerups[PW_YSALAMIRI])
	S1("powerups_redflag", ps->powerups[PW_REDFLAG])
	S1("powerups_blueflag", ps->powerups[PW_BLUEFLAG])
	S1("persistant_team", ps->persistant[PERS_TEAM])
	// forcedata
	S1("fd_forcePowersActive", ps->fd.forcePowersActive)
	S1("fd_saberAnimLevel", ps->fd.saberAnimLevel)
	S1("fd_mtti1", ps->fd.forceMindtrickTargetIndex)
	S1("fd_mtti2", ps->fd.forceMindtrickTargetIndex2)
	S1("fd_mtti3", ps->fd.forceMindtrickTargetIndex3)
	S1("fd_mtti4", ps->fd.forceMindtrickTargetIndex4)
	#undef S1
	#undef SF
	#undef SV
	#undef SI
	fprintf(stderr, "ps_set: unknown field '%s'\n", f); abort();
}

static void es_set(entityState_t *es, int n, char *tok[]) {
	const char *f = tok[0];
	(void)n;
	if (!strcmp(f, "modelindex"))  { es->modelindex  = pi(tok[1]); return; }
	if (!strcmp(f, "modelindex2")) { es->modelindex2 = pi(tok[1]); return; }
	if (!strcmp(f, "generic1"))    { es->generic1    = pi(tok[1]); return; }
	if (!strcmp(f, "powerups"))    { es->powerups    = pi(tok[1]); return; }
	if (!strcmp(f, "eFlags"))      { es->eFlags      = pi(tok[1]); return; }
	fprintf(stderr, "es_set: unknown field '%s'\n", f); abort();
}

// -------------------------- small dump helpers -----------------------------
static void pI(const char *tag, int v) { printf("%s %d\n", tag, v); }
static void pF(const char *tag, float v) { printf("%s %08x\n", tag, f2b(v)); }
static void pV(const char *tag, const vec3_t v) { printf("%s %08x %08x %08x\n", tag, f2b(v[0]), f2b(v[1]), f2b(v[2])); }
static void pS(const char *tag, const char *s) { printf("%s %s\n", tag, s ? s : "(null)"); }

// ------------------------------ trajectory ---------------------------------
static void sec_trajectory(const char *dir) {
	char path[1024]; snprintf(path, sizeof(path), "%s/trajectory.txt", dir);
	char *buf = slurp(path, 0);
	printf("== trajectory ==\n");
	int idx = 0;
	char *save; char *line = strtok_r(buf, "\n", &save);
	for (; line; line = strtok_r(0, "\n", &save)) {
		char copy[512]; strncpy(copy, line, sizeof(copy) - 1); copy[sizeof(copy)-1]=0;
		char *tok[MAXTOK]; int n = tokenize(copy, tok);
		if (n == 0 || tok[0][0] == '#' || strcmp(tok[0], "T")) continue;
		trajectory_t tr; memset(&tr, 0, sizeof(tr));
		tr.trType     = pi(tok[1]);
		tr.trTime     = pi(tok[2]);
		tr.trDuration = pi(tok[3]);
		tr.trBase[0]  = pf(tok[4]); tr.trBase[1] = pf(tok[5]); tr.trBase[2] = pf(tok[6]);
		tr.trDelta[0] = pf(tok[7]); tr.trDelta[1] = pf(tok[8]); tr.trDelta[2] = pf(tok[9]);
		int atTime    = pi(tok[10]);
		vec3_t rp, rd;
		BG_EvaluateTrajectory(&tr, atTime, rp);
		BG_EvaluateTrajectoryDelta(&tr, atTime, rd);
		printf("T %d et %08x %08x %08x ed %08x %08x %08x\n", idx,
			f2b(rp[0]), f2b(rp[1]), f2b(rp[2]), f2b(rd[0]), f2b(rd[1]), f2b(rd[2]));
		idx++;
	}
	free(buf);
}

// ------------------------------- itemlist ----------------------------------
static void sec_itemlist(void) {
	printf("== itemlist ==\n");
	for (int i = 0; i <= bg_numItems; i++) {
		gitem_t *it = &bg_itemlist[i];
		printf("item %d\n", i);
		pS(" classname", it->classname);
		pS(" pickup_sound", it->pickup_sound);
		for (int m = 0; m < MAX_ITEM_MODELS; m++) {
			printf(" world_model%d %s\n", m, it->world_model[m] ? it->world_model[m] : "(null)");
		}
		pS(" view_model", it->view_model);
		pS(" icon", it->icon);
		pI(" quantity", it->quantity);
		pI(" giType", (int)it->giType);
		pI(" giTag", it->giTag);
		pS(" precaches", it->precaches);
		pS(" sounds", it->sounds);
		pS(" description", it->description);
	}
}

// ------------------------------- findid ------------------------------------
static int itemidx(gitem_t *it) { return it ? (int)(it - bg_itemlist) : -1; }
static void sec_findid(void) {
	printf("== findid ==\n");
	// BG_FindItemForWeapon / ForAmmo / ForHoldable Com_Error (fatal -> port
	// panic) when the tag has no item, so only tags that exist are queried
	// (WP_NONE/AMMO_NONE/AMMO_EMPLACED/HI_NONE are omitted — their "not-found"
	// behavior is a shared abort, not an observable index). BG_FindItemForPowerup
	// and BG_FindItem return NULL on miss, so their not-found cases ARE dumped.
	for (int w = WP_NONE + 1; w < WP_NUM_WEAPONS; w++)
		printf("weapon %d %d\n", w, itemidx(BG_FindItemForWeapon(w)));
	static const int ammos[] = { AMMO_FORCE, AMMO_BLASTER, AMMO_POWERCELL,
		AMMO_METAL_BOLTS, AMMO_ROCKETS, AMMO_THERMAL, AMMO_TRIPMINE, AMMO_DETPACK };
	for (unsigned k = 0; k < sizeof(ammos)/sizeof(ammos[0]); k++)
		printf("ammo %d %d\n", ammos[k], itemidx(BG_FindItemForAmmo(ammos[k])));
	for (int h = HI_NONE + 1; h < HI_NUM_HOLDABLE; h++)
		printf("holdable %d %d\n", h, itemidx(BG_FindItemForHoldable(h)));
	for (int p = 0; p <= PW_NUM_POWERUPS; p++)
		printf("powerup %d %d\n", p, itemidx(BG_FindItemForPowerup(p)));
	printf("powerup 999 %d\n", itemidx(BG_FindItemForPowerup(999)));
	const char *names[] = {
		"weapon_saber", "weapon_blaster", "ammo_blaster", "item_shield_sm_instant",
		"team_CTF_redflag", "item_medpac", "item_force_enlighten_light",
		"weapon_stun_baton", "nonexistent_item", "",
	};
	for (unsigned k = 0; k < sizeof(names)/sizeof(names[0]); k++)
		printf("find \"%s\" %d\n", names[k], itemidx(BG_FindItem(names[k])));
}

// ---------------------------- weapon/ammo data -----------------------------
static void sec_weapondata(void) {
	printf("== weapondata ==\n");
	for (int w = 0; w < WP_NUM_WEAPONS; w++) {
		weaponData_t *d = &weaponData[w];
		printf("wd %d %d %d %d %d %d %d %d %d %d %d %d %d %d %d\n", w,
			d->ammoIndex, d->ammoLow, d->energyPerShot, d->fireTime, d->range,
			d->altEnergyPerShot, d->altFireTime, d->altRange, d->chargeSubTime,
			d->altChargeSubTime, d->chargeSub, d->altChargeSub, d->maxCharge, d->altMaxCharge);
	}
	printf("== ammodata ==\n");
	for (int a = 0; a < AMMO_MAX; a++) printf("ad %d %d\n", a, ammoData[a].max);
}

// -------------------------------- grab -------------------------------------
static void sec_grab(const char *dir) {
	char path[1024]; snprintf(path, sizeof(path), "%s/grab.txt", dir);
	char *buf = slurp(path, 0);
	printf("== grab ==\n");
	int gametype = 0;
	entityState_t ent; memset(&ent, 0, sizeof(ent));
	playerState_t ps; memset(&ps, 0, sizeof(ps));
	int havePs = 0;
	char *save; char *line = strtok_r(buf, "\n", &save);
	for (; line; line = strtok_r(0, "\n", &save)) {
		char copy[512]; strncpy(copy, line, sizeof(copy) - 1); copy[sizeof(copy)-1]=0;
		char *tok[MAXTOK]; int n = tokenize(copy, tok);
		if (n == 0 || tok[0][0] == '#') continue;
		if (!strcmp(tok[0], "reset")) {
			memset(&ent, 0, sizeof(ent)); memset(&ps, 0, sizeof(ps));
			gametype = 0; havePs = 1;
		} else if (!strcmp(tok[0], "nullps")) {
			havePs = 0;
		} else if (!strcmp(tok[0], "gametype")) {
			gametype = pi(tok[1]);
		} else if (!strcmp(tok[0], "ent")) {
			es_set(&ent, n - 1, tok + 1);
		} else if (!strcmp(tok[0], "ps")) {
			ps_set(&ps, n - 1, tok + 1);
		} else if (!strcmp(tok[0], "run")) {
			qboolean r = BG_CanItemBeGrabbed(gametype, &ent, havePs ? &ps : NULL);
			printf("grab %s %d\n", tok[1], r ? 1 : 0);
		} else {
			fprintf(stderr, "grab: unknown cmd '%s'\n", tok[0]); abort();
		}
	}
	free(buf);
}

// -------------------------------- ps2es ------------------------------------
static void dump_es(const char *label, playerState_t *ps, entityState_t *s) {
	printf("es %s\n", label);
	pI(" eType", s->eType);
	pI(" number", s->number);
	pI(" pos.trType", (int)s->pos.trType);
	pV(" pos.trBase", s->pos.trBase);
	pV(" pos.trDelta", s->pos.trDelta);
	pI(" apos.trType", (int)s->apos.trType);
	pV(" apos.trBase", s->apos.trBase);
	pI(" trickedentindex", s->trickedentindex);
	pI(" trickedentindex2", s->trickedentindex2);
	pI(" trickedentindex3", s->trickedentindex3);
	pI(" trickedentindex4", s->trickedentindex4);
	pI(" forceFrame", s->forceFrame);
	pI(" emplacedOwner", s->emplacedOwner);
	pF(" speed", s->speed);
	pI(" genericenemyindex", s->genericenemyindex);
	pI(" activeForcePass", s->activeForcePass);
	pV(" angles2", s->angles2);
	pI(" legsAnim", s->legsAnim);
	pI(" torsoAnim", s->torsoAnim);
	pI(" legsFlip", s->legsFlip);
	pI(" torsoFlip", s->torsoFlip);
	pI(" clientNum", s->clientNum);
	pI(" eFlags", s->eFlags);
	pI(" eFlags2", s->eFlags2);
	pI(" saberInFlight", s->saberInFlight);
	pI(" saberEntityNum", s->saberEntityNum);
	pI(" saberMove", s->saberMove);
	pI(" forcePowersActive", s->forcePowersActive);
	pI(" bolt1", s->bolt1);
	pI(" otherEntityNum2", s->otherEntityNum2);
	pI(" saberHolstered", s->saberHolstered);
	pI(" event", s->event);
	pI(" eventParm", s->eventParm);
	pI(" weapon", s->weapon);
	pI(" groundEntityNum", s->groundEntityNum);
	pI(" powerups", s->powerups);
	pI(" loopSound", s->loopSound);
	pI(" generic1", s->generic1);
	pI(" modelindex2", s->modelindex2);
	pI(" constantLight", s->constantLight);
	pV(" origin2", s->origin2);
	pI(" isJediMaster", s->isJediMaster);
	pI(" time2", s->time2);
	pI(" fireflag", s->fireflag);
	pI(" heldByClient", s->heldByClient);
	pI(" ragAttach", s->ragAttach);
	pI(" iModelScale", s->iModelScale);
	pI(" brokenLimbs", s->brokenLimbs);
	pI(" hasLookTarget", s->hasLookTarget);
	pI(" lookTarget", s->lookTarget);
	printf(" customRGBA %d %d %d %d\n", s->customRGBA[0], s->customRGBA[1], s->customRGBA[2], s->customRGBA[3]);
	pI(" m_iVehicleNum", s->m_iVehicleNum);
	// ps side-effect: entityEventSequence advances on the events[] path
	pI(" ps.entityEventSequence", ps->entityEventSequence);
}

static void sec_ps2es(const char *dir) {
	char path[1024]; snprintf(path, sizeof(path), "%s/ps.txt", dir);
	char *buf = slurp(path, 0);
	printf("== ps2es ==\n");
	playerState_t ps; memset(&ps, 0, sizeof(ps));
	int snap = 0;
	char *save; char *line = strtok_r(buf, "\n", &save);
	for (; line; line = strtok_r(0, "\n", &save)) {
		char copy[512]; strncpy(copy, line, sizeof(copy) - 1); copy[sizeof(copy)-1]=0;
		char *tok[MAXTOK]; int n = tokenize(copy, tok);
		if (n == 0 || tok[0][0] == '#') continue;
		if (!strcmp(tok[0], "reset")) {
			memset(&ps, 0, sizeof(ps)); snap = 0;
		} else if (!strcmp(tok[0], "snap")) {
			snap = pi(tok[1]);
		} else if (!strcmp(tok[0], "ps")) {
			ps_set(&ps, n - 1, tok + 1);
		} else if (!strcmp(tok[0], "run")) {
			entityState_t s; memset(&s, 0, sizeof(s));
			BG_PlayerStateToEntityState(&ps, &s, snap ? qtrue : qfalse);
			dump_es(tok[1], &ps, &s);
		} else {
			fprintf(stderr, "ps2es: unknown cmd '%s'\n", tok[0]); abort();
		}
	}
	free(buf);
}

// BG_PlayerStateToEntityStateExtraPolate over the SAME ps.txt scenarios with a
// fixed extrapolation time and snap=0. It differs from the base only in
// pos.trType (TR_LINEAR_STOP) + pos.trTime/trDuration, so it reuses dump_es and
// appends those two fields.
void BG_PlayerStateToEntityStateExtraPolate(playerState_t *ps, entityState_t *s, int time, qboolean snap);
#define XP_TIME 12345
static void sec_ps2esxp(const char *dir) {
	char path[1024]; snprintf(path, sizeof(path), "%s/ps.txt", dir);
	char *buf = slurp(path, 0);
	printf("== ps2esxp ==\n");
	playerState_t ps; memset(&ps, 0, sizeof(ps));
	char *save; char *line = strtok_r(buf, "\n", &save);
	for (; line; line = strtok_r(0, "\n", &save)) {
		char copy[512]; strncpy(copy, line, sizeof(copy) - 1); copy[sizeof(copy)-1]=0;
		char *tok[MAXTOK]; int n = tokenize(copy, tok);
		if (n == 0 || tok[0][0] == '#') continue;
		if (!strcmp(tok[0], "reset")) {
			memset(&ps, 0, sizeof(ps));
		} else if (!strcmp(tok[0], "snap")) {
			/* ignored: extrapolate dumped with snap=0 only */
		} else if (!strcmp(tok[0], "ps")) {
			ps_set(&ps, n - 1, tok + 1);
		} else if (!strcmp(tok[0], "run")) {
			entityState_t s; memset(&s, 0, sizeof(s));
			BG_PlayerStateToEntityStateExtraPolate(&ps, &s, XP_TIME, qfalse);
			dump_es(tok[1], &ps, &s);
			pI(" pos.trTime", s.pos.trTime);
			pI(" pos.trDuration", s.pos.trDuration);
		}
	}
	free(buf);
}

int main(int argc, char **argv) {
	if (argc != 2) { fprintf(stderr, "usage: %s <fixture-dir>\n", argv[0]); return 2; }
	const char *dir = argv[1];
	printf("== bgmisc ==\n");
	sec_trajectory(dir);
	sec_itemlist();
	sec_findid();
	sec_weapondata();
	sec_grab(dir);
	sec_ps2es(dir);
	sec_ps2esxp(dir);
	printf("== end ==\n");
	return 0;
}

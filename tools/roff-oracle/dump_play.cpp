// roff-oracle — Golden B (playback trace). Drives Play + N x UpdateEntities
// against the deterministic host, recording per frame the SetLerp writes
// (trType/trTime/trBase/trDelta on s.pos & s.apos, as raw IEEE-754 bits), the
// note-track VM_Call emissions (GAME_ROFF_NOTETRACK_CALLBACK args), mIsRoffing,
// next_roff_time, and the kill/erase decisions via mROFFEntList size. This pins
// ApplyROFF, ProcessNote, ClearLerp, PurgeEnt and UpdateEntities ordering. Covers
// non-translated, translated (AngleVectors path), v2 note firing, the
// roff-not-found error+kill path, and PurgeEnt. See docs/subsystems/roff.md
// § Verification strategy.
//
// Same `#define private public` idiom as dump_cache.cpp (STL first); oracle
// RoffSystem.h never edited.
#include <vector>
#include <map>
#include <string>
#define private public
#define protected public
#include "RoffSystem.h"
#undef private
#undef protected

#include "server/server.h"
#include "host.h"
#include <cstdio>
#include <cstring>
#include <cstdint>

static uint32_t fbits(float f) { uint32_t u; memcpy(&u, &f, 4); return u; }

static void dump_traj(const char *tag, trajectory_t &t) {
	printf("  %-4s type=%d time=%d base=0x%08x,0x%08x,0x%08x delta=0x%08x,0x%08x,0x%08x\n",
	       tag, (int)t.trType, t.trTime,
	       fbits(t.trBase[0]), fbits(t.trBase[1]), fbits(t.trBase[2]),
	       fbits(t.trDelta[0]), fbits(t.trDelta[1]), fbits(t.trDelta[2]));
}

static void dump_ent(int entnum) {
	sharedEntity_t *e = SV_GentityNum(entnum);
	dump_traj("pos", e->s.pos);
	dump_traj("apos", e->s.apos);
	printf("  r.mIsRoffing=%d next_roff_time=%d curOrigin=0x%08x,0x%08x,0x%08x\n",
	       (int)e->r.mIsRoffing, e->next_roff_time,
	       fbits(e->r.currentOrigin[0]), fbits(e->r.currentOrigin[1]), fbits(e->r.currentOrigin[2]));
}

// Clear all system state between scenarios (harness setup, not oracle behaviour):
// Clean() unloads every roff; the ent list has no public clear, so drain it here.
static void reset() {
	theROFFSystem.Clean(qfalse);
	for (auto *e : theROFFSystem.mROFFEntList) delete e;
	theROFFSystem.mROFFEntList.clear();
	theROFFSystem.mID = 0;
	host_reset_entities();
	host_note_clear();
}

static void run_playback(const char *label, const char *file, int entnum,
                         qboolean translate, float pitch, float yaw, float roll,
                         int startTime, int frames) {
	reset();
	int id = theROFFSystem.Cache(file, qfalse);
	host_set_time(startTime);
	host_set_ent_angles(entnum, pitch, yaw, roll);
	int frameTime = theROFFSystem.mROFFList[id]->mFrameTime;
	printf("### %s: file=%s id=%d ent=%d translate=%d startAngles=(%g,%g,%g) frameTime=%d\n",
	       label, file, id, entnum, (int)translate, pitch, yaw, roll, frameTime);
	theROFFSystem.Play(entnum, id, translate, qfalse);
	for (int fr = 0; fr < frames; fr++) {
		printf("- frame %d @ svs.time=%d entList=%zu\n", fr, host_get_time(),
		       theROFFSystem.mROFFEntList.size());
		theROFFSystem.UpdateEntities(qfalse);
		dump_ent(entnum);
		for (int n = 0; n < host_note_count(); n++) {
			printf("  NOTE callNum=%d entnum=%d text=\"%s\"\n",
			       host_note_callnum(n), host_note_entnum(n), host_note_text(n));
		}
		host_note_clear();
		printf("  entList after=%zu\n", theROFFSystem.mROFFEntList.size());
		host_set_time(host_get_time() + frameTime);
	}
	printf("\n");
}

// roff-not-found: Play, then Unload the roff so UpdateEntities' map lookup misses
// -> error print + mKill + ClearLerp (TR_STATIONARY on both trajectories).
static void run_not_found() {
	reset();
	int id = theROFFSystem.Cache("v1_basic.rof", qfalse);
	host_set_time(1000);
	int entnum = 4;
	theROFFSystem.Play(entnum, id, qfalse, qfalse);
	printf("### roff-not-found: cached id=%d then Unload before UpdateEntities\n", id);
	theROFFSystem.Unload(id);
	printf("- UpdateEntities (roff gone) entList=%zu\n", theROFFSystem.mROFFEntList.size());
	theROFFSystem.UpdateEntities(qfalse);
	dump_ent(entnum);
	printf("  entList after=%zu\n\n", theROFFSystem.mROFFEntList.size());
}

// PurgeEnt: two roffing ents, purge one by id (success), purge a missing id (fail
// + error print).
static void run_purge() {
	reset();
	int id = theROFFSystem.Cache("v1_basic.rof", qfalse);
	host_set_time(1000);
	theROFFSystem.Play(5, id, qfalse, qfalse);
	theROFFSystem.Play(6, id, qfalse, qfalse);
	printf("### purge_ent: two ents playing, entList=%zu\n", theROFFSystem.mROFFEntList.size());
	qboolean pr = theROFFSystem.PurgeEnt(5, qfalse);
	printf("purge_ent(5) = %d entList=%zu\n", (int)pr, theROFFSystem.mROFFEntList.size());
	qboolean pr2 = theROFFSystem.PurgeEnt(99, qfalse);
	printf("purge_ent(99 missing) = %d entList=%zu\n\n", (int)pr2, theROFFSystem.mROFFEntList.size());
}

int main() {
	printf("=== scenario 1: non-translated v1 playback ===\n");
	run_playback("non-translated v1", "v1_basic.rof", 1, qfalse, 0, 0, 0, 1000, 5);

	printf("=== scenario 2: translated v1 playback (yaw 90) ===\n");
	run_playback("translated v1", "v1_basic.rof", 2, qtrue, 0, 90, 0, 1000, 5);

	printf("=== scenario 3: v2 note firing ===\n");
	run_playback("v2 notes", "v2_notes.rof", 3, qfalse, 0, 0, 0, 2000, 4);

	printf("=== scenario 4: roff-not-found error path ===\n");
	run_not_found();

	printf("=== scenario 5: purge_ent ===\n");
	run_purge();

	return 0;
}

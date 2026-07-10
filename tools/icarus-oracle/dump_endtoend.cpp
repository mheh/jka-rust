// End-to-end ICARUS golden dumper (icarus.md § Verification strategy, unit 3:
// "Sequencer + TaskManager + Instance (end-to-end)"). Drives ICARUS_Init ->
// ICARUS_InitEnt -> ICARUS_RunScript on a committed .IBI fixture, then advances
// the mock clock frame-by-frame calling CTaskManager::Update (the per-entity
// heartbeat sv_game.cpp:769 dispatches). The golden is the ordered
// VM_Call(gvm, GAME_ICARUS_*) callback trace the sequencer/taskmanager emit plus
// the final script-variable + signal state. The Rust port's sequencer stack must
// reproduce this through its own EngineHostView/MockHost front door.
#include "exe_headers.h"
#include "../game/g_public.h"
#include "../server/server.h"
#include "icarus.h"
#include "GameInterface.h"
#include "Q3_Registers.h"

#include <cstdio>
#include <cstring>
#include <vector>

extern std::vector<int> g_vmTrace;
extern sharedEntity_t   g_entities[MAX_GENTITIES];

// GAME_ICARUS_* callnum -> name (g_public.h:770-787), for a readable trace.
struct NameEnt { int v; const char *n; };
static const NameEnt kIcarusCalls[] = {
	{ GAME_ICARUS_PLAYSOUND, "PLAYSOUND" }, { GAME_ICARUS_SET, "SET" },
	{ GAME_ICARUS_LERP2POS, "LERP2POS" }, { GAME_ICARUS_LERP2ORIGIN, "LERP2ORIGIN" },
	{ GAME_ICARUS_LERP2ANGLES, "LERP2ANGLES" }, { GAME_ICARUS_GETTAG, "GETTAG" },
	{ GAME_ICARUS_LERP2START, "LERP2START" }, { GAME_ICARUS_LERP2END, "LERP2END" },
	{ GAME_ICARUS_USE, "USE" }, { GAME_ICARUS_KILL, "KILL" },
	{ GAME_ICARUS_REMOVE, "REMOVE" }, { GAME_ICARUS_PLAY, "PLAY" },
	{ GAME_ICARUS_GETFLOAT, "GETFLOAT" }, { GAME_ICARUS_GETVECTOR, "GETVECTOR" },
	{ GAME_ICARUS_GETSTRING, "GETSTRING" }, { GAME_ICARUS_SOUNDINDEX, "SOUNDINDEX" },
	{ GAME_ICARUS_GETSETIDFORSTRING, "GETSETIDFORSTRING" },
};
static const char *callName(int v)
{
	for (size_t i = 0; i < sizeof(kIcarusCalls)/sizeof(kIcarusCalls[0]); i++)
		if (kIcarusCalls[i].v == v) return kIcarusCalls[i].n;
	return "?";
}

int main(int argc, char **argv)
{
	if (argc != 2) { fprintf(stderr, "usage: %s <fixtures/name>  (reads <name>.IBI)\n", argv[0]); return 2; }
	const char *script = argv[1];

	printf("== icarus_endtoend %s ==\n", script);

	// Mock entity 0: a valid, unfrozen script user with stable name strings.
	sharedEntity_t *ent = &g_entities[0];
	memset(ent, 0, sizeof(*ent));
	ent->s.number = 0;
	ent->r.svFlags = 0;                       // no SVF_ICARUS_FREEZE
	static char cn[] = "func_test", tn[] = "test1", stn[] = "test1";
	ent->classname = cn; ent->targetname = tn; ent->script_targetname = stn;

	// The outbound Q3_* slots marshal their T_G_ICARUS_* args through
	// sv.mSharedMemory (alias of the game's gSharedBuffer[8192]) before VM_Call.
	sv.mSharedMemory = (char *)calloc(1, 8192);

	ICARUS_Init();
	ICARUS_InitEnt(ent);
	printf("init ok\n");

	svs.time = 0;
	int r = ICARUS_RunScript(ent, script);
	printf("runscript ret=%d\n", r);

	// Advance the mock clock and beat the task manager. 200ms/frame for ~6s of
	// script time lets wait()/timed tasks resolve deterministically.
	for (int frame = 0; frame < 30; frame++)
	{
		svs.time += 200;
		if (gTaskManagers[0]) gTaskManagers[0]->Update();
	}

	printf("-- vm_call trace (%zu) --\n", g_vmTrace.size());
	for (size_t i = 0; i < g_vmTrace.size(); i++)
		printf("  %zu %s\n", i, callName(g_vmTrace[i]));

	printf("-- variables --\n");
	for (varFloat_m::iterator i = varFloats.begin(); i != varFloats.end(); ++i)
		printf("  F |%s|=%.3f\n", i->first.c_str(), i->second);
	for (varString_m::iterator i = varStrings.begin(); i != varStrings.end(); ++i)
		printf("  S |%s|=|%s|\n", i->first.c_str(), i->second.c_str());
	for (varString_m::iterator i = varVectors.begin(); i != varVectors.end(); ++i)
		printf("  V |%s|=|%s|\n", i->first.c_str(), i->second.c_str());

	printf("-- signals --\n");
	const char *sigs[] = { "go", "ready" };
	for (size_t i = 0; i < sizeof(sigs)/sizeof(sigs[0]); i++)
		printf("  |%s|=%d\n", sigs[i], iICARUS ? (int)iICARUS->CheckSignal(sigs[i]) : -1);

	printf("== end ==\n");
	return 0;
}

// trmodel-oracle — cache golden (docs/subsystems/tr-model.md § Verification
// strategy, "Cache goldens"). Three modes (one committed golden each):
//  - hitmiss     : first register = disk miss (FS reads); after R_HunkClearCrap +
//                  R_ModelInit the disk images survive in CachedModels, so the
//                  re-register is served from cache with zero FS reads
//                  (pqbAlreadyFound flips true, the Malloc repeat branch).
//  - evict       : level-keyed eviction — RE_RegisterModels_LevelLoadEnd with
//                  r_modelpoolmegs=0 dumps entries whose iLastLevelUsedOn is
//                  stale; surviving CachedModels keys printed in std::map/BTreeMap
//                  sorted order via RE_RegisterModels_Info_f.
//  - dumpnonpure : RE_RegisterModels_DumpNonPure (reached through
//                  RE_RegisterMedia_LevelLoadBegin with sv_pure=1) evicts entries
//                  whose FS_FileIsInPAK checksum no longer matches (the 1/-1
//                  convention, ruling 59) — never *default.gla.
#include <cstdio>
#include <cstring>
#include "tr_local.h"
#include "host.h"

static void bind_cvars() {
	sv_pure          = Cvar_Get("sv_pure", "0", 0);
	r_modelpoolmegs  = Cvar_Get("r_modelpoolmegs", "0", 0);
}

static void mode_hitmiss() {
	R_SVModelInit();
	bind_cvars();

	host_fs_reads = 0;
	qhandle_t h1 = RE_RegisterServerModel("models/test.glm");
	printf("=== first register (disk miss) ===\n");
	printf("handle=%d  FS disk reads=%d\n", h1, host_fs_reads);
	printf("cache after first register:\n");
	RE_RegisterModels_Info_f();

	// Drop the model pool + hash but keep CachedModels, then re-init the null
	// model. The re-register now hits the cached disk images.
	R_HunkClearCrap();
	R_ModelInit();

	host_fs_reads = 0;
	qhandle_t h2 = RE_RegisterServerModel("models/test.glm");
	printf("\n=== re-register after HunkClear+ModelInit (cache hit) ===\n");
	printf("handle=%d  FS disk reads=%d (0 == served from CachedModels)\n", h2, host_fs_reads);
	printf("cache after re-register:\n");
	RE_RegisterModels_Info_f();
}

static void mode_evict() {
	R_SVModelInit();
	bind_cvars();
	r_modelpoolmegs->integer = 0;   // force the pool-megs gate open

	RE_RegisterMedia_LevelLoadBegin("map1", eForceReload_NOTHING);   // level -> 1
	RE_RegisterServerModel("models/test.glm");                       // stamped lvl 1
	printf("=== after level 1 register (GetLevel=%d) ===\n", RE_RegisterMedia_GetLevel());
	RE_RegisterModels_Info_f();

	RE_RegisterMedia_LevelLoadBegin("map2", eForceReload_NOTHING);   // level -> 2
	RE_RegisterServerModel("models/modelb.glm");                     // stamped lvl 2
	printf("\n=== after level 2 register (GetLevel=%d) ===\n", RE_RegisterMedia_GetLevel());
	RE_RegisterModels_Info_f();

	qboolean freed = RE_RegisterModels_LevelLoadEnd(qfalse);
	printf("\n=== LevelLoadEnd(qfalse), r_modelpoolmegs=0 -> evict stale ===\n");
	printf("freed at least one=%d\n", freed);
	printf("survivors (sorted):\n");
	RE_RegisterModels_Info_f();
}

static void mode_dumpnonpure() {
	R_SVModelInit();
	bind_cvars();

	// test.glm + its gla live in a pure PAK (checksums stamped at register);
	// modelb.glm + its gla are disk-only (stamp -1); *default.gla is program-
	// internal (stamp -1 too, but DumpNonPure must never dump it).
	host_pak_add("models/test.glm", 111);
	host_pak_add("skeletons/test.gla", 222);

	RE_RegisterServerModel("models/test.glm");
	RE_RegisterServerModel("models/modelb.glm");
	RE_RegisterServerModel("*default.gla");
	printf("=== registered (before DumpNonPure) ===\n");
	RE_RegisterModels_Info_f();

	sv_pure->integer = 1;
	RE_RegisterMedia_LevelLoadBegin("map2", eForceReload_NOTHING);   // -> DumpNonPure
	printf("\n=== after LevelLoadBegin(sv_pure=1) -> DumpNonPure ===\n");
	printf("survivors (pure pak matches + *default.gla, sorted):\n");
	RE_RegisterModels_Info_f();
}

int main(int argc, char **argv) {
	const char *mode = argc > 1 ? argv[1] : "hitmiss";
	if      (!strcmp(mode, "hitmiss"))     mode_hitmiss();
	else if (!strcmp(mode, "evict"))       mode_evict();
	else if (!strcmp(mode, "dumpnonpure")) mode_dumpnonpure();
	else { fprintf(stderr, "unknown mode %s\n", mode); return 2; }
	return 0;
}

// RMG differential-oracle dumper. Compiled ONCE against the UNMODIFIED Raven
// TUs (cm_terrain.cpp, cm_randomterrain.cpp, RM_Manager.cpp) + the real
// GenericParser2.cpp, WITH -DDEDICATED (ruling 25 / RMG-D1) so the compiled
// behavior is the shipped dedicated server: generation dead, LoadMission
// early-out. Emits one of two canonical dumps; the Rust ports must reproduce
// them byte-for-byte. See README.md for the golden -> doc-section map.
//
//   ./rmg_dump seed        -> golden #1  (RMG_CreateSeed, rmg-terrain.md 1249)
//   ./rmg_dump dedicated   -> golden #4  (dedicated outcome, rmg-terrain.md 1256)
#include <cstdio>
#include <cstring>
#include <stdexcept>

#include "exe_headers.h"
#include "cm_local.h"
#include "cm_landscape.h"
#include "cm_randomterrain.h"
#include "RM_Headers.h"   // rmAutomapSymbol_t + CRMMission stub + real RM_Manager.h

// A DEDICATED-server RMG terrain config infostring. Non-empty heightMap key (so
// the ctor takes the load branch, which under DEDICATED forces imageData=NULL —
// no image load, no generation); terrainDef names the committed .terrain
// fixture (ext_data/RMG/dedicated.terrain).
static const char *CONFIG =
	"\\heightMap\\height.tga\\terrainDef\\dedicated\\numPatches\\256\\terxels\\4"
	"\\physics\\1\\seed\\1234"
	"\\minx\\-1024\\miny\\-1024\\minz\\-256\\maxx\\1024\\maxy\\1024\\maxz\\256";

// Same, but with an EMPTY heightMap value -> the ctor's else branch fatals
// (Com_Error ERR_FATAL) — the live EngineHost::error use (cm_terrain.cpp:190-193).
static const char *CONFIG_NO_HEIGHTMAP =
	"\\heightMap\\\\terrainDef\\dedicated\\numPatches\\256\\terxels\\4"
	"\\physics\\1\\seed\\1234"
	"\\minx\\-1024\\miny\\-1024\\minz\\-256\\maxx\\1024\\maxy\\1024\\maxz\\256";

// -------------------------------------------------------------------------
// Golden #1 — the platform-width holdrand LCG substrate (rmg-terrain.md:1248-1251
// "pinning the engine LCG via EngineHost::flrand/irand", RMG-D4f; determinism
// anchor :1279-1284). This pins the exact deterministic draw sequence that
// RMG_CreateSeed and the ctor seed consume, over the same small ranges
// RMG_CreateSeed draws (irand 4..9 / 0..100 / weighted picks) plus flrand.
//
// DEVIATION (§F.19 / §19): the doc's vehicle — dumping RMG_CreateSeed's
// seed-STRING — is undefined behavior at the RULED width. `holdrand` is
// platform-width `c_ulong` (64-bit on this LP64 build, ruling 2026-07-09,
// jampgame-fork-discovery; matching the Rust Rng), so `result = holdrand >> 17`
// pulls high bits and `irand(a,b)` returns values far outside `[a,b]` (e.g.
// irand(0,50) -> -28577). RMG_CreateSeed's FindPiece then walks its weighted
// table unbounded (cm_randomterrain.cpp:990-1005) and reads out of bounds — a
// crash no Rust port (`.get()` -> None) can reproduce. Per §19 the golden pins
// the DEFINED substrate (the LCG draws) the helper is built on, not the UB
// string. Drift shows as a first-diverging state word or draw.
// -------------------------------------------------------------------------
static void dumpSeed( void ) {
	printf( "== holdrand LCG substrate (platform-width c_ulong; RMG-D4f) ==\n" );
	printf( "sizeof(unsigned long)=%d\n", (int)sizeof( unsigned long ) );
	static const unsigned seeds[] = { 0x89abcdefu, 1, 42, 1234567 };
	for ( int s = 0; s < (int)( sizeof( seeds ) / sizeof( seeds[0] ) ); s++ ) {
		Rand_Init( (int)seeds[s] );
		printf( "-- seed 0x%08x  state=0x%016lx --\n", (unsigned)seeds[s], rng_state() );
		for ( int i = 0; i < 6; i++ ) {
			int   a = irand( 4, 9 );     unsigned long sa = rng_state();
			int   b = irand( 0, 100 );   unsigned long sb = rng_state();
			int   c = irand( 0, 255 );   unsigned long sc = rng_state();
			float d = flrand( -1.0f, 1.0f ); unsigned long sd = rng_state();
			float e = flrand( 0.0f, 2.0f );  unsigned long se = rng_state();
			printf( "  [%d] irand4_9=%d s=0x%016lx | irand0_100=%d s=0x%016lx | "
			        "irand0_255=%d s=0x%016lx | flrandm1_1=%.9g s=0x%016lx | "
			        "flrand0_2=%.9g s=0x%016lx\n",
			        i, a, sa, b, sb, c, sc, d, sd, e, se );
		}
	}
	printf( "== end ==\n" );
}

// -------------------------------------------------------------------------
// Golden #4 — the dedicated-server outcome (rmg-terrain.md:1256-1278). Builds
// the CmLandScape under DEDICATED (its LoadTerrainDef GP2-parses the committed
// .terrain fixture, altitudetexture/water cases reading the stubbed
// CM_GetShaderInfo), runs SetLandScape + LoadMission->false (no SpawnMission),
// and streams the snapshot reads. HEIGHTMAP BYTES ARE EXCLUDED (§F.19 UB:
// mHeightMap is Z_Malloc'd non-zeroing and never written under DEDICATED).
// -------------------------------------------------------------------------
static void dumpDedicated( void ) {
	printf( "== CCMLandScape construction (DEDICATED) ==\n" );
	CCMLandScape *ls = new CCMLandScape( CONFIG, true );

	printf( "dims: width=%d height=%d realArea=%d blockW=%d blockH=%d blockCount=%d terxels=%d\n",
		ls->GetWidth(), ls->GetHeight(), ls->GetRealArea(),
		ls->GetBlockWidth(), ls->GetBlockHeight(), ls->GetBlockCount(), ls->GetTerxels() );
	printf( "patchScalarSize=%.6f\n", ls->GetPatchScalarSize() );
	printf( "get_rand_seed=0x%08lx\n", ls->get_rand_seed() );

	// Snapshot stream: flatten map (memset-0, cm_terrain.cpp:161) — pinned as a
	// checksum + all-zero flag. The heightmap is intentionally NOT read (§F.19).
	const byte *fm    = ls->GetFlattenMap();
	int         area  = ls->GetRealArea();
	unsigned    fsum  = 0;
	bool        allz  = true;
	for ( int i = 0; i < area; i++ ) { fsum += fm[i]; if ( fm[i] ) allz = false; }
	printf( "flattenMap: %d bytes, sum=%u, all-zero=%s\n", area, fsum, allz ? "yes" : "no" );
	printf( "heightMap: EXCLUDED from golden (F.19 UB: unpopulated non-zeroing alloc)\n" );

	// LoadTerrainDef water case (cm_terrain.cpp:87-104) — the CM_GetShaderInfo
	// contract's defined water_contents()/water_surface_flags() (RMG-D8).
	printf( "water: baseHeight=%d height=%.6f contents=%d surfaceFlags=%d\n",
		ls->GetBaseWaterHeight(), ls->GetWaterHeight(),
		ls->GetWaterContents(), ls->GetWaterSurfaceFlags() );

	// LoadTerrainDef altitudetexture case (SetShaders) — per-height flags.
	printf( "altitude flags by height:\n" );
	static const int hs[] = { 0, 16, 32, 63, 64, 128, 255 };
	for ( int i = 0; i < (int)( sizeof( hs ) / sizeof( hs[0] ) ); i++ )
		printf( "  h=%3d surfaceFlags=%d contentFlags=%d\n",
			hs[i], ls->GetSurfaceFlags( hs[i] ), ls->GetContentFlags( hs[i] ) );

	// -- RMG lifecycle through the early-out (RM_Manager.cpp) --
	printf( "== RmManager lifecycle (LoadMission early-out) ==\n" );
	CRMManager mgr;
	mgr.SetLandScape( ls );
	bool r = mgr.LoadMission( qtrue );   // prints the #ifndef FINAL_BUILD banner
	printf( "LoadMission returned=%s\n", r ? "true" : "false" );
	printf( "automapSymbolCount=%d\n", mgr.GetAutomapSymbolCount() );

	// -- the ctor's empty-heightMap ERR_FATAL (live EngineHost::error) --
	printf( "== ctor empty-heightMap error path ==\n" );
	try {
		CCMLandScape *bad = new CCMLandScape( CONFIG_NO_HEIGHTMAP, true );
		printf( "ERROR: expected fatal, got %p\n", (void *)bad );
	} catch ( const std::exception &e ) {
		printf( "caught: %s\n", e.what() );
	}
	printf( "== end ==\n" );
}

int main( int argc, char **argv ) {
	if ( argc != 2 ) { fprintf( stderr, "usage: %s seed|dedicated\n", argv[0] ); return 2; }
	if      ( !strcmp( argv[1], "seed" ) )      dumpSeed();
	else if ( !strcmp( argv[1], "dedicated" ) ) dumpDedicated();
	else { fprintf( stderr, "unknown mode %s\n", argv[1] ); return 2; }
	return 0;
}

// ghoul2-server-oracle — surface-list golden dumper.
//
// Drives the UNMODIFIED generated-surface bookkeeping in oracle G2_surfaces.cpp:
// G2_AddSurface (:513), G2_FindOverrideSurface (:495), G2_RemoveSurface (:475).
// These manage a CGhoul2Info's mSlist of generated (offFlags=G2SURFACEFLAG_
// GENERATED, surface=10000 marker) surface entries with the packed
// genPolySurfaceIndex = ((poly&0xffff)<<16)|(surf&0xffff).
//
// The only model-memory touch on this path is G2_AddSurface -> G2_DecideTraceLod
// (G2_misc.cpp:376), which reads currentModel->mdxm->numLODs to clamp the lod. We
// satisfy that with a tiny in-dumper mdxmHeader_t (numLODs=2) — NO loader, NO
// disk fixture — so the run stays deterministic and asset-free. The name-lookup
// surface paths (G2_SetSurfaceOnOff / G2_IsSurfaceLegal, which walk the real mdxm
// surface hierarchy) are a documented gap (README).
//
// Only integers/markers are dumped, so run-twice byte-identical.
#include "codemp/game/q_shared.h"
#include "codemp/renderer/tr_local.h"
#include "codemp/ghoul2/G2_local.h"
#include <cstdio>

static void dump(const char *label, surfaceInfo_v &s) {
	printf("%-26s size=%d", label, (int)s.size());
	for (size_t i = 0; i < s.size(); i++)
		printf(" | [%zu] off=%d surf=%d poly=%d lod=%d",
			i, s[i].offFlags, s[i].surface, s[i].genPolySurfaceIndex, s[i].genLod);
	printf("\n");
}

int main() {
	// Minimal model just to satisfy G2_DecideTraceLod's currentModel->mdxm->numLODs
	// deref — no loader, no file.
	mdxmHeader_t hdr;
	hdr.numLODs = 2;
	model_t mod;
	mod.mdxm = &hdr;

	CGhoul2Info gh;
	gh.mValid = true;
	gh.mLodBias = 0;
	gh.currentModel = &mod;

	printf("== add generated surfaces ==\n");
	int s0 = G2_AddSurface(&gh, /*surf*/7, /*poly*/3, 0.25f, 0.5f, /*lod*/0);
	printf("add (surf=7,poly=3,lod=0) -> %d\n", s0);
	int s1 = G2_AddSurface(&gh, 9, 1, 0.1f, 0.2f, 1);
	printf("add (surf=9,poly=1,lod=1) -> %d\n", s1);
	dump("after 2 adds", gh.mSlist);

	printf("\n== lod clamp (lod>=numLODs -> numLODs-1) ==\n");
	int s2 = G2_AddSurface(&gh, 4, 4, 0, 0, /*lod=5 -> clamp 1*/5);
	printf("add (surf=4,poly=4,lod=5) -> %d (genLod clamped)\n", s2);
	dump("after clamp add", gh.mSlist);

	printf("\n== find override surface (matches surface==10000 marker) ==\n");
	surfaceInfo_t *f = G2_FindOverrideSurface(10000, gh.mSlist);
	printf("find 10000 -> %s (idx0)\n", f ? "found" : "null");
	printf("find 12345 (absent) -> %s\n",
		G2_FindOverrideSurface(12345, gh.mSlist) ? "found" : "null");

	printf("\n== remove middle then tail (tail resize) ==\n");
	// Remove idx1 (marks surface=-1, no tail resize since idx2 still active).
	printf("remove idx1 -> %d\n", (int)G2_RemoveSurface(gh.mSlist, 1));
	dump("after remove idx1", gh.mSlist);
	// Remove idx2 (tail): now idx1,idx2 both -1 -> resize drops both.
	printf("remove idx2 -> %d\n", (int)G2_RemoveSurface(gh.mSlist, 2));
	dump("after remove idx2 (resize)", gh.mSlist);

	printf("\n== add reuses the freed (-1) slot before growing ==\n");
	int s3 = G2_AddSurface(&gh, 2, 2, 0, 0, 0);
	printf("add (surf=2,poly=2) -> %d (reused freed slot)\n", s3);
	dump("after reuse add", gh.mSlist);

	return 0;
}

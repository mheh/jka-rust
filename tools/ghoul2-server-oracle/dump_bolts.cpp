// ghoul2-server-oracle — bolt-list golden dumper.
//
// Drives the UNMODIFIED bolt-list management in oracle G2_bolts.cpp over a
// synthetic (model-free) CGhoul2Info: G2_Add_Bolt_Surf_Num (:73),
// G2_Find_Bolt_Surface_Num (:44), G2_Find_Bolt_Bone_Num (:24), G2_Remove_Bolt
// (:238), G2_Init_Bolt_List (:283), G2_RemoveRedundantBolts (:289). These are the
// generated-surface / bone-slot bookkeeping paths that need NO model memory
// (surface-index based, not the name-lookup G2_Add_Bolt which reads mdxm/mdxa) —
// the model-name bolt path is a documented gap (README).
//
// Covers the doc's "Bolt goldens" list-management surface; the G2API_GetBoltMatrix
// write-through matrix math (G2SV-D1) needs a live bone cache and is a separate
// gap. Only integers are dumped (bolt slot fields), so run-twice byte-identical.
#include "codemp/game/q_shared.h"
#include "codemp/renderer/tr_local.h"
#include "codemp/ghoul2/G2_local.h"
#include <cstdio>

// Prototypes (declared in G2_local.h are pulled above; re-stated here for the two
// bolt fns G2_local.h omits — the internal finders).
int G2_Find_Bolt_Bone_Num(boltInfo_v &bltlist, const int boneNum);
int G2_Find_Bolt_Surface_Num(boltInfo_v &bltlist, const int surfaceNum, const int flags);

static void dump(const char *label, boltInfo_v &b) {
	printf("%-26s size=%d", label, (int)b.size());
	for (size_t i = 0; i < b.size(); i++)
		printf(" | [%zu] bone=%d surf=%d type=%d used=%d",
			i, b[i].boneNumber, b[i].surfaceNumber, b[i].surfaceType, b[i].boltUsed);
	printf("\n");
}

int main() {
	CGhoul2Info gh;
	gh.mValid = true;                 // asserts are NDEBUG no-ops; kept faithful
	surfaceInfo_v slist;              // 4 generated-surface slots (indices 0..3)
	slist.resize(4);
	boltInfo_v bolts;

	printf("== add generated-surface bolts ==\n");
	int a0 = G2_Add_Bolt_Surf_Num(&gh, bolts, slist, 2);
	printf("add surf 2 -> %d\n", a0);
	int a1 = G2_Add_Bolt_Surf_Num(&gh, bolts, slist, 0);
	printf("add surf 0 -> %d\n", a1);
	dump("after 2 adds", bolts);

	printf("\n== duplicate add bumps boltUsed ==\n");
	int a2 = G2_Add_Bolt_Surf_Num(&gh, bolts, slist, 2);   // existing -> ++used
	printf("re-add surf 2 -> %d\n", a2);
	dump("after re-add", bolts);

	printf("\n== add out-of-range surface (>= slist.size) ==\n");
	int a3 = G2_Add_Bolt_Surf_Num(&gh, bolts, slist, 9);   // 9 >= 4 -> -1
	printf("add surf 9 -> %d\n", a3);

	printf("\n== finders ==\n");
	printf("find surf 2 (flags=G2SURFACEFLAG_GENERATED) -> %d\n",
		G2_Find_Bolt_Surface_Num(bolts, 2, G2SURFACEFLAG_GENERATED));
	printf("find surf 0 (flags=0) -> %d\n", G2_Find_Bolt_Surface_Num(bolts, 0, 0));
	printf("find surf 3 (absent) -> %d\n", G2_Find_Bolt_Surface_Num(bolts, 3, 0));
	printf("find bone -1 (none set) -> %d\n", G2_Find_Bolt_Bone_Num(bolts, 0));

	printf("\n== remove (boltUsed decrement + tail resize) ==\n");
	// surf 2 was added twice (used=2): first remove just decrements.
	int i2 = G2_Find_Bolt_Surface_Num(bolts, 2, G2SURFACEFLAG_GENERATED);
	printf("remove idx %d (used 2->1) -> %d\n", i2, (int)G2_Remove_Bolt(bolts, i2));
	dump("after 1st remove", bolts);
	printf("remove idx %d again (used 1->0, frees slot) -> %d\n", i2,
		(int)G2_Remove_Bolt(bolts, i2));
	dump("after 2nd remove", bolts);

	printf("\n== RemoveRedundantBolts (drop bolts to inactive surfaces) ==\n");
	// Re-add two surface bolts, then mark surface 0 inactive.
	G2_Add_Bolt_Surf_Num(&gh, bolts, slist, 0);
	G2_Add_Bolt_Surf_Num(&gh, bolts, slist, 1);
	dump("before prune", bolts);
	int activeSurfaces[4] = { 0, 1, 1, 1 };   // surface 0 inactive
	int activeBones[4]    = { 1, 1, 1, 1 };
	G2_RemoveRedundantBolts(bolts, slist, activeSurfaces, activeBones);
	dump("after prune (surf0 gone)", bolts);

	printf("\n== Init clears the list ==\n");
	G2_Init_Bolt_List(bolts);
	dump("after init", bolts);

	return 0;
}

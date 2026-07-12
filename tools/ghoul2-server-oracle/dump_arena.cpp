// ghoul2-server-oracle — arena/handle golden dumper.
//
// Drives the UNMODIFIED Ghoul2InfoArray singleton (oracle G2_API.cpp:310-493)
// through New / IsValid / Delete, dumping only the packed handle integers (slot
// index in the low G2_MODEL_BITS=10 bits, the generation counter in the high
// bits) — never a pointer/address — so the dump is run-twice byte-identical.
// This is the doc's "Arena/handle goldens" unit (docs/subsystems/ghoul2-server.md
// § Verification strategy: "New/Delete/IsValid handle values across the
// generation rollover", G2SV-D6).
//
// The arena reaches the engine seam only on slot exhaustion (Com_Error) and, in
// DeleteLow, RemoveBoneCache — neither is hit here (no instance ever builds a
// bone cache), so the run is a pure function of the handle arithmetic.
//
// NOTE (§F19, see README § Normalizations): the "rollover RESET" arm of
// DeleteLow (G2_API.cpp:328-333) is UNREACHABLE without signed-overflow UB — the
// int mId reaches exactly 2^31 (== INT_MIN) at the generation the reset test
// `(mId>>10) > (1<<21)` would first fire, so the test never sees a positive
// over-threshold value. This dumper therefore goldens the DEFINED surface: the
// per-Delete generation bump (+MAX_G2_MODELS), LIFO slot reuse, stale-handle
// invalidation, and multi-slot ordering — and keeps the UB rollover out of the
// shared golden.
#include "codemp/game/q_shared.h"
#include "codemp/renderer/tr_local.h"
#include "codemp/ghoul2/ghoul2_shared.h"
#include <cstdio>

void Ghoul2InfoArray_Free(void);  // G2_API.cpp:487 (no public header decl)

#define G2_MODEL_BITS 10
#define MAX_G2_MODELS 1024
#define G2_INDEX_MASK (MAX_G2_MODELS - 1)

static void show(const char *label, int h) {
	IGhoul2InfoArray &a = TheGhoul2InfoArray();
	printf("%-24s handle=%d idx=%d gen=%d valid=%d\n",
		label, h, h & G2_INDEX_MASK, h >> G2_MODEL_BITS, a.IsValid(h) ? 1 : 0);
}

int main() {
	IGhoul2InfoArray &a = TheGhoul2InfoArray();

	printf("== fresh new / initial generation ==\n");
	// The ctor seeds mIds[i] = MAX_G2_MODELS+i (gen 1) and frees 0..1023 in order,
	// so the first New() pops idx 0 with handle 1024.
	int h0 = a.New();
	show("new h0", h0);
	int h1 = a.New();
	show("new h1", h1);
	int h2 = a.New();
	show("new h2", h2);

	printf("\n== IsValid predicate ==\n");
	printf("IsValid(0)       = %d  (null handle -> false)\n", a.IsValid(0));
	printf("IsValid(h0)      = %d\n", a.IsValid(h0));
	printf("IsValid(h0|junk) = %d  (stale generation -> false)\n",
		a.IsValid(h0 + MAX_G2_MODELS));

	printf("\n== delete + LIFO reuse + generation bump ==\n");
	// DeleteLow bumps mIds[idx] += MAX_G2_MODELS (gen+1) and push_front(idx), so
	// the very next New() re-pops the same slot with an incremented generation.
	a.Delete(h1);                 // idx1 -> gen2, front of free list
	show("after delete h1", h1);  // stale handle now invalid
	int r1 = a.New();             // reuse idx1
	show("reuse -> new r1", r1);
	a.Delete(r1);                 // idx1 -> gen3
	int r2 = a.New();             // reuse idx1 again
	show("reuse -> new r2", r2);
	a.Delete(r2);                 // idx1 -> gen4
	int r3 = a.New();
	show("reuse -> new r3", r3);

	printf("\n== multi-slot free-list ordering ==\n");
	// Delete h0 (idx0) then h2 (idx2): DeleteLow push_front, so the free list
	// front becomes idx2, then idx0. The next two New()s pop in that LIFO order.
	a.Delete(h0);                 // push_front(idx0)
	a.Delete(h2);                 // push_front(idx2) -> now ahead of idx0
	int n1 = a.New();
	show("new after 2 deletes", n1);   // expect idx2 (LIFO)
	int n2 = a.New();
	show("new again", n2);             // expect idx0

	Ghoul2InfoArray_Free();
	return 0;
}

// ghoul2-server-oracle — deterministic host / link stubs.
//
// Provides the small set of engine-seam symbols the UNMODIFIED ghoul2 server
// TUs (G2_API.cpp arena, G2_bolts.cpp, G2_surfaces.cpp) reference on the code
// paths the dumpers drive. Everything the dumpers do NOT reach is dead-stripped
// at link (-Wl,-dead_strip), so only the live-path symbols below need a body.
//
// All behaviour is fully deterministic and no raw pointer/address is ever
// emitted, so every golden is run-twice byte-identical. oracle/ is never edited.
#include "codemp/game/q_shared.h"
#include "codemp/renderer/tr_local.h"
#include <cstdio>
#include <cstdlib>
#include <cstdarg>

// --- console / fatal --------------------------------------------------------
// Com_Error is only reached on arena slot exhaustion (G2_API.cpp:388); the
// dumpers never exhaust the 1024 slots, so this is a link stub that would abort
// loudly if it ever fired (matching the oracle's ERR_FATAL semantics).
extern "C" void Com_Error(int level, const char *fmt, ...) {
	va_list ap; va_start(ap, fmt);
	fprintf(stderr, "Com_Error(%d): ", level); vfprintf(stderr, fmt, ap);
	fprintf(stderr, "\n"); va_end(ap);
	exit(1);
}
extern "C" void Com_Printf(const char *fmt, ...) {
	va_list ap; va_start(ap, fmt); vprintf(fmt, ap); va_end(ap);
}

// --- lod helper -------------------------------------------------------------
// G2_AddSurface (G2_surfaces.cpp:517) calls G2_DecideTraceLod, which lives in
// G2_misc.cpp — a TU that pulls the entire collision/gore/server closure and so
// is out of this harness's standalone-compile scope. Rather than link that whole
// TU, the helper is transcribed VERBATIM here (oracle G2_misc.cpp:376-395) so the
// surface golden's genLod clamp is faithful. The Rust port verifies the real
// G2_DecideTraceLod under misc.rs, not here; this is a harness-local helper.
#include "codemp/ghoul2/ghoul2_shared.h"
int G2_DecideTraceLod(CGhoul2Info &ghoul2, int useLod) {
	int returnLod = useLod;
	if (ghoul2.mLodBias > returnLod)
		returnLod = ghoul2.mLodBias;
	if (returnLod >= ghoul2.currentModel->mdxm->numLODs)
		returnLod = ghoul2.currentModel->mdxm->numLODs - 1;
	return returnLod;
}

// --- bone cache -------------------------------------------------------------
// The arena's DeleteLow frees each instance's CBoneCache (G2_API.cpp:321). The
// arena dumper never populates mBoneCache (no skeleton is ever built), so this
// is never called; a no-op link stub keeps DeleteLow's generation arithmetic —
// the actual golden surface — reachable.
void RemoveBoneCache(class CBoneCache *toDelete) { (void)toDelete; }

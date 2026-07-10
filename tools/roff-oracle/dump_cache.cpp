// roff-oracle — Golden A (parse / cache). Drives the unmodified
// Cache/IsROFF/InitROFF/InitROFF2/FixBadAngles/GetID over the hand-authored
// fixtures and dumps, per cached roff: mROFFEntries, mFrameTime, mLerp,
// mNumNoteTracks, every mMoveRotateList entry AFTER FixBadAngles (as raw IEEE-754
// bits, so the Rust parity test matches byte-for-byte), the note-track strings,
// and the mROFFList ID ordering (ROFF-D4). Also exercises the reject paths
// (bad version / bad count) and re-cache idempotency. See docs/subsystems/roff.md
// § Verification strategy.
//
// Private members are reached with the standard `#define private public` idiom
// (STL headers are pulled in FIRST, un-macro'd, so libstdc++ is unaffected); the
// oracle RoffSystem.h is never edited.
#include <vector>
#include <map>
#include <string>
#define private public
#define protected public
#include "RoffSystem.h"
#undef private
#undef protected

#include "host.h"
#include <cstdio>
#include <cstring>
#include <cstdint>

static uint32_t fbits(float f) { uint32_t u; memcpy(&u, &f, 4); return u; }

static void dump_croff(CROFFSystem::CROFF *c) {
	printf("  entries=%d frameTime=%d lerp=%d numNoteTracks=%d\n",
	       c->mROFFEntries, c->mFrameTime, c->mLerp, c->mNumNoteTracks);
	for (int i = 0; i < c->mROFFEntries; i++) {
		CROFFSystem::TROFF2Entry &e = c->mMoveRotateList[i];
		printf("  [%d] o=0x%08x,0x%08x,0x%08x r=0x%08x,0x%08x,0x%08x startNote=%d numNotes=%d\n",
		       i,
		       fbits(e.mOriginOffset[0]), fbits(e.mOriginOffset[1]), fbits(e.mOriginOffset[2]),
		       fbits(e.mRotateOffset[0]), fbits(e.mRotateOffset[1]), fbits(e.mRotateOffset[2]),
		       e.mStartNote, e.mNumNotes);
	}
	for (int i = 0; i < c->mNumNoteTracks; i++) {
		printf("  note[%d]=\"%s\"\n", i, c->mNoteTrackIndexes[i]);
	}
}

int main() {
	const char *files[] = {
		"v1_basic.rof", "v1_badangle.rof", "v2_notes.rof",
		"fallbackcase.rof", "bad_version.rof", "bad_count.rof",
	};

	printf("=== cache calls ===\n");
	for (const char *f : files) {
		int id = theROFFSystem.Cache(f, qfalse);
		printf("cache(\"%s\") = %d  GetID=%d\n", f, id, theROFFSystem.GetID(f));
	}

	// re-cache idempotency: existing roff returns its id, no new list entry.
	printf("\n=== re-cache idempotency ===\n");
	int again = theROFFSystem.Cache("v1_basic.rof", qfalse);
	printf("recache(\"v1_basic.rof\") = %d\n", again);

	printf("\n=== mROFFList (ascending id order) ===\n");
	for (auto &kv : theROFFSystem.mROFFList) {
		printf("id %d -> \"%s\"\n", kv.first, kv.second->mROFFFilePath);
	}

	printf("\n=== per-roff contents (post-FixBadAngles) ===\n");
	for (auto &kv : theROFFSystem.mROFFList) {
		printf("--- id %d: %s ---\n", kv.first, kv.second->mROFFFilePath);
		dump_croff(kv.second);
	}

	// Exercise the console debug methods (List / List(id)) — their formatted
	// output is part of Golden A (doc method table maps List -> A).
	printf("\n=== List() ===\n");
	theROFFSystem.List();
	printf("\n=== List(id) per roff ===\n");
	for (auto &kv : theROFFSystem.mROFFList) {
		theROFFSystem.List(kv.first);
	}
	printf("\n=== List(missing id 999) ===\n");
	theROFFSystem.List(999);

	return 0;
}

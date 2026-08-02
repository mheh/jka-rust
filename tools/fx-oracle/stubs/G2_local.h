// fx-oracle stub for `oracle/codemp/ghoul2/G2_local.h` and the
// `CGhoul2Info_v` handle wrapper in `oracle/codemp/ghoul2/ghoul2_shared.h`.
//
// The real pair drags in the model cache, the bone cache, the surface and
// collision tables and the whole `G2API_*` surface. The FX code uses four
// members of the wrapper (`mItem`, the int assignment, `kill`, the int
// constructor) plus one API call, so the harness declares those.
//
// The harness never allocates a ghoul2 instance. `mItem` is the scripted bolt
// slot and nothing dereferences it, so the destructor is empty. The real
// destructor frees the info array entry.
// Source: `oracle/codemp/ghoul2/ghoul2_shared.h:328-450`
#ifndef FX_ORACLE_G2_LOCAL_H
#define FX_ORACLE_G2_LOCAL_H

#include "../game/q_shared.h"

class CGhoul2Info_v
{
public:
	int mItem;

	CGhoul2Info_v() { mItem = 0; }
	CGhoul2Info_v(const int item) { mItem = item; }
	~CGhoul2Info_v() {}

	void operator=(const CGhoul2Info_v &other) { mItem = other.mItem; }
	void operator=(const int otherItem) { mItem = otherItem; }

	bool IsValid() const { return mItem != 0; }

	// Zeros the handle without freeing the instance behind it.
	void kill() { mItem = 0; }
};

// `SFxHelper::GetOriginAxisFromBolt` is the one caller. The harness answers
// from the scripted bolt queue.
// Source: `oracle/codemp/ghoul2/G2_local.h:141-142`
qboolean G2API_GetBoltMatrix(CGhoul2Info_v &ghoul2, const int modelIndex, const int boltIndex,
	mdxaBone_t *matrix, const vec3_t angles, const vec3_t position, const int frameNum,
	qhandle_t *modelList, vec3_t scale);

#endif // FX_ORACLE_G2_LOCAL_H

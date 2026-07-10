// trmodel-oracle — load/seam/handle golden. Pins:
//  - ServerLoadMDXM/ServerLoadMDXA header parse + in-place write-backs + the
//    glm<->gla animIndex recursion (§ Verification "Header-parse + endian goldens").
//  - model_mdxm/model_mdxa NULL-parity: SET where model_t.mdxm/.mdxa is non-NULL,
//    NULL otherwise (§ "Seam goldens", TRM-D3(b)).
//  - R_AllocModel/R_GetModelByHandle out-of-range -> models[0] fallback and the
//    MOD_BAD failed-entry retention: fail returns literal 0 but the entry stays
//    hashed under its nonzero index, so re-register returns that index
//    (§ "Handle/pool goldens", TRM-D3/ruling 53).
#include <cstdio>
#include "tr_local.h"
#include "host.h"

static const char *tname(modtype_t t) {
	switch (t) {
		case MOD_BAD:  return "MOD_BAD";
		case MOD_MDXM: return "MOD_MDXM";
		case MOD_MDXA: return "MOD_MDXA";
		default:       return "MOD_other";
	}
}

static void dump_model(const char *tag, qhandle_t h) {
	model_t *m = R_GetModelByHandle(h);
	printf("%s: handle=%d index=%d type=%s dataSize=%d numLods=%d name=\"%s\"\n",
		tag, h, m->index, tname(m->type), m->dataSize, m->numLods, m->name);
	printf("     seam: mdxm=%s mdxa=%s\n",
		m->mdxm ? "SET" : "NULL", m->mdxa ? "SET" : "NULL");
}

int main() {
	R_SVModelInit();
	printf("=== init ===\n");
	printf("numModels=%d  models[0].type=%s\n", tr.numModels, tname(R_GetModelByHandle(0)->type));

	printf("\n=== register models/test.glm (recurses skeletons/test.gla) ===\n");
	qhandle_t hglm = RE_RegisterServerModel("models/test.glm");
	printf("register returned handle=%d, numModels=%d\n", hglm, tr.numModels);
	dump_model("glm", hglm);

	model_t *glm = R_GetModelByHandle(hglm);
	mdxmHeader_t *mm = glm->mdxm;
	printf("  mdxmHeader: ident=0x%08x version=%d numBones=%d numLODs=%d ofsLODs=%d "
	       "numSurfaces=%d ofsSurfHierarchy=%d ofsEnd=%d animIndex=%d animName=\"%s\"\n",
	       mm->ident, mm->version, mm->numBones, mm->numLODs, mm->ofsLODs,
	       mm->numSurfaces, mm->ofsSurfHierarchy, mm->ofsEnd, mm->animIndex, mm->animName);

	// in-place write-back proof: the intel-live middle section poked surf->ident
	// = SF_MDX (:935). Walk to LOD0 surface 0.
	mdxmLOD_t *lod = (mdxmLOD_t*)((byte*)mm + mm->ofsLODs);
	mdxmSurface_t *surf = (mdxmSurface_t*)((byte*)lod + sizeof(mdxmLOD_t)
	                        + mm->numSurfaces * sizeof(mdxmLODSurfOffset_t));
	printf("  LOD0 surf0: ident=%d (SF_MDX=%d) numVerts=%d numTriangles=%d ofsEnd=%d\n",
	       surf->ident, SF_MDX, surf->numVerts, surf->numTriangles, surf->ofsEnd);

	qhandle_t hgla = mm->animIndex;
	dump_model("gla", hgla);
	model_t *gla = R_GetModelByHandle(hgla);
	mdxaHeader_t *ma = gla->mdxa;
	printf("  mdxaHeader: ident=0x%08x version=%d numFrames=%d numBones=%d ofsFrames=%d "
	       "ofsEnd=%d name=\"%s\"\n",
	       ma->ident, ma->version, ma->numFrames, ma->numBones, ma->ofsFrames, ma->ofsEnd, ma->name);

	printf("\n=== re-register (hash hit -> same handle, no new model) ===\n");
	qhandle_t hglm2 = RE_RegisterServerModel("models/test.glm");
	printf("re-register handle=%d (was %d), numModels=%d\n", hglm2, hglm, tr.numModels);

	printf("\n=== version reject (badversion.glm) ===\n");
	qhandle_t hbadver = RE_RegisterServerModel("badversion.glm");
	printf("first register returned=%d, numModels=%d\n", hbadver, tr.numModels);
	qhandle_t hbadver2 = RE_RegisterServerModel("badversion.glm");
	printf("re-register returned=%d (MOD_BAD entry stays hashed under its nonzero index)\n", hbadver2);

	printf("\n=== unknown ident (badident.glm) ===\n");
	qhandle_t hbadid = RE_RegisterServerModel("badident.glm");
	printf("first register returned=%d, numModels=%d\n", hbadid, tr.numModels);
	qhandle_t hbadid2 = RE_RegisterServerModel("badident.glm");
	printf("re-register returned=%d\n", hbadid2);

	printf("\n=== R_GetModelByHandle out-of-range -> models[0] (MOD_BAD) ===\n");
	printf("get(0)=%s  get(99999)=%s  get(-5)=%s\n",
	       tname(R_GetModelByHandle(0)->type),
	       tname(R_GetModelByHandle(99999)->type),
	       tname(R_GetModelByHandle(-5)->type));
	return 0;
}

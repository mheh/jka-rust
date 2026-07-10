// ghoul2-server-oracle stub (seeded from tools/trmodel-oracle) for codemp/renderer/tr_local.h
//
// The real tr_local.h drags in the whole GL renderer (qgl.h, tr_public.h, the
// GL backend, ghoul2_shared.h). Under -DDEDICATED tr_model.cpp reaches none of
// that draw surface (GetRefAPI exports only RE_Shutdown, TRM-D3). This stub
// declares exactly the renderer types/consts the live server model pipeline
// touches, plus the real mdx_format.h (self-contained given MAX_QPATH + vecs).
#ifndef GHOUL2_ORACLE_TR_LOCAL_H
#define GHOUL2_ORACLE_TR_LOCAL_H

#include "../game/q_shared.h"
#include "../qcommon/qcommon.h"

#define MD3_MAX_LODS        3       // qfiles.h:96
#define SHADER_MAX_VERTEXES 1000    // qfiles.h:10
#define SHADER_MAX_INDEXES  (6*SHADER_MAX_VERTEXES) // qfiles.h:11

// surfaceType_t ordinal SF_MDX = 8 (tr_local.h:656-679); the loader pokes
// surf->ident = SF_MDX (:935). Only this member is needed.
#define SF_MDX 8

#include "mdx_format.h"

// --- md3 / qfiles surface (qfiles.h) ------------------------------------------
// The §20 client MD3 path (R_LoadMD3/R_GetTag/R_LerpTag/R_ModelBounds) must
// COMPILE (it shares tr_model's TU) but is never linked-live on the dedicated
// build. These carry only the members that code names; layout is irrelevant
// (never dumped). MD3 idents/version verbatim (qfiles.h:92-93), SF_MD3 = 7.
#define MD3_IDENT   (('3'<<24)+('P'<<16)+('D'<<8)+'I')
#define MD3_VERSION 15
#define SF_MD3      7

typedef struct { int ident, version; char name[MAX_QPATH]; int flags,
	numFrames, numTags, numSurfaces, numSkins, ofsFrames, ofsTags,
	ofsSurfaces, ofsEnd; } md3Header_t;
typedef struct { int ident; char name[MAX_QPATH]; int flags, numFrames,
	numShaders, numVerts, numTriangles, ofsTriangles, ofsShaders, ofsSt,
	ofsXyzNormals, ofsEnd; } md3Surface_t;
typedef struct { vec3_t bounds[2]; vec3_t localOrigin; float radius;
	char name[16]; } md3Frame_t;
typedef struct { char name[MAX_QPATH]; vec3_t origin; vec3_t axis[3]; } md3Tag_t;
typedef struct { char name[MAX_QPATH]; int shaderIndex; } md3Shader_t;
typedef struct bmodel_s { vec3_t bounds[2]; } bmodel_t;

typedef struct world_s world_t;   // fwd for the DEDICATED-out RE_LoadWorldMap proto

// modtype_t — verbatim ordinals (tr_local.h:1103-1115).
typedef enum {
	MOD_BAD,
	MOD_BRUSH,
	MOD_MESH,
	MOD_MDXM,
	MOD_MDXA
} modtype_t;

// shader_t — G2_surfaces.cpp's skin walk reads shader->name (compare against
// "*off"); only that member is named. Layout otherwise irrelevant (never dumped).
typedef struct shader_s { char name[MAX_QPATH]; } shader_t;

// model_t — faithful field order/layout (tr_local.h:1117-1135); the harness
// dumps mod->type/index/dataSize/numLods and the mdxm/mdxa seam pointers.
typedef struct model_s {
	char		name[MAX_QPATH];
	modtype_t	type;
	int			index;
	int			dataSize;
	bmodel_t	*bmodel;
	md3Header_t	*md3[MD3_MAX_LODS];
	mdxmHeader_t *mdxm;
	mdxaHeader_t *mdxa;
	int			numLods;
	qboolean	bspInstance;
} model_t;

#define MAX_MOD_KNOWN 1024          // tr_local.h:1138

// trGlobals_t — only the fields the live server pipeline references
// (tr.models/numModels/numBSPModels/numShaders/numSkins). The huge GL state
// is out of the dedicated surface.
typedef struct trGlobals_s {
	model_t	*models[MAX_MOD_KNOWN];
	int		numModels;
	int		numBSPModels;
	int		numShaders;
	int		numSkins;
} trGlobals_t;

extern trGlobals_t tr;

// Loader cvars read by the pipeline (defined/owned by the harness host).
extern cvar_t *sv_pure;
extern cvar_t *r_modelpoolmegs;
extern cvar_t *r_noServerGhoul2;   // tr_local.h:1591
extern cvar_t *r_lodbias;          // §20 client LOD read (dead here)

// ghoul2 mesh/anim loaders live in tr_ghoul2.cpp (the frozen sibling TU); the
// §20 client RE_RegisterModel_Actual references them, so provide link stubs.
qboolean R_LoadMDXA(model_t *mod, void *buffer, const char *mod_name, qboolean &bAlreadyCached);
qboolean R_LoadMDXM(model_t *mod, void *buffer, const char *mod_name, qboolean &bAlreadyCached);

// skin_t — retexture table (tr_local.h:602-611). G2_surfaces.cpp's
// G2_SetSurfaceOnOffFromSkin walks skin->surfaces[j]->name via R_GetSkinByHandle;
// the surface dumper drives the non-skin surface entries, so this is declared for
// compile + a link stub (never exercised live).
typedef struct { char name[MAX_QPATH]; shader_t *shader; } skinSurface_t;
typedef struct skin_s {
	char		name[MAX_QPATH];
	int			numSurfaces;
	skinSurface_t *surfaces[128];
} skin_t;
skin_t *R_GetSkinByHandle(qhandle_t hSkin);

model_t *R_GetModelByHandle(qhandle_t index);
qhandle_t RE_RegisterServerModel(const char *name);
qhandle_t RE_RegisterModel(const char *name);              // tr_local.h:1700

// Bone-math helpers + the file-scope matrices live in the frozen sibling TU
// tr_ghoul2.cpp; G2API_GetBoltMatrix names them. Link stubs (never exercised by
// the bolt/surface/arena dumpers, which don't drive GetBoltMatrix).
void Create_Matrix(const float *angle, mdxaBone_t *matrix);           // tr_local.h:2306
void Multiply_3x4Matrix(mdxaBone_t *out, mdxaBone_t *in2, mdxaBone_t *in); // :2307
extern mdxaBone_t       worldMatrix;      // tr_ghoul2.cpp:136
extern const mdxaBone_t identityMatrix;   // tr_ghoul2.cpp:128
// RemoveBoneCache — the arena's DeleteLow frees each instance's bone cache
// (G2_API.cpp:321); a link stub, never reached (the arena dumper never builds one).
void RemoveBoneCache(class CBoneCache *toDelete);          // tr_ghoul2.cpp:569
void      RE_RegisterMedia_LevelLoadBegin(const char *psMapName, ForceReload_e eForceReload);
int       RE_RegisterMedia_GetLevel(void);
qboolean  RE_RegisterModels_LevelLoadEnd(qboolean bDeleteEverythingNotUsedThisLevel);
void      RE_RegisterModels_Info_f(void);
void      R_ModelInit(void);
void      R_SVModelInit(void);
void      R_ModelFree(void);
void      R_HunkClearCrap(void);

#endif // GHOUL2_ORACLE_TR_LOCAL_H

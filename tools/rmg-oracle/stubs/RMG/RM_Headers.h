// Stub RM_Headers.h — the real umbrella pulls the entire generation tree
// (CRMMission/CRMInstance/CRMPathManager/CTerrainMap + client.h). Under
// DEDICATED (RMG-D1) LoadMission early-outs before constructing any of it, so
// the harness supplies inline no-op stand-ins for the two generation types
// LoadMission still *names* (CRMMission, CRMObjective) — their bodies never run
// but the symbols must resolve — then includes the REAL RM_Manager.h so the
// CRMManager class-under-test compiles unmodified (§18). oracle never edited.
#pragma once
#include "../qcommon/exe_headers.h"
#include "../qcommon/cm_local.h"
#include "../qcommon/GenericParser2.h"

// Raven rmAutomapSymbol_t / MAX_AUTOMAP_SYMBOLS (client.h:149-151). Relocated to
// mp_qshared in the port (RMG-D4d); here a minimal stub — the array is never
// written under DEDICATED (AddAutomapSymbol is §20-dropped). Source: client.h:149
#define MAX_AUTOMAP_SYMBOLS 512
typedef struct {
	int    mType;
	int    mSide;
	vec3_t mOrigin;
} rmAutomapSymbol_t;

class CRandomTerrain; // forward-declared by cm_landscape.h too

// Generation types LoadMission names past its early-out (never constructed under
// DEDICATED, RMG-D1). Inline no-ops so RM_Manager.cpp links with no external
// generation symbols. Source: RM_Objective.h, RM_Mission.h (§20-dropped).
class CRMObjective {};

class CRMMission {
public:
	CRMMission( CRandomTerrain * ) {}
	~CRMMission( void ) {}
	bool          Load( const char *, const char *, const char * ) { return false; }
	void          Spawn( CRandomTerrain *, qboolean ) {}
	CRMObjective *GetCurrentObjective( void ) { return 0; }
	void          CompleteMission( void ) {}
	void          FailedMission( bool ) {}
	void          CompleteObjective( CRMObjective * ) {}
};

#include "RM_Manager.h"

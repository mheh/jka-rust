// Q3_Registers golden dumper (icarus.md § Verification strategy, unit 2:
// "Q3_Registers"). Compiled against the UNMODIFIED oracle
// codemp/icarus/Q3_Registers.cpp. Drives a scripted Declare/Set/Get/Free/query
// sequence over the varStrings / varFloats / varVectors stores and dumps their
// final (sorted) state plus numVariables. The Rust port's Q3_Registers must
// reproduce this byte-for-byte.
#include "exe_headers.h"   // q_shared.h + qcommon.h, exactly as every oracle icarus TU opens
#include "icarus.h"          // TK_FLOAT/TK_STRING/TK_VECTOR, vec3_t
#include "Q3_Registers.h"    // the var stores + Q3_* API

#include <cstdio>

extern int numVariables;

static void dumpState(const char *tag)
{
	printf("-- %s : numVariables=%d\n", tag, numVariables);
	for (varFloat_m::iterator i = varFloats.begin(); i != varFloats.end(); ++i)
		printf("  F |%s|=%.3f\n", i->first.c_str(), i->second);
	for (varString_m::iterator i = varStrings.begin(); i != varStrings.end(); ++i)
		printf("  S |%s|=|%s|\n", i->first.c_str(), i->second.c_str());
	for (varString_m::iterator i = varVectors.begin(); i != varVectors.end(); ++i)
		printf("  V |%s|=|%s|\n", i->first.c_str(), i->second.c_str());
}

int main(void)
{
	printf("== q3_registers ==\n");
	Q3_InitVariables();

	// Declare each type; duplicate + bad-type are no-ops.
	Q3_DeclareVariable(TK_FLOAT,  "health");
	Q3_DeclareVariable(TK_STRING, "name");
	Q3_DeclareVariable(TK_VECTOR, "spot");
	Q3_DeclareVariable(TK_FLOAT,  "health");   // duplicate -> ignored
	Q3_DeclareVariable(TK_INT,    "bogus");    // unknown type -> ignored
	dumpState("declared");

	// VariableDeclared queries (VTYPE_NONE/FLOAT/STRING/VECTOR).
	printf("decl health=%d name=%d spot=%d ghost=%d\n",
	       Q3_VariableDeclared("health"), Q3_VariableDeclared("name"),
	       Q3_VariableDeclared("spot"),   Q3_VariableDeclared("ghost"));

	// Set values; setting an undeclared name fails.
	Q3_SetFloatVariable("health", 42.5f);
	Q3_SetStringVariable("name", "kyle");
	Q3_SetVectorVariable("spot", "1.0 2.0 3.0");
	printf("set ghost=%d\n", Q3_SetStringVariable("ghost", "x"));
	dumpState("set");

	// Gets.
	float fv = -1.0f; int gf = Q3_GetFloatVariable("health", &fv);
	const char *sv = 0; int gs = Q3_GetStringVariable("name", &sv);
	vec3_t vv = {0,0,0}; int gv = Q3_GetVectorVariable("spot", vv);
	printf("get health ok=%d val=%.3f\n", gf, fv);
	printf("get name ok=%d val=|%s|\n", gs, sv ? sv : "(null)");
	printf("get spot ok=%d val=%.3f %.3f %.3f\n", gv, vv[0], vv[1], vv[2]);
	printf("get ghost ok=%d\n", Q3_GetFloatVariable("ghost", &fv));

	// Free one, then re-query.
	Q3_FreeVariable("name");
	dumpState("freed name");

	// Cap test: declare past MAX_VARIABLES (32) and confirm the guard.
	for (int i = 0; i < 40; i++) { char n[16]; sprintf(n, "v%02d", i); Q3_DeclareVariable(TK_FLOAT, n); }
	printf("after flood numVariables=%d floats=%zu\n", numVariables, varFloats.size());

	printf("== end ==\n");
	return 0;
}

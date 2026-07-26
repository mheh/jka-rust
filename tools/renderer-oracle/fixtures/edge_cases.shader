// Edge cases: parse-error / recovery paths, ParseShader's Com_Error abort
// path, and the "not found in shader text" default-shader fallback (that
// last one has no block here — see edge_cases.names, which asks the dumper
// for a name that is deliberately absent from every .shader fixture).

// unknown general-shader keyword: ParseShader returns qfalse -> FinishShader
// still runs (shader.defaultShader stays true from ClearGlobalShader's
// caller in R_FindShader), producing a defaultShader-flagged shader_t.
textures/oracle_test/edge_unknown_general_keyword
{
	bogusGeneralKeyword 1 2 3
	{
		map textures/oracle_test/flat
	}
}

// unknown stage keyword: ParseStage returns qfalse, which propagates the
// same way.
textures/oracle_test/edge_unknown_stage_keyword
{
	{
		map textures/oracle_test/flat
		bogusStageKeyword 1 2 3
	}
}

// empty stage block: a bare `{ }` is legal — the stage is marked active with
// every field at its zero default (no map keyword at all, so bundle[0].image
// stays NULL).
textures/oracle_test/edge_empty_stage
{
	{
	}
}

// TR_MAX_TEXMODS is 4: a 5th tcMod on one stage hits ParseTexMod's overflow
// guard, which Com_Errors(ERR_DROP, ...). The harness's Com_Error stub
// throws a C++ exception caught per-shader by the driver (see README's
// "Com_Error" stub entry) instead of retail's longjmp-to-safe-point, so the
// golden records an ERROR line and the driver moves on to the next name.
textures/oracle_test/edge_tcmod_overflow
{
	{
		map textures/oracle_test/flat
		tcMod scroll 0.1 0.1
		tcMod scroll 0.2 0.2
		tcMod scroll 0.3 0.3
		tcMod scroll 0.4 0.4
		tcMod scroll 0.5 0.5
	}
}

// missing closing brace at EOF: ParseStage's `while(1)` loop hits an empty
// token (COM_ParseExt returns "" past end of text) and warns "no matching
// '}' found", returning qfalse — same default-shader fallback as the
// unknown-keyword cases above.
textures/oracle_test/edge_truncated_stage
{
	{
		map textures/oracle_test/flat

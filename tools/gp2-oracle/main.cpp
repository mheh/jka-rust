// GP2 differential-oracle dumper. Compiled twice against the UNMODIFIED Raven
// sources (copied into build/ next to stub headers by run.sh):
//   gp2_dump_mp: codemp/qcommon/GenericParser2.cpp
//   gp2_dump_sp: code/game/genericparser2.cpp  (-DGP2_SP -D_JK2EXE)
// Parses a fixture and prints a canonical dump; the Rust parity tests
// (crates/*/tests/gp2_parity.rs) must reproduce it byte-for-byte.
//
// One deliberate normalization: FindPairValue on a pair that exists but has an
// empty value list returns NULL in Raven (callers would crash); the Rust port
// folds that into the default, so the dumper prints "<DEF>" for NULL too.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef GP2_SP
#include "genericparser2.h"
#else
#include "GenericParser2.h"
#endif

static const char *FIXED_PROBES[] = {"zzz_missing", "name||count", "zzz_missing||name", "NAME"};

static void dumpGroup(CGPGroup *group, int depth, FILE *out)
{
	fprintf(out, "G %d |%s|\n", depth, group->GetName());

	for (CGPValue *pair = group->GetPairs(); pair; pair = pair->GetNext())
	{
		fprintf(out, "P %d |%s|", depth, pair->GetName());
		for (CGPObject *value = pair->GetList(); value; value = value->GetNext())
		{
			fprintf(out, "%s|", value->GetName());
		}
		fprintf(out, "\n");
	}

	for (CGPValue *pair = group->GetPairs(); pair; pair = pair->GetNext())
	{
		const char *v = group->FindPairValue(pair->GetName(), "<DEF>");
		fprintf(out, "F %d |%s|%s|\n", depth, pair->GetName(), v ? v : "<DEF>");
	}
	for (size_t i = 0; i < sizeof(FIXED_PROBES) / sizeof(FIXED_PROBES[0]); i++)
	{
		const char *v = group->FindPairValue(FIXED_PROBES[i], "<DEF>");
		fprintf(out, "F %d |%s|%s|\n", depth, FIXED_PROBES[i], v ? v : "<DEF>");
	}

	for (CGPGroup *sub = group->GetSubGroups(); sub; sub = sub->GetNext())
	{
		dumpGroup(sub, depth + 1, out);
	}
}

static void dumpInOrder(CGPGroup *group, int depth, FILE *out)
{
	fprintf(out, "IG %d |%s|\n", depth, group->GetName());
	for (CGPValue *pair = group->GetInOrderPairs(); pair; pair = (CGPValue *)pair->GetInOrderNext())
	{
		fprintf(out, "IP %d |%s|\n", depth, pair->GetName());
	}
	for (CGPGroup *sub = group->GetInOrderSubGroups(); sub; sub = (CGPGroup *)sub->GetInOrderNext())
	{
		dumpInOrder(sub, depth + 1, out);
	}
}

int main(int argc, char **argv)
{
	if (argc != 2)
	{
		fprintf(stderr, "usage: %s <fixture.gp2>\n", argv[0]);
		return 2;
	}

	FILE *f = fopen(argv[1], "rb");
	if (!f)
	{
		fprintf(stderr, "cannot open %s\n", argv[1]);
		return 2;
	}
	fseek(f, 0, SEEK_END);
	long size = ftell(f);
	fseek(f, 0, SEEK_SET);
	// Generous zeroed tail: Raven's tokenizer can scan past the terminator on
	// malformed input; fixtures avoid that, this keeps it deterministic anyway.
	char *buf = (char *)calloc(1, size + 64);
	fread(buf, 1, size, f);
	fclose(f);

	CGenericParser2 parser;
	char *ptr = buf;
	bool ok = parser.Parse(&ptr, true, false);

	printf("== parse %s ==\n", ok ? "ok" : "error");
	dumpGroup(parser.GetBaseParseGroup(), 0, stdout);
	printf("== inorder ==\n");
	dumpInOrder(parser.GetBaseParseGroup(), 0, stdout);
	printf("== write ==\n");
	CTextPool pool(1 << 20);
	parser.Write(&pool);
	fwrite(pool.GetPool(), 1, pool.GetUsed(), stdout);
	printf("== end ==\n");

	return 0;
}

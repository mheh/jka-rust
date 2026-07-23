// huff-oracle stub — huffman.cpp's only #include (line 7 of the oracle TU:
//   #include "../qcommon/exe_headers.h"
// "Anything above this #include will be ignored by the compiler").
//
// The real exe_headers.h drags in the whole platform precompiled-header set.
// The adaptive-Huffman TU only needs: the `byte` scalar, the node/huff/huffman
// struct layout (verbatim from oracle/codemp/qcommon/qcommon.h:1044-1074), a
// throwaway `msg_t` (huffman.cpp mentions it but this harness never calls the
// msg_t entry points), the NYT/INTERNAL_NODE/HMAX symbol constants, and the
// Com_Memset/Com_Memcpy shims. oracle/ is never edited.
#ifndef HUFF_ORACLE_EXE_HEADERS_H
#define HUFF_ORACLE_EXE_HEADERS_H

#include <cstring>

typedef unsigned char byte;

// qcommon.h:1055 / :1044 / :1045
#define HMAX 256                 /* Maximum symbol */
#define NYT HMAX                 /* NYT = Not Yet Transmitted */
#define INTERNAL_NODE (HMAX + 1)

// qcommon.h:1047-1053 — verbatim layout.
typedef struct nodetype {
	struct nodetype *left, *right, *parent; /* tree structure */
	struct nodetype *next, *prev;           /* doubly-linked list */
	struct nodetype **head;                 /* highest ranked node in block */
	int weight;
	int symbol;
} node_t;

// qcommon.h:1057-1069 — verbatim layout.
typedef struct {
	int blocNode;
	int blocPtrs;

	node_t *tree;
	node_t *lhead;
	node_t *ltail;
	node_t *loc[HMAX + 1];
	node_t **freelist;

	node_t nodeList[768];
	node_t *nodePtrs[768];
} huff_t;

// qcommon.h:1071-1074.
typedef struct {
	huff_t compressor;
	huff_t decompressor;
} huffman_t;

// qcommon.h:18-26 (minimal; the msg_t entry points are not exercised here).
typedef struct {
	byte *data;
	int maxsize;
	int cursize;
} msg_t;

#define Com_Memset memset
#define Com_Memcpy memcpy

#endif // HUFF_ORACLE_EXE_HEADERS_H

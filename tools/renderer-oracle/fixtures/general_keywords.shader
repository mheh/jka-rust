// General (shader-level) keyword coverage for oracle/codemp/renderer/tr_shader.cpp's
// ParseShader() dispatch table. Hand-written, not copied from any shipped asset —
// see tools/renderer-oracle/README.md for the keyword-coverage checklist this
// fixture (together with stage_keywords.shader) is cross-referenced against.
// One shader block per keyword (or closely related group) for easy review.

textures/oracle_test/gen_cull_none
{
	cull none
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_cull_twosided
{
	cull twosided
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_cull_disable
{
	cull disable
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_cull_back
{
	cull back
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_cull_backside
{
	cull backside
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_cull_backsided
{
	cull backsided
	{
		map textures/oracle_test/flat
	}
}

// invalid cull parm: warns, cullType stays at its zero-initialized default
// (CT_FRONT_SIDED).
textures/oracle_test/gen_cull_invalid
{
	cull sideways
	{
		map textures/oracle_test/flat
	}
}

// no cull keyword at all: default CT_FRONT_SIDED.
textures/oracle_test/gen_cull_default
{
	{
		map textures/oracle_test/flat
	}
}

// ---- sort: every named value in ParseSort's dispatch table, plus a raw
// numeric fallback. ----
textures/oracle_test/gen_sort_portal
{
	sort portal
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_sort_sky
{
	sort sky
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_sort_opaque
{
	sort opaque
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_sort_decal
{
	sort decal
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_sort_seethrough
{
	sort seeThrough
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_sort_banner
{
	sort banner
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_sort_additive
{
	sort additive
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_sort_nearest
{
	sort nearest
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_sort_underwater
{
	sort underwater
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_sort_inside
{
	sort inside
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_sort_mid_inside
{
	sort mid_inside
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_sort_middle
{
	sort middle
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_sort_mid_outside
{
	sort mid_outside
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_sort_outside
{
	sort outside
	{
		map textures/oracle_test/flat
	}
}

// unnamed numeric sort value (ParseSort's final `else` branch: atof()).
textures/oracle_test/gen_sort_numeric
{
	sort 7.5
	{
		map textures/oracle_test/flat
	}
}

// ---- portal (top-level shortcut that also sets sort = SS_PORTAL) ----
textures/oracle_test/gen_portal
{
	portal
	{
		map textures/oracle_test/flat
	}
}

// ---- skyparms <outerbox> <cloudheight> <innerbox> ----
// full outerbox (exercises R_FindImageFile x6 for the rt/lf/bk/ft/up/dn
// suffixes) with an explicit cloud height and "-" (unsupported) innerbox.
textures/oracle_test/gen_skyparms_full
{
	skyparms textures/oracle_test/sky 512 -
}

// "-" outerbox (skipped) and cloudHeight 0, which falls back to the
// hardcoded default of 512.
textures/oracle_test/gen_skyparms_default_height
{
	skyparms - 0 -
}

// ---- deformVertexes / deform (alias) — every ParseDeform subtype ----
textures/oracle_test/gen_deform_wave
{
	deformVertexes wave 100 sin 0.1 0.2 0.3 0.4
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_deform_wave_zerodiv
{
	// div-by-zero guard: deformationSpread falls back to 100.0 with a warning.
	deformVertexes wave 0 sin 0.1 0.2 0.3 0.4
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_deform_normal
{
	deform normal 0.25 1.5
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_deform_move
{
	deformVertexes move 1 2 3 triangle 0.1 0.2 0.3 0.4
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_deform_bulge
{
	deformVertexes bulge 10 20 30
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_deform_projectionshadow
{
	deformVertexes projectionShadow
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_deform_autosprite
{
	deformVertexes autosprite
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_deform_autosprite2
{
	deformVertexes autosprite2
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_deform_text3
{
	deformVertexes text3
	{
		map textures/oracle_test/flat
	}
}

// out-of-range text index ('9'-'0' = 9, outside 0-7) clamps to DEFORM_TEXT0.
textures/oracle_test/gen_deform_text_clamped
{
	deformVertexes text9
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_deform_unknown
{
	deformVertexes bogusSubtype
	{
		map textures/oracle_test/flat
	}
}

// MAX_SHADER_DEFORMS is 3: a 4th deform command is dropped with a warning
// (numDeforms stays at 3).
textures/oracle_test/gen_deform_overflow
{
	deformVertexes autosprite
	deformVertexes autosprite2
	deformVertexes projectionShadow
	deformVertexes bulge 1 2 3
	{
		map textures/oracle_test/flat
	}
}

// ---- standalone boolean/value keywords ----
textures/oracle_test/gen_nomipmaps
{
	nomipmaps
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_nopicmip
{
	nopicmip
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_noglfog
{
	noglfog
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_polygonoffset
{
	polygonOffset
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_notc
{
	noTC
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_entitymergable
{
	entityMergable
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_light
{
	light 300
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_clamptime
{
	clampTime 500.5
	{
		map textures/oracle_test/flat
	}
}

// ---- fogParms ( r g b ) depthForOpaque ----
textures/oracle_test/gen_fogparms
{
	fogParms ( 0.25 0.5 0.75 ) 128.5
	{
		map textures/oracle_test/flat
	}
}

// ---- material / q3map_material (deprecated alias) ----
textures/oracle_test/gen_material
{
	material solidmetal
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_q3map_material
{
	q3map_material glass
	{
		map textures/oracle_test/flat
	}
}

// ---- sun / q3map_sun <r> <g> <b> <intensity> <theta> <phi> ----
// (writes to the shared tr.sunLight/tr.sunDirection globals, not shader_t
// itself — included for grammar coverage; the dump shows the shader is
// otherwise a plain single-stage shader.)
textures/oracle_test/gen_sun
{
	sun 1.0 0.9 0.8 1.5 45 60
	{
		map textures/oracle_test/flat
	}
}

// ---- surfacelight / q3map_surfacelight <value> ----
textures/oracle_test/gen_surfacelight
{
	surfacelight 200
	{
		map textures/oracle_test/flat
	}
}

// ---- lightColor: parsed and then unconditionally skipped (SkipRestOfLine) ----
textures/oracle_test/gen_lightcolor
{
	lightColor 1 1 1
	{
		map textures/oracle_test/flat
	}
}

// ---- tesssize: parsed and skipped ----
textures/oracle_test/gen_tesssize
{
	tesssize 64
	{
		map textures/oracle_test/flat
	}
}

// ---- q3map_* / qer_* catch-alls: skipped regardless of the exact suffix ----
textures/oracle_test/gen_q3map_catchall
{
	q3map_nonplanar
	q3map_lightimage textures/oracle_test/flat
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_qer_catchall
{
	qer_editorimage textures/oracle_test/flat
	qer_trans 0.5
	{
		map textures/oracle_test/flat
	}
}

// ---- surfaceParm: a representative spread of infoParms[] entries ----
textures/oracle_test/gen_surfaceparm_nonsolid
{
	surfaceParm nonsolid
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_surfaceparm_water
{
	surfaceParm water
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_surfaceparm_trigger
{
	surfaceParm trigger
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_surfaceparm_sky
{
	surfaceParm sky
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_surfaceparm_slick
{
	surfaceParm slick
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_surfaceparm_nodraw
{
	surfaceParm nodraw
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_surfaceparm_trans
{
	surfaceParm trans
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_surfaceparm_detail
{
	surfaceParm detail
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/gen_surfaceparm_unknown
{
	surfaceParm bogusParm
	{
		map textures/oracle_test/flat
	}
}

// combining several standalone flags + a sky (0 stages allowed for sky
// shaders — ParseShader's "s == 0 && !shader.sky && !CONTENTS_FOG" guard).
textures/oracle_test/gen_sky_zero_stages
{
	surfaceParm sky
	surfaceParm nolightmap
	skyparms textures/oracle_test/sky 640 -
	nomipmaps
}

// fog content flag also permits 0 stages (the other half of that guard).
textures/oracle_test/gen_fog_zero_stages
{
	surfaceParm fog
	fogParms ( 0.1 0.1 0.2 ) 256
}

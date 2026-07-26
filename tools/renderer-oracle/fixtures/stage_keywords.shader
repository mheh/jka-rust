// Stage-level keyword coverage for oracle/codemp/renderer/tr_shader.cpp's
// ParseStage() dispatch table (the `{ ... }` block inside a shader). Hand-written,
// not copied from any shipped asset. One shader block per keyword (or closely
// related group) for easy review; see README.md for the coverage checklist.

// ---- map: plain / $whiteimage / $lightmap ----
textures/oracle_test/stage_map_plain
{
	{
		map textures/oracle_test/flat
	}
}

textures/oracle_test/stage_map_whiteimage
{
	{
		map $whiteimage
	}
}

// Driven (via stage_keywords.names' plain-name entry, lightmapIndex[0] stays
// LIGHTMAP_NONE) to hit the "no lightmap available" branch --
// stage->bundle[0].image falls back to tr.whiteImage. The sibling entry
// below, driven with a `:0` lightmap-index override, hits the "available"
// branch instead (tr.numLightmaps is set to 1 by the harness -- see README).
textures/oracle_test/stage_map_lightmap
{
	{
		map $lightmap
	}
}

textures/oracle_test/stage_map_lightmap_available
{
	{
		map $lightmap
	}
}

// ---- clampmap ----
textures/oracle_test/stage_clampmap
{
	{
		clampmap textures/oracle_test/flat
	}
}

// ---- animMap / clampanimMap / oneshotanimMap <freq> <img>... ----
textures/oracle_test/stage_animmap
{
	{
		animMap 5.5 textures/oracle_test/anim0 textures/oracle_test/anim1 textures/oracle_test/anim2
	}
}

textures/oracle_test/stage_clampanimmap
{
	{
		clampanimMap 10 textures/oracle_test/anim0 textures/oracle_test/anim1
	}
}

textures/oracle_test/stage_oneshotanimmap
{
	{
		oneshotanimMap 3 textures/oracle_test/anim0 textures/oracle_test/anim1
	}
}

// ---- videoMap (exercises the harness's CIN_PlayCinematic stub) ----
textures/oracle_test/stage_videomap
{
	{
		videoMap testcinematic.roq
	}
}

// ---- alphaFunc: every NameToAFunc entry, plus an unknown name ----
textures/oracle_test/stage_alphafunc_gt0
{
	{
		map textures/oracle_test/flat
		alphaFunc GT0
	}
}

textures/oracle_test/stage_alphafunc_lt128
{
	{
		map textures/oracle_test/flat
		alphaFunc LT128
	}
}

textures/oracle_test/stage_alphafunc_ge128
{
	{
		map textures/oracle_test/flat
		alphaFunc GE128
	}
}

textures/oracle_test/stage_alphafunc_ge192
{
	{
		map textures/oracle_test/flat
		alphaFunc GE192
	}
}

textures/oracle_test/stage_alphafunc_invalid
{
	{
		map textures/oracle_test/flat
		alphaFunc BOGUS
	}
}

// ---- depthFunc: lequal / equal / disable, plus an unknown name ----
textures/oracle_test/stage_depthfunc_lequal
{
	{
		map textures/oracle_test/flat
		depthFunc lequal
	}
}

textures/oracle_test/stage_depthfunc_equal
{
	{
		map textures/oracle_test/flat
		depthFunc equal
	}
}

textures/oracle_test/stage_depthfunc_disable
{
	{
		map textures/oracle_test/flat
		depthFunc disable
	}
}

textures/oracle_test/stage_depthfunc_invalid
{
	{
		map textures/oracle_test/flat
		depthFunc bogus
	}
}

// ---- detail ----
textures/oracle_test/stage_detail
{
	{
		map textures/oracle_test/flat
		detail
	}
}

// ---- blendFunc: named shortcuts, explicit src/dst pairs, unknown names ----
textures/oracle_test/stage_blendfunc_add
{
	{
		map textures/oracle_test/flat
		blendFunc add
	}
}

textures/oracle_test/stage_blendfunc_filter
{
	{
		map textures/oracle_test/flat
		blendFunc filter
	}
}

textures/oracle_test/stage_blendfunc_blend
{
	{
		map textures/oracle_test/flat
		blendFunc blend
	}
}

textures/oracle_test/stage_blendfunc_explicit
{
	{
		map textures/oracle_test/flat
		blendFunc GL_SRC_ALPHA GL_ONE_MINUS_SRC_ALPHA
	}
}

textures/oracle_test/stage_blendfunc_explicit_all_src
{
	{
		map textures/oracle_test/flat
		blendFunc GL_DST_COLOR GL_ONE
	}
}

textures/oracle_test/stage_blendfunc_unknown
{
	{
		map textures/oracle_test/flat
		blendFunc GL_BOGUS_SRC GL_BOGUS_DST
	}
}

// ---- rgbGen: every NameToGenFunc-independent rgbGen keyword ----
textures/oracle_test/stage_rgbgen_wave
{
	{
		map textures/oracle_test/flat
		rgbGen wave sin 0.1 0.2 0.3 0.4
	}
}

textures/oracle_test/stage_rgbgen_const
{
	{
		map textures/oracle_test/flat
		rgbGen const ( 0.25 0.5 0.75 )
	}
}

textures/oracle_test/stage_rgbgen_identity
{
	{
		map textures/oracle_test/flat
		rgbGen identity
	}
}

textures/oracle_test/stage_rgbgen_identitylighting
{
	{
		map textures/oracle_test/flat
		rgbGen identityLighting
	}
}

textures/oracle_test/stage_rgbgen_entity
{
	{
		map textures/oracle_test/flat
		rgbGen entity
	}
}

textures/oracle_test/stage_rgbgen_oneminusentity
{
	{
		map textures/oracle_test/flat
		rgbGen oneMinusEntity
	}
}

textures/oracle_test/stage_rgbgen_vertex
{
	{
		map textures/oracle_test/flat
		rgbGen vertex
	}
}

textures/oracle_test/stage_rgbgen_exactvertex
{
	{
		map textures/oracle_test/flat
		rgbGen exactVertex
	}
}

textures/oracle_test/stage_rgbgen_lightingdiffuse
{
	{
		map textures/oracle_test/flat
		rgbGen lightingDiffuse
	}
}

textures/oracle_test/stage_rgbgen_lightingdiffuseentity
{
	{
		map textures/oracle_test/flat
		rgbGen lightingDiffuseEntity
	}
}

textures/oracle_test/stage_rgbgen_oneminusvertex
{
	{
		map textures/oracle_test/flat
		rgbGen oneMinusVertex
	}
}

textures/oracle_test/stage_rgbgen_unknown
{
	{
		map textures/oracle_test/flat
		rgbGen bogusGen
	}
}

// ---- alphaGen: every alphaGen keyword ----
textures/oracle_test/stage_alphagen_wave
{
	{
		map textures/oracle_test/flat
		alphaGen wave square 0.1 0.2 0.3 0.4
	}
}

textures/oracle_test/stage_alphagen_const
{
	{
		map textures/oracle_test/flat
		alphaGen const 0.5
	}
}

textures/oracle_test/stage_alphagen_identity
{
	{
		map textures/oracle_test/flat
		alphaGen identity
	}
}

textures/oracle_test/stage_alphagen_entity
{
	{
		map textures/oracle_test/flat
		alphaGen entity
	}
}

textures/oracle_test/stage_alphagen_oneminusentity
{
	{
		map textures/oracle_test/flat
		alphaGen oneMinusEntity
	}
}

textures/oracle_test/stage_alphagen_vertex
{
	{
		map textures/oracle_test/flat
		alphaGen vertex
	}
}

textures/oracle_test/stage_alphagen_lightingspecular
{
	{
		map textures/oracle_test/flat
		alphaGen lightingSpecular
	}
}

textures/oracle_test/stage_alphagen_oneminusvertex
{
	{
		map textures/oracle_test/flat
		alphaGen oneMinusVertex
	}
}

textures/oracle_test/stage_alphagen_dot
{
	{
		map textures/oracle_test/flat
		alphaGen dot
	}
}

textures/oracle_test/stage_alphagen_oneminusdot
{
	{
		map textures/oracle_test/flat
		alphaGen oneMinusDot
	}
}

textures/oracle_test/stage_alphagen_portal
{
	{
		map textures/oracle_test/flat
		alphaGen portal 128
	}
}

// missing range parameter: defaults shader.portalRange to 256 with a warning.
textures/oracle_test/stage_alphagen_portal_default
{
	{
		map textures/oracle_test/flat
		alphaGen portal
	}
}

textures/oracle_test/stage_alphagen_unknown
{
	{
		map textures/oracle_test/flat
		alphaGen bogusGen
	}
}

// ---- tcGen / texgen (alias): environment / lightmap / texture / base / vector ----
textures/oracle_test/stage_tcgen_environment
{
	{
		map textures/oracle_test/flat
		tcGen environment
	}
}

textures/oracle_test/stage_tcgen_lightmap
{
	{
		map textures/oracle_test/flat
		tcGen lightmap
	}
}

textures/oracle_test/stage_tcgen_texture
{
	{
		map textures/oracle_test/flat
		tcGen texture
	}
}

textures/oracle_test/stage_texgen_base
{
	{
		map textures/oracle_test/flat
		texgen base
	}
}

textures/oracle_test/stage_tcgen_vector
{
	{
		map textures/oracle_test/flat
		tcGen vector ( 1 0 0 ) ( 0 1 0 )
	}
}

textures/oracle_test/stage_tcgen_unknown
{
	{
		map textures/oracle_test/flat
		tcGen bogusGen
	}
}

// ---- tcMod: every ParseTexMod subtype, plus an unknown subtype ----
textures/oracle_test/stage_tcmod_turb
{
	{
		map textures/oracle_test/flat
		tcMod turb 0.1 0.2 0.3 0.4
	}
}

textures/oracle_test/stage_tcmod_scale
{
	{
		map textures/oracle_test/flat
		tcMod scale 2.0 3.0
	}
}

textures/oracle_test/stage_tcmod_scroll
{
	{
		map textures/oracle_test/flat
		tcMod scroll 0.5 -0.5
	}
}

textures/oracle_test/stage_tcmod_stretch
{
	{
		map textures/oracle_test/flat
		tcMod stretch sawtooth 0.1 0.2 0.3 0.4
	}
}

textures/oracle_test/stage_tcmod_transform
{
	{
		map textures/oracle_test/flat
		tcMod transform 1.0 0.0 0.0 1.0 0.25 0.5
	}
}

textures/oracle_test/stage_tcmod_rotate
{
	{
		map textures/oracle_test/flat
		tcMod rotate 45
	}
}

textures/oracle_test/stage_tcmod_entitytranslate
{
	{
		map textures/oracle_test/flat
		tcMod entityTranslate
	}
}

textures/oracle_test/stage_tcmod_unknown
{
	{
		map textures/oracle_test/flat
		tcMod bogusMod
	}
}

// multiple tcMods stack in bundle[0].texMods[]; TR_MAX_TEXMODS is 4, so a
// stage with two tcMod lines exercises the array without overflowing (the
// overflow case — a 5th tcMod, which Com_Errors — lives in edge_cases.shader).
textures/oracle_test/stage_tcmod_stacked
{
	{
		map textures/oracle_test/flat
		tcMod scroll 0.1 0.1
		tcMod rotate 10
	}
}

// ---- depthwrite ----
textures/oracle_test/stage_depthwrite
{
	{
		map textures/oracle_test/flat
		depthwrite
	}
}

// ---- glow (JKA addition) ----
textures/oracle_test/stage_glow
{
	{
		map textures/oracle_test/flat
		glow
	}
}

// ---- surfaceSprites (JKA/"VERTIGON" addition): vertical / oriented / effect,
// plus every ssXxx optional parameter and an invalid type/param. ----
textures/oracle_test/stage_surfacesprites_vertical
{
	{
		map textures/oracle_test/flat
		surfaceSprites vertical 16 24 4 64
	}
}

textures/oracle_test/stage_surfacesprites_oriented
{
	{
		map textures/oracle_test/flat
		surfaceSprites oriented 8 8 2 40
	}
}

textures/oracle_test/stage_surfacesprites_effect
{
	{
		map textures/oracle_test/flat
		surfaceSprites effect 10 10 1 32
	}
}

textures/oracle_test/stage_surfacesprites_invalid_type
{
	{
		map textures/oracle_test/flat
		surfaceSprites bogusType 10 10 1 32
	}
}

textures/oracle_test/stage_surfacesprites_full_options
{
	{
		map textures/oracle_test/flat
		surfaceSprites vertical 16 24 4 64
		ssFademax 100
		ssFadescale 0.5
		ssVariance 2 3
		ssHangdown
		ssWind 5
		ssWindidle 2
		ssVertskew 0.25
		ssFXDuration 500
		ssFXGrow 1 2
		ssFXAlphaRange 0.2 0.8
	}
}

textures/oracle_test/stage_surfacesprites_anyangle
{
	{
		map textures/oracle_test/flat
		surfaceSprites vertical 16 24 4 64
		ssAnyangle
	}
}

textures/oracle_test/stage_surfacesprites_faceup
{
	{
		map textures/oracle_test/flat
		surfaceSprites vertical 16 24 4 64
		ssFaceup
	}
}

textures/oracle_test/stage_surfacesprites_weatherfx
{
	{
		map textures/oracle_test/flat
		surfaceSprites effect 10 10 1 32
		ssFXWeather
	}
}

textures/oracle_test/stage_surfacesprites_unknown_param
{
	{
		map textures/oracle_test/flat
		surfaceSprites vertical 16 24 4 64
		ssBogusParam 1 2
	}
}

// ---- a "real-shaped" composite: two active stages so FinishShader's
// lightmap-merge/collapse bookkeeping (with GL multitexture disabled per the
// harness's stub inventory, see README) runs over more than one stage. ----
textures/oracle_test/stage_composite_two_pass
{
	cull none
	{
		map $lightmap
		rgbGen identity
	}
	{
		map textures/oracle_test/flat
		blendFunc GL_DST_COLOR GL_ZERO
		rgbGen identity
	}
}

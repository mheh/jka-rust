//! `gpu_images` — the GPU-side twin of `mp_renderer`'s image registry
//! (R4a backend #1, wave 2).
//!
//! `mp_renderer` is CPU-only forever, so `R_CreateImage` stops at a decoded
//! RGBA8 buffer parked in `TrImageState::pending_uploads`, keyed by the
//! `ImageHandle` its `Arena<ImageAsset>` slot got. This module is the other
//! half: it drains that table into `wgpu` textures and keeps one bind group
//! per image, ready for [`crate::pipeline2d`] to bind.
//!
//! The R2 design (`docs/subsystems/renderer-r2-design.md`) calls for a
//! `SecondaryArena<ImageHandle, GpuImage>` — an index-parallel side table
//! sharing the CPU arena's slots. This wave realises that intent as a
//! `HashMap<ImageHandle, GpuImage>`: same keying, same lifetime rule (an entry
//! dies with its `ImageAsset`), without the slot-vector machinery, which buys
//! nothing until image counts and per-frame lookups are real.
//!
//! DEC-37 ruling 3: everything here is render-thread state. Nothing a trap
//! query can reach.

use std::collections::HashMap;

use mp_renderer::gl_constants::GL_REPEAT;
use mp_renderer::render_state::image_asset::ImageHandle;
use mp_renderer::render_state::render_assets::RenderAssets;
use mp_renderer::tr_image::{PendingUpload, TrImageState};
use wgpu::{AddressMode, BindGroup, BindGroupLayout, Sampler, Texture, TextureView};

use crate::gpu::Gpu;

/// One uploaded image: the texture, its view, and the ready-to-bind group
/// pairing that view with the sampler the image's wrap mode selects.
///
/// `texture` and `view` are held because a `BindGroup` does not keep its
/// resources alive in a form we can re-derive; dropping the `GpuImage` frees
/// all three together.
pub struct GpuImage {
    #[allow(dead_code)]
    texture: Texture,
    view: TextureView,
    bind_group: BindGroup,
    /// `wrapClampMode == GL_REPEAT` — the world pipeline picks the wrapping
    /// sampler for a tiling diffuse texture and the clamping one otherwise.
    repeat: bool,
    /// The PBR material set that attaches to this authored texture (D21). The
    /// first PBR frame that resolves this image derives the set from
    /// `derive_source` and caches it here. A default `pbr=0` boot never derives,
    /// so it pays no sidecar cost. The white fallback and the two neutral
    /// defaults keep `None` forever and bind the shared neutral normal and
    /// roughness instead.
    sidecar: Option<SidecarSet>,
    /// The decoded diffuse pixels and their size, kept so the first PBR frame
    /// can derive the sidecar without a texture read-back. The derivation takes
    /// this source and drops it. An image built as a fallback or a neutral
    /// default carries `None` and never derives.
    derive_source: Option<DeriveSource>,
}

/// The decoded diffuse pixels one `GpuImage` keeps for a lazy PBR sidecar
/// derivation (D21). The store holds this until the first PBR frame resolves the
/// image, then the derivation takes it. A default `pbr=0` boot holds one such
/// copy per image until the toggle first turns on.
struct DeriveSource {
    name: String,
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

/// A derived or authored PBR material set that attaches to an authored
/// texture's own `GpuImage` (D21). The material never gets its own asset
/// identity. It rides the diffuse texture's existing `ImageHandle`, so a
/// toggle and a fallback are next-frame per-draw operations.
///
/// Stage B derives this set at upload time from the diffuse pixels. The fields
/// are the normal and roughness texture views the PBR pass reads, plus their
/// backing textures kept alive. The pass builds the combined PBR bind group
/// per draw, since the diffuse and lightmap vary per draw, so this set holds no
/// bind group of its own.
pub struct SidecarSet {
    #[allow(dead_code)]
    normal_texture: Texture,
    #[allow(dead_code)]
    roughness_texture: Texture,
    normal: TextureView,
    roughness: TextureView,
}

/// The uploaded-image store, plus the shared sampler pair and the fallback
/// white texel every handle-less (or unresolvable) draw binds.
pub struct GpuImages {
    layout: BindGroupLayout,
    /// `GL_REPEAT` — bilinear, wrapping.
    sampler_repeat: Sampler,
    /// `GL_CLAMP`/`GL_CLAMP_TO_EDGE` — bilinear, clamped. Every 2D image
    /// `R_RegisterShaderNoMip` registers lands here.
    sampler_clamp: Sampler,
    /// The unmipmapped nearest sampler Raven's `mFilterMode != 0` clouds bind.
    /// Weather sets both the min and the mag filter, and neither weather filter uses mips.
    /// The weather image always loads with `GL_CLAMP`, so one nearest-clamp sampler covers every cloud.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1364-1365`
    sampler_nearest: Sampler,
    /// The stand-in a draw binds when it names no image, names one that never
    /// uploaded, or resolves through a shader with no stage image: one opaque
    /// white texel, so `texture * vertex_color` reduces to the vertex colour.
    white: GpuImage,
    /// The shared neutral normal map the PBR pass binds when a draw's diffuse
    /// carries no sidecar (D21). One 1x1 flat-normal texel `[128, 128, 255,
    /// 255]`, which decodes to the unperturbed surface normal `(0, 0, 1)`. Held
    /// once, never copied per asset.
    neutral_normal: GpuImage,
    /// The shared neutral roughness map the PBR pass binds when a draw's diffuse
    /// carries no sidecar (D21). One 1x1 mid-roughness texel `[128, 128, 128,
    /// 255]`, roughness `0.5`. Held once, never copied per asset.
    neutral_roughness: GpuImage,
    images: HashMap<ImageHandle, GpuImage>,
}

impl GpuImages {
    /// Builds the bind-group layout, both samplers, and the white fallback.
    pub fn new(gpu: &Gpu) -> GpuImages {
        let device = gpu.device();

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mp_renderer_gpu 2d texture layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let sampler_repeat = create_sampler(device, AddressMode::Repeat, "repeat");
        let sampler_clamp = create_sampler(device, AddressMode::ClampToEdge, "clamp");
        let sampler_nearest = create_nearest_sampler(device);

        let white = create_image(
            gpu,
            &layout,
            &sampler_clamp,
            "white fallback",
            &[0xff, 0xff, 0xff, 0xff],
            1,
            1,
            false,
        );

        // The two shared neutral PBR defaults (D21). Every PBR draw whose
        // diffuse carries no sidecar binds these, so they are built once and
        // never copied per asset. The `layout` and `sampler_clamp` reuse the 2D
        // resources, since a 1x1 texel never tiles.
        let neutral_normal = create_image(
            gpu,
            &layout,
            &sampler_clamp,
            "pbr neutral normal",
            &[128, 128, 255, 255],
            1,
            1,
            false,
        );
        let neutral_roughness = create_image(
            gpu,
            &layout,
            &sampler_clamp,
            "pbr neutral roughness",
            &[128, 128, 128, 255],
            1,
            1,
            false,
        );

        GpuImages {
            layout,
            sampler_repeat,
            sampler_clamp,
            sampler_nearest,
            white,
            neutral_normal,
            neutral_roughness,
            images: HashMap::new(),
        }
    }

    /// The layout every bind group here is built against — [`Pipeline2d`]
    /// builds its pipeline layout from it, so the two agree by construction.
    ///
    /// [`Pipeline2d`]: crate::pipeline2d::Pipeline2d
    pub fn layout(&self) -> &BindGroupLayout {
        &self.layout
    }

    /// Number of images uploaded so far (the white fallback is not counted).
    pub fn len(&self) -> usize {
        self.images.len()
    }

    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    /// Drains `img_state.pending_uploads` into GPU textures, returning how
    /// many images were uploaded.
    ///
    /// Draining, not copying: the staged pixels exist only to cross the
    /// crate boundary once (see `TrImageState::pending_uploads`' own doc
    /// comment on why they are retained at all — the oracle `Z_Free`s them
    /// right after `qglTexImage2D`), so an uploaded entry is removed and the
    /// sys-RAM copy released, matching that free.
    ///
    /// A staged entry whose `ImageAsset` has already gone (a purge between
    /// registration and the next frame) is dropped without uploading: the
    /// `wrapClampMode` that picks its sampler lives on the asset.
    //TODO: Port Upload32's mipmap chain
    // Source: oracle/codemp/renderer/tr_image.cpp:599-762
    // v0 uploads level 0 only and samples it directly. Raven built the whole
    // chain here (`R_MipMap` + `GL_LINEAR_MIPMAP_NEAREST`); minified 2D pics
    // therefore alias slightly versus retail until the chain lands.
    pub fn upload_pending(
        &mut self,
        gpu: &Gpu,
        img_state: &mut TrImageState,
        assets: &RenderAssets,
    ) -> usize {
        let staged: Vec<(ImageHandle, PendingUpload)> = img_state.pending_uploads.drain().collect();
        self.upload_staged(gpu, staged, assets)
    }

    /// The same upload, for staged pixels that already crossed a thread
    /// boundary inside a `FramePackage` and so are owned rather than drained
    /// from a live `TrImageState`.
    pub fn upload_staged(
        &mut self,
        gpu: &Gpu,
        staged: Vec<(ImageHandle, PendingUpload)>,
        assets: &RenderAssets,
    ) -> usize {
        if staged.is_empty() {
            return 0;
        }

        let mut uploaded = 0;
        for (handle, pending) in staged {
            let Some(asset) = assets.images.get(handle) else {
                eprintln!(
                    "mp_renderer_gpu: staged upload for a vanished image slot \
                     {}; dropped",
                    handle.index()
                );
                continue;
            };

            let repeat = asset.wrap_clamp_mode == GL_REPEAT;
            let sampler = if repeat {
                &self.sampler_repeat
            } else {
                &self.sampler_clamp
            };
            let width = pending.width.max(0) as u32;
            let height = pending.height.max(0) as u32;
            let mut image = create_image(
                gpu,
                &self.layout,
                sampler,
                &asset.img_name,
                &pending.pixels,
                width,
                height,
                repeat,
            );
            // Keep the decoded pixels for a lazy PBR sidecar derivation (D21).
            // The load path derives nothing here, so a default `pbr=0` boot pays
            // no sidecar VRAM and runs no derivation pass. The store holds one
            // pixel copy per image until the first PBR frame resolves it. The
            // faithful backend never reads the sidecar.
            image.derive_source = Some(DeriveSource {
                name: asset.img_name.clone(),
                pixels: pending.pixels,
                width,
                height,
            });
            self.images.insert(handle, image);
            uploaded += 1;
        }
        uploaded
    }

    /// The bind group for `handle`, or the white fallback when it is `None`
    /// or names an image that never uploaded.
    pub fn bind_group(&self, handle: Option<ImageHandle>) -> &BindGroup {
        handle
            .and_then(|h| self.images.get(&h))
            .map(|image| &image.bind_group)
            .unwrap_or(&self.white.bind_group)
    }

    /// Whether `handle` has an uploaded texture — the executor's test for
    /// "resolved" before it decides to log a fallback.
    pub fn contains(&self, handle: ImageHandle) -> bool {
        self.images.contains_key(&handle)
    }

    /// The uploaded image for `handle`, or the white fallback when `handle` is
    /// `None` or names an image that never uploaded.
    fn image_or_white(&self, handle: Option<ImageHandle>) -> &GpuImage {
        handle
            .and_then(|h| self.images.get(&h))
            .unwrap_or(&self.white)
    }

    /// Derives the PBR sidecar for `handle` if it has none yet (D21). The
    /// derivation takes the kept pixels and caches the set on the image, so a
    /// later frame reuses it. A `None` handle, a purged image, or an already
    /// derived image is a no-op. This is the only path that derives a sidecar,
    /// so a default `pbr=0` boot never runs it.
    fn ensure_sidecar(&mut self, gpu: &Gpu, handle: Option<ImageHandle>) {
        let Some(h) = handle else {
            return;
        };
        let Some(image) = self.images.get_mut(&h) else {
            return;
        };
        if image.sidecar.is_some() {
            return;
        }
        let Some(src) = image.derive_source.take() else {
            return;
        };
        image.sidecar = Some(build_sidecar(gpu, &src.name, &src.pixels, src.width, src.height));
    }

    /// Builds a two-texture bind group (diffuse plus lightmap) for the world
    /// pipeline against `layout`. The diffuse texture keeps its own wrap mode
    /// so a tiling world surface repeats. The lightmap always clamps, matching
    /// the oracle's `GL_CLAMP` lightmap upload.
    ///
    /// This builds a fresh bind group per call, so the world backend makes one
    /// per surface per frame. The first world wave keeps one draw call per
    /// surface, so the extra bind groups cost nothing a batching pass will not
    /// remove later.
    ///
    /// Source: `oracle/codemp/renderer/tr_bsp.cpp:241` (`GL_CLAMP` lightmaps)
    pub fn world_bind_group(
        &self,
        gpu: &Gpu,
        layout: &BindGroupLayout,
        diffuse: Option<ImageHandle>,
        lightmap: Option<ImageHandle>,
    ) -> BindGroup {
        let diffuse_image = self.image_or_white(diffuse);
        let lightmap_image = self.image_or_white(lightmap);
        let diffuse_sampler = if diffuse_image.repeat {
            &self.sampler_repeat
        } else {
            &self.sampler_clamp
        };

        gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mp_renderer_gpu world texture bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_image.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(diffuse_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&lightmap_image.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.sampler_clamp),
                },
            ],
        })
    }

    /// The weather bind group for one cloud: its image with the clamp wrap `R_FindImageFile` gave it, and the filter its `mFilterMode` chose.
    /// A cloud binds one image and no lightmap, so slot 2 takes the white texel and the world shader's second sample reduces to one.
    /// `layout` is the world pipeline's group-1 layout, which `Pipeline3d` owns, the same way `world_bind_group` and `view_bind_group` take it.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1319-1320,1364-1365`
    pub fn weather_bind_group(
        &self,
        gpu: &Gpu,
        layout: &BindGroupLayout,
        handle: Option<ImageHandle>,
        nearest: bool,
    ) -> BindGroup {
        let image = self.image_or_white(handle);
        let sampler = if nearest {
            &self.sampler_nearest
        } else {
            &self.sampler_clamp
        };

        gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mp_renderer_gpu weather texture bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&image.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.white.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.sampler_clamp),
                },
            ],
        })
    }

    /// Builds the world texture bind group with an explicit diffuse view, the shape the distortion stage needs.
    /// The screen image is render-thread scratch with no `ImageHandle`, so it binds by view and takes the clamping sampler.
    ///
    /// Source: `oracle/codemp/renderer/tr_shade.cpp:2167` (`GL_Bind( tr.screenImage )`)
    pub fn view_bind_group(
        &self,
        gpu: &Gpu,
        layout: &BindGroupLayout,
        diffuse: &TextureView,
        lightmap: Option<ImageHandle>,
    ) -> BindGroup {
        let lightmap_image = self.image_or_white(lightmap);

        gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mp_renderer_gpu view texture bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(diffuse),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler_clamp),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&lightmap_image.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.sampler_clamp),
                },
            ],
        })
    }

    /// Builds the four-texture PBR bind group (diffuse, lightmap, normal,
    /// roughness) for the world pipeline against `layout`. The diffuse and
    /// lightmap resolve exactly as [`Self::world_bind_group`] resolves them, so
    /// the authored look is unchanged. The normal and roughness follow the D21
    /// rule: the draw's diffuse image lends its own sidecar set when it carries
    /// one, and the shared neutral normal and roughness stand in otherwise.
    ///
    /// Every uploaded image carries the decoded pixels for a derived set, so the
    /// first PBR frame that resolves the image derives its sidecar and caches it.
    /// Only the white fallback resolves to the shared neutral pair.
    pub fn pbr_world_bind_group(
        &mut self,
        gpu: &Gpu,
        layout: &BindGroupLayout,
        diffuse: Option<ImageHandle>,
        lightmap: Option<ImageHandle>,
    ) -> BindGroup {
        // Derive the diffuse image's sidecar now, on the first PBR frame that
        // resolves it (D21). The derivation takes the kept pixels and caches the
        // result, so a later frame reuses it. This mutable block ends before the
        // immutable resolution below.
        self.ensure_sidecar(gpu, diffuse);

        let diffuse_image = self.image_or_white(diffuse);
        let lightmap_image = self.image_or_white(lightmap);
        let diffuse_sampler = if diffuse_image.repeat {
            &self.sampler_repeat
        } else {
            &self.sampler_clamp
        };

        // D21 sidecar-or-neutral resolution. The white fallback is the one image
        // with no sidecar, so it alone takes the neutral pair.
        let (normal_view, roughness_view) = match &diffuse_image.sidecar {
            Some(set) => (&set.normal, &set.roughness),
            None => (&self.neutral_normal.view, &self.neutral_roughness.view),
        };

        gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mp_renderer_gpu pbr world texture bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_image.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(diffuse_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&lightmap_image.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.sampler_clamp),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(normal_view),
                },
                // The normal map shares the diffuse texcoord, so it wraps exactly
                // as the diffuse wraps. A tiling surface reads the normal outside
                // 0..1, so the clamping sampler would smear the edge texel across
                // the whole face.
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(diffuse_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(roughness_view),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(&self.sampler_clamp),
                },
            ],
        })
    }
}

/// The nearest-filtered clamping sampler a `mFilterMode != 0` weather cloud binds.
/// Both the min and the mag filter are nearest, and neither uses mips.
///
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1364-1365`
fn create_nearest_sampler(device: &wgpu::Device) -> Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("mp_renderer_gpu weather sampler (nearest)"),
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

/// Bilinear in both directions, matching `R_RegisterShaderNoMip`'s
/// `GL_LINEAR` 2D images; `address_mode` carries the image's
/// `wrapClampMode`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp` (`R_RegisterShaderNoMip`)
fn create_sampler(device: &wgpu::Device, address_mode: AddressMode, label: &str) -> Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(&format!("mp_renderer_gpu 2d sampler ({label})")),
        address_mode_u: address_mode,
        address_mode_v: address_mode,
        address_mode_w: address_mode,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    })
}

/// Creates an `Rgba8Unorm` texture from `pixels` (`width * height * 4` bytes)
/// and its bind group. A short buffer is zero-padded rather than panicking —
/// a truncated image should show as black, not take the frame loop down.
fn create_image(
    gpu: &Gpu,
    layout: &BindGroupLayout,
    sampler: &Sampler,
    name: &str,
    pixels: &[u8],
    width: u32,
    height: u32,
    repeat: bool,
) -> GpuImage {
    let width = width.max(1);
    let height = height.max(1);
    let needed = (width as usize) * (height as usize) * 4;

    let padded = if pixels.len() >= needed {
        None
    } else {
        eprintln!(
            "mp_renderer_gpu: image \"{name}\" staged {} bytes for {width}x{height} \
             ({needed} needed); padding with black",
            pixels.len()
        );
        let mut owned = vec![0u8; needed];
        owned[..pixels.len()].copy_from_slice(pixels);
        Some(owned)
    };
    let bytes: &[u8] = match padded.as_deref() {
        Some(owned) => owned,
        None => &pixels[..needed],
    };

    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = gpu.device().create_texture(&wgpu::TextureDescriptor {
        label: Some(name),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    gpu.queue().write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: Some(height),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let bind_group = gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(name),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });

    GpuImage {
        texture,
        view,
        bind_group,
        repeat,
        sidecar: None,
        derive_source: None,
    }
}

/// The height-to-normal gain the diffuse-luminance gradient scales by. A larger
/// value bends the derived normal further from flat, so surface detail reads
/// stronger. This value keeps the detail visible without folding the normal
/// past the surface plane.
const NORMAL_STRENGTH: f32 = 2.0;

/// The lowest roughness the derivation reports, for a flat, near-uniform
/// texture. A value near zero would read as a mirror, which no derived material
/// should claim.
const ROUGHNESS_MIN: f32 = 0.35;

/// The highest roughness the derivation reports, for a busy, high-contrast
/// texture. A value of one is fully matte.
const ROUGHNESS_MAX: f32 = 0.95;

/// The gain the luminance standard deviation scales by before the roughness
/// clamp. A busier texture reports a higher roughness, so a matte-looking wall
/// reads rougher than a smooth panel.
const ROUGHNESS_VARIANCE_GAIN: f32 = 2.5;

/// The luminance of one RGBA8 pixel in the 0..1 range, by the Rec. 601 weights.
fn pixel_luminance(pixels: &[u8], texel: usize) -> f32 {
    let p = texel * 4;
    if p + 2 >= pixels.len() {
        return 0.0;
    }
    let r = pixels[p] as f32;
    let g = pixels[p + 1] as f32;
    let b = pixels[p + 2] as f32;
    (0.299 * r + 0.587 * g + 0.114 * b) / 255.0
}

/// Encodes a signed normal component in -1..1 into an unsigned 0..255 byte.
fn encode_normal_byte(n: f32) -> u8 {
    ((n * 0.5 + 0.5) * 255.0).round().clamp(0.0, 255.0) as u8
}

/// Derives a tangent-space normal map from the diffuse luminance by a
/// central-difference height gradient (`R_CreateImage` gives us no authored
/// normal). Each texel takes the horizontal and vertical luminance slope as a
/// height gradient, then the normal `normalize(-dx, -dy, 1)`. Edge texels clamp
/// the neighbor index, so the border reads flat rather than wrapping.
///
/// This derivation is v1 scaffolding. The future material tool replaces it with
/// authored normal data.
fn derive_normal_map(pixels: &[u8], width: u32, height: u32) -> Vec<u8> {
    let w = width.max(1) as usize;
    let h = height.max(1) as usize;

    let mut lum = vec![0.0f32; w * h];
    for (i, l) in lum.iter_mut().enumerate() {
        *l = pixel_luminance(pixels, i);
    }

    let mut out = vec![0u8; w * h * 4];
    for y in 0..h {
        let yt = y.saturating_sub(1);
        let yb = (y + 1).min(h - 1);
        for x in 0..w {
            let xl = x.saturating_sub(1);
            let xr = (x + 1).min(w - 1);

            let dx = lum[y * w + xr] - lum[y * w + xl];
            let dy = lum[yb * w + x] - lum[yt * w + x];

            let nx = -dx * NORMAL_STRENGTH;
            let ny = -dy * NORMAL_STRENGTH;
            let nz = 1.0f32;
            let inv = 1.0 / (nx * nx + ny * ny + nz * nz).sqrt();

            let o = (y * w + x) * 4;
            out[o] = encode_normal_byte(nx * inv);
            out[o + 1] = encode_normal_byte(ny * inv);
            out[o + 2] = encode_normal_byte(nz * inv);
            out[o + 3] = 255;
        }
    }
    out
}

/// Derives one scalar roughness byte from the diffuse luminance variance. A
/// busy, high-contrast texture reports a higher roughness, so a matte wall reads
/// rougher than a smooth panel. The result clamps to the `ROUGHNESS_MIN`..
/// `ROUGHNESS_MAX` band and stores in a 1x1 texture.
///
/// This derivation is v1 scaffolding. The future material tool replaces it with
/// authored roughness data.
fn derive_roughness_value(pixels: &[u8], width: u32, height: u32) -> u8 {
    let count = (width.max(1) as usize) * (height.max(1) as usize);

    let mut sum = 0.0f32;
    let mut sum_sq = 0.0f32;
    let mut n = 0.0f32;
    for i in 0..count {
        let l = pixel_luminance(pixels, i);
        sum += l;
        sum_sq += l * l;
        n += 1.0;
    }

    let rough = if n == 0.0 {
        ROUGHNESS_MIN
    } else {
        let mean = sum / n;
        let variance = (sum_sq / n - mean * mean).max(0.0);
        let std_dev = variance.sqrt();
        (ROUGHNESS_MIN + std_dev * ROUGHNESS_VARIANCE_GAIN).clamp(ROUGHNESS_MIN, ROUGHNESS_MAX)
    };
    (rough * 255.0).round().clamp(0.0, 255.0) as u8
}

/// Builds the derived PBR material set for one diffuse texture (D21): a
/// full-resolution normal map and a 1x1 roughness texel. Both textures live on
/// the GPU and their views feed the PBR bind group.
fn build_sidecar(gpu: &Gpu, name: &str, pixels: &[u8], width: u32, height: u32) -> SidecarSet {
    let normal_pixels = derive_normal_map(pixels, width, height);
    let (normal_texture, normal) = create_data_texture(
        gpu,
        &format!("{name} pbr normal"),
        &normal_pixels,
        width.max(1),
        height.max(1),
    );

    let rough = derive_roughness_value(pixels, width, height);
    let (roughness_texture, roughness) = create_data_texture(
        gpu,
        &format!("{name} pbr roughness"),
        &[rough, rough, rough, 255],
        1,
        1,
    );

    SidecarSet {
        normal_texture,
        roughness_texture,
        normal,
        roughness,
    }
}

/// Creates an `Rgba8Unorm` texture from `pixels` and its view, with no bind
/// group. The derived normal and roughness textures use this: the PBR pass
/// binds their views into its own combined group, so a per-texture group would
/// go unused.
fn create_data_texture(
    gpu: &Gpu,
    name: &str,
    pixels: &[u8],
    width: u32,
    height: u32,
) -> (Texture, TextureView) {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = gpu.device().create_texture(&wgpu::TextureDescriptor {
        label: Some(name),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    gpu.queue().write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: Some(height),
        },
        size,
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

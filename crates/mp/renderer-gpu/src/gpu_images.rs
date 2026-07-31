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
use mp_renderer::tr_image::TrImageState;
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
    /// The stand-in a draw binds when it names no image, names one that never
    /// uploaded, or resolves through a shader with no stage image: one opaque
    /// white texel, so `texture * vertex_color` reduces to the vertex colour.
    white: GpuImage,
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

        GpuImages {
            layout,
            sampler_repeat,
            sampler_clamp,
            white,
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
        if img_state.pending_uploads.is_empty() {
            return 0;
        }

        let mut uploaded = 0;
        for (handle, pending) in img_state.pending_uploads.drain() {
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
            let image = create_image(
                gpu,
                &self.layout,
                sampler,
                &asset.img_name,
                &pending.pixels,
                pending.width.max(0) as u32,
                pending.height.max(0) as u32,
                repeat,
            );
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
    }
}

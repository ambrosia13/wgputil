use std::num::NonZero;

pub fn bind_buffer_uniform(buffer: &wgpu::Buffer) -> BindingEntry<'_> {
    BindingEntry {
        binding_type: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
        resource: buffer.as_entire_binding(),
    }
}

pub fn bind_buffer_storage(buffer: &wgpu::Buffer, read_only: bool) -> BindingEntry<'_> {
    BindingEntry {
        binding_type: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
        resource: buffer.as_entire_binding(),
    }
}

pub fn bind_texture(
    view: &wgpu::TextureView,
    sample_type: wgpu::TextureSampleType,
    view_dimension: wgpu::TextureViewDimension,
) -> BindingEntry<'_> {
    BindingEntry {
        binding_type: wgpu::BindingType::Texture {
            sample_type,
            view_dimension,
            multisampled: false,
        },
        count: None,
        resource: wgpu::BindingResource::TextureView(view),
    }
}

pub fn bind_textures<'a>(
    views: &'a [&wgpu::TextureView],
    sample_type: wgpu::TextureSampleType,
    view_dimension: wgpu::TextureViewDimension,
) -> BindingEntry<'a> {
    BindingEntry {
        binding_type: wgpu::BindingType::Texture {
            sample_type,
            view_dimension,
            multisampled: false,
        },
        count: Some(views.len()),
        resource: wgpu::BindingResource::TextureViewArray(views),
    }
}

pub fn bind_storage_texture(
    view: &wgpu::TextureView,
    format: wgpu::TextureFormat,
    view_dimension: wgpu::TextureViewDimension,
    access: wgpu::StorageTextureAccess,
) -> BindingEntry<'_> {
    BindingEntry {
        binding_type: wgpu::BindingType::StorageTexture {
            access,
            format,
            view_dimension,
        },
        count: None,
        resource: wgpu::BindingResource::TextureView(view),
    }
}

pub fn bind_storage_textures<'a>(
    views: &'a [&wgpu::TextureView],
    format: wgpu::TextureFormat,
    view_dimension: wgpu::TextureViewDimension,
    access: wgpu::StorageTextureAccess,
) -> BindingEntry<'a> {
    BindingEntry {
        binding_type: wgpu::BindingType::StorageTexture {
            access,
            format,
            view_dimension,
        },
        count: Some(views.len()),
        resource: wgpu::BindingResource::TextureViewArray(views),
    }
}

pub fn bind_sampler(
    sampler: &wgpu::Sampler,
    binding_type: wgpu::SamplerBindingType,
) -> BindingEntry<'_> {
    BindingEntry {
        binding_type: wgpu::BindingType::Sampler(binding_type),
        count: None,
        resource: wgpu::BindingResource::Sampler(sampler),
    }
}

pub fn bind_samplers<'a>(
    samplers: &'a [&wgpu::Sampler],
    binding_type: wgpu::SamplerBindingType,
) -> BindingEntry<'a> {
    BindingEntry {
        binding_type: wgpu::BindingType::Sampler(binding_type),
        count: Some(samplers.len()),
        resource: wgpu::BindingResource::SamplerArray(samplers),
    }
}

/// Entry for creating a linked [`wgpu::BindGroupLayoutEntry`] and [`wgpu::BindGroupEntry`].
pub struct BindingEntry<'a> {
    pub binding_type: wgpu::BindingType,
    pub count: Option<usize>,

    pub resource: wgpu::BindingResource<'a>,
}

impl<'a> BindingEntry<'a> {
    pub fn build(&self, index: usize) -> (wgpu::BindGroupLayoutEntry, wgpu::BindGroupEntry<'a>) {
        (
            wgpu::BindGroupLayoutEntry {
                binding: index as u32,
                visibility: wgpu::ShaderStages::all(), // only a thing on directx, skip specifying
                ty: self.binding_type,
                count: self.count.map(|c| c as u32).and_then(NonZero::new),
            },
            wgpu::BindGroupEntry {
                binding: index as u32,
                resource: self.resource.clone(),
            },
        )
    }
}

/// Create a [`wgpu::BindGroup`] along with its corresponding [`wgpu::BindGroupLayout`], with sequential binding indices.
pub fn create_sequential_linked(
    device: &wgpu::Device,
    label: &str,
    entries: &[BindingEntry],
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let built_entries = entries.iter().enumerate().map(|(i, e)| e.build(i));

    let bind_group_layout_entries: Vec<_> = built_entries.clone().map(|(e, _)| e).collect();
    let bind_group_entries: Vec<_> = built_entries.clone().map(|(_, e)| e).collect();

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &bind_group_layout_entries,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout: &bind_group_layout,
        entries: &bind_group_entries,
    });

    (bind_group_layout, bind_group)
}

/// Create a [`wgpu::BindGroup`] with sequential binding indices.
pub fn create_sequential_with_layout(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::BindGroupLayout,
    entries: &[wgpu::BindingResource],
) -> wgpu::BindGroup {
    let bind_group_entries: Vec<wgpu::BindGroupEntry> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| wgpu::BindGroupEntry {
            binding: i as u32,
            resource: e.clone(),
        })
        .collect();

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &bind_group_entries,
    })
}

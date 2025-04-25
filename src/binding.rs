use std::num::NonZero;

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

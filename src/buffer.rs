use std::num::NonZero;

pub fn write_slice(queue: &wgpu::Queue, buffer: &wgpu::Buffer, data: &[u8], offset: usize) {
    queue
        .write_buffer_with(
            buffer,
            offset as u64,
            NonZero::new(data.len() as u64).unwrap(),
        )
        .unwrap()
        .copy_from_slice(data);
}

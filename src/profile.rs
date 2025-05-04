use std::{
    sync::{mpsc, Arc},
    time::Duration,
};

use crate::GpuHandle;

pub struct TimeQuery {
    started: bool,

    query_set: wgpu::QuerySet,
    resolve_buffer: wgpu::Buffer,
    readback_buffer: Arc<wgpu::Buffer>,
}

impl TimeQuery {
    pub fn new(device: &wgpu::Device) -> Self {
        let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: None,
            ty: wgpu::QueryType::Timestamp,
            count: 2, // one for before timestamp, one for after
        });

        let resolve_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 2 * 8, // 2 u64s, 8 bytes each
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 2 * 8, // 2 u64s, 8 bytes each
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let readback_buffer = Arc::new(readback_buffer);

        Self {
            started: false,
            query_set,
            resolve_buffer,
            readback_buffer,
        }
    }

    pub fn compute_timestamp_writes(&self) -> wgpu::ComputePassTimestampWrites {
        wgpu::ComputePassTimestampWrites {
            query_set: &self.query_set,
            beginning_of_pass_write_index: Some(0),
            end_of_pass_write_index: Some(1),
        }
    }

    pub fn render_timestamp_writes(&self) -> wgpu::RenderPassTimestampWrites {
        wgpu::RenderPassTimestampWrites {
            query_set: &self.query_set,
            beginning_of_pass_write_index: Some(0),
            end_of_pass_write_index: Some(1),
        }
    }

    pub fn write_start_timestamp(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if self.started {
            panic!("Attempted to write a start timestamp more than once");
        }

        self.started = true;
        encoder.write_timestamp(&self.query_set, 0);
    }

    pub fn write_end_timestamp(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if !self.started {
            panic!("Attempted to write an end timestamp without first starting");
        }

        self.started = false;
        encoder.write_timestamp(&self.query_set, 1);

        // after the timestamp is written, resolve the query and prepare for readback
        //self.resolve(encoder);
    }

    fn resolve(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.resolve_query_set(&self.query_set, 0..2, &self.resolve_buffer, 0);

        // Copy the data to a mapped buffer so it can be read on the cpu
        encoder.copy_buffer_to_buffer(
            &self.resolve_buffer,
            0,
            &self.readback_buffer,
            0,
            self.resolve_buffer.size(),
        );
    }

    pub fn read(&self, gpu: &GpuHandle) -> Duration {
        let mut encoder = gpu.device.create_command_encoder(&Default::default());

        // resolve with temporary command encoder instead of the frame encoder
        self.resolve(&mut encoder);

        gpu.queue.submit(std::iter::once(encoder.finish()));

        let (tx, rx) = mpsc::channel();

        let buffer = self.readback_buffer.clone();

        self.readback_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                match result {
                    Ok(()) => {
                        let view = buffer.slice(..).get_mapped_range();
                        let timestamps: &[u64] = bytemuck::cast_slice(&view);

                        let time_start = timestamps[0];
                        let time_end = timestamps[1];

                        tx.send((time_start, time_end)).unwrap();
                    }
                    Err(e) => log::error!("Buffer map failed: {}", e),
                }

                buffer.unmap();
            });

        gpu.device.poll(wgpu::MaintainBase::Wait);

        let (start, end) = rx.recv().unwrap();

        let timestamp_period = gpu.queue.get_timestamp_period() as f64;
        let nanoseconds = (end - start) as f64 * timestamp_period;

        Duration::from_nanos(nanoseconds as u64)
    }
}

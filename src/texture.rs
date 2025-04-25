use std::path::Path;

use crate::{Error, TextureError};

pub fn load_raw<P>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: P,
    desc: &wgpu::TextureDescriptor,
) -> Result<wgpu::Texture, Error>
where
    P: AsRef<Path>,
{
    let texture = device.create_texture(desc);
    let bytes = std::fs::read(path)?;

    let bytes_per_pixel = desc
        .format
        .block_copy_size(None)
        .ok_or(TextureError::InvalidFormat(desc.format))?;

    let bytes_per_row = bytes_per_pixel * desc.size.width;
    let rows_per_image = match desc.dimension {
        wgpu::TextureDimension::D3 => Some(desc.size.height),
        _ => None,
    };

    queue.write_texture(
        texture.as_image_copy(),
        &bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_row),
            rows_per_image,
        },
        desc.size,
    );

    Ok(texture)
}

pub fn from_dynamic_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    image: &image::DynamicImage,
    label: &str,
    target_format: wgpu::TextureFormat,
    texture_usage: wgpu::TextureUsages,
) -> Result<wgpu::Texture, Error> {
    let format_error: Error = TextureError::InvalidFormat(target_format).into();

    let byte_buf: Vec<u8>;
    let short_buf: Vec<u16>;
    let float_buf: Vec<f32>;

    let bytes: &[u8] = match target_format {
        wgpu::TextureFormat::R8Unorm => match image {
            image::DynamicImage::ImageLuma8(buf) => buf.as_raw(),
            _ => return Err(format_error),
        },
        wgpu::TextureFormat::R16Unorm => match image {
            image::DynamicImage::ImageLuma16(buf) => bytemuck::cast_slice(buf.as_raw()),
            _ => return Err(format_error),
        },
        wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => match image {
            image::DynamicImage::ImageRgba8(buf) => bytemuck::cast_slice(buf.as_raw()),
            _ => {
                byte_buf = image.to_rgba8().into_raw();
                &byte_buf
            }
        },
        wgpu::TextureFormat::Rgba16Unorm => match image {
            image::DynamicImage::ImageRgba16(buf) => bytemuck::cast_slice(buf.as_raw()),
            _ => {
                short_buf = image.to_rgba16().into_raw();
                bytemuck::cast_slice(&short_buf)
            }
        },
        wgpu::TextureFormat::Rgba32Float => match image {
            image::DynamicImage::ImageRgba32F(buf) => bytemuck::cast_slice(buf.as_raw()),
            _ => {
                float_buf = image.to_rgba32f().into_raw();
                bytemuck::cast_slice(&float_buf)
            }
        },
        _ => return Err(format_error),
    };

    let bytes_per_pixel = target_format
        .block_copy_size(None)
        .ok_or(TextureError::InvalidFormat(target_format))?;

    let bytes_per_row = bytes_per_pixel * image.width();
    let rows_per_image = None; // image crate only allows 1D or 2D images

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: target_format,
        usage: texture_usage,
        view_formats: &[],
    });

    queue.write_texture(
        texture.as_image_copy(),
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_row),
            rows_per_image,
        },
        texture.size(),
    );

    Ok(texture)
}

pub fn copy(encoder: &mut wgpu::CommandEncoder, src: &wgpu::Texture, dst: &wgpu::Texture) {
    if src.size() != dst.size() {
        log::error!("Attempted to copy textures of different sizes");
    }

    encoder.copy_texture_to_texture(src.as_image_copy(), dst.as_image_copy(), src.size());
}

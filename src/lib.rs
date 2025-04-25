use thiserror::Error;

pub mod binding;
pub mod buffer;
pub mod shader;
pub mod state;
pub mod texture;

pub(crate) mod util;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Texture error: {0}")]
    Texture(#[from] TextureError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("wgpu error: {0}")]
    Wgpu(#[from] wgpu::Error),
}

#[derive(Error, Debug)]
pub enum TextureError {
    #[error("Invalid texture format {0:?}")]
    InvalidFormat(wgpu::TextureFormat),
}

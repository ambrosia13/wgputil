use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use crate::{util, Error};

/// The shader backend to use.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum ShaderBackend {
    Wgsl,
    Spirv,
}

struct ShaderMetadata {
    pub name: String,
    pub path: PathBuf,
    pub backend: ShaderBackend,
}

/// Contains data about shader source, including label, path, and backend (if not WGSL).
/// Used for managing shaders that may fail to compile, or whose contents may need to be
/// reloaded/reread from the source file.
pub struct ShaderSource {
    metadata: ShaderMetadata,
    source: Option<Vec<u8>>,
}

impl ShaderSource {
    fn load<P: AsRef<Path>>(path: P, backend: ShaderBackend) -> Self {
        let name = util::name_from_path(&path).unwrap_or_default();
        let path = path.as_ref().to_owned();

        let metadata = ShaderMetadata {
            name,
            path,
            backend,
        };

        fn read_shader_source<U: AsRef<Path>>(
            path: U,
            backend: ShaderBackend,
        ) -> std::io::Result<Vec<u8>> {
            match backend {
                ShaderBackend::Wgsl => {
                    let source = std::fs::read_to_string(&path)?;
                    Ok(source.into_bytes())
                }
                ShaderBackend::Spirv => Ok(std::fs::read(&path)?),
            }
        }

        let source = read_shader_source(&metadata.path, backend).ok();

        Self { metadata, source }
    }

    /// Create a WGSL [`ShaderSource`] given a path
    pub fn load_wgsl<P: AsRef<Path>>(path: P) -> Self {
        Self::load(path, ShaderBackend::Wgsl)
    }

    /// Create a Spir-V [`ShaderSource`] given a path
    pub fn load_spirv<P: AsRef<Path>>(path: P) -> Self {
        Self::load(path, ShaderBackend::Spirv)
    }

    /// Reread the contents of the shader from the source file, using the
    /// path given at creation.
    pub fn reload(&mut self) {
        let path = &self.metadata.path;
        *self = Self::load(path, self.metadata.backend);
    }

    /// Returns whether the shader source is fallback.
    pub fn is_fallback(&self) -> bool {
        self.source.is_none()
    }

    /// Makes this [`ShaderSource`] use the fallback shader.
    pub fn make_fallback(&mut self) {
        self.source = None;
    }

    pub fn backend(&self) -> ShaderBackend {
        self.metadata.backend
    }

    fn source_str(&self) -> Option<&str> {
        match self.backend() {
            ShaderBackend::Wgsl => Some(std::str::from_utf8(self.source.as_ref()?).unwrap()),
            ShaderBackend::Spirv => panic!("Can't get source strings for binary Spir-V format"),
        }
    }

    #[allow(unused)]
    fn source_words(&self) -> Option<Cow<'_, [u32]>> {
        match self.backend() {
            ShaderBackend::Wgsl => panic!("Can't get source words for wgsl"),
            ShaderBackend::Spirv => Some(wgpu::util::make_spirv_raw(self.source.as_ref()?)),
        }
    }

    fn descriptor(&self) -> wgpu::ShaderModuleDescriptor {
        match self.is_fallback() {
            false => match self.backend() {
                ShaderBackend::Wgsl => {
                    let source_str = self.source_str();

                    wgpu::ShaderModuleDescriptor {
                        label: Some(&self.metadata.name),
                        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source_str.unwrap())),
                    }
                }
                ShaderBackend::Spirv => {
                    todo!()
                }
            },
            true => self.fallback_descriptor(),
        }
    }

    /// Returns the shader module descriptor used for fallback shaders.
    pub fn fallback_descriptor(&self) -> wgpu::ShaderModuleDescriptor<'_> {
        wgpu::ShaderModuleDescriptor {
            label: Some(&self.metadata.name),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("assets/fallback.wgsl"))),
        }
    }
}

/// Creates a [`wgpu::ShaderModule`] given the [`ShaderSource`]. If creation fails for whatever
/// reason (e.g. compile error), then a [`wgpu::Error`] validation error is returned containing
/// the description of the error.
///
/// Either handle the error accordingly, or call [`ShaderSource::make_fallback`] on the source,
/// and then call this function again to create a fallback (basically empty) shader module.
pub fn create(device: &wgpu::Device, source: &ShaderSource) -> Result<wgpu::ShaderModule, Error> {
    device.push_error_scope(wgpu::ErrorFilter::Validation);

    let module = device.create_shader_module(source.descriptor());

    let compile_error = pollster::block_on(device.pop_error_scope());
    if let Some(error) = compile_error {
        return Err(error.into());
    }

    Ok(module)
}

/// Attempts to create the shader module, and if it fails, automatically creates a fallback shader.
pub fn create_or_fallback(
    device: &wgpu::Device,
    source: &mut ShaderSource,
) -> (wgpu::ShaderModule, Option<Error>) {
    match create(device, source) {
        Ok(s) => (s, None),
        Err(e) => {
            source.make_fallback();
            (create(device, source).unwrap(), Some(e))
        }
    }
}

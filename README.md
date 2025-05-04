# wgputil

A set of convenience and utility functions for wgpu. Doesn't abstract anything away, while still reducing boilerplate code.

# Features

## wgpu Initialization

- Initialize wgpu using `SurfaceState`, which contains information about the surface as well as handles to GPU resources
- Optional to use if you want to initialize wgpu yourself; the rest of this library does not depend on `SurfaceState`

## Bind Groups

- Create "linked" bind group layouts and bind groups together with less boilerplate
- Convenience functions to create shared bind group and bind group layout entries (`BindingEntry`) for GPU resources (textures, buffers, samplers)
    - includes functions for bindless resources / binding arrays
    - currently, shader visibility is ignored as it's only relevant with DirectX, which is unlikely to be selected as the GPU backend over Vulkan

## Shaders

- Read shader source from the file system using `ShaderSource`, which provides a fallback shader if the file was not found or if there was a shader compilation error.
- Create shader modules from `ShaderSource`, with the option to use a fallback shader if there was a compile error, or handle the error yourself (which is polled for you)

## Textures

- Load textures from binary data, given a path and a texture descriptor
- Load textures from various image formats (e.g. png, jpg)

## Timestamps

- Convenience struct `TimeQuery`, which manages query & readback buffers for you, making it easy to read timestamps
- Record timestamps in render/compute passes, or directly on a command encoder (Vulkan only feature)
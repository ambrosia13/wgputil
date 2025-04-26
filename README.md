A set of convenience functions for wgpu. Doesn't abstract anything away. Includes:
- function & struct to initialize and hold wgpu/winit state, and to draw to the screen (doesn't include the main window loop)
- reading textures from a file (images as well as raw binary)
- shader compilation utilities (e.g. fallible shader compilation)
- less verbose creation of bind groups and bind group layouts

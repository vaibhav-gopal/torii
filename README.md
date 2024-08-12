# Torii

Torii is a WIP and mainly for personal use at the moment
> Very incomplete at the moment (baby steps)

Torii ***aims to be*** a lightweight, modular, cross-platform **game engine** and **graphics visualization** api.

Torii uses the **Vulkan Graphics API** and is built in **Rust**.

## TODO
- [x] basic windowing and context creation
- [ ] abstracted instance, physical and logical device creation
- [ ] abstracted queue and pipeline creation (default graphics pipeline)
- [ ] abstracted surface, and swapchain creation
- [ ] Test a basic game
  - [ ] create very minimal ECS system
  - [ ] dynamically render primitive meshes
  - [ ] load textures and mipmapping support
  - [ ] load models from external software (blender)
- [ ] abstracted features to include mesh shader pipeline
- [ ] abstracted features to include compute shader support
- [ ] create build system to compile engine with application code (dylib?)
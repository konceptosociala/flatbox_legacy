<p align="center">
    <a href="https://konceptosociala.eu.org/software/sonja"><img src="sonja.svg" height="156" width="156" alt="sonja"></a>
</p>    

<p align="center">
  <img src="https://img.shields.io/badge/Status-Alpha-blue?style=flat-square" alt="status">
  <a href="crates.io/crates/sonja"><img src="https://img.shields.io/crates/v/sonja.svg?style=flat-square" alt="crates"></a>
  <img src="https://img.shields.io/github/stars/konceptosociala/sonja?style=flat-square&color=orange">
  <a href="https://github.com/konceptosociala/sonja/issues"><img src="https://img.shields.io/github/issues/konceptosociala/sonja?color=green&style=flat-square"></a>
</p>

<p align="center">
    3D rusty game engine, using API Vulkan and implementing paradigm of ECS
</p>

## WARNING
The crate is in a very early stage of development. Use it at your own risk

## Features

### Rendering
- [x] Screenshots
- [x] Egui
- [x] Custom materials
- [x] PBR Material
- [ ] Shadows `(WIP)`
- [ ] Animation `(WIP)`
- [ ] Model loading `(WIP)`
  - [x] Wavefront (`.obj`)
  - [ ] glTF (`.gltf/.glb`)

### Physics
- [x] Rigid bodies, colliders
- [ ] Joints
- [ ] Debug

### Misc
- [x] Wide error handling
- [x] Extension system
  - [x] Custom runner
  - [x] Custom event handlers
- [x] Dynamic scene saving/loading
- [x] Save-Load system
  - [x] ECS World
  - [x] Resources
- [ ] Lua scripts

### Audio
- [x] 2D
- [x] 3D
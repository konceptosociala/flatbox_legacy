<p align="center">
    <a href="https://konceptosociala.eu.org/despero"><img src="despero.svg" height="156" width="156" alt="despero"></a>
</p>    

<p align="center">
  <img src="https://img.shields.io/badge/Status-Alpha-blue?style=flat-square" alt="status">
  <a href="crates.io/crates/despero"><img src="https://img.shields.io/crates/v/despero.svg?style=flat-square" alt="crates"></a>
  <img src="https://img.shields.io/github/stars/konceptosociala/despero?style=flat-square&color=orange">
  <a href="https://github.com/konceptosociala/despero/issues"><img src="https://img.shields.io/github/issues/konceptosociala/despero?color=green&style=flat-square"></a>
</p>

<p align="center">
    Rusty game engine, using API Vulkan and implementing paradigm of ECS
</p>

## WARNING
The crate is in a very early stage of development. Use it at your own risk

## To fix:
- Texture coordinates
- Event reading (multiple event types are slow)
- Synchronize light descriptor sets with commandbuffer

## To implement:

- [x] `Prelude`
- [x] Reorganize `InstanceData` (Material) and `VertexData` (Mesh)
- [x] Resource processing (as hecs ECS)
    - [x] `Texture`
    - [x] `Material`
        - [x] `DefaultMat`
        - [x] Universal `Material`
    - [x] `Mesh`
- [x] ECS
    - [x] `Light`
    - [x] `ModelBundle`
    - [x] `Camera`
- [ ] Loading models
    - [x] Wavefront
    - [ ] Animations
    - [ ] COLLADA
- [ ] Resource compressing (lz4)
    - [ ] Scene (scn)
    - [ ] Model (mdl)
    - [ ] Texture (tex)
- [ ] `RenderTexture`
- [ ] Realistic lights
- [ ] Shadows
    - [ ] Simple
    - [ ] Soft
- [x] UI (egui)
- [ ] Physics (rapier3d)
- [ ] Scripting (mlua)
    - [ ] Simple scripts
    - [ ] Script as resource
- [ ] Scenes
    - [x] Save World (Serialize RON)
    - [ ] Save Materials and Textures
    - [ ] Load World (Deserialize RON)
    - [ ] RON-scenes (scn)
- [ ] Particle Systems (sonja)
- [ ] Game Settings
- [ ] Conditional systems
    - [ ] Pause

<p align="center">
	<a href="https://konceptosociala.eu.org/despero"><img src="despero.svg" height="128" width="128" alt="despero"></a>
</p>	

<p align="center">
  <img src="https://img.shields.io/badge/Status-Alpha-blue?style=flat-square" alt="status">
  <img src="https://img.shields.io/crates/v/despero.svg?style=flat-square" alt="crates">
  <img src="https://img.shields.io/github/stars/konceptosociala/despero?style=flat-square&color=orange">
  <img src="https://img.shields.io/github/issues/konceptosociala/despero?color=green&style=flat-square">
</p>

<p align="center">
	Rusty game engine, using API Vulkan and implementing paradigm of ECS
</p>

# Tasklisto
## To fix:
- Texture coordinates
- Event reading (multiple event types are slow)
- Synchronize light descriptor sets with commandbuffer

## To implement:

- [x] `Prelude`
- [x] Reorganize `InstanceData` (Material) and `VertexData` (Mesh)
- [ ] Resource processing (as hecs ECS)
	- [x] `Texture`
	- [ ] `Material`
		- [x] `DefaultMat`
		- [ ] Universal `Material`
	- [x] `Mesh`
- [x] ECS
	- [x] `Light`
	- [x] `ModelBundle`
	- [x] `Camera`
- [ ] Loading models
- [ ] `RenderTexture`
- [ ] Realistic lights
- [ ] Shadows
	- [ ] Simple
	- [ ] Soft
- [ ] UI (egui)
- [ ] Physics (rapier3d)
- [ ] Scripting (mlua)
	- [ ] Simple scripts
	- [ ] Script as resource
- [ ] GLTF Scenes
	- [ ] Processing
	- [ ] GLTFExtras custom parameters
	- [ ] Animation
- [ ] Particle Systems
- [ ] Game Settings
- [ ] Conditional systems

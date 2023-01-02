![despero_banner](banner.svg)

# Despero

Rusty game engine, using API Vulkan and implementing paradigm of ECS

# Tasklisto
## To fix:
- Texture coordinates
- Event reading (`dyn Event` -> `generic E`)
- Reorganize `Renderer`
- Multithreading (`Arc`) for `Renderer`

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

![despero_banner](banner.svg)

# Despero

Rustlingva ludmotoro, uzata API Vulkan kaj realigata koncepton de EKS (ento-komponanto-sistemo)

# Tasklisto

- [ ] `Prelude`
- [ ] Reorganizi `InstanceData` (Material) kaj `VertexData` (Mesh)
- [ ] Biblioteka privateco
- [x] Enteni ĉefan ĉiklon (eventloop) al Despero struct
- [ ] Refari (universaligi)
- [x] Reorganizi `Despero struct`:
```rust
struct Despero {
	renderer: Renderer,
}
```
- [ ] Realigi EKS
	- [ ] main
	- [ ] lib.rs
	- [ ] light.rs
	- [ ] model.rs
	- [ ] camera.rs
	- [ ] texture.rs

Sorry, I've forgotten some `mut`s in SubWorld declaration 

> Rather, put the call to `Schedule::exexute` inside each iteration of the event loop in `MainEventsCleared`

Oh, I thought, that it updates like every frame, but not after finishing of system execution, so the system is executed some times simultaneous. If it is relevant, so it changes everything. Is there a difference between putting  `Schedule::exexute` in `MainEventsCleared` and `RedrawRequested`? Thx a lot 

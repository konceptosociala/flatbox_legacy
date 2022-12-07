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
	- [x] main
	- [x] lib.rs
	- [ ] light.rs
	- [ ] model.rs
	- [ ] camera.rs
	- [ ] texture.rs

use std::sync::Arc;
use despero::prelude::*;
use ash::vk;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct MyMaterial {
	colour: [f32; 3],
}

impl Material for MyMaterial {
	fn pipeline(renderer: &Renderer) -> Pipeline {
		let vertex_shader = vk::ShaderModuleCreateInfo::builder()
			.code(vk_shader_macros::include_glsl!(
				"./shaders/vertex_simple.glsl", 
				kind: vert,
			));
		
		let fragment_shader = vk::ShaderModuleCreateInfo::builder()
			.code(vk_shader_macros::include_glsl!(
				"./shaders/fragment_simple.glsl",
				kind: frag,
			));
			
		let instance_attributes = vec![
			ShaderInputAttribute {
				binding: 1,
				location: 3,
				offset: 0,
				format: ShaderInputFormat::R32G32B32_SFLOAT,
			},
		];
		
		unsafe {
			Pipeline::init(
				&renderer,
				&vertex_shader,
				&fragment_shader,
				instance_attributes,
				12,
			).expect("Cannot create pipeline")
		}
	}
}

fn main() {	
	let mut despero = Despero::init(WindowBuilder::new().with_title("The Game"));
	let reader = despero.add_event_reader();
	
	despero
		.add_setup_system(bind_mat)
		.add_setup_system(create_models)
		.add_setup_system(create_camera)
		.add_system(handling(reader))
		.add_system(ecs_change)
		.run();
}

fn bind_mat(
	mut renderer: Write<Renderer>,
){
	renderer.bind_material::<MyMaterial>();
}

fn ecs_change(
	world: SubWorld<&mut Transform>,
){
	for (_, t) in &mut world.query::<&mut Transform>() {
		t.rotation *= UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0)), 0.05);
	}
}

fn handling(
	mut event_reader: EventReader
) -> impl FnMut() {
	move || {
		if let Ok(event) = event_reader.read::<KeyboardInput>() {
			println!("Keyboard events: {:?}", event);
		}
	}
}

fn create_models(
	mut cmd: Write<CommandBuffer>,
	mut renderer: Write<Renderer>,
){
	// Create texture
	let _texture = renderer.create_texture("assets/uv.jpg", Filter::LINEAR);
	// Load model from OBJ
	cmd.spawn(ModelBundle {
		mesh: Mesh::load_obj("assets/model.obj").swap_remove(0),
		material: renderer.create_material(Arc::new(MyMaterial { colour: [1.0, 0.0, 1.0] })),
		transform: Transform::default(),
	});
	// Add light
	cmd.spawn((DirectionalLight {
		direction: Vector3::new(-1., -1., 0.),
		illuminance: [0.5, 0.5, 0.5],
	},));	
}

fn create_camera(
	mut cmd: Write<CommandBuffer>,
){
	cmd.spawn(CameraBundle{
		camera: 
			Camera::builder()
				.is_active(true)
				.build(),
		transform: Transform::default(),
	});
}

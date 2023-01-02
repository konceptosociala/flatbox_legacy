use despero::prelude::*;

fn main() {	
	let mut despero = Despero::init(WindowBuilder::new().with_title("The Game"));
	let reader = despero.add_event_reader();
	
	despero
		.add_setup_system(create_models)
		.add_setup_system(create_camera)
		.add_system(handling(reader))
		.add_system(ecs_change)
		.run();
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
	let texture = renderer.create_texture("assets/uv.jpg", Filter::LINEAR);
	// Load model from OBJ
	cmd.spawn(ModelBundle {
		mesh: Mesh::load_obj("assets/model.obj").swap_remove(0),
		material: DefaultMat::new(
			texture,
			0.0,
			1.0,
		),
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

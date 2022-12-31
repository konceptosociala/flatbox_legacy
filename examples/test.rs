use despero::prelude::*;

fn main() {	
	let mut despero = Despero::init(WindowBuilder::new().with_title("The Game"));
	let reader = despero.add_event_reader();
	
	despero
		.add_setup_system(create_models)
		.add_setup_system(create_camera)
		.add_system(handling(reader))
		.run();
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
	let texture1 = renderer.create_texture("assets/image2.jpg", Filter::LINEAR);
	let texture2 = renderer.create_texture("assets/image.jpg", Filter::LINEAR);
	// Create textured plane
	cmd.spawn(ModelBundle {
		mesh: Mesh::plane(),
		material: DefaultMat::new(
			Matrix4::new_translation(&Vector3::new(1.5, 0., 0.3)),
			texture1,
			0.0,
			1.0,
		),
		transform: Transform::default(),
	});
	// Load model from OBJ
	cmd.spawn(ModelBundle {
		mesh: despero::take(Mesh::load_obj("assets/model.obj"), 0).unwrap(),
		material: DefaultMat::new(
			Matrix4::new_translation(&Vector3::new(-1.5, 1.0, 1.3)),
			texture2,
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
	
	cmd.spawn((PointLight {
		position: nalgebra::Point3::new(0.1, -3.0, -3.0),
		luminous_flux: [100.0, 100.0, 100.0],
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

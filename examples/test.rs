use winit::window::WindowBuilder;
use hecs_schedule::*;
use despero::prelude::*;
use despero::ecs::event::*;
use nalgebra as na;

fn main() {	
	let mut despero = Despero::init(WindowBuilder::new().with_title("The Game"))
		.add_setup_system(create_models)
		.add_setup_system(create_camera);
		
	let mut reader = EventReader::new(&mut despero.event_writer);
	let handling1 = move || {
		while let Ok(event) = reader.read() {
			println!("HANDLER 1: {:?}", event);
		}
	};
		
	let mut reader = EventReader::new(&mut despero.event_writer);
	let handling2 = move || {
		while let Ok(event) = reader.read() {
			println!("HANDLER 2: {:?}", event);
		}
	};
	
	despero
		.add_system(handling1)
		.add_system(handling2)
		.run();
}

fn create_models(
	mut cmd: Write<CommandBuffer>,
	mut renderer: Write<Renderer>,
){
	// Model
	let texture = renderer.texture_from_file("assets/image.jpg", Filter::LINEAR);
	let mut quad = Model::quad();
	quad.insert_visibly(TexturedInstanceData::new(
		Matrix4::new_translation(&Vector3::new(2.0, 0., 0.3)),
		texture,
		0.0,
		1.0,
	));
	let transform = Transform::default();
	cmd.spawn((quad, transform));
	
	// Lights
	let mut lights = LightManager::default();
	lights.add_light(DirectionalLight {
		direction: na::Vector3::new(-1., -1., 0.),
		illuminance: [0.5, 0.5, 0.5],
	});
	//~ lights.add_light(PointLight {
		//~ position: na::Point3::new(0.1, -3.0, -3.0),
		//~ luminous_flux: [100.0, 100.0, 100.0],
	//~ });
	//~ lights.add_light(PointLight {
		//~ position: na::Point3::new(0.1, -3.0, -3.0),
		//~ luminous_flux: [100.0, 100.0, 100.0],
	//~ });
	//~ lights.add_light(PointLight {
		//~ position: na::Point3::new(0.1, -3.0, -3.0),
		//~ luminous_flux: [100.0, 100.0, 100.0],
	//~ });
	lights.update_buffer(&mut renderer).expect("Cannot update lights");
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

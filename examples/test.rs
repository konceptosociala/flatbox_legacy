use winit::window::WindowBuilder;
use hecs_schedule::*;
use despero::prelude::*;
use despero::engine::light::*;
use nalgebra as na;

fn main() {
	Despero::init(WindowBuilder::new().with_title("The Game"))
		.add_setup_system(create_models)
		.add_setup_system(create_camera)
		.add_setup_system(setup_test)
		.add_system(loop_test)
		.run();
}

fn loop_test(){
	Debug::info("hello");
}

fn setup_test(){
	Debug::warn("bye");
}

fn create_models(
	mut cmd: Write<CommandBuffer>,
	mut renderer: Write<Renderer>,
){
	let _t1 = renderer.texture_from_file("assets/image.jpg", Filter::LINEAR).expect("Cannot create texture");
	
	let mut quad = Model::quad();
	quad.insert_visibly(TexturedInstanceData::new(
        Matrix4::new_translation(&Vector3::new(2.0, 0., 0.3)),
        _t1,
        0.0,
        1.0,
    ));
	let transform = Transform::default();
	
	cmd.spawn((quad, transform));
	
	let mut lights = LightManager::default();
	lights.add_light(DirectionalLight {
		direction: na::Vector3::new(-1., -1., 0.),
		illuminance: [0.5, 0.5, 0.5],
	});
	lights.add_light(PointLight {
		position: na::Point3::new(0.1, -3.0, -3.0),
		luminous_flux: [100.0, 100.0, 100.0],
	});
	lights.add_light(PointLight {
		position: na::Point3::new(0.1, -3.0, -3.0),
		luminous_flux: [100.0, 100.0, 100.0],
	});
	lights.add_light(PointLight {
		position: na::Point3::new(0.1, -3.0, -3.0),
		luminous_flux: [100.0, 100.0, 100.0],
	});
	
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























/*	    
	
	let mut sphere = Model::sphere(3);
	for i in 0..10 {
		for j in 0..10 {
			sphere.insert_visibly(InstanceData::new(
				na::Matrix4::new_translation(&na::Vector3::new(i as f32 - 5., j as f32 + 5., 10.0))
					* na::Matrix4::new_scaling(0.5),
				[0., 0., 0.8],
				i as f32 * 0.1,
				j as f32 * 0.1,
			));
		}
	}*/
	
	// Lights
	

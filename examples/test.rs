use winit::window::WindowBuilder;
use hecs_schedule::*;
use despero::prelude::*;

fn main() {
	Despero::init(WindowBuilder::new().with_title("The Game"))
		.add_system(create_models)
		.add_system(create_camera)
		.run();
}

fn create_models(
	cmd: CommandBuffer,
){
	//
}

fn create_camera(
	cmd: CommandBuffer,
){
	cmd.spawn(CameraBundle{
		camera: Camera::builder().build(),
		transform: Transform::default(),
	});
}























/*
	// Textures
	let texture_id = despero.texture_from_file("assets/image.jpg", Filter::LINEAR)?;
	let second_texture_id = despero.texture_from_file("assets/image2.jpg", Filter::LINEAR)?;
	let third_texture_id = despero.texture_from_file("assets/image.jpg", Filter::NEAREST)?;
	
	//Models
	let mut quad = Model::quad();
	quad.insert_visibly(TexturedInstanceData::from_matrix_and_texture(
		na::Matrix4::identity(),
		texture_id,
	));
	quad.insert_visibly(TexturedInstanceData::from_matrix_and_texture(
        na::Matrix4::new_translation(&na::Vector3::new(2.0, 0., 0.3)),
        second_texture_id,
    ));
	quad.insert_visibly(TexturedInstanceData::from_matrix_and_texture(
        na::Matrix4::new_translation(&na::Vector3::new(5.0, 0., 0.3)),
        third_texture_id,
    ));
    
    despero.models = vec![quad];
    despero.run();
    
    Ok(())
	
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
	/*let mut lights = LightManager::default();
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
	
	lights.update_buffer(
		&despero.device, 
		&mut despero.allocator, 
		&mut despero.lightbuffer, 
		&mut despero.descriptor_sets_light
	)?;*/

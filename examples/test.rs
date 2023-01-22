pub mod modules;

use std::sync::Arc;
use despero::prelude::*;

use modules::materials::*;

fn main() {	
	let mut despero = Despero::init(WindowBuilder::new().with_title("The Game"));
	let reader = despero.add_event_reader();
	
	despero
		.add_setup_system(bind_mat)
		.add_setup_system(create_models)
		.add_setup_system(create_camera)
		.add_system(handling(reader))
		.add_system(ecs_change)
		.add_system(gui)
		.run();
}

fn gui(ctx: Read<egui::Context>) {
	egui::CentralPanel::default().show(&ctx, |ui| {
		ui.add(egui::Label::new("Hello World!"));
		ui.label("A shorter and more convenient way to add a label.");
		if ui.button("Click me").clicked() {
			panic!("I love the life!");
		}
	});
}

fn bind_mat(
	mut renderer: Write<Renderer>,
){
	renderer.bind_material::<MyMaterial>();
	renderer.bind_material::<TexMaterial>();
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
) -> impl FnMut(SubWorld<&mut Camera>) {
	move |camera_world: SubWorld<&mut Camera>| {
		if let Ok(event) = event_reader.read::<KeyboardInput>() {
			for (_, camera) in &mut camera_world.query::<&mut Camera>() {
				match event.virtual_keycode.unwrap() {
					KeyCode::Up => camera.turn_up(0.02),
					KeyCode::Down => camera.turn_down(0.02),
					KeyCode::Left => camera.turn_left(0.02),
					KeyCode::Right => camera.turn_right(0.02),
					_ => {},
				}
			}
		}
	}
}

fn create_models(
	mut cmd: Write<CommandBuffer>,
	mut renderer: Write<Renderer>,
){
	let txt1 = renderer.create_texture("assets/uv.jpg", Filter::NEAREST) as u32;
	let txt2 = renderer.create_texture("assets/image.jpg", Filter::LINEAR) as u32;
	
	cmd.spawn(ModelBundle {
		mesh: Mesh::load_obj("assets/model.obj").swap_remove(0),
		material: renderer.create_material(Arc::new(MyMaterial {
			colour: [0.7, 0.0, 0.0]
		})),
		transform: Transform::from_translation(Vector3::new(1.0, 0.0, 0.0)),
	});
	
	cmd.spawn(ModelBundle {
		mesh: Mesh::load_obj("assets/model.obj").swap_remove(0),
		material: renderer.create_material(Arc::new(MyMaterial {
			colour: [0.0, 0.6, 1.0]
		})),
		transform: Transform::from_translation(Vector3::new(-1.0, 0.0, 0.0)),
	});
	
	cmd.spawn(ModelBundle {
		mesh: Mesh::load_obj("assets/model.obj").swap_remove(0),
		material: renderer.create_material(Arc::new(TexMaterial {
			texture_id: txt1
		})),
		transform: Transform::from_translation(Vector3::new(1.0, 0.0, -2.0)),
	});
	
	cmd.spawn(ModelBundle {
		mesh: Mesh::load_obj("assets/model.obj").swap_remove(0),
		material: renderer.create_material(Arc::new(TexMaterial {
			texture_id: txt2
		})),
		transform: Transform::from_translation(Vector3::new(-1.0, 0.0, -2.0)),
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

use sonja::prelude::*;

fn main() {
    Sonja::init(WindowBuilder::default())
        .default_systems()
        .add_setup_system(create_character)
        .add_system(move_character)
        .run();
}

fn create_character(
    mut cmd: Write<CommandBuffer>,
    mut assets: Write<AssetManager>,
    mut physics: Write<PhysicsHandler>,
){
    let albedo = assets.create_texture("assets/textures/uv.jpg", Filter::Linear);
    let material = assets.create_material(DefaultMat::builder().albedo(albedo).build());
    let plane = Model::plane();

    cmd.spawn((
        plane.clone(),
        Transform::from_rotation(UnitQuaternion::from_axis_angle(&Vector3::x_axis(), to_radian(-90.0))),
        material,
        physics.new_instance(RigidBodyBuilder::fixed().build(), ColliderBuilder::from(plane.mesh.unwrap()).build()),
    ));

    cmd.spawn(CameraBundle {
        camera: Camera::builder()
            .camera_type(CameraType::FirstPerson)
            .is_active(true)
            .build(),
        transform: Transform::new(
            Vector3::new(-1.75, 2.5, 3.0),
            UnitQuaternion::from_axis_angle(&Vector3::x_axis(), to_radian(30.0)) *
            UnitQuaternion::from_axis_angle(&Vector3::y_axis(), to_radian(30.0)), 
            1.0,
        ) 
    });

    let cube = Model::cube();

    cmd.spawn((
        cube.clone(),
        Transform::new(
            Vector3::new(0.0, 3.0, 0.5), 
            UnitQuaternion::identity(),
            0.5,
        ),
        material,
        physics.new_instance(RigidBodyBuilder::dynamic().build(), ColliderBuilder::from(cube.mesh.unwrap()).build()),
    ));
}

fn move_character(

){

}
use despero::prelude::*;

pub mod modules;
use modules::materials::*;

struct LatestPos(Point2<f32>);

fn main() {
    Despero::init(WindowBuilder {
        title: Some("PBR Test"),
        fullscreen: Some(true),
        ..Default::default()
    })
    
        .default_systems()
        
        .add_setup_system(bind_mat)
        .add_setup_system(create_scene)
        .add_system(process_scene)
        
        .run();
}

fn bind_mat(
    mut renderer: Write<Renderer>
){
    renderer.bind_material::<TexMaterial>();
    info!("Material's been bound");
}

fn create_scene(
    mut cmd: Write<CommandBuffer>,
    mut renderer: Write<Renderer>,
){
    let diffuse = renderer.create_texture("assets/pbr_test/diffuse.jpg", Filter::LINEAR) as u32;
    
    cmd.spawn(
        ModelBundle::builder()
            .mesh(Mesh::plane())
            .material(renderer.create_material(
                DefaultMat::builder()
                    .texture_id(diffuse)
                    .metallic(0.0)
                    .roughness(0.5)
                    .build()
            ))
            .transform(Transform::from_rotation(UnitQuaternion::from_axis_angle(&Vector3::x_axis(), to_radian(-45.0))))
            .build()
    );
    
    cmd.spawn((
        Camera::builder()
            .is_active(true)
            .camera_type(CameraType::FirstPerson)
            .build(),
        Transform::from_translation(Vector3::new(0.0, 0.0, 3.0)),
        LatestPos(Point2::origin()),
    ));
    
    cmd.spawn((
        DirectionalLight {
            direction: Vector3::new(-1., -1., 0.),
            illuminance: [0.5, 0.5, 0.5],
        },
    ));
    
    let sky_tex = renderer.create_texture("assets/StandardCubeMap.png", Filter::LINEAR) as u32;
    
    cmd.spawn(
        ModelBundle::builder()
            .mesh(Mesh::load_obj("assets/skybox.obj").swap_remove(0))
            .material(
                renderer.create_material(TexMaterial {
                    texture_id: sky_tex,
                })
            )
            .transform(Transform::from_translation(Vector3::new(0.0, 0.0, 0.0)))
            .build()
    );
}

fn process_scene(
    camera_world: SubWorld<(&Camera, &mut LatestPos, &mut Transform)>,
    gui_events: Read<EventHandler<GuiContext>>,
    //~ time: Read<Time>,
){
    if let Some(ctx) = gui_events.read() {
        if let Some(current) = ctx.pointer_hover_pos() {
            for (_, (_, mut latest, mut t)) in &mut camera_world.query::<(&Camera, &mut LatestPos, &mut Transform)>(){
                let delta_x = current.x - latest.0.x;
                let delta_y = current.y - latest.0.y;                
                *latest = LatestPos(Point2::new(current.x, current.y));

                t.rotation *= 
                    UnitQuaternion::from_axis_angle(&Vector3::x_axis(), delta_y * 0.01) * 
                    UnitQuaternion::from_axis_angle(&Vector3::y_axis(), -delta_x * 0.01);
            }
        }
    }
}

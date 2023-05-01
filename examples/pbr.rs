use despero::prelude::*;

pub mod modules;
use modules::materials::*;

#[derive(Clone, Default, Debug)]
struct CameraConfiguration {
    limit: (f32, f32),
    target_x: f32,
    target_y: f32,
    latest_pos: Point2<f32>,
}

fn main() {
    Despero::init(WindowBuilder {
        title: Some("PBR Test"),
        fullscreen: Some(true),
        renderer: Some(RenderType::Forward),
        ..Default::default()
    })
    
        .default_systems()
        
        .add_setup_system(create_scene)
        .add_system(process_scene)
        
        .run();
}

fn create_scene(
    mut cmd: Write<CommandBuffer>,
    mut asset_manager: Write<AssetManager>,
    mut renderer: Write<Renderer>,
){
    renderer.bind_material::<TexMaterial>();
    
    let diffuse = asset_manager.create_texture("assets/pbr_test/diffuse.jpg", Filter::Linear, &mut renderer);
    
    cmd.spawn(
        ModelBundle::builder()
            .mesh(Mesh::plane())
            .material(asset_manager.create_material(
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
        CameraConfiguration {
            limit: (-85.0, 85.0),
            ..Default::default()
        },
    ));
    
    cmd.spawn((
        DirectionalLight {
            direction: Vector3::new(-1., -1., 0.),
            illuminance: [0.5, 0.5, 0.5],
        },
    ));
    
    let sky_tex = asset_manager.create_texture("assets/StandardCubeMap.png", Filter::Linear, &mut renderer);
    
    cmd.spawn(
        ModelBundle::builder()
            .mesh(Mesh::load_obj("assets/skybox.obj").swap_remove(0))
            .material(
                asset_manager.create_material(TexMaterial {
                    texture_id: sky_tex.unwrap() as u32,
                })
            )
            .transform(Transform::from_translation(Vector3::new(0.0, 0.0, 0.0)))
            .build()
    );
}

fn process_scene(
    camera_world: SubWorld<(&Camera, &mut CameraConfiguration, &mut Transform)>,
    gui_events: Read<EventHandler<GuiContext>>,
    time: Read<Time>,
){
    if let Some(ctx) = gui_events.read() {
        if let Some(current) = ctx.pointer_hover_pos() {
            for (_, (_, mut conf, mut t)) in &mut camera_world.query::<(&Camera, &mut CameraConfiguration, &mut Transform)>(){       
                let (delta_x, delta_y) = {
                    if conf.latest_pos == Point2::origin() {
                        (0.0, 0.0)
                    } else {
                        (current.x - conf.latest_pos.x, current.y - conf.latest_pos.y)
                    }
                };
                
                conf.latest_pos = Point2::new(current.x, current.y);
                
                let local_x = t.local_x();
                
                let (tx, ty) = (conf.target_x.clone(), conf.target_y.clone());
                
                conf.target_x += delta_y * 0.0005 * time.delta_time().as_millis() as f32;
                conf.target_y -= delta_x * 0.0005 * time.delta_time().as_millis() as f32;
                
                conf.target_x = conf.target_x.clamp(to_radian(conf.limit.0), to_radian(conf.limit.1));

                t.rotation *= 
                    UnitQuaternion::from_axis_angle(&local_x, conf.target_x - tx) * 
                    UnitQuaternion::from_axis_angle(&Vector3::y_axis(), conf.target_y - ty);
            }
        }
    }
}
use serde::{Deserialize, Serialize};
use flatbox::prelude::*;

#[derive(Clone, Default, Debug)]
struct CameraConfiguration {
    limit: (f32, f32),
    target_x: f32,
    target_y: f32,
    latest_pos: Point2<f32>,
}

#[repr(C)]
#[derive(Material, Clone, Default, Debug, Serialize, Deserialize)]
#[material(
    vertex = "examples/shaders/my_material.vs",
    fragment = "examples/shaders/my_material.fs",
    topology = "line_list"
)]
pub struct MyMaterial {
    #[color]
    pub color: [f32; 3],
    #[texture]
    pub albedo: u32,
    pub blank: i32,
}

struct SkyBoxComp;
struct EnableMouse(bool);

fn main() {
    Flatbox::init(WindowBuilder {
        title: "PBR Test",
        fullscreen: true,
        renderer: RenderType::Forward,
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
) -> FlatboxResult<()> {    
    renderer.bind_material::<MyMaterial>();

    create_skybox(&mut cmd, &mut asset_manager)?;
    create_plane(&mut cmd, &mut asset_manager);
    create_box(&mut cmd, &mut asset_manager);
    create_camera(&mut cmd);
    create_lights(&mut cmd);
    enable_mouse(&mut cmd, true);

    Ok(())
}

fn process_scene(
    camera_world: SubWorld<(&Camera, &mut CameraConfiguration, &mut Transform)>,
    skybox_world: SubWorld<(&mut Transform, &SkyBoxComp)>,
    light_world: SubWorld<&mut PointLight>,
    enable_mouse_world: SubWorld<&EnableMouse>,
    events: Read<Events>,
    time: Read<Time>,
){
    let gui_events = events.get_handler::<GuiContext>().unwrap();
    
    let mut skybox = skybox_world.query::<(&mut Transform, &SkyBoxComp)>();
    let (_, (mut skybox_transform, _)) = skybox.iter().next().unwrap();

    let mut enable_mouse = enable_mouse_world.query::<&EnableMouse>();
    let (_, enable_mouse) = enable_mouse.iter().next().unwrap();

    if let Some(ctx) = gui_events.read() {
        egui::SidePanel::left("my_left_panel").show(&ctx, |ui| {
            for (e, mut light) in &mut light_world.query::<&mut PointLight>(){
                ui.label(format!("Light {e:?}"));
                ui.add(egui::Slider::new(&mut light.position.x, -5.0..=5.0).text("Light X"));
                ui.add(egui::Slider::new(&mut light.position.y, -5.0..=5.0).text("Light X"));
                ui.add(egui::Slider::new(&mut light.position.z, -5.0..=5.0).text("Light X"));
                ui.separator();
            }
        });

        if let Some(current) = ctx.pointer_hover_pos() {
            if !enable_mouse.0 { return; }

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

                skybox_transform.translation.x = t.translation.x;
                skybox_transform.translation.y = t.translation.y;
                skybox_transform.translation.z = -t.translation.z;
            }
        }
    }
}

fn create_textures(
    asset_manager: &mut AssetManager
) -> (AssetHandle<'T'>, AssetHandle<'T'>, AssetHandle<'T'>, AssetHandle<'T'>) {
    (
        asset_manager.create_texture("assets/textures/pbr_test/diffuse.jpg", Filter::Linear), 
        asset_manager.create_texture("assets/textures/pbr_test/rgh.jpg", Filter::Linear),
        asset_manager.create_texture("assets/textures/pbr_test/nrm.jpg", Filter::Linear), 
        asset_manager.create_texture("assets/textures/pbr_test/ao.jpg", Filter::Linear),
    )
}

fn create_plane_material(
    asset_manager: &mut AssetManager,
) -> AssetHandle<'M'> {
    let (albedo, roughness, normal, ao) = create_textures(asset_manager);

    asset_manager.create_material(
        DefaultMat::builder()
            .albedo(albedo)
            .roughness_map(roughness)
            .normal_map(normal)
            .ao_map(ao)
            .metallic(0.0)
            .roughness(1.0)
            .build()
    )
}

fn create_box_material(
    asset_manager: &mut AssetManager,
) -> AssetHandle<'M'> {
    asset_manager.create_material(
        MyMaterial::builder()
            .albedo(AssetHandle::BUILTIN_ALBEDO)
            .color(Color::new(1.0, 0.5, 0.0))
            .build()
    )
}

fn create_plane(
    cmd: &mut CommandBuffer,
    asset_manager: &mut AssetManager,
){
    let material = create_plane_material(asset_manager);

    cmd.spawn(
        ModelBundle::builder()
            .model(Model::plane())
            .material(material)
            .transform(Transform::from_rotation(UnitQuaternion::from_axis_angle(&Vector3::x_axis(), to_radian(-45.0))))
            .build()
    );
}

fn create_box(
    cmd: &mut CommandBuffer,
    asset_manager: &mut AssetManager
){
    let material = create_box_material(asset_manager);

    cmd.spawn(
        ModelBundle::builder()
            .model(Model::cube())
            .material(material)
            .transform(Transform::from_translation(Vector3::new(0.0, 1.5, 0.0)))
            .build()
    );
}


fn create_skybox(
    cmd: &mut CommandBuffer,
    asset_manager: &mut AssetManager,
) -> FlatboxResult<()> {
    asset_manager.skybox = Some(SkyBox(Texture::new_from_path(
        "assets/textures/cubemap2.jpg", 
        Filter::Nearest, 
        TextureType::Cubemap,
    )));

    cmd.spawn((
        Model::load_obj("assets/models/skybox.obj")?,
        asset_manager.create_material(SkyBoxMat),
        Transform::from_scale(Scale3::new(2.0, 2.0, 2.0)),
        SkyBoxComp,
    ));

    Ok(())
}

fn create_camera(cmd: &mut CommandBuffer){
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
}

fn create_lights(cmd: &mut CommandBuffer){
    cmd.spawn((
        DirectionalLight {
            direction: Vector3::new(-1., -1., 0.),
            illuminance: [0.5, 0.5, 0.5],
        },
    ));

    cmd.spawn((
        PointLight {
            position: Point3::new(-1.0, 0.0, 1.0),
            luminous_flux: [23.47, 21.31, 20.79],
        },
    ));

    cmd.spawn((
        PointLight {
            position: Point3::new(1.0, 0.0, 1.0),
            luminous_flux: [23.47, 21.31, 20.79],
        },
    ));
}

fn enable_mouse(cmd: &mut CommandBuffer, enabled: bool){
    cmd.spawn((EnableMouse(enabled),));
}
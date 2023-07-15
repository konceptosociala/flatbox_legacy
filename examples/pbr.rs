use sonja::prelude::*;

#[derive(Clone, Default, Debug)]
struct CameraConfiguration {
    limit: (f32, f32),
    target_x: f32,
    target_y: f32,
    latest_pos: Point2<f32>,
}

fn main() {
    Sonja::init(WindowBuilder {
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
) -> SonjaResult<()> {    
    let albedo = asset_manager.create_texture("assets/textures/pbr_test/diffuse.jpg", Filter::Linear);
    let roughness = asset_manager.create_texture("assets/textures/pbr_test/rgh.jpg", Filter::Linear);
    let normal = asset_manager.create_texture("assets/textures/pbr_test/nrm.jpg", Filter::Linear);
    let ao = asset_manager.create_texture("assets/textures/pbr_test/ao.jpg", Filter::Linear);

    let material = asset_manager.create_material(
        DefaultMat::builder()
            .albedo(albedo)
            .roughness_map(roughness)
            .normal_map(normal)
            .ao_map(ao)
            .metallic(0.0)
            .roughness(1.0)
            .build()
    );

    cmd.spawn(
        ModelBundle::builder()
            .model(Model::plane())
            .material(material)
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
        
    // asset_manager.skybox = Some(SkyBox(Texture::new_from_path(
    //     "assets/textures/cubemap.jpg", 
    //     Filter::Nearest, 
    //     TextureType::Cubemap,
    // )));

    // cmd.spawn(
    //     ModelBundle::builder()
    //         .model(Model::new("assets/models/skybox.obj")?)
    //         .material(asset_manager.create_material(SkyBoxMat))
    //         .transform(Transform::default())
    //         .build()
    // );

    Ok(())
}

fn process_scene(
    camera_world: SubWorld<(&Camera, &mut CameraConfiguration, &mut Transform)>,
    light_world: SubWorld<&mut PointLight>,
    events: Read<Events>,
    time: Read<Time>,
){
    let gui_events = events.get_handler::<GuiContext>().unwrap();

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

        // if let Some(current) = ctx.pointer_hover_pos() {
        if let Some(current) = Option::<gui::Pos2>::None {
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

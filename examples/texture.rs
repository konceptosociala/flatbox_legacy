use ron::ser::PrettyConfig;
use flatbox::prelude::*;

struct TextureSwitcher {
    first: AssetHandle<'T'>,
    second: AssetHandle<'T'>,
    third: AssetHandle<'T'>,
}

fn main() {
    Flatbox::init(WindowBuilder::default())
        .default_systems()
        .add_setup_system(texture)    
        .add_system(ui_system) 
        .run();
}

fn texture(
    mut assets: Write<AssetManager>,
    mut cmd: Write<CommandBuffer>,
) -> FlatboxResult<()> {
    let loaded = Texture::new_from_path("assets/textures/uv.jpg", Filter::Nearest, TextureType::Plain);
    let solid = Texture::new_solid(Color::new(172, 0, 255), TextureType::Plain, 16, 16);
    let generic = Texture::new_from_raw(&[
        0, 200, 255, 255,
        0, 200, 255, 255,
        255, 200, 0, 255,
        255, 200, 0, 255,
    ], Filter::Nearest, TextureType::Plain, 2, 2);

    let ser = ron::ser::to_string_pretty(&(loaded, solid, generic), PrettyConfig::default())?;
    debug!("Serialized: {ser}");
    let (loaded, solid, generic) = ron::from_str::<(Texture, Texture, Texture)>(&ser)?;
    debug!("Deserialized: ({loaded:#?}, {solid:#?}, {generic:#?})");
    
    let first = add_texture(&mut assets, loaded);
    let second = add_texture(&mut assets, solid);
    let third = add_texture(&mut assets, generic);

    cmd.spawn(ModelBundle {
        model: Model::load_obj("assets/models/model.obj")?,
        material: assets.create_material(DefaultMat::builder().albedo(second.clone()).build()),
        transform: Transform::from_rotation(UnitQuaternion::from_axis_angle(
            &Unit::new_normalize(Vector3::new(1.0, 1.0, 1.0)),
            to_radian(30.0),
        )),
    });

    cmd.spawn(CameraBundle {
        camera: Camera::builder()
            .is_active(true)
            .camera_type(CameraType::LookAt)
            .build(),
        transform: Transform::from_translation(Vector3::new(0.0, 0.0, 3.0)),
    });

    cmd.spawn((TextureSwitcher { first, second, third },));

    Ok(())
}

fn ui_system(
    assets: Read<AssetManager>,
    events: Read<Events>,
    switcher_world: SubWorld<&TextureSwitcher>,
    model_world: SubWorld<With<&AssetHandle<'M'>, (&Model, &Transform)>>
){
    let mut switcher = switcher_world.query::<&TextureSwitcher>();
    let (_, switcher) = switcher.iter().next().unwrap();

    let gui_events = events.get_handler::<GuiContext>().unwrap();
    if let Some(ctx) = gui_events.read() {
        gui::SidePanel::left("my_panel").show(&ctx, |ui| {
            ui.label("Press key (1,2,3) to switch texture");

            for (_, handle) in &mut model_world.query::<With<&AssetHandle<'M'>, (&Model, &Transform)>>(){
                let material = assets.get_material_downcast::<DefaultMat>(*handle).unwrap();
                ui.label(format!("Current texture is {}", match material.albedo {
                    2 => "loaded",
                    3 => "solid",
                    4 => "generic",
                    _ => "<Unknown>",
                }));
            }
        });

        if ctx.input().key_pressed(gui::Key::Num1) {
            for (_, handle) in &mut model_world.query::<With<&AssetHandle<'M'>, (&Model, &Transform)>>(){
                let mut material = assets.get_material_downcast_mut::<DefaultMat>(*handle).unwrap();
                material.albedo = switcher.first.into();
            }
        }

        if ctx.input().key_pressed(gui::Key::Num2) {
            for (_, handle) in &mut model_world.query::<With<&AssetHandle<'M'>, (&Model, &Transform)>>(){
                let mut material = assets.get_material_downcast_mut::<DefaultMat>(*handle).unwrap();
                material.albedo = switcher.second.into();
            }
        }

        if ctx.input().key_pressed(gui::Key::Num3) {
            for (_, handle) in &mut model_world.query::<With<&AssetHandle<'M'>, &Model>>(){
                let mut material = assets.get_material_downcast_mut::<DefaultMat>(*handle).unwrap();
                material.albedo = switcher.third.into();
            }
        }
    }
}

fn add_texture(assets: &mut AssetManager, texture: Texture) -> AssetHandle<'T'> {
    let handle = AssetHandle::<'T'>::from_index(assets.textures.len());
    assets.textures.push(texture);
    handle
}
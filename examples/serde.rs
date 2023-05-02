use serde::{Serialize, Deserialize};
use despero::prelude::*;
use despero::world_serializer;

#[derive(Default, Deserialize, Serialize)]
pub struct WorldSaver {
    components: Vec<String>,
}

impl WorldSaver {
    pub fn new() -> Self {
        WorldSaver::default()
    }
}

world_serializer!(
    WorldSaver, 
        Mesh, 
        Transform, 
        AssetHandle,
        Camera
);


fn main() {    
    Despero::init(WindowBuilder::default())
        .default_systems()
        .add_setup_system(setup)
        .add_system(gui_system)
        .run();
}

fn setup(
    mut renderer: Write<Renderer>,
    mut asset_manager: Write<AssetManager>,
    mut cmd: Write<CommandBuffer>,
){
    let texture_id = asset_manager.create_texture("assets/uv.jpg", Filter::Nearest, &mut renderer);    
    let mesh = Mesh::load_obj("assets/model.obj").swap_remove(0);
    
    cmd.spawn(ModelBundle {
        mesh,
        material: asset_manager.create_material(
            DefaultMat::builder()
                .texture_id(texture_id)
                .metallic(0.0)
                .roughness(1.0)
                .build(),
        ),
        transform: Transform::default(),
    });
    
    cmd.spawn(CameraBundle{
        camera: 
            Camera::builder()
                .is_active(true)
                .build(),
        transform: Transform::from_translation(Vector3::new(0.0, 0.0, 5.0)),
    });
}

fn gui_system(
    gui_events: Read<EventHandler<GuiContext>>,
    world: Read<World>,
    mut cmd: Write<CommandBuffer>,
    model_world: SubWorld<Without<&mut Transform, &Camera>>,
){
    for (_, mut t) in &mut model_world.query::<Without<&mut Transform, &Camera>>(){
        t.rotation *= UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::new(1.0, 1.0, 1.0)), to_radian(1.0));
    }
    
    if let Some(ctx) = gui_events.read() {
        
        gui::SidePanel::left("my_panel").show(&ctx, |ui| {
            ui.label("World (de-)serialization test");
            
            let mut ws = WorldSaver::new();
            
            if ui.button("Save world").clicked() {
                match ws.save("assets/world.ron", &world) {
                    Ok(()) => info!("World saved!"),
                    Err(e) => error!("World not saved: {:?}", e),
                };
            }
            
            if ui.button("Load world").clicked() {
                cmd.write(move |world: &mut World| {
                    match ws.load("assets/world.ron", world) {
                        Ok(()) => info!("World loaded!"),
                        Err(e) => error!("World not loaded: {:?}", e),
                    }
                })
            }
            
        });
    }
}

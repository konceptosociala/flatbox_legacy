use serde::{Serialize, Deserialize};
use despero::prelude::*;
use despero::world_serializer;

pub mod modules;
use modules::materials::*;

#[derive(Deserialize, Serialize)]
struct WorldSaver;

impl WorldSaver {
    pub fn new() -> Self {
        WorldSaver
    }
}

world_serializer!(WorldSaver, Mesh, Transform, MaterialHandle);

fn main() {    
    let mut despero = Despero::init(WindowBuilder::new().with_title("The Game"));
    
    let egui_reader = despero.add_event_reader();
    
    despero
        .add_setup_system(bind_mat)
        .add_setup_system(create_models)
        .add_setup_system(create_camera)        
        .add_system(ecs_change)
        .add_system(egui_handling(egui_reader))
        .run();
}

fn bind_mat(
    mut renderer: Write<Renderer>
){
    renderer.bind_material::<MyMaterial>();
    renderer.bind_material::<TexMaterial>();
    info!("Material's been bound");
}

fn egui_handling(
    mut event_reader: EventReader,
) -> impl FnMut(Read<World>) {
    move |world: Read<World>| {
        
        if let Ok(ctx) = event_reader.read::<GuiContext>() {
            egui::SidePanel::left("my_panel").show(&ctx, |ui| {
                ui.label("Click to save the world");
                if ui.button("Save").clicked() {
                    let mut ws = WorldSaver::new();
                    match ws.save("assets/world.ron", &world) {
                        Ok(()) => debug!("World saved!"),
                        Err(e) => error!("World not saved: {:?}", e),
                    };
                }
            });
        }
        
    }
}

fn ecs_change(
    world: SubWorld<&mut Transform>,
){
    for (_, mut t) in &mut world.query::<&mut Transform>() {
        t.rotation *= UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0)), 0.05);
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
        material: renderer.create_material(MyMaterial {
            colour: [0.7, 0.0, 0.0]
        }),
        transform: Transform::from_translation(Vector3::new(1.0, 0.0, 0.0)),
    });
    
    cmd.spawn(ModelBundle {
        mesh: Mesh::load_obj("assets/model.obj").swap_remove(0),
        material: renderer.create_material(MyMaterial {
            colour: [0.0, 0.6, 1.0]
        }),
        transform: Transform::from_translation(Vector3::new(-1.0, 0.0, 0.0)),
    });
    
    cmd.spawn(ModelBundle {
        mesh: Mesh::load_obj("assets/model.obj").swap_remove(0),
        material: renderer.create_material(
            DefaultMat::builder()
                .texture_id(txt1)
                .metallic(0.0)
                .roughness(1.0)
                .build(),
        ),
        transform: Transform::from_translation(Vector3::new(1.0, 0.0, -2.0)),
    });
    
    cmd.spawn(ModelBundle {
        mesh: Mesh::load_obj("assets/model.obj").swap_remove(0),
        material: renderer.create_material(TexMaterial {
            texture_id: txt2
        }),
        transform: Transform::from_translation(Vector3::new(-1.0, 0.0, -2.0)),
    });
    
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

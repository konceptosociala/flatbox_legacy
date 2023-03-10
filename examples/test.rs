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
    Despero::init(WindowBuilder::new().with_title("The Game"))
        .add_setup_system(bind_mat)
        .add_setup_system(create_models)
        .add_setup_system(create_camera)        
        .add_system(ecs_change)
        .add_system(egui_handling)
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
    gui_events: Write<EventHandler<GuiContext>>,
){
    if let Some(ctx) = gui_events.read() {
        
        egui::SidePanel::left("my_panel").show(&ctx, |ui| {
            if ui.input().key_pressed(Key::A) {
                error!("`A` is pressed!!!");
            }
            
            ui.label("Click to say hello to the world");
            if ui.button("Hello World!").clicked() {
                //~ let mut ws = WorldSaver::new();
                //~ match ws.save("assets/world.ron", &world) {
                    //~ Ok(()) => debug!("World saved!"),
                    //~ Err(e) => error!("World not saved: {:?}", e),
                //~ };
                debug!("Hello World");
            }
            
        });
        
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
    mut physics_handler: Write<PhysicsHandler>,
){
    let txt1 = renderer.create_texture("assets/uv.jpg", Filter::NEAREST) as u32;
    let txt2 = renderer.create_texture("assets/image.jpg", Filter::LINEAR) as u32;
    
    let mesh = Mesh::load_obj("assets/model.obj").swap_remove(0);
    let mesh_flat = Mesh::load_obj("assets/model_flat.obj").swap_remove(0);

    cmd.spawn(ModelBundle {
        mesh: mesh_flat.clone(),
        material: renderer.create_material(
            DefaultMat::builder()
                .texture_id(txt1)
                .metallic(0.0)
                .roughness(1.0)
                .build(),
        ),
        transform: Transform::from_translation(Vector3::new(1.0, 0.0, -1.0)),
    });
    
    cmd.spawn(ModelBundle {
        mesh: mesh.clone(),
        material: renderer.create_material(
            DefaultMat::builder()
                .texture_id(txt1)
                .metallic(0.0)
                .roughness(1.0)
                .build(),
        ),
        transform: Transform::from_translation(Vector3::new(1.0, 0.0, 1.0)),
    });
    
    let mut phys_builder = EntityBuilder::new();
    phys_builder
        .add_bundle(ModelBundle {
            mesh: mesh.clone(),
            material: renderer.create_material(MyMaterial {
                colour: [0.0, 0.6, 1.0]
            }),
            transform: Transform::from_translation(Vector3::new(-1.0, 2.0, 0.5)),
        })
        .add(physics_handler.new_instance(
            RigidBodyBuilder::dynamic().build(),
            ColliderBuilder::cuboid(0.5, 0.5, 0.5).build(),
        ));
    cmd.spawn(phys_builder.build());
    
    let mut static_builder = EntityBuilder::new();
    static_builder
        .add_bundle(ModelBundle {
            mesh: mesh.clone(),
            material: renderer.create_material(TexMaterial {
                texture_id: txt2
            }),
            transform: Transform::from_translation(Vector3::new(-1.0, -2.0, 0.0)),
        })
        .add(physics_handler.new_instance(
            RigidBodyBuilder::fixed().build(),
            ColliderBuilder::cuboid(0.5, 0.5, 0.5).build(),
        ));
    cmd.spawn(static_builder.build());
    
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

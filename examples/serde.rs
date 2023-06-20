use serde::{Serialize, Deserialize};
use sonja::prelude::*;
use sonja::impl_save_load;

#[derive(Default, Deserialize, Serialize)]
pub struct WorldSaver {
    components: Vec<String>,
}

impl WorldSaver {
    pub fn new() -> Self {
        WorldSaver::default()
    }
}

impl_save_load! {
    loader: WorldSaver, 
    components: [
        Model, 
        Transform, 
        AssetHandle<'M'>,
        Camera
    ]
}

fn main() {
    Sonja::init(WindowBuilder::default())
        .default_systems()
        .add_setup_system(setup)
        .add_system(gui_system)
        .add_system(system)
        .add_system(rotation)
        .run();
}

fn setup(
    mut asset_manager: Write<AssetManager>,
    mut cmd: Write<CommandBuffer>,
) -> SonjaResult<()> {
    let texture_id = asset_manager.create_texture("assets/textures/uv.jpg", Filter::Nearest);    
    let model = Model::new("assets/models/model.obj")?;
    
    cmd.spawn(ModelBundle {
        model,
        material: asset_manager.create_material(
            DefaultMat::builder()
                .albedo(texture_id)
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

    Ok(())
}

fn rotation(model_world: SubWorld<Without<&mut Transform, &Camera>>){
    for (_, mut t) in &mut model_world.query::<Without<&mut Transform, &Camera>>(){
        t.rotation *= UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::new(1.0, 1.0, 1.0)), to_radian(1.0));
    }
}

fn gui_system(
    mut cmd: Write<CommandBuffer>,
    mut asset_manager: Write<AssetManager>,
    mut physics_handler: Write<PhysicsHandler>,
    mut renderer: Write<Renderer>,
    world: Read<World>,
    events: Read<Events>,
) -> SonjaResult<()> {    
    let gui_events = events.get_handler::<GuiContext>().unwrap();
    if let Some(ctx) = gui_events.read() {
        
        gui::SidePanel::left("my_panel").show(&ctx, |ui| {
            ui.label("World (de-)serialization test");
            
            let mut ws = WorldSaver::new();
            
            if ui.button("Save world").clicked() {
                ws.save(
                    &world,
                    &asset_manager,
                    &physics_handler,
                    "assets/saves/world.tar.lz4",
                ).expect("Cannot save world");
            }
            
            if ctx.input().key_pressed(gui::Key::Backspace) {
            // if ui.button("Load world").clicked() {
                let (world, assets, physics) = ws.load("assets/saves/world.tar.lz4")
                    .expect("Cannot load world");

                asset_manager.cleanup(&mut renderer);
                asset_manager.textures = assets.textures;
                asset_manager.audio = assets.audio;

                *physics_handler = physics;
                
                cmd.write(move |w: &mut World| {
                    *w = world;
                    w.spawn((CachedMaterials {
                        materials: assets.materials,
                    },));
                });
            }
        });
    }

    Ok(())
}

fn system(
    cached_world: SubWorld<&mut CachedMaterials>,
    mut asset_manager: Write<AssetManager>,
    mut cmd: Write<CommandBuffer>,
){
    for texture in &asset_manager.textures {
        if !texture.is_generated() {
            debug!("Texture {} is not generated yet!", texture.path.display());
            return;
        }
    }

    for (e, mut cache) in &mut cached_world.query::<&mut CachedMaterials>(){
        debug!("Textures are generated!");
        debug!("Loading cached materials...");
        asset_manager.materials = std::mem::take(&mut cache.materials);
        cmd.despawn(e);
    }
}

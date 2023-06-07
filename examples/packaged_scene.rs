use despero::prelude::*;

fn main() {
    Despero::init(WindowBuilder::default())
    
        .default_systems()
        .add_setup_system(load_scene)        
        .run();
}

fn load_scene(
    mut cmd: Write<CommandBuffer>,
    mut asset_manager: Write<AssetManager>,
) -> DesperoResult<()> {
    let scene = Scene::load_packaged("assets/packages/my_scene.lvl")?;

    cmd.spawn_scene(scene, &mut asset_manager);

    Ok(())
}
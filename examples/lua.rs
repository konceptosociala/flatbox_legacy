use sonja::prelude::*;

fn main() {
    Sonja::init(WindowBuilder::default())
        .default_systems()
        .add_setup_system(setup)
        .add_system(process_script)
        .run();
}

fn setup(
    mut cmd: Write<CommandBuffer>,
){
    cmd.spawn((
        Transform::from_translation(Vector3::new(1.0, 2.0, 3.0)),
        Script::new("assets/scripts/script.lua"),
    ));
}

fn process_script(
    lua: Read<LuaManager>,
    transform_world: SubWorld<(&mut Transform, Added<Transform>)>,
    script_world: SubWorld<&Script>,
){
    for (_, (mut transform, is_added)) in &mut transform_world.query::<(&mut Transform, Added<Transform>)>() {
        if is_added {
            debug!("Transform added");
            let wrapper = LuaPointer::new(&mut *transform);
            lua.set_global("TransformWrapper", wrapper).unwrap();
        }
    }

    for (_, script) in &mut script_world.query::<&Script>() {
        lua.execute(&script).unwrap();
    }
}
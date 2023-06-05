use despero::prelude::*;

fn main() {    
    Despero::init(WindowBuilder::default())
        .default_systems()
        .add_setup_system(create_audio)
        .add_system(rotate)
        .run();
}

fn create_audio(
    mut asset_manager: Write<AssetManager>,
    mut cmd: Write<CommandBuffer>,
) -> DesperoResult<()> {
    let mut sound = Sound::new_from_file("assets/birds.mp3")?;
    let cast = asset_manager.audio.new_cast();
    sound.set_cast(&cast);
    let handle = asset_manager.audio.push_sound(sound);

    asset_manager.audio.play(handle)?;

    let texture_id = asset_manager.create_texture("assets/uv.jpg", Filter::Nearest);    
    let mesh = Mesh::load_obj("assets/model.obj").swap_remove(0);
    
    cmd.spawn((
        mesh,
        asset_manager.create_material(
            DefaultMat::builder()
                .albedo(texture_id)
                .metallic(0.0)
                .roughness(1.0)
                .build(),
        ),
        Transform::default(),
        cast,
    ));

    cmd.spawn((
        Camera::builder()
            .camera_type(CameraType::FirstPerson)
            .is_active(true)
            .build(),
        AudioListener::new(
            &mut asset_manager.audio,
        ),
        Transform::from_translation(Vector3::new(0.0, 0.0, 5.0)),
    ));

    Ok(())
}

pub fn rotate(
    player_world: SubWorld<(&mut Transform, &AudioListener)>,
    object_world: SubWorld<(&mut Transform, &AudioCast)>,
){
    for (_, (mut t, _)) in &mut player_world.query::<(&mut Transform, &AudioListener)>(){
        t.rotation *= UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0)), to_radian(5.0));
    }

    for (_, (mut t, _)) in &mut object_world.query::<(&mut Transform, &AudioCast)>(){
        t.rotation *= UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::new(1.0, 1.0, 1.0)), to_radian(5.0));
    }
}

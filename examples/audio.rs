use sonja::prelude::*;
use sonja::audio_storage;

fn main() {    
    Sonja::init(WindowBuilder::default())
        .default_systems()
        .add_setup_system(create_audio)
        .add_system(rotate)
        .run();
}

fn create_audio(
    mut asset_manager: Write<AssetManager>,
    mut cmd: Write<CommandBuffer>,
) -> SonjaResult<()> {
    let handle = asset_manager.audio.create_sound("assets/audio/birds.mp3")?;
    let texture_id = asset_manager.create_texture("assets/textures/uv.jpg", Filter::Linear);    
    
    cmd.spawn((
        Model::new("assets/models/model.obj")?,
        asset_manager.create_material(
            DefaultMat::builder()
                .albedo(texture_id)
                .metallic(0.0)
                .roughness(1.0)
                .build(),
        ),
        Transform::default(),
        asset_manager.audio.new_cast(),
        audio_storage![handle],
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

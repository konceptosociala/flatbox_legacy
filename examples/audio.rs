use despero::prelude::*;

fn main() {    
    Despero::init(WindowBuilder::default())
        .default_systems()
        .add_setup_system(audio_playback)
        .run();
}

fn audio_playback(
    audio_manager: Read<AudioManager>,
){
    let sound_data = StaticSoundData::from_file("assets/wind.wav", StaticSoundSettings::default()).unwrap();
    audio_manager.inner().play(sound_data).unwrap();
}

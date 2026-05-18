use rodio::source::SineWave;
use rodio::Source;
use std::error::Error;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

pub struct AudioList {
    
}

pub fn play_music(path: PathBuf) {

    std::thread::spawn(move || {

        let stream_handle = rodio::DeviceSinkBuilder::open_default_sink().expect("Not found");
        let mixer = stream_handle.mixer();

        let file = std::fs::File::open(path).expect("Not found");
        let player = rodio::play(mixer, BufReader::new(file)).expect("Error play");
        player.set_volume(0.1);

        player.sleep_until_end();
    });


}
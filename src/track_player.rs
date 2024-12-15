pub struct Player {
    pub track: Option<crate::track_metadata::TrackMetadata>,
    pub position: u32,
    pub volume: f32,
    stream: rodio::OutputStream,
    stream_handle: rodio::OutputStreamHandle,
    sink: Option<rodio::Sink>,
}

impl Default for Player {
    fn default() -> Self {
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        
        Self {
            track: None,
            position: 0,
            volume: 0.5,
            stream,
            stream_handle,
            sink: None,
        }
    }
}

impl Player {
    pub fn play(&mut self, track: crate::track_metadata::TrackMetadata) {
        self.track = Some(track);
        let path = std::path::Path::new(&self.track.as_ref().unwrap().media_segment.uri);
        let file = std::fs::File::open(path).unwrap();
        match self.stream_handle.play_once(file) {
            Ok(s) => {
                self.sink = Some(s);
                self.sink.as_ref().unwrap().set_volume(self.volume);
            }
            Err(e) => {
                eprintln!("Error playing track: {}", e);
            }
        }
    }

    pub fn stop (&mut self) {
        if let Some(s) = self.sink.take() {
            s.stop();
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        if let Some(s) = &self.sink {
            s.set_volume(self.volume);
        }
    }
}

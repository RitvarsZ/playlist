pub struct Player {
    pub track: Option<crate::track_metadata::TrackMetadata>,
    pub position: f32, // 0..1
    pub volume: f32,
    _stream: rodio::OutputStream,
    stream_handle: rodio::OutputStreamHandle,
    sink: Option<rodio::Sink>,
    pub data_dir: std::path::PathBuf,
}

impl Default for Player {
    fn default() -> Self {
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        
        Self {
            track: None,
            position: 0.0,
            volume: 0.5,
            _stream: stream,
            stream_handle,
            sink: None,
            data_dir: std::env::temp_dir().join("playlists"),
        }
    }
}

impl Player {
    pub fn load(&mut self, track: Option<crate::track_metadata::TrackMetadata>) {
        self.stop();
        self.position = 0.0;
        self.track = track;
    }

    pub fn play(&mut self) {
        if self.track.is_none() {
            return;
        }

        if let Some(s) = &self.sink {
            s.play();
            return;
        }

        let path = std::path::Path::new(&self.track.as_ref().unwrap().media_segment.uri);
        let file = std::fs::File::open(path).unwrap();
        match self.stream_handle.play_once(file) {
            Ok(s) => {
                self.sink = Some(s);
                self.sink.as_ref().unwrap().set_volume(self.volume);
                
                let _ = self.sink.as_ref().unwrap().try_seek(
                    std::time::Duration::from_secs_f32(self.position * self.track.as_ref().unwrap().media_segment.duration)
                );
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

    pub fn seek (&mut self, pos: f32) {
        self.position = pos;
        if let Some(s) = &self.sink {
            if s.empty() {
                self.play();
                self.seek(pos);
                return;
            }

            let _ = s.try_seek(std::time::Duration::from_secs_f32(pos * self.track.as_ref().unwrap().media_segment.duration));
        } else {
            self.play();
            self.seek(pos);
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        if let Some(s) = &self.sink {
            s.set_volume(self.volume);
        }
    }

    pub fn update(&mut self, ui: &mut eframe::egui::Ui) {
        if let Some(s) = &self.sink {
            self.position = s.get_pos().as_secs_f32() / self.track.as_ref().unwrap().media_segment.duration;
            ui.ctx().request_repaint();
        }
    }

    // Draw waveform preview.
    pub fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        use eframe::egui;
        if let Some(track) = &self.track {
            let filename = track.id.to_string() + track.title.as_str() + track.artist.as_str() + ".png";
            let path = self.data_dir.join(filename.clone());

            // draw waveform.
            if path.exists() {
                let bytes = std::fs::read(path).unwrap();
                let uri = "bytes://".to_string() + filename.as_str();
                egui::Image::from_bytes(uri, bytes)
                    .sense(egui::Sense::click())
                    .bg_fill(egui::Color32::from_rgb(255, 255, 255))
                    .paint_at(ui, ui.min_rect());
            }

            // Draw playhead.
            let playhead = self.position;
            let playhead_x = ui.min_rect().left() + playhead * ui.min_rect().width();
            ui.set_opacity(0.5);
            ui.painter().rect_filled(
                egui::Rect::from_min_max(
                    [ui.min_rect().left(), ui.min_rect().top()].into(),
                    [playhead_x, ui.min_rect().bottom()].into(),
                ),
                0.0,
                egui::Color32::from_rgb(200, 200, 0),
            );

            let clicked = ui.interact(ui.min_rect(), egui::Id::new("waveform seek"),egui::Sense::click());
            if clicked.clicked() {
                let screen_x = clicked.hover_pos().unwrap().x;
                let x = (screen_x - ui.min_rect().left()) / ui.min_rect().width();
                self.seek(x.max(0.0));
            }
        }
    }
}


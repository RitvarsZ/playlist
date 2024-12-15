use std::{cell::RefCell, rc::Rc};
use crate::track_metadata::TrackMetadata;

/**
*
* Playlist containing parsed csv metadata
* mapped together with a corresponding m3u8 import.
*
*/
pub struct Playlist {
    pub tracks: Vec<TrackMetadata>,
    ui_id: eframe::egui::Id,
    selected_track: Rc<RefCell<Option<TrackMetadata>>>,
}

impl Playlist {
    pub fn new(selected_track: Rc<RefCell<Option<TrackMetadata>>>) -> Self {
        Self {
            tracks: vec![],
            ui_id: eframe::egui::Id::new(rand::random::<u64>()),
            selected_track,
        }
    }

    /**
     * Remove track and return it if contained.
     */
    pub fn maybe_remove_track(&mut self, track_id: u32) -> Option<TrackMetadata> {
        let index = self.tracks.iter().position(|track| track.id == track_id);
        if let Some(index) = index {
            Some(self.tracks.remove(index))
        } else {
            None
        }
    }

    pub fn add_track(&mut self, track: TrackMetadata) {
        self.tracks.push(track);
    }

    /**
     * Import track metadata from csv file and a corresponding m3u8 file.
     */
    pub fn import(&mut self) {
        self.tracks = vec![];
        // Import the playlist from a csv file.
        let file = rfd::FileDialog::new()
            .add_filter("csv", &["txt", "csv"])
            .set_directory("/")
            .pick_file();

        if let Some(file) = file {
            println!("Importing playlist from file: {:?}", file);
            let r = csv::ReaderBuilder::default()
                .has_headers(true)
                .delimiter(b'\t')
                .from_path(file);

            if let Ok(mut reader) = r {
                reader.deserialize().for_each(|result| {
                    if let Ok(track) = result {
                        self.tracks.push(track);
                    } else {
                        println!("Error deserializing track: {:?}", result.err());
                    }
                });
            } else {
                println!("Error reading file: {:?}", r.err());
            }
        }

        // map over tracks
        for (i, track) in self.tracks.iter().enumerate() {
            println!("{}: {}", i, track.title);
        }

        let file = rfd::FileDialog::new()
            .add_filter("m3u8", &["m3u8"])
            .set_directory("/")
            .pick_file();

        if let Some(file) = file {
            println!("Importing playlist from file: {:?}", file);
            let bytes = std::fs::read(file).unwrap();
            match m3u8_rs::parse_playlist(&bytes) {
                Result::Ok((_, m3u8_rs::Playlist::MediaPlaylist(pl))) => {
                    //print playlist
                    println!("{:?}", pl);
                    // map each segment to each track metadata entry.
                    // if lengths differ, throw an error.
                    if pl.segments.len() != self.tracks.len() {
                        println!("{} != {}", pl.segments.len(), self.tracks.len());
                        panic!("Length of segments and tracks differ.");
                    }

                    for (i, segment) in pl.segments.iter().enumerate() {
                        self.tracks[i].media_segment = segment.clone();
                    }
                }
                Result::Err(e) => panic!("Parsing error: \n{}", e),
                _ => println!("Unknown playlist type"),
            }
        }
    }

    /**
     *
     * Export the playlist to a m3u8 file.
     *
     */
    pub fn export(&mut self) {
        let file = rfd::FileDialog::new()
            .add_filter("m3u8", &["m3u8"])
            .set_directory("/")
            .save_file();

        if let Some(file) = file {
            println!("Exporting playlist to file: {:?}", file);
            let mut playlist = m3u8_rs::MediaPlaylist::default();
            println!("p: {:?}", playlist);

            let mut duration = 0.;
            for track in self.tracks.iter() {
                playlist.segments.push(track.media_segment.clone());
                duration += track.media_segment.duration;
            }
            playlist.target_duration = duration as u64;

            let f = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(file);

            match f {
                Ok(mut f) => {
                    match self.write_to(&mut f) {
                        Ok(_) => println!("Playlist written to file."),
                        Err(e) => println!("Error writing playlist: {:?}", e),
                    }
                }
                Err(e) => println!("Error opening file: {:?}", e),
            }

        }
    }

    /** 
    * Partial copy paste from m3u8_rs
    * keeping only things we need.
    */
    pub fn write_to<T: std::io::Write>(&self, w: &mut T) -> std::io::Result<()> {
        writeln!(w, "#EXTM3U")?;

        for track in &self.tracks {
            match m3u8_rs::WRITE_OPT_FLOAT_PRECISION.load(core::sync::atomic::Ordering::Relaxed) {
                core::usize::MAX => {
                    write!(w, "#EXTINF:{},", track.media_segment.duration)?;
                }
                n => {
                    write!(w, "#EXTINF:{:.*},", n, track.media_segment.duration)?;
                }
            };

            if let Some(ref v) = track.media_segment.title {
                writeln!(w, "{}", v)?;
            } else {
                writeln!(w)?;
            }

            writeln!(w, "{}", track.media_segment.uri)?;
        }

        Ok(())
    }

    pub fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        ui.push_id(self.ui_id, |ui| {
            use eframe::egui::{Align, Layout};
            use egui_extras::{Column, TableBuilder};
            ui.style_mut().interaction.selectable_labels = false;

            let mut table = TableBuilder::new(ui)
                .id_salt(self.ui_id)
                .cell_layout(Layout::left_to_right(Align::Center))
                .auto_shrink(false)
                .column(Column::initial(30.)) // #
                .column(
                    // Title
                    Column::initial(125.)
                        .resizable(true)
                        .clip(true)
                        .at_least(30.),
                )
                .column(
                    // Artist
                    Column::initial(125.)
                        .resizable(true)
                        .clip(true)
                        .at_least(30.),
                )
                .column(Column::initial(50.).resizable(false).clip(false)) // BPM
                .column(Column::initial(40.).at_least(40.).resizable(false).clip(false)) // Key
                .column(Column::initial(40.).resizable(false).clip(false)) // Time
                .column(
                    // My Tag
                    Column::initial(70.)
                        .resizable(true)
                        .clip(true)
                        .at_least(30.),
                )
                .column(
                    // Message
                    Column::initial(70.)
                        .resizable(true)
                        .clip(true)
                        .at_least(30.),
                )
                .column(Column::remainder().clip(false).at_least(30.0)) // Path
                .striped(true);

            table = table.sense(eframe::egui::Sense::click());

            table
                .header(30.0, |mut header| {
                    [
                        "#", "Title", "Artist", "BPM", "Key", "Time", "My Tag", "Message", "Path",
                    ]
                    .iter()
                    .for_each(|&label| {
                        header.col(|ui| {
                            ui.label(label);
                        });
                    });
                })
                .body(|body| {
                    body.rows(24.0, self.tracks.len(), |mut row| {
                        let row_index = row.index();
                        let track = &self.tracks[row_index];
                        {
                            self.selected_track.borrow().as_ref().inspect(|t| {
                                row.set_selected(t.id == track.id);
                            });
                        }
                        let columns = [
                            track.id.to_string(),
                            track.title.to_string(),
                            track.artist.to_string(),
                            track.bpm.to_string(),
                            track.key.to_string(),
                            track.time.to_string(),
                            track.my_tag.to_string(),
                            track.message.to_string(),
                            track.media_segment.uri.to_string(),
                        ];

                        for col_data in columns.iter() {
                            row.col(|ui| {
                                if col_data == &track.key {
                                    self.show_key_col(ui, track);
                                }
                                else {
                                    ui.label(col_data);
                                }
                            });
                        }
                        self.toggle_row_selection(track.clone(), &row.response());
                    })
                });
        });
    }

    fn show_key_col(&self, ui: &mut eframe::egui::Ui, track: &TrackMetadata) {
        let label = eframe::egui::widgets::Label::new(&track.key);
        if let Some(selected_track) = self.selected_track.borrow().as_ref() {
            let key_compare = crate::track_metadata::compare_keys(&selected_track.key, &track.key);
            if let Ok(key_compare) = key_compare {
                if let Some(color) = crate::track_metadata::color_from_key_compare(key_compare) {
                    eframe::egui::Frame::default().fill(color).show(ui, |ui| {
                        ui.set_width(30.);
                        ui.colored_label(eframe::egui::Color32::from_rgb(255, 255, 255), label.text());
                    });
                } else {
                    ui.add(label);
                }
            }
        } else {
            ui.add(label);
        }
    }

    fn toggle_row_selection(&mut self, track: TrackMetadata, row_response: &eframe::egui::Response) {
        if row_response.clicked() {
            let mut selected = self.selected_track.borrow_mut();
            if let Some(selected_track) = selected.as_ref() {
                if selected_track.id == track.id {
                    selected.take();
                } else {
                    selected.replace(track);
                }
            } else {
                selected.replace(track);
            }
        }
    }
}

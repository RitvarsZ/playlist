#![windows_subsystem = "windows"]
use std::{cell::RefCell, rc::Rc};
use std::thread;
use eframe::egui;

mod playlist;
mod track_metadata;
mod track_player;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Playlists",
        options,
        Box::new(|_cc| {
            Ok(Box::<App>::default())
        })
    )
}

struct App {
    import_table: playlist::Playlist,
    export_table: playlist::Playlist,
    selected_track: Rc<RefCell<Option<track_metadata::TrackMetadata>>>,
    player: track_player::Player,
}

impl Default for App {
    fn default() -> Self {
        let selected_track = Rc::new(RefCell::new(None));

        Self {
            import_table: playlist::Playlist::new(selected_track.clone()),
            export_table: playlist::Playlist::new(selected_track.clone()),
            selected_track,
            player: track_player::Player::default(),
        }
    }
}

impl App {
    fn update_player(&mut self) {
        let selected_track = self.selected_track.borrow().clone();
        let player_track = self.player.track.clone();

        if selected_track.is_none() && player_track.is_some() {
            self.player.load(None);
        } else if selected_track.as_ref().is_some_and(|t| t.id != player_track.map_or(0, |t| t.id)) {
            self.player.load(selected_track);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);

        self.update_player();
        let mut import_clicked = false;
        let mut export_clicked = false;
        let mut play_clicked = false;
        let mut stop_clicked = false;

        egui::TopBottomPanel::top("top panel").min_height(30.0).show(ctx, |ui| {
            use egui_extras::{Size, StripBuilder};
            StripBuilder::new(ui)
                .size(Size::initial(20.))
                .size(Size::initial(60.))
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        ui.horizontal_centered(|ui| {
                            import_clicked = ui.button("Import Table").clicked();
                            export_clicked = ui.button("Export Table").clicked();
                            play_clicked = ui.button("Play").clicked();
                            stop_clicked = ui.button("Stop").clicked();

                            ui.add(egui::Slider::from_get_set(
                                0.0..=1.0,
                                |volume| {
                                    if let Some(volume) = volume {
                                        self.player.set_volume(volume as f32)
                                    }

                                    self.player.volume as f64
                                }
                            ));
                        });
                    });

                    strip.cell(|ui| {
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(0,0,0))
                            .inner_margin(egui::Margin::same(10.))
                            .show(ui, |ui| {
                                ui.set_height(60.);
                                ui.set_width(ui.available_width());
                                self.player.update(ui);
                                self.player.ui(ui);
                            });
                    });
                });

        });
        egui::CentralPanel::default().show(ctx, |ui| {
            use egui_extras::{Size, StripBuilder};
            StripBuilder::new(ui)
                .size(Size::remainder().at_least(100.0)) // for the import table
                .size(Size::initial(30.)) // for the separator
                .size(Size::remainder().at_least(100.0)) // for the export table
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(0,0,0))
                            .inner_margin(egui::Margin::same(10.))
                            .show(ui, |ui| {
                                egui::ScrollArea::horizontal().show(ui, |ui| {
                                    self.import_table.ui(ui);
                                });
                            });
                    });
                    strip.cell(|ui| {
                        ui.separator();
                        ui.horizontal_centered(|ui| {
                            ui.label("Export:");
                        });
                        ui.separator();
                    });
                    strip.cell(|ui| {
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(0,0,0))
                            .inner_margin(egui::Margin::same(10.))
                            .show(ui, |ui| {
                                egui::ScrollArea::horizontal().show(ui, |ui| {
                                    self.export_table.ui(ui);
                                });
                            });
                    });
                });
        });

        if import_clicked {
            self.import_table.import();
            // spawn threads to generate previews.
            let tracks = std::sync::Arc::new(self.import_table.tracks.clone());

            let handles: Vec<_> = tracks.iter().map(|track| {
                let track = track.clone();
                println!("generating preview for: {}", track.title);
                thread::spawn(move || {
                    let datadir = std::env::temp_dir().join("playlists");
                    std::fs::create_dir_all(&datadir).unwrap();
                    let filename = track.id.to_string() + track.title.as_str() + track.artist.as_str() + ".png";

                    if datadir.join(filename.as_str()).exists() {
                        return;
                    }

                    let path = std::path::Path::new(&track.media_segment.uri);
                    let decoder = rodio::Decoder::new(std::fs::File::open(path).unwrap());

                    audio_visualizer::waveform::png_file::waveform_static_png_visualize(
                        &decoder.unwrap().collect::<Vec<i16>>(),
                        audio_visualizer::Channels::Mono,
                        datadir.to_str().unwrap(),
                        filename.as_str(),
                    );
                })
            }).collect();

            for handle in handles {
                if let Err(e) = handle.join() { eprintln!("Thread encountered an error: {:?}", e) }
            }
        }

        if export_clicked {
            self.export_table.export();
        }

        let arrow_down = ctx.input(|i| i.clone().consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
        let arrow_up = ctx.input(|i| i.clone().consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));
        if arrow_down {
            if let Some(t) = self.selected_track.borrow().as_ref() {
                self.import_table.maybe_move_track_down(t);
                self.export_table.maybe_move_track_down(t);
            }
        } else if arrow_up {
            if let Some(t) = self.selected_track.borrow().as_ref() {
                self.import_table.maybe_move_track_up(t);
                self.export_table.maybe_move_track_up(t);
            } 
        }
        

        let space = ctx.input(|i| i.clone().consume_key(egui::Modifiers::NONE, egui::Key::Space));
        if space {
            if let Some(t) = self.selected_track.borrow().as_ref() {
                if let Some(track) = self.import_table.maybe_remove_track(t.id) {
                    self.export_table.add_track(track);
                } else if let Some(track) = self.export_table.maybe_remove_track(t.id) {
                    self.import_table.add_track(track);
                }
            }
        
        }

        if stop_clicked {
            self.player.stop()
        }

        if play_clicked {
            self.player.play()
        }
    }
}


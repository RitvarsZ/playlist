use std::{cell::RefCell, rc::Rc};

use eframe::egui;

mod playlist;
mod track_metadata;

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

/** 
*
* Plalists app is an app for reordering recordbox playlists.
* 1. Import a csv file with song information (title, artists, key, bpm)
* 2. Imported songs show up in a table.
* 3. Imported songs can be dragged to another table, called the export table.
* 4. Songs in both tables can be previewed.
* 5. When selecting a song on the export table - songs that match bpm and key
*    show up on the import table (indicating that they are a good match to mix).
* 6. Export table can be exported back to csv.
*
* */
struct App {
    import_table: playlist::Playlist,
    export_table: playlist::Playlist,
    selected_track: Rc<RefCell<Option<track_metadata::TrackMetadata>>>,
}

impl Default for App {
    fn default() -> Self {
        let selected_track = Rc::new(RefCell::new(None));

        Self {
            import_table: playlist::Playlist::new(selected_track.clone()),
            export_table: playlist::Playlist::new(selected_track.clone()),
            selected_track,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut import_clicked = false;
        let mut export_clicked = false;
        egui::TopBottomPanel::top("top panel").min_height(30.0).show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                import_clicked = ui.button("Import Table").clicked();
                export_clicked = ui.button("Export Table").clicked();
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
    }
}


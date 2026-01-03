use super::Args;

use cat_nipa::{Game, NpaEntry, parse_head, read_entries, read_entry_data};
use color_eyre::Result;
use egui::{FontData, FontDefinitions, FontFamily};
use egui_ltreeview::{Action, Activate};
use std::path::{Component, PathBuf};

pub fn run(args: Args) -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 240.0])
            .with_min_inner_size([320.0, 240.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "Cat Nipa",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            let mut app = App {
                file: args.file,
                game: Some(args.game),
                entries: Vec::new(),
            };

            let mut fonts = FontDefinitions::default();

            fonts.font_data.insert(
                "Pretendard".to_owned(),
                std::sync::Arc::new(FontData::from_static(include_bytes!(
                    "../../assets/fonts/PretendardJP-Regular.otf"
                ))),
            );

            fonts
                .families
                .get_mut(&FontFamily::Proportional)
                .unwrap()
                .push("Pretendard".to_owned());

            fonts
                .families
                .get_mut(&FontFamily::Monospace)
                .unwrap()
                .push("Pretendard".to_owned());

            cc.egui_ctx.set_fonts(fonts);

            if let Some(file_path) = &app.file {
                let file = std::fs::File::open(file_path).unwrap();
                let mut reader = std::io::BufReader::new(file);

                let header = parse_head(&mut reader).unwrap();

                app.entries = read_entries(
                    &mut reader,
                    &header,
                    app.game.unwrap() == Game::LamentoTrail || app.game.unwrap() == Game::Lamento,
                )
                .unwrap();

                app.entries.sort_by_key(|b| b.file_path.clone());

                log::info!("{:#?}", app.entries);
            }

            Ok(Box::new(app))
        }),
    )
    .expect("Failed to run app");

    Ok(())
}

#[derive(Default)]
struct App {
    file: Option<PathBuf>,
    game: Option<Game>,
    entries: Vec<NpaEntry>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.heading("Cat Nipa");

            if let Some(file_path) = &self.file {
                ui.label(file_path.to_string_lossy().to_string());
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                let (_, actions) = egui_ltreeview::TreeView::new("entries".into())
                    .allow_multi_selection(false)
                    .show(ui, |builder| {
                        let mut last_level = 0;
                        for (index, entry) in self.entries.iter().enumerate() {
                            let level = entry
                                .file_path
                                .parent()
                                .map(|parent| {
                                    parent
                                        .components()
                                        .filter(|component| {
                                            matches!(component, Component::Normal(_))
                                        })
                                        .count()
                                })
                                .unwrap_or(0);

                            if level < last_level {
                                for _ in 0..(last_level - level) {
                                    builder.close_dir();
                                }
                            }

                            last_level = level;

                            if entry.is_directory() {
                                builder
                                    .dir(index, format!("🗀 {}", entry.file_path.to_string_lossy()));
                            } else {
                                builder.leaf(
                                    index,
                                    entry
                                        .file_path
                                        .file_name()
                                        .unwrap()
                                        .to_string_lossy()
                                        .to_string(),
                                );
                            }
                        }
                    });

                for action in actions {
                    if let Action::Activate(Activate { selected, .. }) = action {
                        if let Some(entry) = self.entries.get(*selected.first().unwrap()) {
                            println!("Selected: {}", entry.file_path.to_string_lossy());
                        }
                    }
                }
            });
        });

        ctx.request_repaint();
    }
}

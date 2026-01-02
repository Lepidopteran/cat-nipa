use std::{
    fs,
    ops::Deref,
    path::{Component, Path, PathBuf},
};

use cat_nipa::{Game, NpaEntry, parse_head, read_entries, read_entry_data};
use clap::Parser;
use color_eyre::Result;
use egui_ltreeview::{Action, Activate};
use indicatif::ProgressBar;
use log::debug;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Input File
    #[arg(value_name = "FILE", short = 'i', long = "input")]
    file: Option<PathBuf>,

    /// Which Game
    #[arg(value_enum, short, long, default_value_t = Game::ChaosHead)]
    game: Game,

    /// Output Directory
    #[arg(short, long = "output", value_name = "DIR")]
    output_dir: Option<PathBuf>,

    /// Run in GUI mode
    #[arg(long = "gui", default_value_t = false)]
    run_gui: bool,
}

pub fn run() -> Result<()> {
    let args = Args::parse();

    if args.run_gui || args.file.is_none() {
        run_gui(args)
    } else {
        run_cli(args)
    }
}

fn run_cli(args: Args) -> Result<()> {
    let file_path = args.file.expect("No input file specified");
    let file = std::fs::File::open(&file_path).unwrap();
    let mut reader = std::io::BufReader::new(file);

    let header = parse_head(&mut reader).unwrap();

    let entries = read_entries(
        &mut reader,
        &header,
        args.game == Game::LamentoTrail || args.game == Game::Lamento,
    )
    .unwrap();

    let output_directory = args
        .output_dir
        .unwrap_or_else(|| PathBuf::from(file_path.file_stem().expect("Input file has no stem")));

    if !output_directory.exists() {
        fs::create_dir(&output_directory).unwrap();
    }

    let progress_bar = ProgressBar::new(header.total_count as u64);

    for entry in entries {
        let file_name = entry.file_path.to_string_lossy().to_string();
        let path = output_directory.join(file_name);

        if entry.is_directory() {
            fs::create_dir_all(path).unwrap();
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("Failed to create parent directory");
            }

            fs::write(
                path,
                read_entry_data(&mut reader, &header, &entry, args.game).unwrap(),
            )
            .unwrap();
        }

        progress_bar.inc(1);
    }

    progress_bar.finish();

    Ok(())
}

fn run_gui(args: Args) -> Result<()> {
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
                                builder.dir(index, format!("🗀 {}", entry.file_path.to_string_lossy()));
                            } else {
                                builder.leaf(index, entry.file_path.file_name().unwrap().to_string_lossy().to_string());
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

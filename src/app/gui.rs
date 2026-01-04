use super::Args;

use cat_nipa::{Game, NpaEntry, parse_head, read_entries, read_entry, read_entry_data};
use color_eyre::{Result, eyre::eyre};
use egui::{ColorImage, FontData, FontDefinitions, FontFamily, TextureHandle, ahash::HashMap};
use egui_ltreeview::Action;
use log::debug;
use std::{
    fs::File,
    ops::Not,
    path::{Component, Path, PathBuf},
};
use strum::IntoEnumIterator;

mod image_viewer;

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

            Ok(Box::new(if let Some(file_path) = args.file {
                App::new(file_path, Some(args.game))
            } else {
                App::new_without_file(Some(args.game))
            }))
        }),
    )
    .expect("Failed to run app");

    Ok(())
}

#[derive(Default)]
struct App {
    file_path: Option<PathBuf>,
    game: Option<Game>,
    entries: Vec<NpaEntry>,
    selected: Vec<usize>,
    cached_data: HashMap<usize, Vec<u8>>,
    cached_textures: HashMap<usize, TextureHandle>,
    auto_select_game_on_failure: bool,
}

impl App {
    pub fn new(file_path: PathBuf, game: Option<Game>) -> Self {
        let mut temp_reader = File::open(&file_path).expect("Failed to open file");
        let temp_header = parse_head(&mut temp_reader).expect("Failed to parse head");

        let add_encrypted_bytes = (0..temp_header.total_count)
            .find_map(|index| {
                let entry = read_entry(&mut temp_reader, index as usize, &temp_header, true);

                entry
                    .ok()
                    .and_then(|entry| entry.is_directory().not().then_some(entry))
            })
            .expect("No file entry could be found")
            .file_path
            .extension()
            .is_some();

        let mut file = File::open(&file_path).expect("Failed to open file");
        let header = parse_head(&mut file).expect("Failed to parse head");

        let mut entries =
            read_entries(&mut file, &header, add_encrypted_bytes).expect("Failed to read entries");

        entries.sort_by_key(|b| b.file_path.clone());

        Self {
            game,
            entries,
            file_path: Some(file_path),
            auto_select_game_on_failure: true,
            ..Default::default()
        }
    }

    pub fn new_without_file(game: Option<Game>) -> Self {
        Self {
            game,
            auto_select_game_on_failure: true,
            ..Default::default()
        }
    }

    pub fn cache_data(&mut self, key: usize, data: Vec<u8>, invalidate: bool) {
        if self.cached_data.contains_key(&key) && !invalidate {
            return;
        }

        self.cached_data.insert(key, data);
    }

    pub fn get_texture(&mut self, key: usize, context: &egui::Context) -> Result<&TextureHandle> {
        if !self.cached_textures.contains_key(&key) {
            self.load_texture(key, context)?;
        }

        Ok(self
            .cached_textures
            .get(&key)
            .expect("Texture was just loaded"))
    }

    pub fn load_texture(&mut self, key: usize, context: &egui::Context) -> Result<&TextureHandle> {
        let data = self
            .cached_data
            .get(&key)
            .ok_or_else(|| eyre!("Data not cached"))?;

        let entry = self
            .entries
            .get(key)
            .ok_or_else(|| eyre!("Entry not found"))?;

        if !infer::is_image(data) {
            return Err(eyre!("Data is not an image"));
        }

        let image = image::load_from_memory(data)?;
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();

        let texture = context.load_texture(
            entry.file_path.display().to_string(),
            egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
            egui::TextureOptions::NEAREST,
        );

        self.cached_textures.insert(key, texture);

        Ok(self
            .cached_textures
            .get(&key)
            .expect("Failed to get texture"))
    }

    pub fn auto_select_game(&mut self, entry_index: usize) -> Result<()> {
        if !self.auto_select_game_on_failure {
            return Ok(());
        }

        let entry = self
            .entries
            .get(entry_index)
            .cloned()
            .ok_or_else(|| eyre!("Selected entry does not exist"))?;

        if entry.is_directory() {
            return Err(eyre!("Selected entry is a directory"));
        }

        let path = self
            .file_path
            .as_ref()
            .ok_or_else(|| eyre!("No file path"))
            .cloned()?;

        let mut file = File::open(path).expect("Failed to open file");
        let header = parse_head(&mut file).expect("Failed to parse head");

        let extension = entry
            .file_path
            .extension()
            .map(|ext| ext.to_string_lossy().to_string())
            .ok_or_else(|| eyre!("Selected entry has no extension"))?;

        Game::iter()
            .any(|game| {
                debug!("Trying {game:?} to auto-select game");

                let data = read_entry_data(&mut file, &header, &entry, game);
                let valid = data.as_ref().is_ok_and(|data| {
                    if infer::is_supported(extension.as_str()) {
                        infer::is(data, extension.as_str())
                    } else {
                        std::str::from_utf8(data).is_ok()
                    }
                });

                if valid {
                    self.cache_data(
                        entry_index,
                        data.expect("Failed reading data, This should never happen!"),
                        false,
                    );

                    self.game.replace(game);
                }

                valid
            })
            .then_some(())
            .ok_or_else(|| eyre!("No decryption key could be found that could decode the file"))
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.heading("Cat Nipa");

            if let Some(file_path) = &self.file_path.clone() {
                ui.label(file_path.to_string_lossy().to_string());
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
                                    builder.dir(
                                        index,
                                        format!(
                                            "🗀 {}",
                                            entry.file_path.file_name().unwrap().to_string_lossy()
                                        ),
                                    );
                                } else {
                                    builder.leaf(
                                        index,
                                        format!(
                                            "{} {}",
                                            match entry
                                                .file_path
                                                .extension()
                                                .map(|extension| extension
                                                    .to_string_lossy()
                                                    .to_string())
                                                .unwrap_or("".to_string())
                                                .as_str()
                                            {
                                                "png" | "jpeg" | "jpg" | "gif" | "bmp" | "webp" =>
                                                    "🖻",
                                                "wav" | "ogg" | "flac" | "m4a" | "mp3" => "♫",
                                                "mp4" | "mkv" | "webm" | "mov" | "avi" | "wmv"
                                                | "flv" | "ngs" => "🎞",
                                                _ => "🗎",
                                            },
                                            entry.file_path.file_name().unwrap().to_string_lossy()
                                        ),
                                    );
                                }
                            }
                        });

                    for action in actions {
                        if let Action::SetSelected(selected) = action {
                            debug!("Selected: {selected:?}");

                            for index in &selected {
                                let entry = self
                                    .entries
                                    .get(*index)
                                    .cloned()
                                    .expect("Failed to get entry");

                                debug!("Entry: {entry:?}");

                                if entry.is_directory() || self.cached_data.contains_key(index) {
                                    continue;
                                }

                                let mut file = File::open(file_path).expect("Failed to open file");
                                let head = parse_head(&mut file).expect("Failed to parse head");

                                let data =
                                    read_entry_data(&mut file, &head, &entry, self.game.unwrap());

                                let valid = data.as_ref().is_ok_and(|data| {
                                    is_valid_data(
                                        data,
                                        &entry.file_path.extension().unwrap().to_string_lossy(),
                                    )
                                });

                                if valid {
                                    self.cache_data(*index, data.unwrap(), false);
                                } else if self.auto_select_game_on_failure {
                                    log::warn!(
                                        "Entry data for {:?} is invalid, auto-selecting game...",
                                        entry.file_path.to_string_lossy()
                                    );

                                    let _ = self.auto_select_game(*index).map_err(|err| {
                                        log::error!("Failed to auto-select game: {}", err);
                                    });
                                } else {
                                    log::error!("Entry data for {:?} is invalid", entry.file_path);
                                }
                            }

                            self.selected = selected;
                        }
                    }
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(entry_index) = self.selected.first().cloned()
                && let Ok(texture) = self.get_texture(entry_index, ctx)
            {
                image_viewer::image_viewer(ui.make_persistent_id(format!("image-viewer-{entry_index}")), ui, texture);
            }
        });

        ctx.request_repaint();
    }
}

pub fn is_valid_data(data: &[u8], extension: &str) -> bool {
    if infer::is_supported(extension) {
        infer::is(data, extension)
    } else {
        std::str::from_utf8(data).is_ok()
    }
}

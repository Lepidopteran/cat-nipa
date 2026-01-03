use std::{fs, path::PathBuf};

use super::Args;
use cat_nipa::{Game, parse_head, read_entries, read_entry_data};
use color_eyre::Result;
use indicatif::ProgressBar;

pub fn run(args: Args) -> Result<()> {
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

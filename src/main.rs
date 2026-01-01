use std::{
    fs,
    path::{Path, PathBuf},
};

use cat_nipa::{Game, parse_head, read_entries, read_entry_data};
use clap::Parser;
use indicatif::ProgressBar;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    file: PathBuf,

    #[arg(value_enum, short, long)]
    game: Game,

    #[arg(short, long = "output")]
    output_dir: Option<PathBuf>,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    let file = std::fs::File::open(&args.file).unwrap();
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
        .unwrap_or_else(|| PathBuf::from(args.file.file_stem().expect("input file has no stem")));

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
}

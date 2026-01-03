use std::path::PathBuf;

use cat_nipa::Game;
use clap::Parser;
use color_eyre::Result;

mod cli;
mod gui;

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
        gui::run(args)
    } else {
        cli::run(args)
    }
}

use color_eyre::Result;

mod app;

fn main() -> Result<()> {
    env_logger::init();
    color_eyre::install().ok();

    app::run()
}

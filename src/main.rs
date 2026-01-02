use color_eyre::Result;

mod app;

fn main() -> Result<()> {
    env_logger::init();

    app::run()
}

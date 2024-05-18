use std::time::Duration;
use std::{io, thread};

use clap::Parser;
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use log::LevelFilter;
use servicepoint2::Connection;

use crate::app::App;

mod app;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, default_value = "localhost:2342")]
    destination: String,
    #[arg(short, long, default_value_t = 0.5f64)]
    probability: f64,
}

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let cli = Cli::parse();
    let connection = Connection::open(&cli.destination)
        .expect("Could not connect. Did you forget `--destination`?");

    let entered_alternate = execute!(io::stdout(), EnterAlternateScreen).is_ok();

    crossterm::terminal::enable_raw_mode().expect("could not enable raw terminal mode");

    let mut app = App::new(connection, &cli);
    while app.step() {
        thread::sleep(Duration::from_millis(30));
    }

    crossterm::terminal::disable_raw_mode().expect("could not disable raw terminal mode");

    if entered_alternate {
        execute!(io::stdout(), LeaveAlternateScreen).expect("could not leave alternate screen");
    }
}

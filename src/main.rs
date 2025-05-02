use crate::app::App;
use clap::Parser;

mod app;
mod game;
mod print;
mod rules;
mod simulation;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, default_value = "172.23.42.29:2342")]
    destination: String,
}

fn main() {
    let mut app = App::new(Cli::parse());
    while !app.terminated() {
        app.run_iteration();
    }
}

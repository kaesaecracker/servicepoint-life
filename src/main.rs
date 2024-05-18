use std::io::stdout;
use std::thread;
use std::time::Duration;

use clap::Parser;
use crossterm::{event, execute, queue};
use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::style::{Print, PrintStyledContent, Stylize};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen};
use log::LevelFilter;
use rand::{distributions, Rng};
use servicepoint2::{
    ByteGrid, CompressionCode, Connection, Origin, PIXEL_WIDTH, PixelGrid, TILE_HEIGHT, TILE_WIDTH,
};
use servicepoint2::Command::{BitmapLinearWin, CharBrightness};

use crate::game::Game;

mod game;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, default_value = "localhost:2342")]
    destination: String,
    #[arg(short, long, default_value_t = 0.4f64)]
    probability: f64,
    #[arg(short, long, default_value_t = true)]
    extended: bool,
}

// TODO: itsa spaghetti! 👌
fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let cli = Cli::parse();
    let connection = Connection::open(&cli.destination)
        .expect("Could not connect. Did you forget `--destination`?");

    let entered_alternate = execute!(stdout(), EnterAlternateScreen, EnableLineWrap).is_ok();

    enable_raw_mode().expect("could not enable raw terminal mode");

    let mut left = Game::default();
    let mut right = Game::default();

    let mut pixels = PixelGrid::max_sized();
    let mut luma = ByteGrid::new(TILE_WIDTH as usize, TILE_HEIGHT as usize);

    let mut split_pixel = PIXEL_WIDTH as usize / 2;

    let mut close_requested = false;
    while !close_requested {
        left.step();
        right.step();

        for x in 0..pixels.width() {
            let left_or_right = if x < split_pixel {
                &left.field
            } else {
                &right.field
            };
            for y in 0..pixels.height() {
                let set = left_or_right.get(x, y) || x == split_pixel;
                pixels.set(x, y, set);
            }
        }

        let split_tile = split_pixel / 8;
        for x in 0..luma.width() {
            let left_or_right = if x < split_tile {
                &left.luma
            } else {
                &right.luma
            };
            for y in 0..luma.height() {
                let set = if x == split_tile {
                    255
                } else {
                    left_or_right.get(x, y)
                };
                luma.set(x, y, set);
            }
        }

        let pixel_cmd =
            BitmapLinearWin(Origin(0, 0), pixels.clone(), CompressionCode::Uncompressed);
        connection
            .send(pixel_cmd.into())
            .expect("could not send pixels");

        connection
            .send(CharBrightness(Origin(0, 0), luma.clone()).into())
            .expect("could not send brightness");

        while event::poll(Duration::from_secs(0)).expect("could not poll") {
            match event::read().expect("could not read event") {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char('h') => {
                            println_info("[h] help");
                            println_info("[q] quit");
                            println_info("[a] reset left field");
                            println_info("[d] reset right field");
                            println_info("[→] move divider right");
                            println_info("[←] move divider left");
                        }
                        KeyCode::Char('q') => {
                            println_warning("terminating");
                            close_requested = true;
                        }
                        KeyCode::Char('a') => {
                            println_debug("generating new random field for left");
                            left = make_random_field(cli.probability);
                        }
                        KeyCode::Char('d') => {
                            println_info("generating new random field for right");
                            right = make_random_field(cli.probability);
                        }
                        KeyCode::Right => {
                            split_pixel += 1;
                        }
                        KeyCode::Left => {
                            split_pixel -= 1;
                        }
                        key_code => {
                            println_debug(format!("unhandled KeyCode {key_code:?}"));
                        }
                    }
                }
                event => {
                    println_debug(format!("unhandled event {event:?}"));
                }
            }
        }

        thread::sleep(Duration::from_millis(30));
    }

    disable_raw_mode().expect("could not disable raw terminal mode");

    if entered_alternate {
        execute!(stdout(), LeaveAlternateScreen).expect("could not leave alternate screen");
    }
}

fn println_info(text: impl Into<String>) {
    println_command(PrintStyledContent(text.into().white()))
}

fn println_debug(text: impl Into<String>) {
    println_command(PrintStyledContent(text.into().grey()))
}

fn println_warning(text: impl Into<String>) {
    println_command(PrintStyledContent(text.into().red()))
}

fn println_command(command: impl crossterm::Command) {
    queue!(stdout(), command, Print("\r\n"));
}

fn make_random_field(probability: f64) -> Game {
    let mut field = PixelGrid::max_sized();
    let mut rng = rand::thread_rng();
    let d = distributions::Bernoulli::new(probability).unwrap();
    for x in 0..field.width() {
        for y in 0..field.height() {
            field.set(x, y, rng.sample(d));
        }
    }

    let mut luma = ByteGrid::new(TILE_WIDTH as usize, TILE_HEIGHT as usize);
    for x in 0..luma.width() {
        for y in 0..luma.height() {
            luma.set(x, y, rng.gen());
        }
    }

    Game { field, luma }
}

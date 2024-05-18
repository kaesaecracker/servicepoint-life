use std::io::stdout;
use std::time::Duration;
use std::{io, thread};

use clap::Parser;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::style::{Print, PrintStyledContent, Stylize};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{event, execute};
use log::LevelFilter;
use rand::{distributions, Rng};
use servicepoint2::Command::{BitmapLinearWin, CharBrightness};
use servicepoint2::{
    ByteGrid, CompressionCode, Connection, Origin, PixelGrid, PIXEL_WIDTH, TILE_HEIGHT, TILE_WIDTH,
};

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

    let entered_alternate = execute!(io::stdout(), EnterAlternateScreen).is_ok();

    crossterm::terminal::enable_raw_mode().expect("could not enable raw terminal mode");

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
                            execute!(
                                stdout(),
                                Print("h for help\r\n"),
                                Print("q to quit\r\n"),
                                Print("a to reset left field\r\n"),
                                Print("d to reset right field\r\n")
                            )
                            .unwrap();
                        }
                        KeyCode::Char('q') => {
                            execute!(stdout(), PrintStyledContent("terminating\r\n".red()))
                                .unwrap();
                            close_requested = true;
                        }
                        KeyCode::Char('a') => {
                            execute!(
                                stdout(),
                                PrintStyledContent(
                                    "generating new random field for left\r\n".grey()
                                )
                            )
                            .unwrap();
                            left = make_random_field(cli.probability);
                        }
                        KeyCode::Char('d') => {
                            execute!(
                                stdout(),
                                PrintStyledContent(
                                    "generating new random field for right\r\n".grey()
                                )
                            )
                            .unwrap();
                            right = make_random_field(cli.probability);
                        }
                        KeyCode::Right => {
                            split_pixel += 1;
                        }
                        KeyCode::Left => {
                            split_pixel -= 1;
                        }
                        key_code => {
                            execute!(
                                stdout(),
                                PrintStyledContent(
                                    format!("unhandled KeyCode {key_code:?}\r\n").dark_grey()
                                )
                            )
                            .unwrap();
                        }
                    }
                }
                event => {
                    execute!(
                        stdout(),
                        PrintStyledContent(format!("unhandled event {event:?}\r\n").dark_grey()),
                    )
                    .unwrap();
                }
            }
        }

        thread::sleep(Duration::from_millis(30));
    }

    crossterm::terminal::disable_raw_mode().expect("could not disable raw terminal mode");

    if entered_alternate {
        execute!(stdout(), LeaveAlternateScreen).expect("could not leave alternate screen");
    }
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

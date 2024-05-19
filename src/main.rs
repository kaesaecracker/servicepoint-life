use std::io::stdout;
use std::num::Wrapping;
use std::thread;
use std::time::{Duration, Instant};

use clap::Parser;
use crossterm::{event, execute};
use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen,
};
use log::LevelFilter;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use servicepoint2::{ByteGrid, CompressionCode, Connection, FRAME_PACING, Grid, Origin, PIXEL_WIDTH, PixelGrid, TILE_HEIGHT, TILE_WIDTH};
use servicepoint2::Command::{BitmapLinearWin, CharBrightness};

use crate::game::Game;
use crate::print::{println_debug, println_info, println_warning};
use crate::rules::Rules;

mod game;
mod rules;
mod print;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, default_value = "localhost:2342")]
    destination: String,
}

fn main() {
    let connection = init();

    let mut left_pixels = Game {
        rules: Rules::day_and_night(),
        field: PixelGrid::max_sized(),
    };
    let mut right_pixels = Game {
        rules: Rules::seeds(),
        field: PixelGrid::max_sized(),
    };
    let mut left_luma = Game {
        rules: Rules::continuous_game_of_life(),
        field: ByteGrid::new(TILE_WIDTH, TILE_HEIGHT),
    };
    let mut right_luma = Game {
        rules: Rules::continuous_game_of_life(),
        field: ByteGrid::new(TILE_WIDTH, TILE_HEIGHT),
    };

    randomize(&mut left_luma.field);
    randomize(&mut left_pixels.field);
    randomize(&mut right_luma.field);
    randomize(&mut right_pixels.field);

    let mut pixels = PixelGrid::max_sized();
    let mut luma = ByteGrid::new(TILE_WIDTH, TILE_HEIGHT);

    let mut split_pixel = PIXEL_WIDTH / 2;
    let mut split_speed = 1;

    let mut iteration = Wrapping(0u8);

    loop {
        let start = Instant::now();

        left_pixels.step();
        right_pixels.step();

        left_luma.step();
        right_luma.step();

        iteration += Wrapping(1u8);
        split_pixel = usize::clamp(split_pixel + split_speed, 0, pixels.width() - 1);

        draw_pixels(&mut pixels, &left_pixels.field, &right_pixels.field, split_pixel);
        draw_luma(&mut luma, &left_luma.field, &right_luma.field, split_pixel / 8);
        send_to_screen(&connection, &pixels, &luma);

        while event::poll(Duration::from_secs(0)).expect("could not poll") {
            match parse_event(event::read().expect("could not read event")) {
                AppEvent::None => {}
                AppEvent::RandomizeLeftPixels => {
                    randomize(&mut left_pixels.field);
                }
                AppEvent::RandomizeRightPixels => {
                    randomize(&mut right_pixels.field);
                }
                AppEvent::RandomizeLeftLuma => {
                    randomize(&mut left_luma.field);
                }
                AppEvent::RandomizeRightLuma => {
                    randomize(&mut right_luma.field);
                }
                AppEvent::Accelerate => {
                    split_speed += 1;
                }
                AppEvent::Decelerate => {
                    split_speed -= 1;
                }
                AppEvent::Close => {
                    de_init();
                    return;
                }
            }
        }


        let tick_time = start.elapsed();
        if tick_time < FRAME_PACING {
            thread::sleep(FRAME_PACING - tick_time);
        }
    }
}

enum AppEvent {
    None,
    Close,
    RandomizeLeftPixels,
    RandomizeRightPixels,
    RandomizeLeftLuma,
    RandomizeRightLuma,
    Accelerate,
    Decelerate,
}

fn parse_event(event: Event) -> AppEvent {
    match event {
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            match key_event.code {
                KeyCode::Char('h') => {
                    println_info("[h] help");
                    println_info("[q] quit");
                    println_info("[d] randomize left pixels");
                    println_info("[e] randomize left luma");
                    println_info("[r] randomize right pixels");
                    println_info("[f] randomize right luma");
                    println_info("[→] move divider right");
                    println_info("[←] move divider left");
                    AppEvent::None
                }
                KeyCode::Char('q') => {
                    println_warning("terminating");
                    AppEvent::Close
                }
                KeyCode::Char('d') => {
                    println_debug("randomizing left pixels");
                    AppEvent::RandomizeLeftPixels
                }
                KeyCode::Char('e') => {
                    println_info("randomizing left luma");
                    AppEvent::RandomizeLeftLuma
                }
                KeyCode::Char('f') => {
                    println_info("randomizing right pixels");
                    AppEvent::RandomizeRightPixels
                }
                KeyCode::Char('r') => {
                    println_info("randomizing right luma");
                    AppEvent::RandomizeRightLuma
                }
                KeyCode::Right => {
                    AppEvent::Accelerate
                }
                KeyCode::Left => {
                    AppEvent::Decelerate
                }
                key_code => {
                    println_debug(format!("unhandled KeyCode {key_code:?}"));
                    AppEvent::None
                }
            }
        }
        event => {
            println_debug(format!("unhandled event {event:?}"));
            AppEvent::None
        }
    }
}

fn draw_pixels(pixels: &mut PixelGrid, left: &PixelGrid, right: &PixelGrid, split_index: usize) {
    for x in 0..pixels.width() {
        let left_or_right = if x < split_index { left } else { right };
        for y in 0..pixels.height() {
            let set = x == split_index || left_or_right.get(x, y);
            pixels.set(x, y, set);
        }
    }
}

fn draw_luma(luma: &mut ByteGrid, left: &ByteGrid, right: &ByteGrid, split_tile: usize) {
    for x in 0..luma.width() {
        let left_or_right = if x < split_tile { left } else { right };
        for y in 0..luma.height() {
            let set = if x == split_tile {
                255
            } else {
                left_or_right.get(x, y)
            };
            luma.set(x, y, set);
        }
    }
}

fn send_to_screen(connection: &Connection, pixels: &PixelGrid, luma: &ByteGrid) {
    let pixel_cmd =
        BitmapLinearWin(Origin(0, 0), pixels.clone(), CompressionCode::Uncompressed);
    connection
        .send(pixel_cmd.into())
        .expect("could not send pixels");

    connection
        .send(CharBrightness(Origin(0, 0), luma.clone()).into())
        .expect("could not send brightness");
}

fn randomize<TGrid, TValue>(field: &mut TGrid)
    where TGrid: Grid<TValue>, Standard: Distribution<TValue>
{
    let mut rng = rand::thread_rng();

    for y in 0..field.height() {
        for x in 0..field.width() {
            field.set(x, y, rng.gen());
        }
    }
}

fn init() -> Connection {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    execute!(stdout(), EnterAlternateScreen, EnableLineWrap).expect("could not enter alternate screen");
    enable_raw_mode().expect("could not enable raw terminal mode");

    Connection::open(Cli::parse().destination)
        .expect("Could not connect. Did you forget `--destination`?")
}

fn de_init() {
    disable_raw_mode().expect("could not disable raw terminal mode");
    execute!(stdout(), LeaveAlternateScreen).expect("could not leave alternate screen");
}

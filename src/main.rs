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
use servicepoint2::{
    ByteGrid, CompressionCode, Connection, FRAME_PACING, Grid, Origin, PixelGrid, TILE_HEIGHT,
    TILE_WIDTH,
};
use servicepoint2::Command::{BitmapLinearWin, CharBrightness};

use crate::game::Game;
use crate::print::{println_debug, println_info, println_warning};
use crate::rules::{generate_bb3, generate_u8b3};

mod game;
mod print;
mod rules;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, default_value = "172.23.42.29:2342")]
    destination: String,
}

fn main() {
    let connection = init();

    let mut left_pixels = Game {
        rules: generate_bb3(),
        field: PixelGrid::max_sized(),
    };
    let mut right_pixels = Game {
        rules: generate_bb3(),
        field: PixelGrid::max_sized(),
    };
    let mut left_luma = Game {
        rules: generate_u8b3(),
        field: ByteGrid::new(TILE_WIDTH, TILE_HEIGHT),
    };
    let mut right_luma = Game {
        rules: generate_u8b3(),
        field: ByteGrid::new(TILE_WIDTH, TILE_HEIGHT),
    };

    randomize(&mut left_luma.field);
    randomize(&mut left_pixels.field);
    randomize(&mut right_luma.field);
    randomize(&mut right_pixels.field);

    let mut pixels = PixelGrid::max_sized();
    let mut luma = ByteGrid::new(TILE_WIDTH, TILE_HEIGHT);

    let mut split_pixel = 0;
    let mut split_speed: i32 = 1;

    let mut iteration = Wrapping(0u8);

    let mut target_duration = FRAME_PACING;

    loop {
        let start = Instant::now();

        left_pixels.step();
        right_pixels.step();

        if iteration % Wrapping(10) == Wrapping(0) {
            left_luma.step();
            right_luma.step();
        }

        iteration += Wrapping(1u8);

        if split_speed > 0 && split_pixel == pixels.width() {
            split_pixel = 0;

            (left_luma, right_luma) = (right_luma, left_luma);
            (left_pixels, right_pixels) = (right_pixels, left_pixels);

            randomize(&mut left_pixels.field);
            randomize(&mut left_luma.field);
            left_pixels.rules = generate_bb3();
            left_luma.rules = generate_u8b3();
        } else if split_speed < 0 && split_pixel == 0 {
            split_pixel = pixels.width();

            (left_luma, right_luma) = (right_luma, left_luma);
            (left_pixels, right_pixels) = (right_pixels, left_pixels);

            randomize(&mut right_pixels.field);
            randomize(&mut right_luma.field);
            right_pixels.rules = generate_bb3();
            right_luma.rules = generate_u8b3();
        }

        split_pixel =
            i32::clamp(split_pixel as i32 + split_speed, 0, pixels.width() as i32) as usize;

        draw_pixels(
            &mut pixels,
            &left_pixels.field,
            &right_pixels.field,
            split_pixel,
        );
        draw_luma(
            &mut luma,
            &left_luma.field,
            &right_luma.field,
            split_pixel / 8,
        );
        send_to_screen(&connection, &pixels, &luma);

        while event::poll(Duration::from_secs(0)).expect("could not poll") {
            match event::read().expect("could not read event").try_into() {
                Err(_) => {}
                Ok(AppEvent::RandomizeLeftPixels) => {
                    randomize(&mut left_pixels.field);
                    println_debug("randomized left pixels");
                }
                Ok(AppEvent::RandomizeRightPixels) => {
                    randomize(&mut right_pixels.field);
                    println_info("randomized right pixels");
                }
                Ok(AppEvent::RandomizeLeftLuma) => {
                    randomize(&mut left_luma.field);
                    println_info("randomized left luma");
                }
                Ok(AppEvent::RandomizeRightLuma) => {
                    randomize(&mut right_luma.field);
                    println_info("randomized right luma");
                }
                Ok(AppEvent::SeparatorAccelerate) => {
                    split_speed += 1;
                    println_info(format!("increased separator speed to {split_speed}"));
                }
                Ok(AppEvent::SeparatorDecelerate) => {
                    split_speed -= 1;
                    println_info(format!("decreased separator speed to {split_speed}"));
                }
                Ok(AppEvent::Close) => {
                    println_warning("terminating");
                    de_init();
                    return;
                }
                Ok(AppEvent::SimulationSpeedUp) => {
                    target_duration = target_duration.saturating_sub(Duration::from_millis(1));
                    println_info(format!("increased simulation speed to {} ups", 1f64 / target_duration.as_secs_f64()));
                }
                Ok(AppEvent::SimulationSpeedDown) => {
                    target_duration = target_duration.saturating_add(Duration::from_millis(1));
                    println_info(format!("decreased simulation speed to {} ups", 1f64 / target_duration.as_secs_f64()));
                }
            }
        }

        let tick_time = start.elapsed();
        if tick_time < target_duration {
            thread::sleep(target_duration - tick_time);
        }
    }
}

enum AppEvent {
    Close,
    RandomizeLeftPixels,
    RandomizeRightPixels,
    RandomizeLeftLuma,
    RandomizeRightLuma,
    SeparatorAccelerate,
    SeparatorDecelerate,
    SimulationSpeedUp,
    SimulationSpeedDown,
}

impl TryFrom<Event> for AppEvent {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
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
                        println_info("[→] accelerate divider right");
                        println_info("[←] accelerate divider left");
                        Err(())
                    }
                    KeyCode::Char('q') => Ok(AppEvent::Close),
                    KeyCode::Char('d') => Ok(AppEvent::RandomizeLeftPixels),
                    KeyCode::Char('e') => Ok(AppEvent::RandomizeLeftLuma),
                    KeyCode::Char('f') => Ok(AppEvent::RandomizeRightPixels),
                    KeyCode::Char('r') => Ok(AppEvent::RandomizeRightLuma),
                    KeyCode::Right => Ok(AppEvent::SeparatorAccelerate),
                    KeyCode::Left => Ok(AppEvent::SeparatorDecelerate),
                    KeyCode::Up => Ok(AppEvent::SimulationSpeedUp),
                    KeyCode::Down => Ok(AppEvent::SimulationSpeedDown),
                    key_code => {
                        println_debug(format!("unhandled KeyCode {key_code:?}"));
                        Err(())
                    }
                }
            }
            event => {
                println_debug(format!("unhandled event {event:?}"));
                Err(())
            }
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
            let set = u8::max(48, left_or_right.get(x, y));

            let set = set as f32 / u8::MAX as f32 * 12f32;

            luma.set(x, y, set as u8);
        }
    }
}

fn send_to_screen(connection: &Connection, pixels: &PixelGrid, luma: &ByteGrid) {
    let pixel_cmd = BitmapLinearWin(Origin(0, 0), pixels.clone(), CompressionCode::Uncompressed);
    connection
        .send(pixel_cmd.into())
        .expect("could not send pixels");

    connection
        .send(CharBrightness(Origin(0, 0), luma.clone()).into())
        .expect("could not send brightness");
}

fn randomize<TGrid, TValue>(field: &mut TGrid)
    where
        TGrid: Grid<TValue>,
        Standard: Distribution<TValue>,
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

    execute!(stdout(), EnterAlternateScreen, EnableLineWrap)
        .expect("could not enter alternate screen");
    enable_raw_mode().expect("could not enable raw terminal mode");

    Connection::open(Cli::parse().destination)
        .expect("Could not connect. Did you forget `--destination`?")
}

fn de_init() {
    disable_raw_mode().expect("could not disable raw terminal mode");
    execute!(stdout(), LeaveAlternateScreen).expect("could not leave alternate screen");
}

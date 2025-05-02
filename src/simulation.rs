use crate::{
    game::Game,
    print::{println_debug, println_info},
    rules::{generate_bb3, generate_u8b3},
};
use rand::{distributions::Standard, prelude::Distribution, Rng};
use servicepoint::{
    Bitmap, Brightness, BrightnessGrid, Grid, Value, ValueGrid, PIXEL_HEIGHT, PIXEL_WIDTH,
    TILE_HEIGHT, TILE_SIZE, TILE_WIDTH,
};
use std::num::Wrapping;

pub(crate) struct Simulation {
    pub(crate) left_pixels: Game<bool>,
    pub(crate) right_pixels: Game<bool>,
    pub(crate) left_luma: Game<u8>,
    pub(crate) right_luma: Game<u8>,
    split_pixel: usize,
    pub(crate) split_speed: i32,
    iteration: Wrapping<u8>,
}

pub enum SimulationEvent {
    RandomizeLeftPixels,
    RandomizeRightPixels,
    RandomizeLeftLuma,
    RandomizeRightLuma,
    SeparatorAccelerate,
    SeparatorDecelerate,
}

impl Simulation {
    pub fn new() -> Simulation {
        let left_pixels = Game {
            rules: generate_bb3(),
            field: make_randomized(PIXEL_WIDTH, PIXEL_HEIGHT),
        };
        let right_pixels = Game {
            rules: generate_bb3(),
            field: make_randomized(PIXEL_WIDTH, PIXEL_HEIGHT),
        };
        let left_luma = Game {
            rules: generate_u8b3(),
            field: make_randomized(TILE_WIDTH, TILE_HEIGHT),
        };
        let right_luma = Game {
            rules: generate_u8b3(),
            field: make_randomized(TILE_WIDTH, TILE_HEIGHT),
        };

        Self {
            left_pixels,
            right_pixels,
            left_luma,
            right_luma,
            split_pixel: 0,
            split_speed: 1,
            iteration: Wrapping(0u8),
        }
    }

    fn swap_left_right(&mut self) {
        std::mem::swap(&mut self.left_luma, &mut self.right_luma);
        std::mem::swap(&mut self.left_pixels, &mut self.right_pixels);
    }

    fn regenerate(pixels: &mut Game<bool>, luma: &mut Game<u8>) {
        randomize(&mut pixels.field);
        randomize(&mut luma.field);
        pixels.rules = generate_bb3();
        luma.rules = generate_u8b3();
    }

    pub(crate) fn run_iteration(&mut self) {
        self.left_pixels.step();
        self.right_pixels.step();

        if self.iteration % Wrapping(10) == Wrapping(0) {
            self.left_luma.step();
            self.right_luma.step();
        }

        self.iteration += Wrapping(1u8);

        if self.split_speed > 0 && self.split_pixel == self.left_pixels.field.width() {
            self.split_pixel = 0;
            self.swap_left_right();
            Self::regenerate(&mut self.left_pixels, &mut self.left_luma);
        } else if self.split_speed < 0 && self.split_pixel == 0 {
            self.split_pixel = self.left_pixels.field.width();
            self.swap_left_right();
            Self::regenerate(&mut self.right_pixels, &mut self.right_luma);
        }

        self.split_pixel = i32::clamp(
            self.split_pixel as i32 + self.split_speed,
            0,
            self.left_pixels.field.width() as i32,
        ) as usize;
    }

    pub(crate) fn draw_state(&self, pixels: &mut Bitmap, luma: &mut BrightnessGrid) {
        for x in 0..pixels.width() {
            let left_or_right = if x < self.split_pixel {
                &self.left_pixels.field
            } else {
                &self.right_pixels.field
            };
            for y in 0..pixels.height() {
                let set = x == self.split_pixel || left_or_right.get(x, y);
                pixels.set(x, y, set);
            }
        }

        let split_tile = self.split_pixel / TILE_SIZE;
        for x in 0..luma.width() {
            let left_or_right = if x < split_tile {
                &self.left_luma.field
            } else {
                &self.right_luma.field
            };
            for y in 0..luma.height() {
                let set = left_or_right.get(x, y) as f32 / u8::MAX as f32 * u8::from(Brightness::MAX) as f32;
                let set = Brightness::try_from(set as u8).unwrap();
                luma.set(x, y, set);
            }
        }
    }

    pub(crate) fn handle_event(&mut self, event: SimulationEvent) {
        match event {
            SimulationEvent::RandomizeLeftPixels => {
                randomize(&mut self.left_pixels.field);
                println_debug("randomized left pixels");
            }
            SimulationEvent::RandomizeRightPixels => {
                randomize(&mut self.right_pixels.field);
                println_info("randomized right pixels");
            }
            SimulationEvent::RandomizeLeftLuma => {
                randomize(&mut self.left_luma.field);
                println_info("randomized left luma");
            }
            SimulationEvent::RandomizeRightLuma => {
                randomize(&mut self.right_luma.field);
                println_info("randomized right luma");
            }
            SimulationEvent::SeparatorAccelerate => {
                self.split_speed += 1;
                println_info(format!("increased separator speed to {}", self.split_speed));
            }
            SimulationEvent::SeparatorDecelerate => {
                self.split_speed -= 1;
                println_info(format!("decreased separator speed to {}", self.split_speed));
            }
        }
    }
}

fn make_randomized<T: Value>(width: usize, height: usize) -> ValueGrid<T>
where
    Standard: Distribution<T>,
{
    let mut grid = ValueGrid::new(width, height);
    randomize(&mut grid);
    grid
}

fn randomize<T: Value>(field: &mut ValueGrid<T>)
where
    Standard: Distribution<T>,
{
    let mut rng = rand::thread_rng();

    for y in 0..field.height() {
        for x in 0..field.width() {
            field.set(x, y, rng.gen());
        }
    }
}

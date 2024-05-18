use std::time::Duration;

use crossterm::event;
use crossterm::event::KeyCode::Modifier;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use log::{debug, info, warn};
use rand::{distributions, Rng};
use servicepoint2::Command::CharBrightness;
use servicepoint2::{
    ByteGrid, Command, CompressionCode, Connection, Origin, PixelGrid, TILE_HEIGHT, TILE_WIDTH,
};

use crate::Cli;

pub(crate) struct App {
    connection: Connection,
    probability: f64,
    field: PixelGrid,
}

impl App {
    #[must_use]
    pub fn new(connection: Connection, cli: &Cli) -> Self {
        Self {
            connection,
            probability: cli.probability,
            field: Self::make_random_field(cli.probability),
        }
    }

    pub fn step(&mut self) -> bool {
        self.send_image();
        self.change_brightness();
        self.field = self.game_iteration();

        self.handle_events()
    }

    fn game_iteration(&self) -> PixelGrid {
        let mut next = self.field.clone();
        for x in 0..self.field.width() {
            for y in 0..self.field.height() {
                let old_state = self.field.get(x, y);
                let neighbors = self.count_neighbors(x, y);

                let new_state =
                    matches!((old_state, neighbors), (true, 2) | (true, 3) | (false, 3));
                next.set(x, y, new_state);
            }
        }
        next
    }

    fn send_image(&self) {
        let command = Command::BitmapLinearWin(
            Origin(0, 0),
            self.field.clone(),
            CompressionCode::Uncompressed,
        );

        self.connection
            .send(command.into())
            .expect("could not send");
    }

    fn count_neighbors(&self, x: usize, y: usize) -> u8 {
        let x = x as i32;
        let y = y as i32;
        let mut count = 0;
        for nx in x - 1..=x + 1 {
            for ny in y - 1..=y + 1 {
                if nx == x && ny == y {
                    continue; // the cell itself does not count
                }

                if nx < 0
                    || ny < 0
                    || nx >= self.field.width() as i32
                    || ny >= self.field.height() as i32
                {
                    continue; // pixels outside the grid do not count
                }

                if !self.field.get(nx as usize, ny as usize) {
                    continue; // dead cells do not count
                }

                count += 1;
            }
        }

        count
    }

    fn make_random_field(probability: f64) -> PixelGrid {
        let mut field = PixelGrid::max_sized();
        let mut rng = rand::thread_rng();
        let d = distributions::Bernoulli::new(probability).unwrap();
        for x in 0..field.width() {
            for y in 0..field.height() {
                field.set(x, y, rng.sample(d));
            }
        }
        field
    }

    fn handle_events(&mut self) -> bool {
        if !event::poll(Duration::from_secs(0)).expect("could not poll") {
            return true;
        }

        match event::read().expect("could not read event") {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_press(key_event)
            }
            event => {
                debug!("unhandled event {event:?}");
                return true;
            }
        }
    }

    fn handle_key_press(&mut self, event: KeyEvent) -> bool {
        match event.code {
            KeyCode::Char('q') => {
                warn!("q pressed, terminating");
                return false;
            }
            KeyCode::Char(' ') => {
                info!("generating new random field");
                self.field = Self::make_random_field(self.probability);
            }
            key_code => {
                debug!("unhandled KeyCode {key_code:?}");
            }
        }
        true
    }

    fn change_brightness(&self) {
        let mut rng = rand::thread_rng();

        if !rng.gen_ratio(1, 10) {
            return;
        }

        let min_size = 1;
        let x = rng.gen_range(0..TILE_WIDTH - min_size);
        let y = rng.gen_range(0..TILE_HEIGHT - min_size);

        let w = rng.gen_range(min_size..=TILE_WIDTH - x);
        let h = rng.gen_range(min_size..=TILE_HEIGHT - y);

        let origin = Origin(x, y);
        let mut luma = ByteGrid::new(w as usize, h as usize);

        for y in 0..h as usize {
            for x in 0..w as usize {
                luma.set(x, y, rng.gen());
            }
        }

        self.connection
            .send(CharBrightness(origin, luma).into())
            .expect("could not send brightness");
    }
}

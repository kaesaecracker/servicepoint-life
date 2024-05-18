use rand::Rng;
use servicepoint2::{ByteGrid, PixelGrid, TILE_HEIGHT, TILE_WIDTH};

pub(crate) struct Game {
    pub field: PixelGrid,
    pub luma: ByteGrid,
}

impl Game {
    pub fn step(&mut self) {
        let mut rng = rand::thread_rng();

        self.field = self.field_iteration();

        if rng.gen_ratio(1, 10) {
            self.luma = self.luma_iteration();
        }
    }

    fn field_iteration(&self) -> PixelGrid {
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

    fn luma_iteration(&self) -> ByteGrid {
        let mut rng = rand::thread_rng();

        let min_size = 1;
        let window_x = rng.gen_range(0..TILE_WIDTH as usize - min_size);
        let window_y = rng.gen_range(0..TILE_HEIGHT as usize - min_size);

        let w = rng.gen_range(min_size..=TILE_WIDTH as usize - window_x);
        let h = rng.gen_range(min_size..=TILE_HEIGHT as usize - window_y);

        let mut new_luma = self.luma.clone();
        for inner_y in 0..h {
            for inner_x in 0..w {
                let x = window_x + inner_x;
                let y = window_y + inner_y;
                let old_value = self.luma.get(x, y);
                let new_value = i32::clamp(
                    old_value as i32 + rng.gen_range(-64..=64),
                    u8::MIN as i32,
                    u8::MAX as i32,
                ) as u8;

                new_luma.set(x, y, new_value);
            }
        }

        new_luma
    }
}

impl Default for Game {
    fn default() -> Self {
        Self {
            field: PixelGrid::max_sized(),
            luma: ByteGrid::new(TILE_WIDTH as usize, TILE_HEIGHT as usize),
        }
    }
}

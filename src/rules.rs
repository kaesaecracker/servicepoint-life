use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;

use crate::print::println_info;

pub struct Rules<TState, TKernel, const KERNEL_SIZE: usize>
    where TState: Copy + PartialEq, TKernel: Copy
{
    pub kernel: [[TKernel; KERNEL_SIZE]; KERNEL_SIZE],
    pub count_neighbor: fn(neighbor_state: TState, kernel_value: TKernel) -> i32,
    pub next_state: Box<dyn Fn(TState, i32) -> TState>,
}

pub const MOORE_NEIGHBORHOOD: [[bool; 3]; 3] = [
    [true, true, true],
    [true, false, true],
    [true, true, true]
];

pub const NEUMANN_NEIGHBORHOOD: [[bool; 3]; 3] = [
    [false, true, false],
    [true, false, true],
    [false, true, false]
];

pub const DIAGONALS_NEIGHBORHOOD: [[bool; 3]; 3] = [
    [true, false, true],
    [false, true, false],
    [true, false, true]
];

pub fn count_true_neighbor(neighbor_state: bool, kernel_value: bool) -> i32
{
    if neighbor_state && kernel_value { 1 } else { 0 }
}

impl Rules<bool, bool, 3> {
    #[must_use]
    pub fn random_bb3() -> Self {
        match rand::thread_rng().gen_range(0..=6) {
            0 => Self::game_of_life(),
            1 => Self::high_life(),
            2 => Self::seeds(),
            3 => Self::day_and_night(),
            4 => Self::mazecetric(),
            5 => Self::generate_bb3_moore(),
            6 => Self::generate_bb3_neumann(),
            _ => panic!(),
        }
    }

    #[must_use]
    pub fn game_of_life() -> Self {
        println_info("game of life");
        Self {
            kernel: MOORE_NEIGHBORHOOD,
            count_neighbor: count_true_neighbor,
            next_state: Box::new(|old_state, neighbors|
                matches!((old_state, neighbors), (true, 2) | (true, 3) | (false, 3))),
        }
    }

    #[must_use]
    pub fn high_life() -> Self {
        println_info("high life");
        Self {
            kernel: MOORE_NEIGHBORHOOD,
            count_neighbor: count_true_neighbor,
            next_state: Box::new(|old_state, neighbors|
                matches!((old_state, neighbors), (true, 2) | (true, 3) | (false, 3)| (false, 6))),
        }
    }

    #[must_use]
    pub fn seeds() -> Self {
        println_info("seeds");
        Self {
            kernel: MOORE_NEIGHBORHOOD,
            count_neighbor: count_true_neighbor,
            next_state: Box::new(|state, neighbors|
                matches!((state, neighbors), (false, 2))),
        }
    }

    #[must_use]
    pub fn day_and_night() -> Self {
        println_info("day_and_night");
        Self {
            kernel: MOORE_NEIGHBORHOOD,
            count_neighbor: count_true_neighbor,
            next_state: Box::new(|state, neighbors| {
                match (state, neighbors) {
                    (false, 3) => true,
                    (false, 6) => true,
                    (false, 7) => true,
                    (false, 8) => true,
                    (true, 3) => true,
                    (true, 4) => true,
                    (true, 6) => true,
                    (true, 7) => true,
                    (true, 8) => true,
                    _ => false,
                }
            }),
        }
    }

    #[must_use]
    pub fn mazecetric() -> Self {
        println_info("mazecetric");
        Self {
            kernel: MOORE_NEIGHBORHOOD,
            count_neighbor: count_true_neighbor,
            next_state: Box::new(|state, neighbors| {
                match (state, neighbors) {
                    (false, 3) => true,
                    (true, 0) => false,
                    (true, n) if n < 5 => true,
                    _ => false,
                }
            }),
        }
    }

    #[must_use]
    pub fn generate_bb3_moore() -> Self {
        let mut rng = thread_rng();

        let birth = Self::generate_neighbor_counts(rng.gen_range(0..=8), &mut rng);
        let survive = Self::generate_neighbor_counts(rng.gen_range(0..=8), &mut rng);

        println_info(format!("generated bb3 moore: Birth {birth:?} Survival {survive:?}"));

        Self {
            kernel: MOORE_NEIGHBORHOOD,
            count_neighbor: count_true_neighbor,
            next_state: Box::new(move |old_state, neighbors| {
                old_state && survive.contains(&neighbors)
                    || !old_state && birth.contains(&neighbors)
            }),
        }
    }

    #[must_use]
    pub fn generate_bb3_neumann() -> Rules<bool, bool, 3> {
        let mut rng = thread_rng();

        let birth = Self::generate_neighbor_counts(rng.gen_range(0..=4), &mut rng);
        let survive = Self::generate_neighbor_counts(rng.gen_range(0..=4), &mut rng);

        println_info(format!("generated bb3 neumann: Birth {birth:?} Survival {survive:?}"));

        Self {
            kernel: NEUMANN_NEIGHBORHOOD,
            count_neighbor: count_true_neighbor,
            next_state: Box::new(move |old_state, neighbors| {
                old_state && survive.contains(&neighbors)
                    || !old_state && birth.contains(&neighbors)
            }),
        }
    }

    fn generate_neighbor_counts(count: u8, rng: &mut ThreadRng) -> Vec<i32> {
        let mut result = vec!();
        for _ in 0..count {
            loop {
                let candidate = rng.gen_range(0..=8);

                if !result.contains(&candidate) {
                    result.push(candidate);
                    break;
                }
            }
        }

        result
    }
}

impl Rules<u8, bool, 3> {
    #[must_use]
    pub fn random_u8b3() -> Self {
        match rand::thread_rng().gen_range(0..3) {
            0 => Self::brians_brain(),
            1 => Self::continuous_game_of_life(),
            2 => Self::equalizer(),
            _ => panic!(),
        }
    }

    #[must_use]
    pub fn brians_brain() -> Self {
        const ALIVE: u8 = u8::MAX;
        const DYING: u8 = ALIVE / 2;
        const DEAD: u8 = 0;

        Self {
            kernel: MOORE_NEIGHBORHOOD,
            count_neighbor: |state, kernel| {
                if kernel && state == u8::MAX { 1 } else { 0 }
            },
            next_state: Box::new(|state, neighbors| {
                match (state, neighbors) {
                    (ALIVE, _) => DYING,
                    (DYING, _) => DEAD,
                    (DEAD, 2) => ALIVE,
                    (random_state, _) => if random_state > DYING {
                        ALIVE
                    } else {
                        DEAD
                    }
                }
            }),
        }
    }


    #[must_use]
    pub fn continuous_game_of_life() -> Self {
        Self {
            kernel: MOORE_NEIGHBORHOOD,
            count_neighbor: |state, kernel| {
                if kernel && state >= u8::MAX / 2 { 1 } else { 0 }
            },
            next_state: Box::new(|old_state, neighbors| {
                let is_alive = old_state >= u8::MAX / 2;
                let delta = match (is_alive, neighbors) {
                    (true, 2) | (true, 3) | (false, 3) => 10,
                    _ => -10,
                };

                i32::clamp(old_state as i32 + delta, u8::MIN as i32, u8::MAX as i32) as u8
            }),
        }
    }

    #[must_use]
    pub fn equalizer() -> Self {
        Self {
            kernel: DIAGONALS_NEIGHBORHOOD,
            count_neighbor: |state, kernel| {
                let state = state as i32;
                if kernel {
                    state
                } else {
                    0
                }
            },
            next_state: Box::new(|old_state, neighbors| {
                if old_state % 42 == 0 {
                    return u8::MAX;
                }

                if old_state % 23 == 0 {
                    return u8::MIN;
                }

                let average_health = neighbors / 5;
                let delta = if average_health > old_state as i32 {
                    10
                } else {
                    -10
                };

                i32::clamp(old_state as i32 + delta, u8::MIN as i32, u8::MAX as i32) as u8
            }),
        }
    }
}


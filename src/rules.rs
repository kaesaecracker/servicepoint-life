use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;

use crate::print::println_info;

pub struct Rules<TState, TKernel, const KERNEL_SIZE: usize>
    where TState: Copy + PartialEq, TKernel: Copy
{
    pub kernel: [[TKernel; KERNEL_SIZE]; KERNEL_SIZE],
    pub count_neighbor: Box<dyn Fn(TState, TKernel) -> i32>,
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
    [false, false, false],
    [true, false, true]
];

pub fn count_true_neighbor(neighbor_state: bool, kernel_value: bool) -> i32
{
    if neighbor_state && kernel_value { 1 } else { 0 }
}

#[must_use]
pub fn generate_bb3() -> Rules<bool, bool, 3> {
    let mut rng = thread_rng();

    let is_moore = rng.gen_bool(1.0 / 2.0);
    let kernel = if is_moore { MOORE_NEIGHBORHOOD } else { NEUMANN_NEIGHBORHOOD };
    let max_neighbors = if is_moore { 8 } else { 4 };

    let birth = generate_neighbor_counts(rng.gen_range(1..=max_neighbors), &mut rng, &[0]);
    let survive = generate_neighbor_counts(rng.gen_range(1..=max_neighbors), &mut rng, &[]);

    println_info(format!("generated bb3: Birth {birth:?} Survival {survive:?}, kernel: {kernel:?}"));

    Rules {
        kernel,
        count_neighbor: Box::new(count_true_neighbor),
        next_state: Box::new(move |old_state, neighbors| {
            old_state && survive.contains(&neighbors)
                || !old_state && birth.contains(&neighbors)
        }),
    }
}

fn generate_neighbor_counts(count: u8, rng: &mut ThreadRng, exclude: &[i32]) -> Vec<i32> {
    let mut result = vec!();
    for _ in 0..count {
        let value = rng.gen_range(0..=count) as i32;
        if !exclude.contains(&value) {
            result.push(value);
        }
    }
    result
}

#[must_use]
pub fn generate_u8b3() -> Rules<u8, bool, 3> {
    let mut rng = thread_rng();

    let kernel = match rng.gen_range(0..3) {
        0 => MOORE_NEIGHBORHOOD,
        1 => NEUMANN_NEIGHBORHOOD,
        2 => DIAGONALS_NEIGHBORHOOD,
        _ => panic!()
    };

    let alive_threshold = rng.gen();

    let birth = generate_neighbor_counts(rng.gen_range(1..=9), &mut rng, &[0]);
    let survive = generate_neighbor_counts(rng.gen_range(1..=9 - birth.len()) as u8, &mut rng, &[]);

    let add = rng.gen_range(5..40);
    let sub = rng.gen_range(5..40);

    println_info(format!("generated u8b3: Birth {birth:?} Survival {survive:?}, kernel: {kernel:?}, alive_thresh: {alive_threshold}, delta: {add}/{sub}"));

    Rules {
        kernel,
        count_neighbor: Box::new(|state, kernel| {
            if kernel { state as i32 } else { 0 }
        }),
        next_state: Box::new(move |old_state, neighbors| {
            let neighbors = neighbors / alive_threshold as i32;
            let old_is_alive = old_state >= alive_threshold;
            let new_is_alive = old_is_alive && survive.contains(&neighbors)
                || !old_is_alive && birth.contains(&neighbors);
            let delta = if new_is_alive { add } else { -sub };
            i32::clamp(old_state as i32 + delta, u8::MIN as i32, u8::MAX as i32) as u8
        }),
    }
}

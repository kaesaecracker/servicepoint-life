pub struct Rules<TState, TKernel, const KERNEL_SIZE: usize>
    where TState: Copy + PartialEq, TKernel: Copy
{
    pub kernel: [[TKernel; KERNEL_SIZE]; KERNEL_SIZE],
    pub count_neighbor: fn(neighbor_state: TState, kernel_value: TKernel) -> i32,
    pub next_state: fn(state: TState, kernel_result: i32) -> TState,
}

pub const MOORE_NEIGHBORHOOD: [[bool; 3]; 3] = [
    [true, true, true],
    [true, false, true],
    [true, true, true]
];

pub fn count_true_neighbor(neighbor_state: bool, kernel_value: bool) -> i32
{
    if neighbor_state && kernel_value { 1 } else { 0 }
}

impl Rules<bool, bool, 3> {
    #[must_use]
    pub fn game_of_life() -> Self {
        Self {
            kernel: MOORE_NEIGHBORHOOD,
            count_neighbor: count_true_neighbor,
            next_state: |old_state, neighbors|
                matches!((old_state, neighbors), (true, 2) | (true, 3) | (false, 3)),
        }
    }

    #[must_use]
    pub fn high_life() -> Self {
        Self {
            kernel: MOORE_NEIGHBORHOOD,
            count_neighbor: count_true_neighbor,
            next_state: |old_state, neighbors|
                matches!((old_state, neighbors), (true, 2) | (true, 3) | (false, 3)| (false, 6)),
        }
    }

    #[must_use]
    pub fn seeds() -> Self {
        Self {
            kernel: MOORE_NEIGHBORHOOD,
            count_neighbor: count_true_neighbor,
            next_state: |state, neighbors|
                matches!((state, neighbors), (false, 2)),
        }
    }

    #[must_use]
    pub fn day_and_night() -> Self {
        Self {
            kernel: MOORE_NEIGHBORHOOD,
            count_neighbor: count_true_neighbor,
            next_state: |state, neighbors| {
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
            },
        }
    }
}

impl Rules<u8, bool, 3> {
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
            next_state: |state, neighbors| {
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
            },
        }
    }

    pub fn continuous_game_of_life() -> Self {
        Self {
            kernel: MOORE_NEIGHBORHOOD,
            count_neighbor: |state, kernel| {
                if kernel && state >= u8::MAX / 2 { 1 } else { 0 }
            },
            next_state: |old_state, neighbors| {
                let is_alive = old_state >= u8::MAX / 2;
                let delta = match (is_alive, neighbors) {
                    (true, 2) | (true, 3) | (false, 3) => 10,
                    _ => -10,
                };

                i32::clamp(old_state as i32 + delta, u8::MIN as i32, u8::MAX as i32) as u8
            },
        }
    }
}

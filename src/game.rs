use servicepoint2::Grid;

use crate::rules::Rules;

pub(crate) struct Game<TState, TGrid, TKernel, const KERNEL_SIZE: usize>
where
    TGrid: Grid<TState>,
    TState: Copy + PartialEq,
    TKernel: Copy,
{
    pub field: TGrid,
    pub rules: Rules<TState, TKernel, KERNEL_SIZE>,
}

impl<TState, TGrid, TKernel, const KERNEL_SIZE: usize> Game<TState, TGrid, TKernel, KERNEL_SIZE>
where
    TGrid: Grid<TState>,
    TState: Copy + PartialEq,
    TKernel: Copy,
{
    pub fn step(&mut self) {
        self.field = self.field_iteration();
    }

    fn field_iteration(&self) -> TGrid {
        let mut next = TGrid::new(self.field.width(), self.field.height());
        for x in 0..self.field.width() {
            for y in 0..self.field.height() {
                let old_state = self.field.get(x, y);
                let neighbors = self.count_neighbors(x, y);
                let new_state = (self.rules.next_state)(old_state, neighbors);
                next.set(x, y, new_state);
            }
        }
        next
    }

    fn count_neighbors(&self, x: usize, y: usize) -> i32 {
        let x = x as i32;
        let y = y as i32;
        let mut count = 0;

        let kernel = &self.rules.kernel;
        assert_eq!(KERNEL_SIZE % 2, 1);
        let offset = KERNEL_SIZE as i32 / 2;

        for (kernel_y, kernel_row) in kernel.iter().enumerate() {
            let offset_y = kernel_y as i32 - offset;

            for (kernel_x, kernel_value) in kernel_row.iter().enumerate() {
                let offset_x = kernel_x as i32 - offset;
                let neighbor_x = x + offset_x;
                let neighbor_y = y + offset_y;

                if neighbor_x < 0
                    || neighbor_y < 0
                    || neighbor_x >= self.field.width() as i32
                    || neighbor_y >= self.field.height() as i32
                {
                    continue; // pixels outside the grid do not count
                }

                let neighbor_state = self.field.get(neighbor_x as usize, neighbor_y as usize);
                count += (self.rules.count_neighbor)(neighbor_state, *kernel_value);
            }
        }

        count
    }
}

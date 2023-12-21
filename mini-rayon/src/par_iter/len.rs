pub const THRESHOLD: usize = 10 * 1024;

pub struct ParallelLen {
    pub max_len: usize,
    pub cost: usize,
    pub sparse: bool,
}

impl ParallelLen {
    pub fn let_cost(&self, mid: usize) -> ParallelLen {
        ParallelLen {
            max_len: mid,
            cost: self.cost / 2,
            sparse: self.sparse,
        }
    }

    pub fn right_cost(&self, mid: usize) -> ParallelLen {
        ParallelLen {
            max_len: self.max_len - mid,
            cost: self.cost / 2,
            sparse: self.sparse,
        }
    }
}
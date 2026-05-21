#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub test: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { test: false }
    }
}

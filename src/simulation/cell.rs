#[derive(Debug, Clone, Copy,PartialEq)]
pub enum CellTypes{Solid,Liquid,Gas}

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub cell_type: CellTypes,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { cell_type: CellTypes::Liquid }
    }
}

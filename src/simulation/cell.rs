#[derive(Debug, Clone, Copy,PartialEq)]
pub enum CellTypes{Solid,Liquid,Gas}

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub cell_type: CellTypes,
    pub color: (i32,i32,i32)
}

impl Default for Cell {
    fn default() -> Self {
        Cell { cell_type: CellTypes::Liquid, color: (0,0,0) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]

pub enum CellTypes {
    Solid,
    Liquid,
    Gas,
}

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub cell_type: CellTypes,
    pub color: (f32, f32, f32),

    pub u: f32,
    pub v: f32,
    pub du: f32,
    pub dv: f32,
    pub prev_u: f32,
    pub prev_v: f32,
    pub p: f32,
    pub s: f32,
    pub particle_density: f32,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            cell_type: CellTypes::Liquid,
            color: (0.0, 0.0, 0.0),

            u: 0.0,
            v: 0.0,
            du: 0.0,
            dv: 0.0,
            prev_u: 0.0,
            prev_v: 0.0,
            p: 0.0,
            s: 1.0,
            particle_density: 0.0,
        }
    }
}

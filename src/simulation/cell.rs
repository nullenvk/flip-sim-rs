#[derive(Debug, Clone, Copy,PartialEq)]

pub enum CellTypes{Solid,Liquid,Gas}

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub cell_type: CellTypes,
    pub color: (f64,f64,f64),

    pub u: f64,
    pub v: f64,
    pub du: f64,
    pub dv: f64,
    pub prev_u: f64,
    pub prev_v: f64,
    pub p: f64,
    pub s: f64,
    pub particle_density: f64,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { 
            cell_type: CellTypes::Liquid, 
            color: (0.0,0.0,0.0),

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

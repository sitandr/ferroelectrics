

use std::{mem::replace, vec};

use eframe::emath::RectTransform;
use egui::{Painter, Pos2, Color32, Rect, Vec2, Rounding};
use fnv::FnvHashMap;
use rand::Rng;


#[derive( Debug, serde::Deserialize, serde::Serialize)]
pub struct Simulation{
    pub gen: FieldGenerator,

    pub cells: CellBox,

    pub germs: GermGenesis

    // transform: RectTransform,
    // shapes: Vec<Shape>
}


impl Simulation{
    pub fn new(width: usize, height: usize) -> Self{
        Simulation{cells: CellBox::new(width, height),
             gen: FieldGenerator { t: 0, time_up: 500, time_down: 500, amplitude: 0.4},
             germs: GermGenesis::StartRandom { number: 10 },
       }
    }

    pub fn step<T: Rng>(&mut self, mut rng: T){

        let (f, tend) = self.gen.field();
        self.gen.tick();
        
        self.cells.step(f, &tend, &mut rng);
        self.germs.tick(f, &mut self.cells, &mut rng);
        match tend {
            FieldTend::ReverseDown => {self.germs.activate_start(-1.0, &mut self.cells, &mut rng)},
            FieldTend::ReverseUp => {self.germs.activate_start(1.0, &mut self.cells, &mut rng)},
            FieldTend::Stable => {},
        }
    }

    pub fn get_polarization(&self) -> f64{
        (self.cells.polarization_counter as f64)/((self.cells.width*self.cells.height) as f64)
    }

    /// Call "set_transform" to generate shapes to paint
    pub fn paint(&self, painter: &Painter, transform: RectTransform) {
        
        for (&i, &color_c) in self.cells.active.iter(){
            let (x, y) = self.cells.index2coord(i);
            let x = x as f32 * 0.9 + (self.cells.width as f32)/20.0;
            let y = y as f32 * 0.9 + (self.cells.height as f32)/20.0;
            let point = transform * Pos2::new((x as f32)/(self.cells.width as f32), (y as f32)/(self.cells.height as f32));
                painter.rect_filled(Rect::from_center_size(point,
                     transform.scale() * Vec2::new(1.0/self.cells.width as f32, 1.0/self.cells.height as f32)*1.1),
                      Rounding::none(),
                      Self::color_gradient(color_c/4.0, Color32::from_rgb(40, 0, 130), Color32::from_rgb(200, 250, 50)));

        }
    }

    fn color_gradient(v: f32, c1: Color32, c2: Color32) -> Color32{
        let c1 = c1.linear_multiply(1.0 - v);
        let c2 = c2.linear_multiply(v);
        Color32::from_rgb(c1.r() + c2.r(), c1.g() + c2.g(), c1.b() + c2.b())
    }

    pub fn reset<T: Rng>(&mut self, mut rng: T){
        self.cells.clear();
        self.germs.activate_once(&mut self.cells, &mut rng);
        self.gen.reset();
    }
}

enum FieldTend{
    ReverseDown,
    ReverseUp,
    Stable
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct FieldGenerator{
    #[serde(skip)]
    t: u32,
    pub time_up: u32,
    pub time_down: u32,
    pub amplitude: f32
}

impl FieldGenerator{
    pub fn reset(&mut self){
        self.t = 0;
    }
    /// returns field value, than 
    fn tick(&mut self){
        self.t += 1;
        if self.t > self.time_down + self.time_up{
            self.t = 0;
        }
    }

    fn field(&self) -> (f32, FieldTend){
        match self.t{
            0 => (0.0, FieldTend::ReverseUp),
            t if t < self.time_up => (self.amplitude, FieldTend::Stable),
            t if t == self.time_up => (0.0, FieldTend::ReverseDown),
            t if t > self.time_up => (-self.amplitude, FieldTend::Stable),
            _ => unreachable!()
        }
    }
}

type Coord = (usize, usize);
type Neighbours = [Option<usize>;4];//[usize; 8];

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ActivationFunc{
    Linear,
    Quadratic,
    Cubic,
    SquareRoot,
    Treshold,
    Switch
}

impl ActivationFunc{
    fn func(&self, x: f32) -> f32{
        match &self {
            ActivationFunc::Linear => {x},
            ActivationFunc::Quadratic => {x*x},
            ActivationFunc::Cubic => {x*x*x},
            ActivationFunc::SquareRoot => {x.sqrt()},
            ActivationFunc::Treshold => {if x > 0.5 {1.0} else {0.0}},
            ActivationFunc::Switch => {if x > 0.0 {1.0} else {0.0}},
        }
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum GermGenesis{
    StartRandom{
        number: u32 // both fixed up and down
    },
    StartFixed{
        number: u32,
        #[serde(skip)]
        fixed: Vec<usize> // list of those that can't change
    },
    ContinuousRandom{
        chance: f32
    }
}

impl GermGenesis{
    /// should be called at start of generation or reset (creates and saves fixed)
    pub fn activate_once<T: Rng>(&mut self, cells: &mut CellBox, rng: &mut T){ 
        if let Self::StartFixed { number, ref mut fixed } = self {
            *fixed = vec![];
            for field in [-1.0, 1.0]{
                for _ in 0..*number{
                    fixed.push(cells.random_activate(rng, field));
                }
            }
        }
    }

    pub fn new_fixed<T: Rng>(cells: &mut CellBox, rng: &mut T, n: u32) -> Self{
        let mut s = Self::StartFixed { number: n, fixed: Vec::with_capacity((2*n).try_into().unwrap()) };
        s.activate_once(cells, rng);
        s
    }

    /// should be called each field reverse
    fn activate_start<T: Rng>(&mut self, field: f32, cells: &mut CellBox, rng: &mut T){
        if let Self::StartRandom { number } =  self {
            for _ in 0..*number{
                cells.random_activate(rng, field);
            }
        }
    }

    /// should be called each tick (prevents destroying for fixed and creates new for continuous)
    fn tick<T: Rng>(&mut self, field: f32, cells: &mut CellBox, rng: &mut T){
        if let Self::StartFixed { fixed, .. } = self{
            for i in fixed.iter(){
                cells.active.entry(*i).and_modify(|j| *j = 0.0);
            }
        }
        else if let Self::ContinuousRandom{chance} = self {
            let r = rng.gen::<f32>();
            if r < *chance{
                cells.random_activate(rng, field);
            }
        }
    }
}

#[derive(Debug,serde::Deserialize, serde::Serialize)]
pub struct CellBox{
    #[serde(skip)]
    cells: Vec<Cell>,
    #[serde(skip)]
    active: FnvHashMap<usize, f32>,

    #[serde(skip)]
    polarization_counter: i32,

    pub width: usize,
    pub height: usize,

    pub x_spread: f32,
    pub y_spread: f32,
    pub activation_func: ActivationFunc
}

#[derive(Debug, Clone)]
struct Cell{
    polarization: bool
}

impl Cell{
    fn new() -> Self{
        Self { polarization: false }
    }
    fn get_polarization(&self) -> i32{
        return if self.polarization {1} else {-1}
    }

    fn activation<T: Rng>(&self, pol_coeff: f32, field: f32, rng: &mut T, func: &ActivationFunc) -> bool{
        let r = rng.gen::<f32>();
        r < (-1.0/field.abs()/func.func(pol_coeff)).exp()
    }
}

impl CellBox{

    pub fn clear(&mut self){
        let init: Cell = Cell::new();
        self.cells = vec![init; self.width*self.height];
        self.active.clear();
        self.polarization_counter = 0;
    }

    fn new(width: usize, height: usize) -> Self{
        let init: Cell = Cell::new();
        Self { cells: vec![init; width*height],
             active: FnvHashMap::default(),
             width, height,
             polarization_counter: 0,
            x_spread: 1.0,
            y_spread: 0.5,
            activation_func: ActivationFunc::Quadratic, }
    }

    fn bool2charge(b: bool) -> i32{
        return if b {1} else {-1}
    }

    fn index2coord(&self, i: usize) -> Coord{
        (i%self.width, i/self.width)
    }

    fn coord2index(&self, (x, y): (i32, i32)) -> Option<usize>{
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height{
            return None;
        }
        Some(x as usize + y as usize*self.width)
    }

    fn get_neighbours(&self, i: usize) -> Neighbours{
        let (x, y) = self.index2coord(i);
        let x = x as i32;
        let y = y as i32;
        [/*(x-1, y-1),*/ (x, y-1), /*(x+1, y-1),*/
         (x-1, y), (x+1, y),
         /*(x-1, y+1),*/ (x, y+1), /*(x+1, y+1)*/].map(|s| self.coord2index(s))
    }

    fn random_activate<T: Rng>(&mut self, rng: &mut T, field: f32) -> usize{
        let i = rng.gen_range(0..self.width*self.height);
        if field * (self.cells[i].get_polarization() as f32) < 0.0{
            self.activate_cell(i, field, &mut Default::default())
        }
        i
    }

    /// Field there is used to activate neighbours (check whether they are already properly polarised)
    /// Old active data is used to transfer neighbour weight from previous iteration 
    fn activate_cell(&mut self, cell_id: usize, electric_field: f32, old_active: &FnvHashMap<usize, f32>){
        let activation = electric_field > 0.0;
        assert_eq!(!self.cells[cell_id].polarization, activation);

        self.cells[cell_id].polarization = activation;
        self.polarization_counter += Self::bool2charge(activation);
        self.activate_neighbours(cell_id, electric_field, old_active);
    }


    fn activate_neighbours(&mut self, cell_id: usize, electric_field: f32, old_active: &FnvHashMap<usize, f32>){
        for (i, n_id) in self.get_neighbours(cell_id).into_iter().enumerate().filter_map(|(i, j)| j.and_then(|v| Some((i, v)))){
            if electric_field * ((self.cells[n_id].get_polarization() as f32)) < 0.0{
                let pol_coeff = match i {
                    0|3 => self.y_spread,
                    1|2 => self.x_spread,
                    _ => unreachable!()
                };
                let e = self.active.entry(n_id).or_default();
                *e += pol_coeff + old_active.get(&n_id).unwrap_or(&mut 0.0);
            }
        }
    }

    fn step<T: Rng>(&mut self, electric_field: f32, tend: &FieldTend, rng: &mut T){
        let new_vec = FnvHashMap::with_capacity_and_hasher((self.active.len() as f32 *1.4) as usize, Default::default());
        // create map for new iteration

        let active = replace(&mut self.active, new_vec); // save old active cells

        if let FieldTend::Stable = tend{ // field is stable
            for (&cell_id, &cell_accum) in active.iter(){ // iterate over *old cells and weights*

                if electric_field * (self.cells[cell_id].get_polarization() as f32) < 0.0{
                    if self.cells[cell_id].activation(cell_accum, electric_field, rng, &self.activation_func){
                        assert_eq!(electric_field < 0.0, self.cells[cell_id].polarization);

                        self.activate_cell(cell_id, electric_field, &active); // reverse and activate neighbours
                    }
                    else{
                        let e = self.active.entry(cell_id).or_default();
                        *e += cell_accum;
                    }
                }
            }
        }
        else{ // fild is going to change
            let effective_field = match tend{
                FieldTend::ReverseDown => -1.0,
                FieldTend::ReverseUp => 1.0,
                FieldTend::Stable => unreachable!(),
            };
            for (&cell_id, _) in active.iter(){
                self.activate_neighbours(cell_id, effective_field, &active);
            }
        }

        
        //println!("{:?}, {:?}", active, self.active);
    }
}

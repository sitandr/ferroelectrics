

use std::{mem::{replace}};

use eframe::{emath::RectTransform};
use egui::{Painter, Pos2, Color32, Stroke, Rect, Vec2, Rounding};
use rand::Rng;


#[derive( Debug)]
pub struct Simulation{
    pub gen: FieldGenerator,

    pub germ_num: u32,
    pub cells: CellBox,

    // transform: RectTransform,
    // shapes: Vec<Shape>
}


impl Simulation{
    pub fn new(width: usize, height: usize) -> Self{
        Simulation{cells: CellBox::new(width, height), gen: FieldGenerator { t: 0, time_up: 500, time_down: 500, amplitude: 0.4},
        germ_num: 5,
       // transform: RectTransform::identity(Rect::NOTHING),
    /*shapes: vec![]*/}
    }

    pub fn step(&mut self){
        let mut rng = rand::thread_rng();

        let (f, tend) = self.gen.field();
        self.gen.tick();
        
        self.cells.step(f, &tend, &mut rng);
        match tend {
            FieldTend::ReverseDown => {self.cells.random_activate(self.germ_num, &mut rng, -1.0)},
            FieldTend::ReverseUp => {self.cells.random_activate(self.germ_num, &mut rng, 1.0)},
            FieldTend::Stable => {},
        }
    }

    pub fn get_polarization(&self) -> f64{
        (self.cells.polarization_counter as f64)/((self.cells.width*self.cells.height) as f64)
    }

    /*pub fn set_transform(&mut self, transform: RectTransform){
        if transform != self.transform{
            println!("Regenerated");
            self.transform = transform;
            // self.generate_shapes();
        }
    }*/

    /// Call "set_transform" to generate shapes to paint
    pub fn paint(&self, painter: &Painter, transform: RectTransform) {
        
        
        /*for (i, c) in self.cells.cells.iter().enumerate(){
            if c.polarization {
                let (x, y) = self.cells.index2coord(i);
                let point = transform * Pos2::new((x as f32)/(self.cells.width as f32), (y as f32)/(self.cells.height as f32));
                painter.rect(Rect::from_center_size(point, transform.scale() * Pos2::new(1.0/self.cells.width as f32, 1.0/self.cells.height as f32).to_vec2()),
                 1.0, Color32::from_rgb(100, 200, 100), Stroke::new(1.0, Color32::from_gray(64)))
            }
        }*/
        // painter.extend(self.shapes.iter().enumerate().filter_map(|(i, s)| if self.cells.cells[i].polarization {Some(s.clone())}else{None}));
        for &i in self.cells.active.iter(){
            let (x, y) = self.cells.index2coord(i);
            let x = x as f32 * 0.9 + (self.cells.width as f32)/20.0;
            let y = y as f32 * 0.9 + (self.cells.height as f32)/20.0;
            let color_c = self.cells.cells[i].pol_coeff;
            let point = transform * Pos2::new((x as f32)/(self.cells.width as f32), (y as f32)/(self.cells.height as f32));
                painter.rect_filled(Rect::from_center_size(point,
                     transform.scale() * Vec2::new(1.0/self.cells.width as f32, 1.0/self.cells.height as f32)*1.1),
                      Rounding::none(), Color32::from_rgb((150.0*color_c) as u8, (250.0*color_c) as u8, 150));
            //println!("{:?}", (150.0*self.cells.cells[i].pol_coeff).round() as u8);
        }
    }

    /*fn generate_shapes(&mut self){
        let l = self.cells.cells.len();
        self.shapes = (0..l).map(|i|{
                let (x, y) = self.cells.index2coord(i);
                let point = self.transform * Pos2::new((x as f32)/(self.cells.width as f32), (y as f32)/(self.cells.height as f32));
                Shape::Rect(RectShape{rect: Rect::from_center_size(point, self.transform.scale() * Pos2::new(1.0/self.cells.width as f32, 1.0/self.cells.height as f32).to_vec2()),
                rounding: Rounding::none(), fill: Color32::from_rgb(100, 200, 100), stroke: Stroke::new(1.0, Color32::from_gray(64))}
                )
            }
        ).collect();
    }*/
}

enum FieldTend{
    ReverseDown,
    ReverseUp,
    Stable
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct FieldGenerator{
    t: u32,
    pub time_up: u32,
    pub time_down: u32,
    pub amplitude: f32
}

impl FieldGenerator{
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct CellBox{
    cells: Vec<Cell>,
    active: Vec<usize>,

    polarization_counter: i32,

    width: usize,
    height: usize,

    pub x_spread: f32,
    pub y_spread: f32,
    pub activation_func: ActivationFunc
}

#[derive(Debug, Clone)]
struct Cell{
    polarization: bool,
    pol_coeff: f32
}

impl Cell{
    fn new() -> Self{
        Self { polarization: false , pol_coeff: 0.0 }
    }
    fn get_polarization(&self) -> i32{
        return if self.polarization {1} else {-1}
    }

    fn activation<T: Rng>(&self, field: f32, rng: &mut T, func: &ActivationFunc) -> bool{
        let r = rng.gen::<f32>();
        r < (-1.0/field.abs()/func.func(self.pol_coeff)).exp()
    }
}

impl CellBox{
    fn new(width: usize, height: usize) -> Self{
        let init: Cell = Cell::new();
        Self { cells: vec![init; width*height],
             active: vec![],
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

    fn random_activate<T: Rng>(&mut self, n: u32, rng: &mut T, field: f32){
        for _ in 0..n {
            let i = rng.gen_range(0..self.width*self.height);
            if field * (self.cells[i].get_polarization() as f32) < 0.0{
                self.activate_cell(i, field)
            }   
        }
    }

    /// Field there is used to activate neighbours (check whether they are already properly polarised)
    /// 
    fn activate_cell(&mut self, cell_id: usize, electric_field: f32){
        let activation = electric_field > 0.0;
        assert_eq!(!self.cells[cell_id].polarization, activation);
        self.cells[cell_id].polarization = activation;
        self.cells[cell_id].pol_coeff = 0.0;
        self.polarization_counter += Self::bool2charge(activation);
        self.activate_neighbours(cell_id, electric_field);
    }

    fn activate_neighbours(&mut self, cell_id: usize, electric_field: f32){
        for (i, n_id) in self.get_neighbours(cell_id).into_iter().filter_map(|j| j).enumerate(){
            if electric_field * ((self.cells[n_id].get_polarization() as f32)) < 0.0{
                self.cells[n_id].pol_coeff += match i {
                    0|3 => self.y_spread,
                    1|2 => self.x_spread,
                    _ => unreachable!()
                };
                self.active.push(n_id)
            }
        }
    }

    fn step<T: Rng>(&mut self, electric_field: f32, tend: &FieldTend, rng: &mut T){
        let new_vec = Vec::with_capacity((self.active.len() as f32*1.1) as usize);
        let active = replace(&mut self.active, new_vec);
        if let FieldTend::Stable = tend{
            for cell_id in active{
                assert_ne!(electric_field, 0.0);
                if electric_field * (self.cells[cell_id].get_polarization() as f32) < 0.0{
                    if self.cells[cell_id].activation(electric_field, rng, &self.activation_func){
                        assert_eq!(electric_field < 0.0, self.cells[cell_id].polarization);
                        self.activate_cell(cell_id, electric_field)
                    }
                    else{
                        self.active.push(cell_id);
                    }
                }
            }
        }
        else{
            let effective_field = match tend{
                FieldTend::ReverseDown => -1.0,
                FieldTend::ReverseUp => 1.0,
                FieldTend::Stable => unreachable!(),
            };
            for cell_id in active{
                self.activate_neighbours(cell_id, effective_field);
            }
        }

        
        //println!("{:?}, {:?}", active, self.active);
    }
}

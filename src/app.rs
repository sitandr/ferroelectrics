use std::vec;

use eframe::emath;
use egui::{Painter, Rect, Pos2, Stroke, Color32, plot::{Plot, Line, PlotPoints}};
use rand::{rngs::StdRng, SeedableRng};

use crate::physics::{Simulation, ActivationFunc, GermGenesis};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    simulation: Simulation,
    #[serde(skip)]
    paused: bool,
    #[serde(skip)]
    points: Vec<(f64, f64)>,
    #[serde(skip)]
    time: f64,
    #[serde(skip)]
    rng: StdRng,
    #[serde(skip)]
    double_step: bool,
    seed: i32 // seed that will be used after reset <0 => random
}

impl Default for App {
    
    fn default() -> Self {
        Self {
            time: 0.0,
            points: vec![],
            double_step: false,
            simulation:  Simulation::new(100, 100),
            paused: false,
            rng: StdRng::from_entropy(),
            seed: -1
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let mut app: Self =  eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            app.reset();
            app
        }
        else{
            Default::default()
        }
    }

    pub fn reset(&mut self){
        if self.seed >= 0{
            self.rng = StdRng::seed_from_u64(self.seed as u64);
        }
        
        self.simulation.reset(&mut self.rng);
        
        self.points.clear();
        self.time = 0.0;
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });
        
        

        egui::Window::new("Параметры").show(ctx, |ui| {

            ui.checkbox(&mut self.paused, "Приостановить");
            ui.checkbox(&mut self.double_step, "Двойной шаг");

            egui::ComboBox::from_label("Зародышеобразование")
                .selected_text(match self.simulation.germs {
                    GermGenesis::StartRandom { .. } => "Случайные",
                    GermGenesis::StartFixed { .. } => "Фиксированные",
                    GermGenesis::ContinuousRandom { .. } => "Постепенные",
                })
                .show_ui(ui, |ui| {
                    /*ui.selectable_value(&mut simulation.germs, GermGenesis::StartRandom{number: 5}, "Случайные зародыши при каждом переключении");
                    ui.selectable_value(&mut simulation.germs, GermGenesis::new_fixed(&mut simulation.cells, &mut self.rng, 5), "Фиксированные дефекты");
                    ui.selectable_value(&mut simulation.germs, GermGenesis::ContinuousRandom { chance: 0.1 }, "Случайное постепенное образование");
                    */
                    if ui.selectable_label(if let GermGenesis::StartRandom{..} = self.simulation.germs {true} else {false},
                         "Случайные зародыши").clicked(){
                            self.simulation.germs = GermGenesis::StartRandom{number: 5};
                    }
                    if ui.selectable_label(if let GermGenesis::StartFixed{..} = self.simulation.germs {true} else {false},
                         "Фиксированные зародыши").clicked(){
                            self.simulation.germs = GermGenesis::new_fixed(&mut self.simulation.cells, &mut self.rng, 5);
                    }
                    if ui.selectable_label(if let GermGenesis::ContinuousRandom{..} = self.simulation.germs {true} else {false},
                        "Постепенное зарождение").clicked(){
                            self.simulation.germs = GermGenesis::ContinuousRandom { chance: 0.2 };
                   }
                }
            );

            match &mut self.simulation.germs {
                GermGenesis::StartRandom { number }| GermGenesis::StartFixed { number, .. }=> {
                    ui.add(egui::Slider::new(number, 0..=100).text("Число зародышей"));
                },
                GermGenesis::ContinuousRandom { chance } => {
                    ui.add(egui::Slider::new(chance, 0.0..=1.0).text("Шанс зародышеобразования на тик"));
                },
            }

           // ui.add(egui::Slider::new(&mut simulation.germs, 1..=10).text("Число зародышей"));
            ui.add(egui::Slider::new(&mut self.simulation.cells.x_spread, 0.0..=2.0).text("Скорость по x"));
            ui.add(egui::Slider::new(&mut self.simulation.cells.y_spread, 0.0..=2.0).text("Скорость по y"));

            let act_func = &mut self.simulation.cells.activation_func;
            egui::ComboBox::from_label("Функция активации")
                .selected_text(match act_func {
                    ActivationFunc::Linear => "Linear",
                    ActivationFunc::Quadratic => "Quadratic",
                    ActivationFunc::Cubic => "Cubic",
                    ActivationFunc::SquareRoot => "SquareRoot",
                    ActivationFunc::Treshold => "Treshold",
                    ActivationFunc::Switch => "Switch",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(act_func, ActivationFunc::Linear, "Linear");
                    ui.selectable_value(act_func, ActivationFunc::Quadratic, "Quadratic");
                    ui.selectable_value(act_func, ActivationFunc::Cubic, "Cubic");
                    ui.selectable_value(act_func, ActivationFunc::SquareRoot, "SquareRoot");
                    ui.selectable_value(act_func, ActivationFunc::Treshold, "Treshold");
                    ui.selectable_value(act_func, ActivationFunc::Switch, "Switch");
                }
            );


            ui.label("Сигнал");
            ui.add(egui::Slider::new(&mut self.simulation.gen.time_up, 1..=500_000).logarithmic(true).text("Время поля \"вверх\""));
            ui.add(egui::Slider::new(&mut self.simulation.gen.time_down, 1..=500_000).logarithmic(true).text("Время поля \"вниз\""));
            ui.add(egui::Slider::new(&mut self.simulation.gen.amplitude, 0.001..=5.0).text("Амплитуда поля"));

            ui.add(egui::Separator::default());

            if ui.add(egui::Slider::new(&mut self.simulation.cells.width, 1..=500).text("Ширина")).changed(){
                self.reset();
            }
            if ui.add(egui::Slider::new(&mut self.simulation.cells.height, 1..=500).text("Высота")).changed(){
                self.reset();
            };

            ui.add(egui::Separator::default());

            ui.add(egui::Slider::new(&mut self.seed, -1..=i32::MAX).text("Seed"));

            if self.paused{
                let mut points_text = self.points.iter().map(|(_, j)| format!("{:.4}", j)).collect::<Vec<_>>().join(";");
                ui.label("Данные:");
                ui.add(egui::TextEdit::singleline(&mut points_text));
            }

            if ui.button("Сбросить").clicked() {
                self.reset();
            }
            
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            if !self.paused{
                self.time += 0.01;
                self.simulation.step(&mut self.rng);
                if self.double_step{
                    self.simulation.step(&mut self.rng)
                }
                let /*mut*/ measure: f64 = self.simulation.get_polarization() as f64;
                ui.ctx().request_repaint();

                if self.time % 0.05 < 0.01{
                    self.points.push((self.time, measure));
                }
            }
            
            let mut rect = ui.available_rect_before_wrap();
            if rect.height() > rect.width(){
                rect.set_height(rect.width())
            }
            else{
                rect.set_width(rect.height())
            }
            
            let painter = Painter::new(
                ui.ctx().clone(),
                ui.layer_id(),
                rect,
            );
            let rect = painter.clip_rect();
            let to_screen = emath::RectTransform::from_to(
                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                rect,
            );
            // simulation.set_transform(to_screen);
            self.simulation.paint(&painter, to_screen);
            painter.rect_stroke(rect, 1.0, Stroke::new(1.0, Color32::from_gray(16)));
            // Make sure we allocate what we used (everything)
            ui.expand_to_include_rect(painter.clip_rect());
        
            egui::warn_if_debug_build(ui);
        });

        if true {
            egui::Window::new("Поляризация").show(ctx, |ui| {
                Plot::new("data").include_y(0.0).include_x(0.0).auto_bounds_y().auto_bounds_x().show(ui, |plot_ui| plot_ui.line(Line::new(
                    self.points.iter().map(|&(x, p)| {
                        [x, p]}).collect::<PlotPoints>())));
            });
        }
    }
}

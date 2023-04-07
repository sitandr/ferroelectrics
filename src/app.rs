use std::vec;

use eframe::emath;
use egui::{Painter, Rect, Pos2, Stroke, Color32, plot::{Plot, Line, PlotPoints}};

use crate::physics::{Simulation};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    #[serde(skip)]
    simulation: Simulation,
    #[serde(skip)]
    paused: bool,
    #[serde(skip)]
    points: Vec<(f64, f64)>,
    #[serde(skip)]
    time: f64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            time: 0.0,
            points: vec![],
            simulation:  Simulation::new(200, 200),
            paused: false
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
            let app: Self =  eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            //app.simulation.random_initiation();
            app
        }
        else{
            Default::default()
        }
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
        let Self {simulation, time, points, ..} = self;

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
            ui.add(egui::Slider::new(&mut simulation.germ_num, 1..=10).text("Число зародышей"));
            if ui.button("Перегенировать").clicked() {
                //simulation.random_initiation();
                *time = 0.0;
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            if !self.paused{
                self.time += 0.01;
                simulation.step();
                let /*mut*/ measure: f64 = simulation.get_polarization() as f64;
                ui.ctx().request_repaint();

                if self.time % 0.1 < 0.01{
                    points.push((self.time, measure));
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
            simulation.paint(&painter, to_screen);
            painter.rect_stroke(rect, 1.0, Stroke::new(1.0, Color32::from_gray(16)));
            // Make sure we allocate what we used (everything)
            ui.expand_to_include_rect(painter.clip_rect());
            egui::warn_if_debug_build(ui);
        });

        if true {
            egui::Window::new("Поляризация").show(ctx, |ui| {
                Plot::new("data").include_y(0.0).include_x(0.0).auto_bounds_y().auto_bounds_x().show(ui, |plot_ui| plot_ui.line(Line::new(
                    points.iter().map(|&(x, p)| {
                        [x, p]}).collect::<PlotPoints>())));
            });
        }
    }
}

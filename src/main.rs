use std::f32::consts::TAU;

use eframe::egui;
use egui::{containers::*, widgets::*, *};

fn main() {
    println!("Hello!");
    let native_options = eframe::NativeOptions::default();

    let _ = eframe::run_native(
        "Grust",
        native_options,
        Box::new(|cc: &eframe::CreationContext<'_>| Box::new(MyEguiApp::new(cc))),
    );
}

struct MyEguiApp {
    fractal_clock: FractalClock,
    frames: f64,
}

impl Default for MyEguiApp {
    fn default() -> Self {
        Self {
            frames: 0.0,
            fractal_clock: FractalClock::default(),
        }
    }
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::dark_canvas(&ctx.style()))
            .show(ctx, |ui| {
                self.frames += 0.1;
                self.fractal_clock.ui(ui, Some(self.frames));
            });
    }
}

#[derive(PartialEq)]
pub struct FractalClock {
    paused: bool,
    time: f64,
    zoom: f32,
    start_line_width: f32,
    depth: usize,
    length_factor: f32,
    luminance_factor: f32,
    width_factor: f32,
    line_count: usize,
}

impl Default for FractalClock {
    fn default() -> Self {
        Self {
            paused: false,
            time: 0.0,
            zoom: 0.25,
            start_line_width: 2.5,
            depth: 9,
            length_factor: 0.8,
            luminance_factor: 0.8,
            width_factor: 0.9,
            line_count: 0,
        }
    }
}

impl FractalClock {
    pub fn ui(&mut self, ui: &mut Ui, seconds_since_midnight: Option<f64>) {
        if !self.paused {
            self.time = seconds_since_midnight.unwrap_or_else(|| ui.input(|i| i.time));
            ui.ctx().request_repaint();
        }

        let painter = Painter::new(
            ui.ctx().clone(),
            ui.layer_id(),
            ui.available_rect_before_wrap(),
        );

        self.paint(&painter);
        // Make sure we allocate what we used (everything)
        ui.expand_to_include_rect(painter.clip_rect());

        Frame::popup(ui.style())
            .stroke(Stroke::NONE)
            .show(ui, |ui| {
                ui.set_max_width(270.0);
                CollapsingHeader::new("Settings")
                    .show(ui, |ui| self.options_ui(ui, seconds_since_midnight));
            });
    }

    fn options_ui(&mut self, ui: &mut Ui, seconds_since_midnight: Option<f64>) {
        if seconds_since_midnight.is_some() {
            ui.label(format!(
                "Local time: {:02}:{:02}:{:02}.{:03}",
                (self.time % (24.0 * 60.0 * 60.0) / 3600.0).floor(),
                (self.time % (60.0 * 60.0) / 60.0).floor(),
                (self.time % 60.0).floor(),
                (self.time % 1.0 * 100.0).floor()
            ));
        } else {
            ui.label("The fractal_clock clock is not showing the correct time");
        };
        ui.label(format!("Painted line count: {}", self.line_count));

        ui.checkbox(&mut self.paused, "Paused");
        ui.add(Slider::new(&mut self.zoom, 0.0..=1.0).text("zoom"));
        ui.add(Slider::new(&mut self.start_line_width, 0.0..=5.0).text("Start line width"));
        ui.add(Slider::new(&mut self.depth, 0..=14).text("depth"));
        ui.add(Slider::new(&mut self.length_factor, 0.0..=1.0).text("length factor"));
        ui.add(Slider::new(&mut self.luminance_factor, 0.0..=1.0).text("luminance factor"));
        ui.add(Slider::new(&mut self.width_factor, 0.0..=1.0).text("width factor"));
    }

    fn paint(&mut self, painter: &Painter) {
        let mut shapes: Vec<Shape> = Vec::new();

        let rect = painter.clip_rect();
        let to_screen = emath::RectTransform::from_to(
            Rect::from_center_size(Pos2::ZERO, rect.square_proportions() / self.zoom),
            rect,
        );

        let mut paint_circle = |points: Pos2, color: Color32, width: f32| {
            let point = to_screen * points;
            let radius = 10.0;
            let top_left: Pos2 = Pos2::new(point.x - radius, point.y - radius);
            let bottom_right: Pos2 = Pos2::new(point.x + radius, point.y + radius);
            // culling
            if rect.intersects(Rect::from_min_max(top_left, bottom_right)) {
                shapes.push(Shape::circle_stroke( point, radius, (width, color)));
            }
        };

        paint_circle(Pos2::new(0.0, 0.0), Color32::from_additive_luminance(255), 20.0);
        painter.extend(shapes);
    }
}

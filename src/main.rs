use eframe::egui;

mod graph;

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
    fractal_clock: graph::Graph,
    frames: f64,
}

impl Default for MyEguiApp {
    fn default() -> Self {
        Self {
            frames: 0.0,
            fractal_clock: graph::Graph::default(),
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

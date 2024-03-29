use eframe::egui;

fn main() {
    println!("Hello!");
    let frames = 100;
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc))),
    );
}

#[derive(Default)]
struct MyEguiApp {}

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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        static mut FRAMES: i64 = 0;
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut message = "Hello World! Frame number: ".to_owned();
            let mut frames: String;
            unsafe {
                FRAMES += 1;
                frames = FRAMES.to_string();
            }
            
            message.push_str(&frames);
            ui.heading(message);
        });
    }
}

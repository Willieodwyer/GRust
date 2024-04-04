use eframe::egui;

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
    fractal_clock: Graph,
    frames: f64,
}

impl Default for MyEguiApp {
    fn default() -> Self {
        Self {
            frames: 0.0,
            fractal_clock: Graph::default(),
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

// ------------------------------

use egui::{containers::*, widgets::*, *};

#[derive(PartialEq)]
pub struct Graph {
    paused: bool,
    time: f64,
    zoom: f32,
    start_line_width: f32,
    depth: usize,
    length_factor: f32,
    luminance_factor: f32,
    dampening: f32,
    line_count: usize,
    nodes: [Node; 6],
    adjacency_matrix: [[bool; 6]; 6]
}

impl Default for Graph {
    fn default() -> Self {
        // Nodes
        let node0 = Node::new(-1.1, 1.0);
        let node1 = Node::new(-1.0, 1.0);
        let node2 = Node::new(1.0, -1.0);
        let node3 = Node::new(-1.0, -1.0);
        let node4 = Node::new(1.3, -1.8);
        let node5 = Node::new(-0.01, -1.9);

        let mut nodes: [Node; 6] = [node0, node1, node2, node3, node4, node5];

        let mut adjacency_matrix: [[bool; 6]; 6] = [[false; 6]; 6];
        adjacency_matrix[0][1] = true;
        adjacency_matrix[1][0] = true;

        adjacency_matrix[1][2] = true;
        adjacency_matrix[2][1] = true;

        adjacency_matrix[2][3] = true;
        adjacency_matrix[3][2] = true;

        adjacency_matrix[3][4] = true;
        adjacency_matrix[4][3] = true;

        adjacency_matrix[4][5] = true;
        adjacency_matrix[5][4] = true;

        adjacency_matrix[5][0] = true;
        adjacency_matrix[0][5] = true;

        adjacency_matrix[5][3] = true;
        adjacency_matrix[3][5] = true;

        Self {
            paused: false,
            time: 0.0,
            zoom: 0.15,
            start_line_width: 2.5,
            depth: 9,
            length_factor: 0.8,
            luminance_factor: 0.8,
            dampening: 0.0001,
            line_count: 0,
            nodes, 
            adjacency_matrix
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Node {
    x: f32,
    y: f32,
    force_x: f32,
    force_y: f32,
    velocity_x: f32,
    velocity_y: f32,
}

impl Node {
    fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            force_x: 0.0,
            force_y: 0.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
        }
    }
}

impl Graph {
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

        self.paint(ui, &painter);
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
        ui.add(Slider::new(&mut self.dampening, 0.0..=0.01).text("dampening"));
    }

    fn paint(&mut self, ui: &mut Ui, painter: &Painter) {
        let mut shapes: Vec<Shape> = Vec::new();

        let rect = painter.clip_rect();
        let to_screen = emath::RectTransform::from_to(
            Rect::from_center_size(Pos2::ZERO, rect.square_proportions() / self.zoom),
            rect,
        );

        let paint_line =
            |points: [Pos2; 2], color: Color32, width: f32, shapes: &mut Vec<Shape>| {
                let line = [to_screen * points[0], to_screen * points[1]];

                // culling
                if rect.intersects(Rect::from_two_pos(line[0], line[1])) {
                    shapes.push(Shape::line_segment(line, (width, color)));
                }
            };

        let paint_circle = |points: Pos2, color: Color32, width: f32, shapes: &mut Vec<Shape>| {
            let point = to_screen * points;
            let radius = 10.0;
            let top_left: Pos2 = Pos2::new(point.x - radius, point.y - radius);
            let bottom_right: Pos2 = Pos2::new(point.x + radius, point.y + radius);
            // culling
            if rect.intersects(Rect::from_min_max(top_left, bottom_right)) {
                shapes.push(Shape::circle_filled(point, radius, color));

                // Add text overlay with x and y coordinates
                let text = format!("({}, {})", points.x, points.y);
                let text_position = point + Vec2::new(radius, -radius);
                let text_color = Color32::GRAY;

                shapes.push(ui.fonts(|f| {
                    Shape::text(
                        f,
                        text_position,
                        egui::Align2::LEFT_BOTTOM,
                        text,
                        TextStyle::Monospace.resolve(ui.style()),
                        color,
                    )
                }));
            }
        };

        // Algorithm
        for i in 0..self.nodes.len() {
            let mut v: Node = self.nodes[i];

            v.force_x = 0.0;
            v.force_y = 0.0;

            // Push away from other nodes
            for j in 0..self.nodes.len() {
                if i == j {
                    continue;
                }

                let u = &self.nodes[j];

                let rsq = 0.25 * ((v.x - u.x) * (v.x - u.x) + (v.y - u.y) * (v.y - u.y));

                v.force_x += 10.0 * ((v.x - u.x) / rsq);
                v.force_y += 10.0 * ((v.y - u.y) / rsq);
            }

            // Push away from corners
            for (x,y) in [(100.0,100.0), (-100.0,10.0), (100.0,-100.0), (-100.0,-100.0)]
            {
                let rsq: f32 = 0.25 * ((v.x - x) * (v.x - x) + (v.y - y) * (v.y - y));
                v.force_x += 100.0 * ((v.x - x) / rsq);
                v.force_y += 100.0 * ((v.y - y) / rsq);
            }


            // Attract
            for j in 0..self.nodes.len() {
                if (self.adjacency_matrix[i][j]) {
                    let u: &Node = &self.nodes[j];
                    v.force_x += 5.0 * (u.x - v.x);
                    v.force_y += 5.0 * (u.y - v.y);
                }
            }

            v.velocity_x = (v.velocity_x + v.force_x) * self.dampening;
            v.velocity_y = (v.velocity_y + v.force_y) * self.dampening;
            self.nodes[i] = v;
        }

        for x in 0..self.nodes.len() {
            let mut v = self.nodes[x];
            v.x = v.x + v.velocity_x;
            v.y = v.y + v.velocity_y;
            self.nodes[x] = v;
        }

        // Render
        // - Lines
        for i in 0..self.nodes.len() {
            for j in 0..self.nodes.len() {
                if self.adjacency_matrix[i][j] == true {
                    paint_line(
                        [
                            Pos2::new(self.nodes[i].x, self.nodes[i].y),
                            Pos2::new(self.nodes[j].x, self.nodes[j].y),
                        ],
                        Color32::from_additive_luminance(255),
                        10.0,
                        &mut shapes,
                    );
                }
            }
        }

        // - Nodes
        for node in 0..self.nodes.len() {
            paint_circle(
                Pos2::new(self.nodes[node].x, self.nodes[node].y),
                Color32::from_additive_luminance(255),
                2.0,
                &mut shapes,
            );
        }

        painter.extend(shapes);
    }
}

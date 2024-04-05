use egui::{containers::*, widgets::*, *};
use rand::prelude::*;

#[derive(PartialEq)]
pub struct Graph {
    paused: bool,
    time: f64,
    zoom: f32,
    bounds: f32,
    attraction: f32,
    repulsion: f32,
    velocity: f32,
    nodes: Vec<Node>,
    adjacency_matrix: Vec<Vec<bool>>,
}

impl Graph {
    pub fn new(adj_mtx: Vec<Vec<bool>>) -> Self {
        let mut nodes: Vec<Node> = Vec::new();

        for i in 0..adj_mtx[0].len() {
            let mut rng = rand::thread_rng();
            let random_x: f32 = rng.gen_range(0.0..1.0);
            let random_y: f32 = rng.gen_range(0.0..1.0);
            nodes.push(Node::new(i, random_x, random_y));
        }

        let def = Self {
            paused: false,
            time: 0.0,
            zoom: 0.01,
            bounds: 1.0,
            attraction: 1.0,
            repulsion: 1.0,
            velocity: 1.0,
            nodes: nodes,
            adjacency_matrix: adj_mtx,
        };

        return def;
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            paused: false,
            time: 0.0,
            zoom: 0.01,
            bounds: 1.0,
            attraction: 1.0,
            repulsion: 1.0,
            velocity: 1.0,
            nodes: Vec::new(),
            adjacency_matrix: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Node {
    idx: usize,
    x: f32,
    y: f32,
    force_x: f32,
    force_y: f32,
    velocity_x: f32,
    velocity_y: f32,
}

impl Node {
    fn new(name: usize, x: f32, y: f32) -> Self {
        Self {
            idx: name,
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

        ui.checkbox(&mut self.paused, "Paused");
        ui.add(Slider::new(&mut self.zoom, 0.0..=0.05).text("Zoom"));
        ui.add(Slider::new(&mut self.attraction, -1.0..=10.0).text("Attraction"));
        ui.add(Slider::new(&mut self.repulsion, -1.0..=10.0).text("Repulsion"));
        ui.add(Slider::new(&mut self.bounds, -1.0..=10.0).text("Bounds strength"));
        ui.add(Slider::new(&mut self.velocity, -1.0..=10.0).text("velocity"));
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

        let paint_circle =
            |idx: usize, points: Pos2, color: Color32, width: f32, shapes: &mut Vec<Shape>| {
                let point = to_screen * points;
                let radius = 10.0;
                let top_left: Pos2 = Pos2::new(point.x - radius, point.y - radius);
                let bottom_right: Pos2 = Pos2::new(point.x + radius, point.y + radius);
                // culling
                if rect.intersects(Rect::from_min_max(top_left, bottom_right)) {
                    shapes.push(Shape::circle_filled(point, radius, color));

                    // Add text overlay with x and y coordinates
                    let text = format!("(#{} {}, {})", idx, points.x, points.y);
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

            // Attract
            for j in 0..self.nodes.len() {
                if (self.adjacency_matrix[i][j]) {
                    let u: &Node = &self.nodes[j];
                    v.force_x += 5.0 * self.attraction * (u.x - v.x);
                    v.force_y += 5.0 * self.attraction * (u.y - v.y);
                }
            }

            // Push away from other nodes
            for j in 0..self.nodes.len() {
                if i == j {
                    continue;
                }

                let u = &self.nodes[j];

                let rsq = 0.25 * ((v.x - u.x) * (v.x - u.x) + (v.y - u.y) * (v.y - u.y));

                v.force_x += 10.0 * self.repulsion * ((v.x - u.x) / rsq);
                v.force_y += 10.0 * self.repulsion * ((v.y - u.y) / rsq);
            }

            // Push away from corners
            for point in [10000.0, -10000.0] {
                let rsq_x: f32 = 0.25 * f32::powf((v.x - point), 2.0);
                let rsq_y: f32 = 0.25 * f32::powf((v.y - point), 2.0);
                v.force_x += 10000000.0 * self.bounds * ((v.x - point) / rsq_x);
                v.force_y += 10000000.0 * self.bounds * ((v.y - point) / rsq_y);
            }

            v.velocity_x = (v.velocity_x + v.force_x) * 0.01 * self.velocity;
            v.velocity_y = (v.velocity_y + v.force_y) * 0.01 * self.velocity;
            self.nodes[i] = v;
        }

        for x in 0..self.nodes.len() {
            let mut v: Node = self.nodes[x];
            if v.velocity_x.is_nan()
                || v.velocity_y.is_nan()
                || v.velocity_x.abs() > 100.0
                || v.velocity_y.abs() > 100.0
            {
                // Just give a random velocity between 0 and 1;
                let mut rng = rand::thread_rng();
                v.velocity_x = rng.gen_range(-1.0..1.0);
                v.velocity_y = rng.gen_range(-1.0..1.0);
            }
            v.x = v.x + v.velocity_x;
            v.y = v.y + v.velocity_y;

            if v.x.abs() > 1000.0 {
                v.x = if v.x.is_sign_negative() { 1000.0 } else { -1000.0 };
            }

            if v.y.abs() > 1000.0 {
                v.y = if v.y.is_sign_negative() { 1000.0 } else { -1000.0 };
            }

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
                        Color32::from_additive_luminance(128),
                        10.0,
                        &mut shapes,
                    );
                }
            }
        }

        // - Nodes
        for node in &self.nodes {
            paint_circle(
                node.idx,
                Pos2::new(node.x, node.y),
                Color32::from_additive_luminance(255),
                2.0,
                &mut shapes,
            );
        }

        painter.extend(shapes);
    }
}

use eframe::egui;
use egui::debug_text::print;
use std::{
    fs::{self, File},
    io,
    process::exit,
    str::FromStr,
};

use std::collections::HashSet;
use std::io::{BufRead, BufReader};

mod graph;

fn main() {
    println!("Hello!");
    let file_path = "graph.txt";

    let open_file: Result<File, std::io::Error> = File::open(file_path);
    if open_file.is_err() {
        eprintln!("Graph file: {file_path} not found");
        exit(1);
    }

    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut node_set: HashSet<usize> = HashSet::new();

    let file = open_file.unwrap();
    println!("{:?}", file);

    let reader = io::BufReader::new(file);

    let mut min = usize::max_value();

    for line in reader.lines() {
        let line = line.unwrap();
        let edge: Vec<&str> = line.split(' ').collect();
        if edge.len() != 2 {
            eprint!("Edgelist parse error!");
            exit(-1);
        }

        let left = edge[0].parse::<usize>().unwrap();
        let right = edge[1].parse::<usize>().unwrap();

        if left == right {
            continue;
        }

        if left < min {
            min = left
        };

        if right < min {  
            min = right;
        }

        node_set.insert(left);
        node_set.insert(right);

        edges.push((left, right));
    }

    println!("{:?}", edges);
    println!("{:?}", node_set);

    let mut adjacency_matrix: Vec<Vec<bool>> = vec![vec![false; node_set.len()]; node_set.len()];
    for edge in &edges {
        println!("{:?}", edge);
        adjacency_matrix[edge.0 - min][edge.1 - min] = true;
        adjacency_matrix[edge.1 - min][edge.0 - min] = true;
    }

    println!("{:?}", adjacency_matrix);

    let native_options = eframe::NativeOptions::default();

    let _ = eframe::run_native(
        "Grust",
        native_options,
        Box::new(|cc: &eframe::CreationContext<'_>| Box::new(MyEguiApp::new(cc, adjacency_matrix))),
    );
}

struct MyEguiApp {
    graph: graph::Graph,
    frames: f64,
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>, adj_mtx: Vec<Vec<bool>>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let mut def = Self {
            frames: 0.0,
            graph: graph::Graph::new(adj_mtx),
        };
        def
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::dark_canvas(&ctx.style()))
            .show(ctx, |ui| {
                self.frames += 0.1;
                self.graph.ui(ui, Some(self.frames));
            });
    }
}

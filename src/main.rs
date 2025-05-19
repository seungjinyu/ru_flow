use chrono::Local;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::format;
use std::fs::{self, File};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct Experiment {
    id: String,          // UUID
    name: String,        // name of the experiment
    script_path: String, //path of the script
    args: Vec<String>,   // arguements
    timestamp: String,
    result: Option<serde_json::Value>, // save the result of the experiment
}

fn main() {
    println!("Hello, world!");

    let args: Vec<String> = env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("register") => register_experiment(),
        Some("run") => run_experiment(),
        Some("list") => list_experiments(),
        _ => {
            println!("Usage");
            println!(" register # Register new experiment");
            println!(" run      # run new experiment");
            println!(" list     # Register new experiment");
        }
    }
}

fn register_experiment() {
    println!("register_experiment");

    // create an Uuid
    let id = Uuid::new_v4().to_string();

    // current time
    let timestamp = Local::now().to_rfc3339();

    // create experiment data
    let experiment = Experiment {
        id: id.clone(),
        name: "example".to_string(),
        script_path: "train.py".to_string(),
        args: vec!["--lr=0.01".to_string(), "--batch=32".to_string()],
        timestamp,
        result: None,
    };

    let dir_path = format!("experiment/{}", id);
    fs::create_dir_all(&dir_path).expect("Failed to create directory");

    let file_path = format!("{}/meta.json", dir_path);
    let file = File::create(file_path).expect("Failed to create file");

    serde_json::to_writer_pretty(file, &experiment).expect("Failed to save JSON");

    println!("Experiment register complete");
}
fn run_experiment() {
    println!("run_experiment");
}
fn list_experiments() {
    println!("list_experiment");
}
